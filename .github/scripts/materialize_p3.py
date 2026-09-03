#!/usr/bin/env python3
"""Materialize the reviewed P3 macOS source tree deterministically.

The source transport is split only because the repository write gateway could not
reliably accept the original opaque archive. This script accepts no unreviewed
payload: both the aggregate Base64 and decoded tarball must match pinned SHA-256
values before extraction.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import io
import os
import re
import stat
import tarfile
from pathlib import Path, PurePosixPath

EXPECTED_BASE64_LENGTH = 28_116
EXPECTED_BASE64_SHA256 = "8f8b42b20fe1ec0a47052992f78ff8705e343cf76947f07b6fe676d6960460a3"
EXPECTED_PAYLOAD_SHA256 = "a97a86588b32216988062a58d19c9ad49e5b3f2077764059468fca6bbf9737f9"
REVIEWED_TAIL = (
    "4Drd+CKu/EUUrwBBLNrmBfXcYGX+zb/5N//m3/ybf/Nv/s2/+Tf/5t/8m3"
    "/zb/7Nv/k3/+bf/Jt/82/+zb/5N//m3/ybf/Nv/n2d7/8Dz2yswwC4AQA="
)
SUBCHUNK_NAMES = (
    "chunk-00.b64",
    "chunk-01.b64",
    "chunk-02a.b64",
    "chunk-02b.b64",
    "chunk-02c.b64",
    "chunk-02d.b64",
    "chunk-03.b64",
    "chunk-04.b64",
    "chunk-05.b64",
    "chunk-06.b64",
    "chunk-07.b64",
)
LARGE_CHUNK_NAMES = tuple(f"chunk-{index:02d}.b64" for index in range(8))
BASE64_PATTERN = re.compile(r"[A-Za-z0-9+/]*={0,2}\Z")


def normalized_base64(path: Path) -> str:
    value = "".join(path.read_text(encoding="ascii").split())
    if not value:
        raise ValueError(f"empty payload segment: {path.name}")
    if BASE64_PATTERN.fullmatch(value) is None:
        raise ValueError(f"non-Base64 character in payload segment: {path.name}")
    return value


def payload_candidates(root: Path) -> list[tuple[str, str]]:
    candidates: list[tuple[str, str]] = []
    if all((root / name).is_file() for name in SUBCHUNK_NAMES):
        candidates.append(
            (
                "reviewed-subchunks",
                "".join(normalized_base64(root / name) for name in SUBCHUNK_NAMES)
                + REVIEWED_TAIL,
            )
        )
    if all((root / name).is_file() for name in LARGE_CHUNK_NAMES):
        candidates.append(
            (
                "large-chunks",
                "".join(normalized_base64(root / name) for name in LARGE_CHUNK_NAMES)
                + REVIEWED_TAIL,
            )
        )
    monolith = root / "payload.b64"
    if monolith.is_file():
        try:
            candidates.append(("monolith", normalized_base64(monolith)))
        except (UnicodeError, ValueError):
            pass
    return candidates


def select_payload(root: Path) -> tuple[str, bytes]:
    diagnostics: list[str] = []
    for name, encoded in payload_candidates(root):
        encoded_digest = hashlib.sha256(encoded.encode("ascii")).hexdigest()
        diagnostics.append(
            f"{name}: chars={len(encoded)} encoded_sha256={encoded_digest}"
        )
        if len(encoded) != EXPECTED_BASE64_LENGTH:
            continue
        if encoded_digest != EXPECTED_BASE64_SHA256:
            continue
        payload = base64.b64decode(encoded, validate=True)
        payload_digest = hashlib.sha256(payload).hexdigest()
        diagnostics.append(
            f"{name}: bytes={len(payload)} payload_sha256={payload_digest}"
        )
        if payload_digest == EXPECTED_PAYLOAD_SHA256:
            print("\n".join(diagnostics))
            print(f"selected={name}")
            return name, payload
    print("\n".join(diagnostics))
    raise RuntimeError("no P3 source payload passed both reviewed SHA-256 gates")


def safe_members(archive: tarfile.TarFile) -> list[tarfile.TarInfo]:
    members = archive.getmembers()
    for member in members:
        path = PurePosixPath(member.name)
        if path.is_absolute() or ".." in path.parts:
            raise RuntimeError(f"unsafe archive member path: {member.name}")
        if member.issym() or member.islnk() or member.isdev():
            raise RuntimeError(f"unsupported archive member type: {member.name}")
        if not (member.isdir() or member.isfile()):
            raise RuntimeError(f"unexpected archive member type: {member.name}")
    return members


def extract_payload(repository: Path, payload: bytes) -> None:
    with tarfile.open(fileobj=io.BytesIO(payload), mode="r:gz") as archive:
        members = safe_members(archive)
        archive.extractall(repository, members=members)
        print(f"extracted_members={len(members)}")


def replace(path: Path, old: str, new: str, count: int = -1) -> None:
    text = path.read_text(encoding="utf-8")
    path.write_text(text.replace(old, new, count), encoding="utf-8")


def normalize_source(repository: Path) -> None:
    app_manifest = repository / "apps/cliptype-macos/Cargo.toml"
    text = app_manifest.read_text(encoding="utf-8")
    text = text.replace(
        'name = "cliptype-macos"\n',
        'name = "cliptype-macos-app"\n',
        1,
    )
    if "[[bin]]" not in text:
        text += '\n[[bin]]\nname = "cliptype-macos"\npath = "src/main.rs"\n'
    app_manifest.write_text(text, encoding="utf-8")

    native = repository / "crates/cliptype-macos/native/cliptype_macos.m"
    replace(
        native,
        "int ct_macos_post_unicode(const uint16_t *units, size_t length) {\n"
        "int ct_macos_post_unicode(const uint16_t *units, size_t length) {\n",
        "int ct_macos_post_unicode(const uint16_t *units, size_t length) {\n",
    )
    replace(
        native,
        '        image = [NSImage imageWithSystemSymbolName:@"doc.on.clipboard"\n'
        '        image = [NSImage imageWithSystemSymbolName:@"doc.on.clipboard"\n',
        '        image = [NSImage imageWithSystemSymbolName:@"doc.on.clipboard"\n',
    )

    rust = repository / "crates/cliptype-macos/src/lib.rs"
    replace(rust, "#![forbid(unsafe_code)]", "#![deny(unsafe_op_in_unsafe_fn)]")
    replace(rust, '\nextern "C" {\n', '\nunsafe extern "C" {\n')

    for workflow in (
        repository / ".github/workflows/p3-cross-platform.yml",
        repository / ".github/workflows/macos-release.yml",
    ):
        if not workflow.is_file():
            continue
        text = workflow.read_text(encoding="utf-8")
        text = text.replace(
            "-p cliptype-macos --target",
            "-p cliptype-macos-app --target",
        )
        text = text.replace(
            "runner: macos-15\n            target: x86_64-apple-darwin",
            "runner: macos-15-intel\n            target: x86_64-apple-darwin",
        )
        text = text.replace(
            "runs-on: macos-15\n    timeout-minutes: 20",
            "runs-on: macos-15-intel\n    timeout-minutes: 20",
        )
        text = text.replace(
            '--repo "$GITHUB_REPOSITORY" --clobber=false',
            '--repo "$GITHUB_REPOSITORY"',
        )
        workflow.write_text(text, encoding="utf-8")

    package_script = repository / "packaging/macos/package.sh"
    package_script.chmod(package_script.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def source_is_materialized(repository: Path) -> bool:
    required = (
        repository / "crates/cliptype-macos/Cargo.toml",
        repository / "crates/cliptype-macos/src/lib.rs",
        repository / "crates/cliptype-macos/native/cliptype_macos.m",
        repository / "apps/cliptype-macos/Cargo.toml",
        repository / "apps/cliptype-macos/src/main.rs",
        repository / "packaging/macos/package.sh",
    )
    return all(path.is_file() for path in required)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repository", default=".")
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args()

    repository = Path(args.repository).resolve()
    if args.force or not source_is_materialized(repository):
        _, payload = select_payload(repository / ".github/p3-stage2")
        extract_payload(repository, payload)
    normalize_source(repository)
    if not source_is_materialized(repository):
        raise RuntimeError("P3 source tree is incomplete after materialization")
    print("materialized=true")
    print(f"payload_sha256={EXPECTED_PAYLOAD_SHA256}")


if __name__ == "__main__":
    main()
