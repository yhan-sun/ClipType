#!/usr/bin/env python3
"""Fail closed when P3 evidence manifests appear to contain sensitive data.

The linter never prints a rejected value. Diagnostics name only the manifest and field
location so CI logs cannot become a secondary disclosure channel.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path, PurePosixPath
from typing import Any, Iterable, Sequence
from urllib.parse import urlparse

MAX_MANIFEST_BYTES = 256 * 1024
OPAQUE_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:-]{0,95}$")
OPERATOR_ALIAS_RE = re.compile(r"^[a-z0-9][a-z0-9._-]{1,63}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
EMAIL_RE = re.compile(r"(?<![A-Za-z0-9._%+-])[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}(?![A-Za-z0-9.-])")
URL_RE = re.compile(r"\b(?:https?|file)://", re.IGNORECASE)
WINDOWS_ABSOLUTE_RE = re.compile(r"(?:^|[\s'\"])[A-Za-z]:[\\/]")
POSIX_PRIVATE_PATH_RE = re.compile(r"(?:^|[\s'\"])/(?:Users|home|private|var/folders|tmp)/")
SECRET_ASSIGNMENT_RE = re.compile(
    r"\b(?:password|passwd|secret|token|api[_-]?key|authorization|cookie|clipboard|typed[_-]?text|sample[_-]?text|content)\s*[:=]",
    re.IGNORECASE,
)
PEM_RE = re.compile(r"-----BEGIN [A-Z0-9 ]+-----")
LONG_TOKEN_RE = re.compile(r"[A-Za-z0-9+/=_-]{80,}")
HEX_BLOB_RE = re.compile(r"\b[0-9a-fA-F]{96,}\b")
ALLOWED_EVIDENCE_KINDS = {"path", "url", "sha256", "run", "screenshot", "command"}


class PrivacyError(ValueError):
    """Raised with a field location, never the rejected value."""


def reject(location: str, reason: str) -> None:
    raise PrivacyError(f"{location}: {reason}")


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        size = path.stat().st_size
    except FileNotFoundError as exc:
        raise PrivacyError(f"{path}: file does not exist") from exc
    if size > MAX_MANIFEST_BYTES:
        raise PrivacyError(f"{path}: manifest exceeds {MAX_MANIFEST_BYTES} bytes")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except UnicodeDecodeError as exc:
        raise PrivacyError(f"{path}: manifest is not UTF-8") from exc
    except json.JSONDecodeError as exc:
        raise PrivacyError(f"{path}: invalid JSON at line {exc.lineno}") from exc
    if not isinstance(value, dict):
        raise PrivacyError(f"{path}: top-level value must be an object")
    return value


def validate_opaque_id(value: Any, location: str, *, allow_empty: bool = False) -> None:
    if not isinstance(value, str):
        reject(location, "must be a string")
    if allow_empty and value == "":
        return
    if not OPAQUE_ID_RE.fullmatch(value):
        reject(location, "must be an opaque identifier without whitespace or embedded data")


def validate_operator(value: Any, location: str) -> None:
    if not isinstance(value, str):
        reject(location, "must be a string")
    if value == "":
        return
    if not OPERATOR_ALIAS_RE.fullmatch(value):
        reject(location, "must be a lowercase non-identifying alias, not a name or email address")


def validate_note(value: Any, location: str) -> None:
    if not isinstance(value, str):
        reject(location, "must be a string")
    if not value:
        return
    if len(value) > 240:
        reject(location, "must be 240 characters or fewer")
    if "\n" in value or "\r" in value:
        reject(location, "must be one line")
    if EMAIL_RE.search(value):
        reject(location, "appears to contain an email address")
    if URL_RE.search(value):
        reject(location, "URLs belong in a structured evidence reference")
    if WINDOWS_ABSOLUTE_RE.search(value) or POSIX_PRIVATE_PATH_RE.search(value):
        reject(location, "appears to contain an absolute local path")
    if SECRET_ASSIGNMENT_RE.search(value):
        reject(location, "appears to contain secret, clipboard, sample, or content data")
    if PEM_RE.search(value) or LONG_TOKEN_RE.search(value) or HEX_BLOB_RE.search(value):
        reject(location, "appears to contain an embedded credential, digest blob, or encoded body")
    if any(ord(character) < 32 for character in value):
        reject(location, "contains control characters")


def validate_path(value: Any, location: str) -> None:
    if not isinstance(value, str) or not value:
        reject(location, "path reference must be a non-empty string")
    if "\\" in value:
        reject(location, "path reference must already use normalized forward slashes")
    path = PurePosixPath(value)
    if path.is_absolute() or ".." in path.parts or "." in path.parts:
        reject(location, "path reference must be normalized and repository-relative")
    if value.startswith("~") or ":" in value.split("/", 1)[0]:
        reject(location, "path reference must not identify a local home or drive")


def validate_url(value: Any, location: str) -> None:
    if not isinstance(value, str) or not value:
        reject(location, "URL reference must be a non-empty string")
    parsed = urlparse(value)
    if parsed.scheme != "https" or not parsed.netloc:
        reject(location, "URL reference must use HTTPS")
    if parsed.username or parsed.password:
        reject(location, "URL reference must not contain user information")
    if parsed.query or parsed.fragment:
        reject(location, "URL reference must not contain a query string or fragment")
    if "@" in parsed.netloc or parsed.hostname is None:
        reject(location, "URL authority is malformed")
    if parsed.hostname in {"localhost", "127.0.0.1", "::1"}:
        reject(location, "URL reference must not identify a local service")


def validate_evidence(item: Any, location: str) -> None:
    if not isinstance(item, dict) or set(item) != {"kind", "value"}:
        reject(location, "must contain exactly kind and value")
    kind = item.get("kind")
    if kind not in ALLOWED_EVIDENCE_KINDS:
        reject(f"{location}.kind", "is unsupported")
    value_location = f"{location}.value"
    value = item.get("value")
    if kind == "path":
        validate_path(value, value_location)
    elif kind == "url":
        validate_url(value, value_location)
    elif kind == "sha256":
        if not isinstance(value, str) or not SHA256_RE.fullmatch(value):
            reject(value_location, "must be a lowercase SHA-256 digest")
    else:
        validate_opaque_id(value, value_location)


def validate_manifest(manifest: dict[str, Any], source: Path) -> None:
    validate_opaque_id(manifest.get("run_id"), f"{source}:run_id")
    validate_operator(manifest.get("operator"), f"{source}:operator")

    privacy = manifest.get("privacy")
    expected_privacy = {
        "clipboard_content_collected": False,
        "typed_content_collected": False,
        "credentials_collected": False,
        "log_bodies_embedded": False,
    }
    if privacy != expected_privacy:
        reject(f"{source}:privacy", "must declare all prohibited collection fields false")

    host = manifest.get("host")
    if not isinstance(host, dict) or set(host) != {"system", "release", "machine", "python"}:
        reject(f"{source}:host", "must contain only content-blind OS metadata")
    for key, value in host.items():
        if not isinstance(value, str) or len(value) > 160 or "\n" in value or "\r" in value:
            reject(f"{source}:host.{key}", "is malformed")
        if EMAIL_RE.search(value) or WINDOWS_ABSOLUTE_RE.search(value) or POSIX_PRIVATE_PATH_RE.search(value):
            reject(f"{source}:host.{key}", "appears to contain identity or local path data")

    checks = manifest.get("checks")
    if not isinstance(checks, list):
        reject(f"{source}:checks", "must be a list")
    for index, check in enumerate(checks):
        location = f"{source}:checks[{index}]"
        if not isinstance(check, dict):
            reject(location, "must be an object")
        validate_note(check.get("note"), f"{location}.note")
        evidence = check.get("evidence")
        if not isinstance(evidence, list):
            reject(f"{location}.evidence", "must be a list")
        for evidence_index, item in enumerate(evidence):
            validate_evidence(item, f"{location}.evidence[{evidence_index}]")


def lint_paths(paths: Iterable[Path]) -> None:
    seen: set[Path] = set()
    for path in paths:
        resolved = path.resolve()
        if resolved in seen:
            raise PrivacyError(f"{path}: duplicate manifest path")
        seen.add(resolved)
        validate_manifest(load_manifest(path), path)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifests", nargs="+", type=Path)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        lint_paths(args.manifests)
    except PrivacyError as exc:
        print(f"privacy lint failed: {exc}", file=sys.stderr)
        return 2
    print(f"privacy lint passed for {len(args.manifests)} manifest(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
