#!/usr/bin/env python3
"""Content-blind preflight for ClipType P3 release candidates.

This command verifies repository provenance, bundle metadata, Universal 2 slices,
Developer ID verification, notarization stapling, Gatekeeper assessment, and artifact
digests without printing credentials or application content.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import plistlib
import re
import shutil
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Sequence

SHA_RE = re.compile(r"^[0-9a-f]{40}$")
EXPECTED_REPOSITORY_FILES = (
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "apps/cliptype-macos/Cargo.toml",
    "crates/cliptype-macos/Cargo.toml",
    "packaging/macos/package.sh",
    ".github/workflows/macos-release.yml",
)


class PreflightError(RuntimeError):
    """Raised when the preflight itself cannot execute safely."""


@dataclass(frozen=True)
class CheckResult:
    check: str
    status: str
    detail: str


class Results:
    def __init__(self) -> None:
        self.items: list[CheckResult] = []

    def pass_(self, check: str, detail: str) -> None:
        self.items.append(CheckResult(check, "pass", _single_line(detail)))

    def fail(self, check: str, detail: str) -> None:
        self.items.append(CheckResult(check, "fail", _single_line(detail)))

    def skip(self, check: str, detail: str) -> None:
        self.items.append(CheckResult(check, "skipped", _single_line(detail)))

    @property
    def failed(self) -> bool:
        return any(item.status == "fail" for item in self.items)


def _single_line(value: str, *, limit: int = 500) -> str:
    text = str(value).replace("\r", " ").replace("\n", " ").strip()
    if len(text) > limit:
        text = text[: limit - 3] + "..."
    return text


def _run(command: Sequence[str], *, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(command),
        cwd=cwd,
        text=True,
        capture_output=True,
        check=False,
        env={**os.environ, "LC_ALL": "C", "LANG": "C"},
    )


def _command_detail(process: subprocess.CompletedProcess[str]) -> str:
    output = process.stdout.strip() or process.stderr.strip() or f"exit {process.returncode}"
    return _single_line(output)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def check_repository(results: Results, repo: Path, expected_commit: str, require_clean: bool) -> None:
    if not SHA_RE.fullmatch(expected_commit):
        raise PreflightError("--commit must be a lowercase 40-character Git SHA")
    if not (repo / ".git").exists():
        raise PreflightError(f"not a Git working tree: {repo}")

    process = _run(["git", "rev-parse", "HEAD"], cwd=repo)
    if process.returncode != 0:
        raise PreflightError(f"unable to read Git HEAD: {_command_detail(process)}")
    head = process.stdout.strip()
    if head == expected_commit:
        results.pass_("repository.exact_commit", head)
    else:
        results.fail("repository.exact_commit", f"HEAD {head} != expected {expected_commit}")

    process = _run(["git", "status", "--porcelain=v1", "--untracked-files=all"], cwd=repo)
    if process.returncode != 0:
        raise PreflightError(f"unable to inspect working tree: {_command_detail(process)}")
    dirty = [line for line in process.stdout.splitlines() if line.strip()]
    if not dirty:
        results.pass_("repository.clean_tree", "working tree is clean")
    elif require_clean:
        results.fail("repository.clean_tree", f"working tree has {len(dirty)} changed paths")
    else:
        results.skip("repository.clean_tree", f"working tree has {len(dirty)} changed paths")

    missing = [path for path in EXPECTED_REPOSITORY_FILES if not (repo / path).is_file()]
    if missing:
        results.fail("repository.release_inputs", "missing: " + ", ".join(missing))
    else:
        results.pass_("repository.release_inputs", f"{len(EXPECTED_REPOSITORY_FILES)} inputs present")

    package = repo / "packaging/macos/package.sh"
    if package.is_file() and os.access(package, os.X_OK):
        results.pass_("repository.package_script_executable", "packaging/macos/package.sh")
    else:
        results.fail("repository.package_script_executable", "package.sh is missing or not executable")


def read_bundle_metadata(app: Path) -> tuple[Path, dict[str, Any]]:
    info_path = app / "Contents" / "Info.plist"
    if not info_path.is_file():
        raise PreflightError(f"bundle is missing Info.plist: {info_path}")
    try:
        with info_path.open("rb") as stream:
            info = plistlib.load(stream)
    except (plistlib.InvalidFileException, OSError) as exc:
        raise PreflightError(f"unable to parse Info.plist: {exc}") from exc
    if not isinstance(info, dict):
        raise PreflightError("Info.plist must contain a dictionary")
    executable_name = info.get("CFBundleExecutable")
    if not isinstance(executable_name, str) or not executable_name or "/" in executable_name:
        raise PreflightError("Info.plist has an invalid CFBundleExecutable")
    return app / "Contents" / "MacOS" / executable_name, info


def check_bundle(results: Results, app: Path, *, require_universal: bool) -> None:
    if not app.is_dir() or app.suffix != ".app":
        results.fail("bundle.exists", f"application bundle not found: {app}")
        return
    results.pass_("bundle.exists", str(app))

    try:
        executable, info = read_bundle_metadata(app)
    except PreflightError as exc:
        results.fail("bundle.metadata", str(exc))
        return

    required_metadata = {
        "CFBundleIdentifier": lambda value: isinstance(value, str) and "." in value,
        "CFBundleName": lambda value: isinstance(value, str) and bool(value),
        "CFBundleExecutable": lambda value: isinstance(value, str) and bool(value),
        "CFBundlePackageType": lambda value: value == "APPL",
        "CFBundleShortVersionString": lambda value: isinstance(value, str) and bool(value),
        "CFBundleVersion": lambda value: isinstance(value, str) and bool(value),
        "LSMinimumSystemVersion": lambda value: isinstance(value, str) and bool(value),
    }
    invalid = [key for key, predicate in required_metadata.items() if not predicate(info.get(key))]
    if invalid:
        results.fail("bundle.metadata", "invalid or missing keys: " + ", ".join(invalid))
    else:
        results.pass_(
            "bundle.metadata",
            f"identifier={info['CFBundleIdentifier']} version={info['CFBundleShortVersionString']} build={info['CFBundleVersion']}",
        )

    icon_name = info.get("CFBundleIconFile")
    icon_candidates = []
    if isinstance(icon_name, str) and icon_name:
        icon_candidates.append(app / "Contents" / "Resources" / icon_name)
        if not icon_name.endswith(".icns"):
            icon_candidates.append(app / "Contents" / "Resources" / f"{icon_name}.icns")
    if icon_candidates and any(path.is_file() for path in icon_candidates):
        results.pass_("bundle.icon", "declared icon exists")
    else:
        results.fail("bundle.icon", "CFBundleIconFile is missing or does not resolve to an .icns resource")

    if executable.is_file() and os.access(executable, os.X_OK):
        results.pass_("bundle.executable", str(executable.relative_to(app)))
    else:
        results.fail("bundle.executable", "declared executable is missing or not executable")
        return

    lipo = shutil.which("lipo")
    if lipo is None:
        if require_universal:
            results.fail("bundle.universal2", "lipo is unavailable on this host")
        else:
            results.skip("bundle.universal2", "lipo is unavailable on this host")
        return
    process = _run([lipo, "-archs", str(executable)])
    if process.returncode != 0:
        results.fail("bundle.universal2", _command_detail(process))
        return
    architectures = set(process.stdout.split())
    required = {"arm64", "x86_64"}
    if required.issubset(architectures):
        results.pass_("bundle.universal2", " ".join(sorted(architectures)))
    elif require_universal:
        results.fail("bundle.universal2", "architectures=" + " ".join(sorted(architectures)))
    else:
        results.skip("bundle.universal2", "architectures=" + " ".join(sorted(architectures)))


def check_codesign(results: Results, app: Path, *, required: bool) -> None:
    codesign = shutil.which("codesign")
    if codesign is None:
        if required:
            results.fail("signing.codesign_verify", "codesign is unavailable")
        else:
            results.skip("signing.codesign_verify", "codesign is unavailable")
        return

    verify = _run([codesign, "--verify", "--deep", "--strict", "--verbose=2", str(app)])
    if verify.returncode == 0:
        results.pass_("signing.codesign_verify", _command_detail(verify))
    elif required:
        results.fail("signing.codesign_verify", _command_detail(verify))
    else:
        results.skip("signing.codesign_verify", _command_detail(verify))

    display = _run([codesign, "--display", "--verbose=4", str(app)])
    combined = "\n".join((display.stdout, display.stderr))
    if display.returncode == 0 and "runtime" in combined.lower():
        results.pass_("signing.hardened_runtime", "runtime flag present")
    elif required:
        results.fail("signing.hardened_runtime", "runtime flag was not found")
    else:
        results.skip("signing.hardened_runtime", "signature metadata unavailable or unsigned")

    entitlements = _run([codesign, "--display", "--entitlements", ":-", str(app)])
    if entitlements.returncode == 0:
        body = entitlements.stdout.strip() or entitlements.stderr.strip()
        forbidden = (
            "com.apple.security.get-task-allow",
            "com.apple.security.cs.allow-jit",
            "com.apple.security.cs.disable-library-validation",
            "com.apple.security.cs.allow-unsigned-executable-memory",
        )
        present = [key for key in forbidden if key in body]
        if present:
            results.fail("signing.entitlements", "unexpected sensitive entitlements: " + ", ".join(present))
        else:
            results.pass_("signing.entitlements", "no forbidden release entitlements found")
    elif required:
        results.fail("signing.entitlements", _command_detail(entitlements))
    else:
        results.skip("signing.entitlements", "entitlements are unavailable for unsigned candidate")


def check_notarization(results: Results, app: Path, *, required: bool) -> None:
    xcrun = shutil.which("xcrun")
    if xcrun is None:
        if required:
            results.fail("notarization.stapler", "xcrun is unavailable")
        else:
            results.skip("notarization.stapler", "xcrun is unavailable")
        return
    stapler = _run([xcrun, "stapler", "validate", str(app)])
    if stapler.returncode == 0:
        results.pass_("notarization.stapler", _command_detail(stapler))
    elif required:
        results.fail("notarization.stapler", _command_detail(stapler))
    else:
        results.skip("notarization.stapler", _command_detail(stapler))

    spctl = shutil.which("spctl")
    if spctl is None:
        if required:
            results.fail("notarization.gatekeeper", "spctl is unavailable")
        else:
            results.skip("notarization.gatekeeper", "spctl is unavailable")
        return
    assess = _run([spctl, "--assess", "--type", "execute", "--verbose=4", str(app)])
    if assess.returncode == 0:
        results.pass_("notarization.gatekeeper", _command_detail(assess))
    elif required:
        results.fail("notarization.gatekeeper", _command_detail(assess))
    else:
        results.skip("notarization.gatekeeper", _command_detail(assess))


def check_artifacts(results: Results, paths: Sequence[Path], *, required: bool) -> dict[str, str]:
    hashes: dict[str, str] = {}
    for path in paths:
        check_name = f"artifact.{path.name}"
        if not path.is_file():
            if required:
                results.fail(check_name, f"missing artifact: {path}")
            else:
                results.skip(check_name, f"missing artifact: {path}")
            continue
        digest = sha256_file(path)
        hashes[path.name] = digest
        results.pass_(check_name, f"size={path.stat().st_size} sha256={digest}")
    return hashes


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--commit", required=True)
    parser.add_argument("--app", type=Path)
    parser.add_argument("--artifact", type=Path, action="append", default=[])
    parser.add_argument("--require-clean", action="store_true")
    parser.add_argument("--require-bundle", action="store_true")
    parser.add_argument("--require-universal2", action="store_true")
    parser.add_argument("--require-signing", action="store_true")
    parser.add_argument("--require-notarization", action="store_true")
    parser.add_argument("--require-artifacts", action="store_true")
    parser.add_argument("--json-output", type=Path)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    repo = args.repo.resolve()
    app = args.app.resolve() if args.app else None
    artifacts = [path.resolve() for path in args.artifact]
    results = Results()

    try:
        check_repository(results, repo, args.commit, args.require_clean)
        if app is None:
            if args.require_bundle or args.require_signing or args.require_notarization:
                results.fail("bundle.exists", "--app is required by the selected release gate")
            else:
                results.skip("bundle.exists", "no --app candidate supplied")
        else:
            check_bundle(results, app, require_universal=args.require_universal2)
            check_codesign(results, app, required=args.require_signing)
            check_notarization(results, app, required=args.require_notarization)
        hashes = check_artifacts(results, artifacts, required=args.require_artifacts)
    except PreflightError as exc:
        print(f"preflight error: {_single_line(str(exc))}", file=sys.stderr)
        return 2

    payload = {
        "schema_version": 1,
        "gate": "p3-release-preflight",
        "repository": "yhan-sun/ClipType",
        "commit": args.commit,
        "app": str(app) if app else None,
        "artifact_sha256": hashes,
        "checks": [asdict(item) for item in results.items],
        "passed": not results.failed,
    }
    if args.json_output:
        args.json_output.parent.mkdir(parents=True, exist_ok=True)
        args.json_output.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    for item in results.items:
        print(f"{item.status.upper():7} {item.check}: {item.detail}")
    print("PASS" if payload["passed"] else "FAIL")
    return 0 if payload["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
