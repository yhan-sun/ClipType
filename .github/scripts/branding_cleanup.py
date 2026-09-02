from pathlib import Path

root = Path(__file__).resolve().parents[2]
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
    ".github/scripts/branding_cleanup.py",
    ".github/workflows/branding-cleanup.yml",
]:
    path = root / relative
    if path.exists():
        path.unlink()
