#!/usr/bin/env python3
"""Create and verify content-blind P3 real-host validation evidence.

The gate intentionally stores only test metadata and references to external evidence.
Clipboard contents, typed sample text, screenshots, credentials, and log bodies must not
be embedded in manifests.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform as platform_module
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any, Mapping, Sequence
from urllib.parse import urlparse

SCHEMA_VERSION = 1
GATE_NAME = "p3-real-host"
REPOSITORY = "yhan-sun/ClipType"
ALLOWED_RESULTS = {"pending", "pass", "fail", "blocked", "not-applicable"}
TERMINAL_RESULTS = ALLOWED_RESULTS - {"pending"}
PLATFORMS = {
    "windows-x86_64",
    "macos-arm64",
    "macos-x86_64",
    "release-macos",
}
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
CHECK_ID_RE = re.compile(r"^[a-z0-9][a-z0-9._-]{2,95}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
RFC3339_RE = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$")


class GateError(ValueError):
    """Raised when evidence is unsafe, incomplete, or inconsistent."""


@dataclass(frozen=True)
class CheckDefinition:
    check_id: str
    title: str
    platforms: tuple[str, ...]
    required: bool
    instructions: str


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def canonical_json(value: Mapping[str, Any]) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n").encode(
        "utf-8"
    )


def manifest_digest(manifest: Mapping[str, Any]) -> str:
    unsigned = dict(manifest)
    unsigned.pop("manifest_sha256", None)
    return hashlib.sha256(canonical_json(unsigned)).hexdigest()


def atomic_write_json(path: Path, value: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    with tempfile.NamedTemporaryFile(
        mode="w", encoding="utf-8", dir=path.parent, prefix=f".{path.name}.", delete=False
    ) as stream:
        stream.write(payload)
        stream.flush()
        os.fsync(stream.fileno())
        temporary = Path(stream.name)
    os.replace(temporary, path)


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise GateError(f"file does not exist: {path}") from exc
    except json.JSONDecodeError as exc:
        raise GateError(f"invalid JSON in {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise GateError(f"top-level JSON value must be an object: {path}")
    return value


def clean_text(value: str, *, field: str, limit: int = 512, allow_empty: bool = True) -> str:
    if not isinstance(value, str):
        raise GateError(f"{field} must be a string")
    if not allow_empty and not value.strip():
        raise GateError(f"{field} must not be empty")
    if len(value) > limit:
        raise GateError(f"{field} exceeds {limit} characters")
    if any(ord(character) < 32 and character not in "\t" for character in value):
        raise GateError(f"{field} contains control characters")
    if "\n" in value or "\r" in value:
        raise GateError(f"{field} must be a single line")
    return value


def validate_sha(value: str, *, field: str = "commit") -> str:
    if not SHA_RE.fullmatch(value):
        raise GateError(f"{field} must be a lowercase 40-character Git SHA")
    return value


def validate_rfc3339(value: str, *, field: str) -> str:
    if not isinstance(value, str) or not RFC3339_RE.fullmatch(value):
        raise GateError(f"{field} must be an RFC3339 UTC timestamp ending in Z")
    try:
        datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise GateError(f"{field} is not a valid timestamp") from exc
    return value


def validate_relative_path(value: str, *, field: str) -> str:
    clean_text(value, field=field, limit=300, allow_empty=False)
    normalized = value.replace("\\", "/")
    path = PurePosixPath(normalized)
    if path.is_absolute() or ".." in path.parts or "." in path.parts:
        raise GateError(f"{field} must be a normalized relative path without traversal")
    if not path.parts:
        raise GateError(f"{field} must not be empty")
    return path.as_posix()


def parse_evidence(value: str) -> dict[str, str]:
    if "=" not in value:
        raise GateError("evidence must use KIND=VALUE")
    kind, raw = value.split("=", 1)
    kind = clean_text(kind, field="evidence kind", limit=32, allow_empty=False)
    raw = clean_text(raw, field="evidence value", limit=512, allow_empty=False)
    if kind == "path":
        raw = validate_relative_path(raw, field="evidence path")
    elif kind == "url":
        parsed = urlparse(raw)
        if parsed.scheme != "https" or not parsed.netloc or parsed.username or parsed.password:
            raise GateError("evidence URL must be an HTTPS URL without embedded credentials")
    elif kind == "sha256":
        if not SHA256_RE.fullmatch(raw):
            raise GateError("sha256 evidence must be a lowercase 64-character digest")
    elif kind in {"run", "screenshot", "command"}:
        pass
    else:
        raise GateError(f"unsupported evidence kind: {kind}")
    return {"kind": kind, "value": raw}


def load_catalog(path: Path) -> list[CheckDefinition]:
    raw = load_json(path)
    if raw.get("schema_version") != SCHEMA_VERSION:
        raise GateError("catalog schema_version is unsupported")
    if raw.get("gate") != GATE_NAME:
        raise GateError("catalog gate name is incorrect")
    checks = raw.get("checks")
    if not isinstance(checks, list) or not checks:
        raise GateError("catalog checks must be a non-empty list")

    definitions: list[CheckDefinition] = []
    seen: set[str] = set()
    for index, item in enumerate(checks):
        if not isinstance(item, dict):
            raise GateError(f"catalog check {index} must be an object")
        check_id = item.get("id")
        if not isinstance(check_id, str) or not CHECK_ID_RE.fullmatch(check_id):
            raise GateError(f"catalog check {index} has an invalid id")
        if check_id in seen:
            raise GateError(f"duplicate catalog check id: {check_id}")
        seen.add(check_id)
        title = clean_text(item.get("title", ""), field=f"{check_id}.title", limit=160, allow_empty=False)
        instructions = clean_text(
            item.get("instructions", ""),
            field=f"{check_id}.instructions",
            limit=500,
            allow_empty=False,
        )
        raw_platforms = item.get("platforms")
        if not isinstance(raw_platforms, list) or not raw_platforms:
            raise GateError(f"{check_id}.platforms must be a non-empty list")
        platforms: list[str] = []
        for platform_name in raw_platforms:
            if platform_name not in PLATFORMS:
                raise GateError(f"{check_id} references unsupported platform {platform_name!r}")
            if platform_name in platforms:
                raise GateError(f"{check_id} repeats platform {platform_name!r}")
            platforms.append(platform_name)
        required = item.get("required", True)
        if not isinstance(required, bool):
            raise GateError(f"{check_id}.required must be boolean")
        definitions.append(
            CheckDefinition(
                check_id=check_id,
                title=title,
                platforms=tuple(platforms),
                required=required,
                instructions=instructions,
            )
        )
    return definitions


def host_fingerprint() -> dict[str, str]:
    return {
        "system": clean_text(platform_module.system(), field="system", limit=80),
        "release": clean_text(platform_module.release(), field="release", limit=160),
        "machine": clean_text(platform_module.machine(), field="machine", limit=80),
        "python": clean_text(platform_module.python_version(), field="python", limit=40),
    }


def create_manifest(
    *, catalog_path: Path, platform_name: str, commit: str, run_id: str, operator: str
) -> dict[str, Any]:
    if platform_name not in PLATFORMS:
        raise GateError(f"unsupported platform: {platform_name}")
    validate_sha(commit)
    run_id = clean_text(run_id, field="run_id", limit=96, allow_empty=False)
    operator = clean_text(operator, field="operator", limit=96)
    definitions = load_catalog(catalog_path)
    checks = [
        {
            "id": definition.check_id,
            "required": definition.required,
            "result": "pending",
            "evidence": [],
            "note": "",
            "recorded_at": None,
        }
        for definition in definitions
        if platform_name in definition.platforms
    ]
    if not checks:
        raise GateError(f"catalog has no checks for {platform_name}")
    manifest: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "gate": GATE_NAME,
        "repository": REPOSITORY,
        "commit": commit,
        "platform": platform_name,
        "run_id": run_id,
        "operator": operator,
        "host": host_fingerprint(),
        "started_at": utc_now(),
        "completed_at": None,
        "privacy": {
            "clipboard_content_collected": False,
            "typed_content_collected": False,
            "credentials_collected": False,
            "log_bodies_embedded": False,
        },
        "checks": checks,
    }
    manifest["manifest_sha256"] = manifest_digest(manifest)
    return manifest


def _validate_host(value: Any) -> None:
    if not isinstance(value, dict):
        raise GateError("host must be an object")
    if set(value) != {"system", "release", "machine", "python"}:
        raise GateError("host contains unexpected or missing fields")
    for key, raw in value.items():
        clean_text(raw, field=f"host.{key}", limit=160)


def _validate_privacy(value: Any) -> None:
    expected = {
        "clipboard_content_collected": False,
        "typed_content_collected": False,
        "credentials_collected": False,
        "log_bodies_embedded": False,
    }
    if value != expected:
        raise GateError("privacy declaration must remain content-blind and all false")


def validate_manifest(
    manifest: Mapping[str, Any],
    *,
    catalog_path: Path,
    expected_commit: str | None = None,
    require_complete: bool = False,
    verify_digest: bool = True,
) -> list[str]:
    required_top_level = {
        "schema_version",
        "gate",
        "repository",
        "commit",
        "platform",
        "run_id",
        "operator",
        "host",
        "started_at",
        "completed_at",
        "privacy",
        "checks",
        "manifest_sha256",
    }
    if set(manifest) != required_top_level:
        missing = sorted(required_top_level - set(manifest))
        extra = sorted(set(manifest) - required_top_level)
        raise GateError(f"manifest fields differ from schema; missing={missing}, extra={extra}")
    if manifest["schema_version"] != SCHEMA_VERSION:
        raise GateError("manifest schema_version is unsupported")
    if manifest["gate"] != GATE_NAME or manifest["repository"] != REPOSITORY:
        raise GateError("manifest does not belong to the ClipType P3 gate")
    commit = validate_sha(manifest["commit"])
    if expected_commit is not None and commit != validate_sha(expected_commit, field="expected_commit"):
        raise GateError(f"manifest commit {commit} does not match expected commit {expected_commit}")
    platform_name = manifest["platform"]
    if platform_name not in PLATFORMS:
        raise GateError("manifest platform is unsupported")
    clean_text(manifest["run_id"], field="run_id", limit=96, allow_empty=False)
    clean_text(manifest["operator"], field="operator", limit=96)
    _validate_host(manifest["host"])
    validate_rfc3339(manifest["started_at"], field="started_at")
    completed_at = manifest["completed_at"]
    if completed_at is not None:
        validate_rfc3339(completed_at, field="completed_at")
    _validate_privacy(manifest["privacy"])

    definitions = {
        item.check_id: item
        for item in load_catalog(catalog_path)
        if platform_name in item.platforms
    }
    checks = manifest["checks"]
    if not isinstance(checks, list):
        raise GateError("checks must be a list")
    observed_ids: set[str] = set()
    pending_or_failed: list[str] = []
    for index, check in enumerate(checks):
        if not isinstance(check, dict):
            raise GateError(f"check {index} must be an object")
        if set(check) != {"id", "required", "result", "evidence", "note", "recorded_at"}:
            raise GateError(f"check {index} contains unexpected or missing fields")
        check_id = check["id"]
        if check_id not in definitions:
            raise GateError(f"unknown or wrong-platform check id: {check_id}")
        if check_id in observed_ids:
            raise GateError(f"duplicate manifest check id: {check_id}")
        observed_ids.add(check_id)
        definition = definitions[check_id]
        if check["required"] is not definition.required:
            raise GateError(f"{check_id}.required differs from the catalog")
        result = check["result"]
        if result not in ALLOWED_RESULTS:
            raise GateError(f"{check_id} has unsupported result {result!r}")
        note = clean_text(check["note"], field=f"{check_id}.note", limit=512)
        if "clipboard=" in note.lower() or "typed_text=" in note.lower():
            raise GateError(f"{check_id}.note appears to contain prohibited inline content")
        evidence = check["evidence"]
        if not isinstance(evidence, list):
            raise GateError(f"{check_id}.evidence must be a list")
        if len(evidence) > 12:
            raise GateError(f"{check_id}.evidence has too many entries")
        for item in evidence:
            if not isinstance(item, dict) or set(item) != {"kind", "value"}:
                raise GateError(f"{check_id} has malformed evidence")
            parsed = parse_evidence(f"{item['kind']}={item['value']}")
            if parsed != item:
                raise GateError(f"{check_id} evidence is not normalized")
        recorded_at = check["recorded_at"]
        if result == "pending":
            if recorded_at is not None or evidence or note:
                raise GateError(f"pending check {check_id} must not have evidence, note, or timestamp")
        else:
            if recorded_at is None:
                raise GateError(f"terminal check {check_id} must have recorded_at")
            validate_rfc3339(recorded_at, field=f"{check_id}.recorded_at")
            if result == "pass" and not evidence:
                raise GateError(f"passing check {check_id} must reference evidence")
        if definition.required and result != "pass":
            pending_or_failed.append(f"{check_id}:{result}")

    if observed_ids != set(definitions):
        missing = sorted(set(definitions) - observed_ids)
        raise GateError(f"manifest is missing catalog checks: {missing}")
    is_complete = not pending_or_failed
    if (completed_at is not None) != is_complete:
        raise GateError("completed_at must be set if and only if every required check passed")
    if require_complete and not is_complete:
        raise GateError("required checks are not complete: " + ", ".join(pending_or_failed))
    digest = manifest["manifest_sha256"]
    if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
        raise GateError("manifest_sha256 is malformed")
    if verify_digest and digest != manifest_digest(manifest):
        raise GateError("manifest_sha256 does not match the manifest contents")
    return pending_or_failed


def record_result(
    manifest: dict[str, Any],
    *,
    catalog_path: Path,
    check_id: str,
    result: str,
    evidence: Sequence[dict[str, str]],
    note: str,
) -> None:
    validate_manifest(manifest, catalog_path=catalog_path)
    if result not in TERMINAL_RESULTS:
        raise GateError("record result must be pass, fail, blocked, or not-applicable")
    note = clean_text(note, field="note", limit=512)
    matching = [check for check in manifest["checks"] if check["id"] == check_id]
    if len(matching) != 1:
        raise GateError(f"manifest does not contain exactly one check {check_id!r}")
    check = matching[0]
    if check["required"] and result == "not-applicable":
        raise GateError("required checks cannot be marked not-applicable")
    if result == "pass" and not evidence:
        raise GateError("passing a check requires at least one evidence reference")
    check["result"] = result
    check["evidence"] = list(evidence)
    check["note"] = note
    check["recorded_at"] = utc_now()
    all_required_pass = all(
        item["result"] == "pass" for item in manifest["checks"] if item["required"]
    )
    manifest["completed_at"] = utc_now() if all_required_pass else None
    manifest["manifest_sha256"] = manifest_digest(manifest)
    validate_manifest(manifest, catalog_path=catalog_path)


def reset_result(manifest: dict[str, Any], *, catalog_path: Path, check_id: str) -> None:
    validate_manifest(manifest, catalog_path=catalog_path)
    matching = [check for check in manifest["checks"] if check["id"] == check_id]
    if len(matching) != 1:
        raise GateError(f"manifest does not contain exactly one check {check_id!r}")
    check = matching[0]
    check.update({"result": "pending", "evidence": [], "note": "", "recorded_at": None})
    manifest["completed_at"] = None
    manifest["manifest_sha256"] = manifest_digest(manifest)
    validate_manifest(manifest, catalog_path=catalog_path)


def read_git_head(repo: Path) -> str:
    process = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repo,
        text=True,
        capture_output=True,
        check=False,
    )
    if process.returncode != 0:
        raise GateError(f"unable to read Git HEAD from {repo}: {process.stderr.strip()}")
    return validate_sha(process.stdout.strip(), field="git HEAD")


def verify_gate_set(
    manifests: Sequence[Mapping[str, Any]],
    *,
    catalog_path: Path,
    expected_commit: str,
    require_release: bool,
) -> None:
    if not manifests:
        raise GateError("at least one manifest is required")
    platforms: set[str] = set()
    run_ids: set[str] = set()
    for manifest in manifests:
        validate_manifest(
            manifest,
            catalog_path=catalog_path,
            expected_commit=expected_commit,
            require_complete=True,
        )
        platform_name = manifest["platform"]
        if platform_name in platforms:
            raise GateError(f"duplicate platform manifest: {platform_name}")
        platforms.add(platform_name)
        run_id = manifest["run_id"]
        if run_id in run_ids:
            raise GateError(f"duplicate run_id across manifests: {run_id}")
        run_ids.add(run_id)
    required = {"windows-x86_64", "macos-arm64", "macos-x86_64"}
    if require_release:
        required.add("release-macos")
    missing = sorted(required - platforms)
    if missing:
        raise GateError(f"gate set is missing required platform manifests: {missing}")


def render_report(manifests: Sequence[Mapping[str, Any]], catalog_path: Path) -> str:
    definitions = {item.check_id: item for item in load_catalog(catalog_path)}
    lines = [
        "# ClipType P3 real-host validation evidence",
        "",
        "> This report contains test metadata only. It must not contain clipboard contents, typed samples, credentials, or log bodies.",
        "",
    ]
    for manifest in sorted(manifests, key=lambda value: value["platform"]):
        pending = validate_manifest(manifest, catalog_path=catalog_path)
        status = "PASS" if not pending else "INCOMPLETE"
        lines.extend(
            [
                f"## {manifest['platform']} — {status}",
                "",
                f"- Commit: `{manifest['commit']}`",
                f"- Run ID: `{manifest['run_id']}`",
                f"- Started: `{manifest['started_at']}`",
                f"- Completed: `{manifest['completed_at'] or 'not complete'}`",
                f"- Manifest SHA-256: `{manifest['manifest_sha256']}`",
                "",
                "| Check | Required | Result | Evidence references | Note |",
                "|---|---:|---|---:|---|",
            ]
        )
        for check in manifest["checks"]:
            title = definitions[check["id"]].title.replace("|", "\\|")
            note = check["note"].replace("|", "\\|")
            lines.append(
                f"| `{check['id']}` — {title} | {'yes' if check['required'] else 'no'} | "
                f"{check['result']} | {len(check['evidence'])} | {note} |"
            )
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def default_catalog_path() -> Path:
    return Path(__file__).resolve().parents[1] / "qa" / "p3-real-host-checks.json"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", type=Path, default=default_catalog_path())
    subparsers = parser.add_subparsers(dest="command", required=True)

    catalog = subparsers.add_parser("catalog", help="validate and list the check catalog")
    catalog.add_argument("--platform", choices=sorted(PLATFORMS))

    initialize = subparsers.add_parser("init", help="create a content-blind evidence manifest")
    initialize.add_argument("--platform", required=True, choices=sorted(PLATFORMS))
    initialize.add_argument("--commit", required=True)
    initialize.add_argument("--run-id", required=True)
    initialize.add_argument("--operator", default="")
    initialize.add_argument("--output", type=Path, required=True)
    initialize.add_argument("--force", action="store_true")

    record = subparsers.add_parser("record", help="record one check result")
    record.add_argument("--manifest", type=Path, required=True)
    record.add_argument("--check", required=True)
    record.add_argument("--result", required=True, choices=sorted(TERMINAL_RESULTS))
    record.add_argument("--evidence", action="append", default=[])
    record.add_argument("--note", default="")

    reset = subparsers.add_parser("reset", help="reset one check to pending")
    reset.add_argument("--manifest", type=Path, required=True)
    reset.add_argument("--check", required=True)

    verify = subparsers.add_parser("verify", help="validate one or more manifests")
    verify.add_argument("manifests", nargs="+", type=Path)
    verify.add_argument("--expected-commit")
    verify.add_argument("--require-complete", action="store_true")

    gate_set = subparsers.add_parser("verify-set", help="validate the full multi-host gate")
    gate_set.add_argument("manifests", nargs="+", type=Path)
    gate_set.add_argument("--expected-commit")
    gate_set.add_argument("--repo", type=Path, default=Path.cwd())
    gate_set.add_argument("--require-release", action="store_true")

    report = subparsers.add_parser("report", help="render manifests as Markdown")
    report.add_argument("manifests", nargs="+", type=Path)
    report.add_argument("--output", type=Path)

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    catalog_path = args.catalog.resolve()
    try:
        if args.command == "catalog":
            definitions = load_catalog(catalog_path)
            selected = [
                definition
                for definition in definitions
                if args.platform is None or args.platform in definition.platforms
            ]
            for definition in selected:
                print(
                    f"{definition.check_id}\t{'required' if definition.required else 'optional'}\t"
                    f"{','.join(definition.platforms)}\t{definition.title}"
                )
            return 0

        if args.command == "init":
            if args.output.exists() and not args.force:
                raise GateError(f"output already exists: {args.output}; use --force to replace it")
            manifest = create_manifest(
                catalog_path=catalog_path,
                platform_name=args.platform,
                commit=args.commit,
                run_id=args.run_id,
                operator=args.operator,
            )
            atomic_write_json(args.output, manifest)
            print(f"created {args.output} sha256={manifest['manifest_sha256']}")
            return 0

        if args.command == "record":
            manifest = load_json(args.manifest)
            evidence = [parse_evidence(value) for value in args.evidence]
            record_result(
                manifest,
                catalog_path=catalog_path,
                check_id=args.check,
                result=args.result,
                evidence=evidence,
                note=args.note,
            )
            atomic_write_json(args.manifest, manifest)
            print(f"recorded {args.check}={args.result} in {args.manifest}")
            return 0

        if args.command == "reset":
            manifest = load_json(args.manifest)
            reset_result(manifest, catalog_path=catalog_path, check_id=args.check)
            atomic_write_json(args.manifest, manifest)
            print(f"reset {args.check} in {args.manifest}")
            return 0

        if args.command == "verify":
            for path in args.manifests:
                manifest = load_json(path)
                pending = validate_manifest(
                    manifest,
                    catalog_path=catalog_path,
                    expected_commit=args.expected_commit,
                    require_complete=args.require_complete,
                )
                status = "complete" if not pending else f"incomplete ({len(pending)} required checks)"
                print(f"{path}: valid, {status}, sha256={manifest['manifest_sha256']}")
            return 0

        if args.command == "verify-set":
            expected_commit = args.expected_commit or read_git_head(args.repo.resolve())
            manifests = [load_json(path) for path in args.manifests]
            verify_gate_set(
                manifests,
                catalog_path=catalog_path,
                expected_commit=expected_commit,
                require_release=args.require_release,
            )
            print(f"P3 real-host gate passed for {expected_commit}")
            return 0

        if args.command == "report":
            manifests = [load_json(path) for path in args.manifests]
            rendered = render_report(manifests, catalog_path)
            if args.output is None:
                sys.stdout.write(rendered)
            else:
                args.output.parent.mkdir(parents=True, exist_ok=True)
                args.output.write_text(rendered, encoding="utf-8")
                print(f"wrote {args.output}")
            return 0

        raise AssertionError(f"unhandled command {args.command}")
    except GateError as exc:
        parser.exit(2, f"error: {exc}\n")


if __name__ == "__main__":
    raise SystemExit(main())
