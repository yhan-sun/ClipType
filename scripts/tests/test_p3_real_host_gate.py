from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "p3_real_host_gate.py"
SPEC = importlib.util.spec_from_file_location("p3_real_host_gate", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
GATE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = GATE
SPEC.loader.exec_module(GATE)
CATALOG = ROOT / "qa" / "p3-real-host-checks.json"
COMMIT = "a" * 40


class CatalogTests(unittest.TestCase):
    def test_catalog_is_valid_and_covers_all_platforms(self) -> None:
        definitions = GATE.load_catalog(CATALOG)
        self.assertGreaterEqual(len(definitions), 25)
        covered = {platform for definition in definitions for platform in definition.platforms}
        self.assertEqual(GATE.PLATFORMS, covered)
        self.assertEqual(len(definitions), len({definition.check_id for definition in definitions}))

    def test_catalog_has_content_privacy_check_for_every_platform(self) -> None:
        definitions = {definition.check_id: definition for definition in GATE.load_catalog(CATALOG)}
        privacy = definitions["privacy.no_content_in_diagnostics"]
        self.assertEqual(GATE.PLATFORMS, set(privacy.platforms))
        self.assertTrue(privacy.required)


class ManifestTests(unittest.TestCase):
    def create(self, platform_name: str = "windows-x86_64"):
        return GATE.create_manifest(
            catalog_path=CATALOG,
            platform_name=platform_name,
            commit=COMMIT,
            run_id=f"test-{platform_name}",
            operator="tester",
        )

    def test_init_is_valid_and_content_blind(self) -> None:
        manifest = self.create()
        pending = GATE.validate_manifest(manifest, catalog_path=CATALOG)
        self.assertTrue(pending)
        self.assertEqual(
            manifest["privacy"],
            {
                "clipboard_content_collected": False,
                "typed_content_collected": False,
                "credentials_collected": False,
                "log_bodies_embedded": False,
            },
        )
        self.assertNotIn("hostname", manifest["host"])
        self.assertNotIn("username", manifest["host"])

    def test_pass_requires_evidence(self) -> None:
        manifest = self.create()
        check_id = manifest["checks"][0]["id"]
        with self.assertRaisesRegex(GATE.GateError, "requires at least one evidence"):
            GATE.record_result(
                manifest,
                catalog_path=CATALOG,
                check_id=check_id,
                result="pass",
                evidence=[],
                note="",
            )

    def test_record_rehashes_manifest_and_reset_is_safe(self) -> None:
        manifest = self.create()
        old_digest = manifest["manifest_sha256"]
        check_id = manifest["checks"][0]["id"]
        GATE.record_result(
            manifest,
            catalog_path=CATALOG,
            check_id=check_id,
            result="pass",
            evidence=[{"kind": "run", "value": "local-001"}],
            note="metadata-only evidence",
        )
        self.assertNotEqual(old_digest, manifest["manifest_sha256"])
        GATE.validate_manifest(manifest, catalog_path=CATALOG)
        GATE.reset_result(manifest, catalog_path=CATALOG, check_id=check_id)
        check = next(item for item in manifest["checks"] if item["id"] == check_id)
        self.assertEqual("pending", check["result"])
        self.assertEqual([], check["evidence"])
        self.assertIsNone(check["recorded_at"])
        GATE.validate_manifest(manifest, catalog_path=CATALOG)

    def test_tampering_is_detected(self) -> None:
        manifest = self.create()
        manifest["operator"] = "tampered"
        with self.assertRaisesRegex(GATE.GateError, "does not match"):
            GATE.validate_manifest(manifest, catalog_path=CATALOG)

    def test_wrong_commit_is_detected(self) -> None:
        manifest = self.create()
        with self.assertRaisesRegex(GATE.GateError, "does not match expected"):
            GATE.validate_manifest(
                manifest,
                catalog_path=CATALOG,
                expected_commit="b" * 40,
            )

    def test_required_check_cannot_be_not_applicable(self) -> None:
        manifest = self.create()
        check_id = manifest["checks"][0]["id"]
        with self.assertRaisesRegex(GATE.GateError, "cannot be marked"):
            GATE.record_result(
                manifest,
                catalog_path=CATALOG,
                check_id=check_id,
                result="not-applicable",
                evidence=[],
                note="",
            )

    def test_unsafe_evidence_is_rejected(self) -> None:
        invalid = (
            "path=/tmp/private.txt",
            "path=../private.txt",
            "url=http://example.com/run",
            "url=https://user:secret@example.com/run",
            "sha256=not-a-digest",
            "blob=inline-content",
        )
        for value in invalid:
            with self.subTest(value=value):
                with self.assertRaises(GATE.GateError):
                    GATE.parse_evidence(value)

    def test_safe_evidence_is_normalized(self) -> None:
        self.assertEqual(
            {"kind": "path", "value": "qa/evidence/p3/run.json"},
            GATE.parse_evidence("path=qa\\evidence\\p3\\run.json"),
        )
        self.assertEqual(
            {"kind": "url", "value": "https://github.com/example/actions/runs/1"},
            GATE.parse_evidence("url=https://github.com/example/actions/runs/1"),
        )

    def test_full_platform_manifest_completes_only_after_all_required_pass(self) -> None:
        manifest = self.create("macos-arm64")
        for check in list(manifest["checks"]):
            GATE.record_result(
                manifest,
                catalog_path=CATALOG,
                check_id=check["id"],
                result="pass",
                evidence=[{"kind": "run", "value": f"evidence-{check['id']}"}],
                note="",
            )
        self.assertIsNotNone(manifest["completed_at"])
        self.assertEqual(
            [],
            GATE.validate_manifest(
                manifest,
                catalog_path=CATALOG,
                require_complete=True,
            ),
        )

    def test_report_contains_no_evidence_values(self) -> None:
        manifest = self.create()
        check_id = manifest["checks"][0]["id"]
        GATE.record_result(
            manifest,
            catalog_path=CATALOG,
            check_id=check_id,
            result="pass",
            evidence=[{"kind": "url", "value": "https://example.com/sensitive-reference"}],
            note="metadata-only",
        )
        report = GATE.render_report([manifest], CATALOG)
        self.assertIn(check_id, report)
        self.assertNotIn("sensitive-reference", report)

    def test_atomic_write_round_trip(self) -> None:
        manifest = self.create()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "nested" / "manifest.json"
            GATE.atomic_write_json(path, manifest)
            loaded = GATE.load_json(path)
            self.assertEqual(manifest, loaded)
            GATE.validate_manifest(loaded, catalog_path=CATALOG)


class GateSetTests(unittest.TestCase):
    def complete(self, platform_name: str):
        manifest = GATE.create_manifest(
            catalog_path=CATALOG,
            platform_name=platform_name,
            commit=COMMIT,
            run_id=f"set-{platform_name}",
            operator="tester",
        )
        for check in list(manifest["checks"]):
            GATE.record_result(
                manifest,
                catalog_path=CATALOG,
                check_id=check["id"],
                result="pass",
                evidence=[{"kind": "run", "value": f"run-{platform_name}-{check['id']}"}],
                note="",
            )
        return manifest

    def test_host_gate_requires_windows_and_both_macos_architectures(self) -> None:
        manifests = [
            self.complete("windows-x86_64"),
            self.complete("macos-arm64"),
            self.complete("macos-x86_64"),
        ]
        GATE.verify_gate_set(
            manifests,
            catalog_path=CATALOG,
            expected_commit=COMMIT,
            require_release=False,
        )

    def test_release_gate_is_separate_and_explicit(self) -> None:
        manifests = [
            self.complete("windows-x86_64"),
            self.complete("macos-arm64"),
            self.complete("macos-x86_64"),
        ]
        with self.assertRaisesRegex(GATE.GateError, "release-macos"):
            GATE.verify_gate_set(
                manifests,
                catalog_path=CATALOG,
                expected_commit=COMMIT,
                require_release=True,
            )
        manifests.append(self.complete("release-macos"))
        GATE.verify_gate_set(
            manifests,
            catalog_path=CATALOG,
            expected_commit=COMMIT,
            require_release=True,
        )


if __name__ == "__main__":
    unittest.main()
