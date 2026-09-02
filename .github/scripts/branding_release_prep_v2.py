from pathlib import Path
import runpy

root = Path(__file__).resolve().parents[2]
legacy = root / ".github/scripts/branding_release_prep.py"
text = legacy.read_text(encoding="utf-8")
text = text.replace(
    "BUILD_RS = r'''use std::{",
    "BUILD_RS = r'''#[cfg(windows)]\nuse std::{",
    1,
)
text = text.replace(
    "\n#[derive(Clone, Copy)]\nenum IconKind {",
    "\n#[cfg(windows)]\n#[derive(Clone, Copy)]\nenum IconKind {",
    1,
)
legacy.write_text(text, encoding="utf-8")
runpy.run_path(str(legacy), run_name="__main__")

keep = {
    "README.md",
    "NOTICE.md",
    "RELEASE_ASSET_POLICY.md",
    "cliptype-primary.svg",
    "cliptype-tray.svg",
}
branding = root / "assets/branding"
for path in branding.iterdir():
    if path.is_file() and path.name not in keep:
        path.unlink()

for relative in [
    ".github/scripts/branding_release_prep_v2.py",
    ".github/workflows/branding-release-prep-v2.yml",
    ".github/scripts/branding_cleanup.py",
    ".github/workflows/branding-cleanup.yml",
    ".github/scripts/branding_release_prep.py",
    ".github/workflows/branding-release-prep.yml",
    ".github/scripts/p2_b02_patch.py",
    ".github/workflows/p2-b02-apply.yml",
]:
    path = root / relative
    if path.exists():
        path.unlink()
