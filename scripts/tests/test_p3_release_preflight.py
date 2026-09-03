from __future__ import annotations

import hashlib
import importlib.util
import plistlib
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "p3_release_preflight.py"
SPEC = importlib.util.spec_from_file_location("p3_release_preflight", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
PREFLIGHT = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = PREFLIGHT
SPEC.loader.exec_module(PREFLIGHT)


class DigestTests(unittest.TestCase):
    def test_sha256_file(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "artifact.zip"
            path.write_bytes(b"ClipType")
            self.assertEqual(hashlib.sha256(b"ClipType").hexdigest(), PREFLIGHT.sha256_file(path))


class BundleTests(unittest.TestCase):
    def make_bundle(self, root: Path) -> Path:
        app = root / "ClipType.app"
        macos = app / "Contents" / "MacOS"
        resources = app / "Contents" / "Resources"
        macos.mkdir(parents=True)
        resources.mkdir(parents=True)
        executable = macos / "cliptype-macos"
        executable.write_bytes(b"binary")
        executable.chmod(0o755)
        (resources / "ClipType.icns").write_bytes(b"icon")
        info = {
            "CFBundleIdentifier": "dev.cliptype.app",
            "CFBundleName": "ClipType",
            "CFBundleExecutable": "cliptype-macos",
            "CFBundlePackageType": "APPL",
            "CFBundleShortVersionString": "0.2.0-beta.1",
            "CFBundleVersion": "1",
            "CFBundleIconFile": "ClipType.icns",
            "LSMinimumSystemVersion": "13.0",
        }
        with (app / "Contents" / "Info.plist").open("wb") as stream:
            plistlib.dump(info, stream)
        return app

    def test_bundle_metadata_and_executable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            app = self.make_bundle(Path(directory))
            results = PREFLIGHT.Results()
            with mock.patch.object(PREFLIGHT.shutil, "which", return_value=None):
                PREFLIGHT.check_bundle(results, app, require_universal=False)
            statuses = {item.check: item.status for item in results.items}
            self.assertEqual("pass", statuses["bundle.exists"])
            self.assertEqual("pass", statuses["bundle.metadata"])
            self.assertEqual("pass", statuses["bundle.icon"])
            self.assertEqual("pass", statuses["bundle.executable"])
            self.assertEqual("skipped", statuses["bundle.universal2"])

    def test_universal_requirement_fails_without_lipo(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            app = self.make_bundle(Path(directory))
            results = PREFLIGHT.Results()
            with mock.patch.object(PREFLIGHT.shutil, "which", return_value=None):
                PREFLIGHT.check_bundle(results, app, require_universal=True)
            statuses = {item.check: item.status for item in results.items}
            self.assertEqual("fail", statuses["bundle.universal2"])

    def test_invalid_executable_name_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            app = self.make_bundle(Path(directory))
            info_path = app / "Contents" / "Info.plist"
            with info_path.open("rb") as stream:
                info = plistlib.load(stream)
            info["CFBundleExecutable"] = "../escape"
            with info_path.open("wb") as stream:
                plistlib.dump(info, stream)
            with self.assertRaises(PREFLIGHT.PreflightError):
                PREFLIGHT.read_bundle_metadata(app)


class ArtifactTests(unittest.TestCase):
    def test_missing_required_artifact_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            results = PREFLIGHT.Results()
            hashes = PREFLIGHT.check_artifacts(
                results,
                [Path(directory) / "missing.dmg"],
                required=True,
            )
            self.assertEqual({}, hashes)
            self.assertTrue(results.failed)

    def test_present_artifact_records_digest_not_content(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "artifact.dmg"
            path.write_bytes(b"SECRET_SAMPLE_CONTENT")
            results = PREFLIGHT.Results()
            hashes = PREFLIGHT.check_artifacts(results, [path], required=True)
            self.assertEqual(64, len(hashes[path.name]))
            self.assertFalse(results.failed)
            self.assertNotIn("SECRET_SAMPLE_CONTENT", results.items[0].detail)


if __name__ == "__main__":
    unittest.main()
