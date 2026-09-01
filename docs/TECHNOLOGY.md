# Technology Strategy

## Primary language: Rust

ClipType's core, orchestration, and system adapters will be implemented primarily in **Rust**.

Reasons:

- strong FFI and low-level OS API access;
- memory safety for a long-running process handling sensitive text;
- predictable footprint without a managed runtime requirement;
- good support for Windows bindings, Unix APIs, DBus/Wayland/X11 ecosystems, and C/Objective-C interop;
- ability to localize `unsafe` around native APIs while presenting safe internal contracts.

The project does not select Go or C++ as the primary implementation language. Platform-specific auxiliary UI/helper code MAY use another language only where native platform quality materially benefits and the boundary is small and documented.

## Native integration policy

'Native' means:

- native OS clipboard/input/focus/permission APIs;
- no Electron runtime;
- no browser automation as the injection mechanism;
- platform-appropriate tray/settings/onboarding surfaces;
- installation/permission behavior that follows OS conventions.

## UI strategy

ClipType is tray-first, not window-first. V1 UI surface is intentionally small:

- tray/menu status;
- enable/disable;
- trigger/hotkey configuration;
- injection mode/speed;
- permission onboarding;
- diagnostics/status;
- quit.

The core MUST remain UI-toolkit independent.

Preferred platform presentation direction:

- Windows: native Win32/Windows App SDK-compatible shell where practical through Rust bindings or a very thin native shell.
- macOS: AppKit/SwiftUI-quality native shell; Rust may expose a narrow application API if a Swift shell is chosen.
- Linux: GTK4/libadwaita-quality settings shell where a graphical settings surface is needed; tray support follows desktop realities.

A single cross-platform rendered UI framework is not an architecture requirement. Toolkit selection is deferred until the Windows vertical slice proves the core integration.

## OS binding policy

Prefer official APIs and maintained bindings:

- Windows: `windows`/windows-rs style bindings to Win32 APIs.
- macOS: maintained Rust bindings where mature; otherwise narrow Objective-C/CoreFoundation/CoreGraphics FFI wrappers.
- Linux: maintained X11/Wayland/DBus/libevdev bindings as appropriate.

Direct shelling out to `xdotool`, `wtype`, `wl-paste`, `ydotool`, PowerShell, AppleScript, etc. is useful for research and diagnostics but is not the default production architecture.

## Async/runtime policy

Do not introduce a full async runtime until a concrete need exists. Native desktop event loops and bounded channels may be sufficient for early phases. If async is introduced, justify it with actual concurrent I/O/lifecycle requirements and keep platform-main-thread constraints explicit.

## Configuration format

Human-editable configuration will use TOML unless implementation evidence requires a change. Configuration has a versioned semantic model; unknown/invalid security-sensitive values fail explicitly.

## Logging

Use structured logging with strict field allowlists. The expected Rust ecosystem direction is `tracing`, but dependency selection occurs when implementation begins. No log API may accept clipboard plaintext as a normal field.

## Packaging direction

- Windows: signed installer/package after P2 hardening.
- macOS: signed/notarized `.app`/distribution artifact after macOS support stabilizes.
- Linux: distro-neutral archive first; native packages/AppImage/Flatpak are evaluated after backend support is stable. Packaging must not imply unsupported Wayland capabilities.

## Dependency evaluation checklist

Before adding a dependency, evaluate:

1. license compatibility;
2. maintenance/activity;
3. transitive dependency size;
4. unsafe/FFI surface;
5. platform coverage;
6. security history;
7. whether the standard library/official API already suffices;
8. impact on binary size/startup/packaging.