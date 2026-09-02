from __future__ import annotations

from pathlib import Path
import re
import textwrap

ROOT = Path(__file__).resolve().parents[2]

BUILD_RS = r'''use std::{
    env,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Copy)]
enum IconKind {
    Application,
    Tray,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/branding/cliptype-primary.svg");
    println!("cargo:rerun-if-changed=../../assets/branding/cliptype-tray.svg");

    #[cfg(windows)]
    build_windows_resources().expect("compile ClipType Windows icon resources");
}

#[cfg(windows)]
fn build_windows_resources() -> io::Result<()> {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let app_icon = out.join("cliptype-app.ico");
    let tray_icon = out.join("cliptype-tray.ico");
    write_ico(&app_icon, IconKind::Application)?;
    write_ico(&tray_icon, IconKind::Tray)?;

    let resource = out.join("cliptype-icons.rc");
    let app_path = resource_path(&app_icon);
    let tray_path = resource_path(&tray_icon);
    fs::write(
        &resource,
        format!("1 ICON \"{app_path}\"\r\n2 ICON \"{tray_path}\"\r\n"),
    )?;

    let output = out.join("cliptype-icons.res");
    let compiler = find_resource_compiler().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Windows SDK resource compiler not found")
    })?;
    let status = Command::new(compiler)
        .arg("/nologo")
        .arg(format!("/fo{}", output.display()))
        .arg(&resource)
        .status()?;
    if !status.success() {
        return Err(io::Error::other("Windows resource compiler failed"));
    }

    println!("cargo:rustc-link-arg-bin=cliptype={}", output.display());
    Ok(())
}

#[cfg(windows)]
fn resource_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(windows)]
fn find_resource_compiler() -> Option<PathBuf> {
    if let Some(path) = env::var_os("RC").map(PathBuf::from)
        && path.is_file()
    {
        return Some(path);
    }

    if let Ok(output) = Command::new("where.exe").arg("rc.exe").output()
        && output.status.success()
        && let Some(line) = String::from_utf8_lossy(&output.stdout).lines().next()
    {
        let path = PathBuf::from(line.trim());
        if path.is_file() {
            return Some(path);
        }
    }

    if let (Some(root), Some(version)) = (
        env::var_os("WindowsSdkDir").map(PathBuf::from),
        env::var_os("WindowsSDKVersion"),
    ) {
        let version = version.to_string_lossy().trim_matches(['\\', '/']).to_owned();
        for architecture in ["x64", "x86"] {
            let candidate = root.join("bin").join(&version).join(architecture).join("rc.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    let program_files = env::var_os("ProgramFiles(x86)").map(PathBuf::from)?;
    let root = program_files.join("Windows Kits").join("10").join("bin");
    let mut versions: Vec<_> = fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .collect();
    versions.sort_by_key(|entry| std::cmp::Reverse(entry.file_name()));
    for version in versions {
        for architecture in ["x64", "x86"] {
            let candidate = version.path().join(architecture).join("rc.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(windows)]
fn write_ico(path: &Path, kind: IconKind) -> io::Result<()> {
    const SIZES: [u32; 8] = [16, 20, 24, 32, 48, 64, 128, 256];
    let images: Vec<_> = SIZES
        .into_iter()
        .map(|size| encode_dib(size, render(size, kind)))
        .collect();

    let count = u16::try_from(images.len()).expect("fixed icon count");
    let header_bytes = 6_u32 + u32::from(count) * 16;
    let mut offset = header_bytes;
    let mut file = fs::File::create(path)?;
    write_u16(&mut file, 0)?;
    write_u16(&mut file, 1)?;
    write_u16(&mut file, count)?;

    for (size, image) in SIZES.into_iter().zip(&images) {
        file.write_all(&[if size == 256 { 0 } else { size as u8 }])?;
        file.write_all(&[if size == 256 { 0 } else { size as u8 }])?;
        file.write_all(&[0, 0])?;
        write_u16(&mut file, 1)?;
        write_u16(&mut file, 32)?;
        write_u32(&mut file, u32::try_from(image.len()).expect("bounded icon frame"))?;
        write_u32(&mut file, offset)?;
        offset = offset
            .checked_add(u32::try_from(image.len()).expect("bounded icon frame"))
            .expect("bounded icon file");
    }
    for image in images {
        file.write_all(&image)?;
    }
    Ok(())
}

#[cfg(windows)]
fn encode_dib(size: u32, rgba: Vec<[u8; 4]>) -> Vec<u8> {
    let pixel_bytes = size * size * 4;
    let mask_row = size.div_ceil(32) * 4;
    let mask_bytes = mask_row * size;
    let mut out = Vec::with_capacity((40 + pixel_bytes + mask_bytes) as usize);
    push_u32(&mut out, 40);
    push_i32(&mut out, size as i32);
    push_i32(&mut out, (size * 2) as i32);
    push_u16(&mut out, 1);
    push_u16(&mut out, 32);
    push_u32(&mut out, 0);
    push_u32(&mut out, pixel_bytes);
    push_i32(&mut out, 0);
    push_i32(&mut out, 0);
    push_u32(&mut out, 0);
    push_u32(&mut out, 0);

    for y in (0..size).rev() {
        for x in 0..size {
            let [red, green, blue, alpha] = rgba[(y * size + x) as usize];
            out.extend_from_slice(&[blue, green, red, alpha]);
        }
    }

    for y in (0..size).rev() {
        let start = out.len();
        out.resize(start + mask_row as usize, 0);
        for x in 0..size {
            if rgba[(y * size + x) as usize][3] < 128 {
                let byte = start + (x / 8) as usize;
                out[byte] |= 0x80 >> (x % 8);
            }
        }
    }
    out
}

#[cfg(windows)]
fn render(size: u32, kind: IconKind) -> Vec<[u8; 4]> {
    const SCALE: u32 = 4;
    let large = size * SCALE;
    let mut high = vec![[0_u8; 4]; (large * large) as usize];
    for y in 0..large {
        for x in 0..large {
            let px = (x as f32 + 0.5) / large as f32;
            let py = (y as f32 + 0.5) / large as f32;
            high[(y * large + x) as usize] = sample_icon(px, py, kind);
        }
    }

    let mut output = vec![[0_u8; 4]; (size * size) as usize];
    for y in 0..size {
        for x in 0..size {
            let mut sums = [0_u32; 4];
            for sy in 0..SCALE {
                for sx in 0..SCALE {
                    let pixel = high[((y * SCALE + sy) * large + x * SCALE + sx) as usize];
                    for channel in 0..4 {
                        sums[channel] += u32::from(pixel[channel]);
                    }
                }
            }
            let divisor = SCALE * SCALE;
            output[(y * size + x) as usize] = [
                (sums[0] / divisor) as u8,
                (sums[1] / divisor) as u8,
                (sums[2] / divisor) as u8,
                (sums[3] / divisor) as u8,
            ];
        }
    }
    output
}

#[cfg(windows)]
fn sample_icon(x: f32, y: f32, kind: IconKind) -> [u8; 4] {
    let mut color = [0.0_f32; 4];
    match kind {
        IconKind::Application => {
            if rounded_rect(x, y, 0.035, 0.035, 0.965, 0.965, 0.205) {
                let cyan = (1.0 - (x + y) * 0.38).clamp(0.0, 1.0);
                over(&mut color, [0.025 + cyan * 0.02, 0.20 + cyan * 0.55, 0.88 + cyan * 0.10, 1.0]);
            }
            clipboard(&mut color, x, y, 0.13, 0.10, 0.68, 0.76);
            if capsule(x, y, 0.045, 0.525, 0.49, 0.575)
                || capsule(x, y, 0.075, 0.61, 0.545, 0.655)
                || capsule(x, y, 0.15, 0.69, 0.57, 0.73)
            {
                over(&mut color, [0.05, 0.83, 1.0, 0.96]);
            }
            if rounded_rect(x, y, 0.53, 0.60, 0.92, 0.91, 0.075) {
                over(&mut color, [0.035, 0.105, 0.54, 1.0]);
            }
            if rounded_rect(x, y, 0.575, 0.635, 0.88, 0.865, 0.055) {
                over(&mut color, [0.96, 0.985, 1.0, 1.0]);
            }
            draw_caret(&mut color, x, y, 0.675, 0.68, 0.81, 0.82, [0.02, 0.38, 0.95, 1.0]);
        }
        IconKind::Tray => {
            if circle(x, y, 0.5, 0.5, 0.485) {
                let cyan = (1.0 - (x + y) * 0.42).clamp(0.0, 1.0);
                over(&mut color, [0.015 + cyan * 0.02, 0.20 + cyan * 0.58, 0.90 + cyan * 0.08, 1.0]);
            }
            clipboard(&mut color, x, y, 0.19, 0.16, 0.72, 0.82);
            draw_caret(&mut color, x, y, 0.67, 0.42, 0.86, 0.78, [1.0, 1.0, 1.0, 1.0]);
        }
    }
    [
        (color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[3].clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

#[cfg(windows)]
fn clipboard(color: &mut [f32; 4], x: f32, y: f32, left: f32, top: f32, right: f32, bottom: f32) {
    if rounded_rect(x, y, left, top + 0.10, right, bottom, 0.075) {
        over(color, [0.96, 0.985, 1.0, 1.0]);
    }
    let width = right - left;
    if rounded_rect(
        x,
        y,
        left + width * 0.27,
        top,
        left + width * 0.73,
        top + 0.20,
        0.055,
    ) {
        over(color, [0.90, 0.95, 1.0, 1.0]);
    }
    if circle(x, y, (left + right) * 0.5, top + 0.075, 0.029) {
        over(color, [0.04, 0.30, 0.82, 1.0]);
    }
    for (line_top, line_right, blue) in [
        (top + 0.30, right - width * 0.17, [0.02, 0.66, 0.94, 1.0]),
        (top + 0.41, right - width * 0.22, [0.10, 0.49, 0.93, 1.0]),
        (top + 0.52, right - width * 0.32, [0.24, 0.58, 0.94, 1.0]),
    ] {
        if capsule(x, y, left + width * 0.17, line_top, line_right, line_top + 0.045) {
            over(color, blue);
        }
    }
}

#[cfg(windows)]
fn draw_caret(color: &mut [f32; 4], x: f32, y: f32, left: f32, top: f32, right: f32, bottom: f32, fill: [f32; 4]) {
    let width = right - left;
    let cap = width;
    if rounded_rect(x, y, left, top, right, top + cap * 0.34, cap * 0.15)
        || rounded_rect(x, y, left + width * 0.37, top, left + width * 0.63, bottom, width * 0.12)
        || rounded_rect(x, y, left, bottom - cap * 0.34, right, bottom, cap * 0.15)
    {
        over(color, fill);
    }
}

#[cfg(windows)]
fn over(destination: &mut [f32; 4], source: [f32; 4]) {
    let source_alpha = source[3];
    let destination_alpha = destination[3];
    let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
    if output_alpha <= f32::EPSILON {
        *destination = [0.0; 4];
        return;
    }
    for channel in 0..3 {
        destination[channel] = (source[channel] * source_alpha
            + destination[channel] * destination_alpha * (1.0 - source_alpha))
            / output_alpha;
    }
    destination[3] = output_alpha;
}

#[cfg(windows)]
fn circle(x: f32, y: f32, cx: f32, cy: f32, radius: f32) -> bool {
    (x - cx).mul_add(x - cx, (y - cy) * (y - cy)) <= radius * radius
}

#[cfg(windows)]
fn capsule(x: f32, y: f32, left: f32, top: f32, right: f32, bottom: f32) -> bool {
    rounded_rect(x, y, left, top, right, bottom, (bottom - top) * 0.5)
}

#[cfg(windows)]
fn rounded_rect(x: f32, y: f32, left: f32, top: f32, right: f32, bottom: f32, radius: f32) -> bool {
    if x < left || x > right || y < top || y > bottom {
        return false;
    }
    let cx = x.clamp(left + radius, right - radius);
    let cy = y.clamp(top + radius, bottom - radius);
    (x - cx).mul_add(x - cx, (y - cy) * (y - cy)) <= radius * radius
}

#[cfg(windows)]
fn write_u16(writer: &mut impl Write, value: u16) -> io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

#[cfg(windows)]
fn write_u32(writer: &mut impl Write, value: u32) -> io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

#[cfg(windows)]
fn push_u16(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

#[cfg(windows)]
fn push_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

#[cfg(windows)]
fn push_i32(buffer: &mut Vec<u8>, value: i32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}
'''

BRANDING_README = '''# ClipType branding assets

ClipType uses a detailed primary illustration and a simplified notification-area mark.

- `cliptype-primary.svg` is the primary application and release artwork.
- `cliptype-tray.svg` is the simplified small-size source.
- Windows builds generate multi-frame 16, 20, 24, 32, 48, 64, 128, and 256-pixel ICO resources from deterministic geometry in `apps/cliptype/build.rs`.
- Resource ID `1` is the executable icon; resource ID `2` is loaded by the notification-area shell.

The small icon deliberately omits motion ribbons and the detailed keycap/cursor composition. It uses a blue circular field, a white clipboard, and an insertion caret so the silhouette remains legible at Windows tray sizes.
'''

RELEASE_NOTES = '''# ClipType v0.1.0-beta.1

ClipType's first public Windows **prerelease** turns the clipboard-to-input vertical slice into a daily-usable tray utility. This build is intended for early adopters; application-specific compatibility reports remain welcome, and post-fix interactive validation remains visible in issues #33 and #41.

## Included

- new ClipType application and notification-area icons;
- native Win32 tray shell and reviewed global trigger/cancel hotkey presets;
- explicit `keyboard`, `clipboard`, and capability-gated `auto` modes;
- bounded Unicode keyboard input and revision-guarded current-clipboard paste;
- real characters-per-second pacing with independently sampled bounded jitter;
- optional adjacent-QWERTY typo simulation with paced Backspace correction, disabled by default;
- strict versioned per-user settings with migration and backup recovery;
- current-user start-at-login support;
- per-user install and uninstall scripts that require no elevation;
- content-free diagnostics, privacy-sentinel checks, and bounded shutdown;
- automated Windows Server 2022 and Windows Server 2025 x86_64 compatibility coverage.

## Compatibility declaration

Windows 11 x64 on an unlocked interactive desktop is the recommended client. Windows 10 22H2 x64 is best-effort because the operating system is outside standard Microsoft support. Hosted runners verify builds, controlled native targets, packaging, and lifecycle behaviour; they do not certify every third-party application or every physical keyboard/IME environment.

`auto` mode prefers revision-guarded clipboard paste for non-ASCII text when that capability is available, improving CJK reliability without depending on an active IME. Explicit `keyboard` mode preserves Unicode packet semantics and never silently falls back, so individual targets that reject synthetic Unicode packets can still report a limitation.

Not supported in this release:

- 32-bit Windows or Windows on ARM64;
- Server Core, services, locked desktops, or non-interactive sessions;
- automatic elevation or injection across Windows integrity boundaries;
- a universal exact-caret guarantee inside shared Chromium/Electron render hosts;
- clipboard transformations that require ClipType to rewrite and restore the clipboard.

## Install

1. Download `ClipType-v0.1.0-beta.1-windows-x86_64.zip`.
2. Verify `SHA256SUMS.txt`, the matching Sigstore bundles, and the GitHub artifact attestation.
3. Extract the archive.
4. Run `install.ps1` from PowerShell, or run `cliptype.exe` portably.

The default installation is per-user and does not request administrator privileges.

## Signature and provenance

Release assets and the checksum manifest are signed with Sigstore keyless signing using GitHub Actions OIDC. GitHub artifact attestations bind the files to the workflow and source commit.

The executable is **not Authenticode publisher-signed** because no trusted Windows code-signing certificate is configured. SmartScreen or reputation warnings may therefore appear. Sigstore, SHA-256 checksums, and GitHub attestations prove source provenance but do not make Windows show a trusted publisher identity.

## Human-paced keyboard controls

The version-2 settings schema supports:

```toml
characters_per_second = 40
jitter_percent = 10
typo_probability_percent = 0
```

Typo simulation is opt-in and should remain disabled for passwords, source code, terminals, commands, administrative tools, and exact-data entry. Clipboard mode remains one atomic normal paste and is not artificially slowed character by character.

## Privacy and safety

ClipType does not persist clipboard text, collect clipboard history, transmit clipboard content, read focused-field contents, or log window titles. Clipboard mode uses a content-blind revision check and never replaces, clears, or restores the clipboard. Partial or unknown native input is never blindly retried.

## Known limitations

- Hotkey preset changes apply after a controlled restart.
- The destination application owns rich-text paste, formatting, and command-submission semantics.
- A target change can only be detected before the next action already submitted to Windows.
- A normal-integrity ClipType process cannot inject into a higher-integrity target.
- CJK and Unicode support remains application-dependent in explicit `keyboard` mode.
- Post-fix physical/client validation is tracked in issues #33 and #41.

## Upgrade and uninstall

Re-running `install.ps1` updates the per-user executable and preserves supported settings. `uninstall.ps1` removes product-owned installation and startup state. Pass `-RemoveSettings` to remove the per-user configuration as well.
'''


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        if new in text:
            return
        raise RuntimeError(f"expected text not found in {path}: {old[:80]!r}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def patch_cargo() -> None:
    path = ROOT / "apps/cliptype/Cargo.toml"
    text = path.read_text(encoding="utf-8")
    if 'build = "build.rs"' not in text:
        text = text.replace("publish.workspace = true\n", 'publish.workspace = true\nbuild = "build.rs"\n')
    path.write_text(text, encoding="utf-8")


def patch_tray() -> None:
    path = ROOT / "crates/cliptype-windows/src/tray.rs"
    text = path.read_text(encoding="utf-8")
    if "IDI_CLIPTYPE_TRAY" not in text:
        text = text.replace(
            "const IDI_APPLICATION: *const u16 = 32_512_usize as *const u16;",
            "const IDI_CLIPTYPE_TRAY: *const u16 = 2_usize as *const u16;\nconst IDI_APPLICATION: *const u16 = 32_512_usize as *const u16;",
        )
    old = '''        // SAFETY: loads the process-independent stock application icon.\n        let icon = unsafe { load_icon_w(null_mut(), IDI_APPLICATION) };\n        if icon.is_null() {\n            return Err(TrayError::IconUnavailable);\n        }'''
    new = '''        // SAFETY: resource id 2 is generated and embedded by the application\n        // build script. A stock icon is retained only as a development fallback.\n        let instance = unsafe { get_module_handle_w(null()) };\n        let mut icon = if instance.is_null() {\n            null_mut()\n        } else {\n            unsafe { load_icon_w(instance, IDI_CLIPTYPE_TRAY) }\n        };\n        if icon.is_null() {\n            icon = unsafe { load_icon_w(null_mut(), IDI_APPLICATION) };\n        }\n        if icon.is_null() {\n            return Err(TrayError::IconUnavailable);\n        }'''
    if old in text:
        text = text.replace(old, new, 1)
    elif "load_icon_w(instance, IDI_CLIPTYPE_TRAY)" not in text:
        raise RuntimeError("tray icon block was not found")
    path.write_text(text, encoding="utf-8")


def patch_readme() -> None:
    path = ROOT / "README.md"
    text = path.read_text(encoding="utf-8")
    if "assets/branding/cliptype-primary.svg" not in text:
        text = text.replace(
            "# ClipType\n",
            '# ClipType\n\n<p align="center"><img src="assets/branding/cliptype-primary.svg" alt="ClipType icon" width="180"></p>\n',
            1,
        )
    text = text.replace(
        "The first public release channel is `v0.1.0-beta.1` for Windows x86_64.",
        "The first public prerelease is `v0.1.0-beta.1` for Windows x86_64.",
    )
    text = text.replace(
        "- `keyboard` — bounded Unicode-oriented `SendInput` batches with target, modifier, cancellation, and partial-progress guards.",
        "- `keyboard` — per-action Unicode-oriented `SendInput` with target, modifier, cancellation, characters/s, jitter, and optional corrected-typo controls.",
    )
    text = text.replace(
        "- `auto` — freezes one proven backend per session from payload size and available capabilities.",
        "- `auto` — freezes one proven backend per session and prefers guarded clipboard paste for non-ASCII text when available.",
    )
    path.write_text(text, encoding="utf-8")


def patch_compatibility() -> None:
    path = ROOT / "docs/COMPATIBILITY.md"
    text = path.read_text(encoding="utf-8")
    text = text.replace(
        "ClipType `v0.1.0-beta.1` is the first public Windows x86_64 beta.",
        "ClipType `v0.1.0-beta.1` is the first public Windows x86_64 prerelease.",
    )
    marker = "## Application compatibility\n"
    note = """## Prerelease evidence boundary\n\nThe P2-B02 automated candidate includes per-action pacing, CJK-oriented Auto routing, corrected typo simulation, and the branded Windows resources. Issues #33 and #41 retain the post-fix physical/client observations. Publication as a GitHub prerelease does not convert hosted-runner evidence into a universal per-application claim.\n\n"""
    if note not in text and marker in text:
        text = text.replace(marker, note + marker, 1)
    path.write_text(text, encoding="utf-8")


def patch_release_doc() -> None:
    path = ROOT / "docs/RELEASE.md"
    text = path.read_text(encoding="utf-8")
    section = """\n## Windows icon resources\n\nThe `cliptype` binary embeds deterministic multi-frame application and tray icons during native Windows builds. Release validation must fail if resource compilation fails; a public package must not silently ship the stock Windows icon. The tray shell loads resource id `2`, while resource id `1` is the executable icon displayed by Explorer and shortcuts.\n"""
    if "## Windows icon resources" not in text:
        text = text.rstrip() + "\n" + section
    path.write_text(text, encoding="utf-8")


def patch_workflow_paths() -> None:
    for path in (ROOT / ".github/workflows").glob("*.yml"):
        if path.name == "branding-release-prep.yml":
            continue
        text = path.read_text(encoding="utf-8")
        if "paths:" not in text or "apps/cliptype/build.rs" in text:
            continue
        lines = text.splitlines()
        output: list[str] = []
        inserted = False
        for line in lines:
            output.append(line)
            if not inserted and line.strip() == "paths:":
                indent = line[: len(line) - len(line.lstrip())] + "  "
                output.extend(
                    [
                        f'{indent}- "assets/branding/**"',
                        f'{indent}- "apps/cliptype/build.rs"',
                    ]
                )
                inserted = True
        path.write_text("\n".join(output) + "\n", encoding="utf-8")


def main() -> None:
    (ROOT / "apps/cliptype/build.rs").write_text(BUILD_RS, encoding="utf-8")
    (ROOT / "assets/branding/README.md").write_text(BRANDING_README, encoding="utf-8")
    (ROOT / "docs/releases/v0.1.0-beta.1.md").write_text(RELEASE_NOTES, encoding="utf-8")
    patch_cargo()
    patch_tray()
    patch_readme()
    patch_compatibility()
    patch_release_doc()
    patch_workflow_paths()

    for relative in [
        "assets/branding/.release-prep-probe",
        ".github/workflows/p2-b02-apply.yml",
        ".github/scripts/p2_b02_patch.py",
        ".github/workflows/branding-release-prep.yml",
        ".github/scripts/branding_release_prep.py",
    ]:
        path = ROOT / relative
        if path.exists():
            path.unlink()


if __name__ == "__main__":
    main()
