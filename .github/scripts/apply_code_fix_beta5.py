#!/usr/bin/env python3
import base64
import json
import pathlib
import shutil
import zlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
CHUNK_DIR = ROOT / ".github" / "code-fix-chunks"

encoded = "".join(
    (CHUNK_DIR / f"chunk-{index:02d}.txt").read_text().strip()
    for index in range(5)
)
obj = json.loads(zlib.decompress(base64.b64decode(encoded)))

# Keep the Flutter regression test package-clean under the repository lint set.
flutter_test = "apps/cliptype-flutter/test/completion_diagnostics_test.dart"
obj["payload"][flutter_test] = obj["payload"][flutter_test].replace(
    "import '../lib/l10n/app_localizations.dart';",
    "import 'package:cliptype_flutter/l10n/app_localizations.dart';",
)

for change in obj["manifest"]["changes"]:
    path = ROOT / change["path"]
    text = path.read_text()
    before = change["before"]
    after = change["after"]
    if text.count(before) != 1:
        raise SystemExit(f"anchor mismatch: {change['path']}")
    path.write_text(text.replace(before, after, 1))

for rel, text in obj["payload"].items():
    path = ROOT / rel
    if path.exists():
        raise SystemExit(f"payload exists: {rel}")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)

(ROOT / "release" / "VERSION").write_text("v0.2.0-beta.5\n")

pubspec = ROOT / "apps" / "cliptype-flutter" / "pubspec.yaml"
pubspec_text = pubspec.read_text()
old_version = "version: 0.2.0-beta.4+3"
if pubspec_text.count(old_version) != 1:
    raise SystemExit("pubspec version mismatch")
pubspec.write_text(pubspec_text.replace(old_version, "version: 0.2.0-beta.5+4", 1))

for rel in [
    "README.md",
    "apps/cliptype-flutter/README.md",
    "docs/README.md",
    "docs/COMPATIBILITY.md",
]:
    path = ROOT / rel
    path.write_text(path.read_text().replace("v0.2.0-beta.4", "v0.2.0-beta.5"))

release_notes = ROOT / "docs" / "releases" / "v0.2.0-beta.5.md"
if release_notes.exists():
    raise SystemExit("beta5 release notes already exist")
release_notes.write_text("""# ClipType v0.2.0-beta.5

`v0.2.0-beta.5` is a maintenance prerelease for Code-mode reliability on the macOS Apple Silicon testing preview.

## Fix

- isolates synthetic Core Graphics keyboard events from the state used to observe user-held modifiers;
- prevents Code-mode Command+Right closer navigation from contaminating the following modifier safety check;
- explicitly clears the synthetic Command flag on the final Right-key release;
- keeps target, cancellation, physical-modifier, Accessibility, and partial-input fail-closed protections;
- adds the reported Rust heap-sort fixture and LF/CRLF Code-plan regressions;
- carries content-free completion diagnostics through the macOS Flutter bridge.

## Validation

Publication requires the exact-head Rust, Flutter, Windows release/compatibility/controlled-input gates and macOS arm64 build/package gate to pass. Real-editor physical verification remains separate and this prerelease does not claim universal VS Code/Monaco compatibility.

## Platform boundary

Windows x86_64 remains primary. The macOS arm64 asset remains an ad-hoc-signed testing preview requiring Accessibility consent; it is not Developer ID signed, notarized, Universal 2, or a general macOS release.

## Rollback

Older tags and assets remain immutable.
""")

for rel in [
    ".github/apply-code-fix-beta5.log",
    ".github/rust-code-fix-beta5.log",
    ".github/flutter-code-fix-beta5.log",
    ".github/cliptype-code-fix-trigger",
    ".github/workflows/bootstrap-code-fix-beta5.yml",
    ".github/scripts/apply_code_fix_beta5.py",
]:
    path = ROOT / rel
    if path.exists():
        path.unlink()

if CHUNK_DIR.exists():
    shutil.rmtree(CHUNK_DIR)
