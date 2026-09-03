from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "p3_evidence_privacy_lint.py"
SPEC = importlib.util.spec_from_file_location("p3_evidence_privacy_lint", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
LINT = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = LINT
SPEC.loader.exec_module(LINT)


def valid_manifest() -> dict:
    return {
        "run_id": "mac-arm64-001",
        "operator": "qa-runner",
        "host": {
            "system": "Darwin",
            "release": "25.0.0",
            "machine": "arm64",
            "python": "3.13.7",
        },
        "privacy": {
            "clipboard_content_collected": False,
            "typed_content_collected": False,
            "credentials_collected": False,
            "log_bodies_embedded": False,
        },
        "checks": [
            {
                "note": "grant state observed after returning from System Settings",
                "evidence": [
                    {"kind": "run", "value": "mac-arm64-001-accessibility"},
                    {"kind": "url", "value": "https://github.com/example/actions/runs/1"},
                    {"kind": "path", "value": "qa/evidence/p3/run-001.json"},
                    {"kind": "sha256", "value": "a" * 64},
                ],
            }
        ],
    }


class PrivacyLintTests(unittest.TestCase):
    def test_valid_content_blind_manifest_passes(self) -> None:
        LINT.validate_manifest(valid_manifest(), Path("manifest.json"))

    def test_operator_must_be_non_identifying_alias(self) -> None:
        for value in ("Jane Doe", "person@example.com", "A", "qa/runner"):
            with self.subTest(value=value):
                manifest = valid_manifest()
                manifest["operator"] = value
                with self.assertRaises(LINT.PrivacyError):
                    LINT.validate_manifest(manifest, Path("manifest.json"))

    def test_opaque_evidence_cannot_embed_log_text(self) -> None:
        for value in (
            "this is a copied log line",
            "token=super-secret-value",
            "-----BEGIN PRIVATE KEY-----",
            "A" * 100,
        ):
            with self.subTest(value=value):
                manifest = valid_manifest()
                manifest["checks"][0]["evidence"] = [{"kind": "run", "value": value}]
                with self.assertRaises(LINT.PrivacyError):
                    LINT.validate_manifest(manifest, Path("manifest.json"))

    def test_url_rejects_credentials_query_fragment_and_localhost(self) -> None:
        values = (
            "https://user:secret@example.com/run",
            "https://example.com/run?token=secret",
            "https://example.com/run#credential",
            "https://localhost/run",
            "http://example.com/run",
        )
        for value in values:
            with self.subTest(value=value):
                with self.assertRaises(LINT.PrivacyError):
                    LINT.validate_url(value, "evidence")

    def test_path_rejects_absolute_traversal_and_drive_paths(self) -> None:
        values = (
            "/Users/test/evidence.json",
            "../evidence.json",
            "qa/../evidence.json",
            "C:/Users/test/evidence.json",
            "qa\\evidence\\run.json",
        )
        for value in values:
            with self.subTest(value=value):
                with self.assertRaises(LINT.PrivacyError):
                    LINT.validate_path(value, "evidence")

    def test_note_rejects_content_secrets_identity_and_local_paths(self) -> None:
        values = (
            "clipboard=private text",
            "typed_text: private text",
            "token=abc123",
            "contact person@example.com",
            "see https://example.com/private",
            "saved at /Users/person/Desktop/evidence",
            "saved at C:\\Users\\person\\Desktop\\evidence",
            "A" * 100,
        )
        for value in values:
            with self.subTest(value=value):
                with self.assertRaises(LINT.PrivacyError):
                    LINT.validate_note(value, "note")

    def test_error_never_contains_rejected_value(self) -> None:
        secret = "token=THIS_MUST_NOT_APPEAR"
        with self.assertRaises(LINT.PrivacyError) as raised:
            LINT.validate_note(secret, "manifest.json:checks[0].note")
        self.assertNotIn(secret, str(raised.exception))
        self.assertIn("checks[0].note", str(raised.exception))

    def test_manifest_size_is_bounded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "oversized.json"
            path.write_bytes(b" " * (LINT.MAX_MANIFEST_BYTES + 1))
            with self.assertRaises(LINT.PrivacyError):
                LINT.load_manifest(path)

    def test_file_round_trip(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "manifest.json"
            path.write_text(json.dumps(valid_manifest()), encoding="utf-8")
            LINT.lint_paths([path])


if __name__ == "__main__":
    unittest.main()
