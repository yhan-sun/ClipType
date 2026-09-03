# Technology Strategy

## Primary language: Rust

ClipType's core, orchestration, system adapters, and shared settings presentation are implemented primarily in **Rust**.

Reasons:

- strong FFI and low-level OS API access;
- memory safety for a long-running process handling sensitive text;
- predictable footprint without a managed runtime requirement;
- good support for Windows bindings, Unix APIs, DBus/Wayland/X11 ecosystems, and C/Objective-C interop;
- ability to localize `unsafe` around native APIs while presenting safe internal contracts.

The project does not select Go or C++ as the primary implementation language. Platform-specific auxiliary code MAY use another language only where native platform quality materially benefits and the boundary is small and documented.

## Native integration policy

'Native' means:

- native OS clipboard/input/focus/permission APIs;
- compiled desktop binaries, not Electron or an embedded WebView;
- no browser automation as the injection mechanism;
- platform-appropriate tray/menu-bar/settings/onboarding surfaces;
- installation, permission, login-item, signing, and notarization behavior that follows OS conventions.

A native-compiled cross-platform settings toolkit does not imply that every control is an operating-system-native widget. ClipType states that distinction explicitly.

## P3 UI strategy

ADR-0009 selects a shared `cliptype-ui` crate using **Slint `=1.17.1`** for the Windows and macOS settings window.

- Slint markup and Rust callbacks compile to native machine code.
- No HTML, JavaScript, Electron, Chromium, or system WebView is part of the settings window.
- Product policy remains in `cliptype-core` and `cliptype-app`; UI callbacks invoke typed application services.
- Windows keeps its native Win32 notification-area shell.
- macOS uses an AppKit `NSStatusItem`/`NSMenu` shell and platform Accessibility/login-item adapters.
- The settings UI is required to support keyboard navigation, accessibility metadata, focus indicators, dark/light themes, DPI/Retina scaling, and content-free diagnostics.

The distributed desktop binaries use the Slint Royalty-free Desktop, Mobile, and Web Applications License 2.0. The top-level About screen includes the required `AboutSlint` attribution widget, and release dependency inventories record the selected Slint version/license. The project does not distribute the complete application under GPLv3 merely to consume the UI dependency.

A future move to separate WinUI and SwiftUI settings implementations requires a superseding ADR and evidence that the maintenance cost is justified by measured platform-quality gaps.

## Settings surface

The P3 graphical settings window provides:

- General — enabled, notifications, start at login;
- Shortcuts — local Trigger/Cancel recorders, static validation, platform registration probe, Apply/Reset, and rollback status;
- Typing — Keyboard/Clipboard/Auto, exact characters per second, jitter, corrected typo probability, and Auto threshold;
- Permissions — macOS Accessibility status and fixed remediation;
- About & Updates — version/channel, release notes, licenses, dependency notices, and Slint attribution.

The recorder receives key events only while its control has local focus. It is not a global keylogger and does not install a broad low-level keyboard hook.

## OS binding policy

Prefer official APIs and maintained bindings:

- Windows: `windows-sys`/windows-rs style bindings to Win32 APIs.
- macOS: maintained Rust Objective-C bindings where mature; otherwise narrow Objective-C/CoreFoundation/CoreGraphics/ApplicationServices FFI wrappers with explicit ownership and thread invariants.
- Linux: maintained X11/Wayland/DBus/libevdev bindings as appropriate.

Direct shelling out to `xdotool`, `wtype`, `wl-paste`, `ydotool`, PowerShell, AppleScript, etc. is useful for research and diagnostics but is not the default production architecture.

## macOS technology boundary

The production mechanism is validated by P3-S01 before adapter freeze:

- `NSPasteboard.general` and content-blind `changeCount` for current text/revision evidence;
- Core Graphics `CGEvent` for bounded Unicode/key input and a balanced Command+V chord;
- frontmost-process and Accessibility focused-element identity without reading field contents;
- `AXIsProcessTrustedWithOptions` for explicit permission onboarding only;
- an OS global-hotkey registration mechanism that can probe and atomically replace a Trigger/Cancel pair without unrelated-key capture;
- AppKit status-item/menu lifecycle;
- `SMAppService.mainApp` for supported start-at-login behavior;
- Universal 2 `.app` packaging, hardened runtime, Developer ID Application signing, notarization, and stapling for public distribution.

Unsigned CI candidates remain clearly separated from signed/notarized public artifacts. Apple credentials are never committed to the repository.

## Async/runtime policy

Do not introduce a full async runtime until a concrete need exists. Native desktop event loops and bounded channels are the current design. Slint/AppKit/Win32 main-thread and message-loop constraints remain explicit, while bounded clipboard/input work stays off presentation loops.

## Configuration format

Human-editable configuration uses TOML. Configuration has a versioned semantic model; unknown/invalid security-sensitive values fail explicitly. P3 migrates preset-only shortcuts to native-neutral canonical Trigger/Cancel specifications.

## Logging

Use structured, content-free logging with strict field allowlists. No log API may accept clipboard plaintext, typed plaintext, focused content, window titles, recorded-key history, or persistent content fingerprints as a normal field.

## Packaging direction

- Windows: versioned ZIP/portable executable now; trusted Authenticode signing remains a separate credential boundary.
- macOS: Universal 2 signed/notarized `.app` plus ZIP and/or DMG after macOS adapters and physical evidence stabilize.
- Linux: distro-neutral archive first; native packages/AppImage/Flatpak are evaluated after backend support is stable. Packaging must not imply unsupported Wayland capabilities.

## Dependency evaluation checklist

Before adding a dependency, evaluate:

1. license compatibility and attribution obligations;
2. maintenance/activity and exact version pinning for release-critical dependencies;
3. transitive dependency size;
4. unsafe/FFI surface;
5. platform coverage;
6. security history;
7. whether the standard library/official API already suffices;
8. impact on binary size/startup/packaging;
9. event-loop and main-thread ownership;
10. effect on accessibility, theme behavior, and release signing.
