# Technology Strategy

## Primary language: Rust

ClipType's core, orchestration, and system adapters are implemented primarily in **Rust**.

Reasons:

- strong FFI and low-level OS API access;
- memory safety for a long-running process handling sensitive text;
- predictable footprint without a managed runtime requirement in the core/runtime;
- good support for Windows bindings and C/Objective-C/Swift interop;
- ability to localize `unsafe` around native APIs while exposing safe internal contracts.

Platform-specific auxiliary code may use another language/runtime where native platform quality materially benefits and the boundary is small and documented. The current macOS shell uses Swift/AppKit and Flutter only at the presentation/native-shell boundary.

## Native integration policy

“Native” means:

- native OS clipboard/input/focus/permission APIs;
- compiled desktop binaries, not Electron or an embedded WebView;
- no browser automation as the injection mechanism;
- platform-appropriate tray/menu-bar/settings/onboarding surfaces;
- installation, permission, startup, signing, and distribution behavior that follows OS conventions.

A compiled cross-platform UI toolkit does not imply every control is an operating-system-native widget. ClipType states that distinction explicitly.

## Windows UI strategy

ADR-0009 established `cliptype-ui` using Slint `=1.17.1`. ADR-0010/0012 superseded that decision **for macOS only**; the Slint crate remains a Windows presentation dependency until a separate Windows UI decision.

- Slint markup/Rust callbacks compile to native machine code.
- No HTML, JavaScript, Electron, Chromium, or system WebView is part of the Windows settings UI.
- Product policy remains in `cliptype-core` / `cliptype-app`; UI callbacks invoke typed application services.
- Win32 retains notification-area/message-loop/startup ownership.
- The Windows distributed binary carries the required Slint attribution/license inventory.

A future Windows UI replacement requires a superseding ADR and evidence that the maintenance/quality trade-off is worthwhile.

## macOS Apple Silicon Flutter strategy

ADR-0010 and ADR-0012 define the current macOS product line:

- `apps/cliptype-flutter` is the sole macOS settings/front-end composition root;
- Flutter 3.47.2 / Dart 3.13.2 are pinned in the authoritative P4 gate;
- target is `aarch64-apple-darwin` only;
- one process and one Flutter engine;
- Swift/AppKit retains the status item, native menu, Accessibility remediation, `SMAppService`, global shortcut ownership, application lifecycle, and fixed Flutter channels;
- `cliptype-flutter-bridge` exposes a narrow C ABI carrying bounded settings/commands/content-free states only;
- Rust owns validation, settings semantics, one-session coordination, backend selection, target/modifier/revision safety, pacing, cancellation, and terminal outcomes.

No clipboard text, injected text, target content, user identity, or recorded-key history crosses the Flutter/Swift status boundary.

Flutter controls are not represented as native AppKit widgets. The current public artifact is an **Apple Silicon arm64 testing preview**, not Intel/Rosetta/Universal 2 or a trusted Apple distribution.

## macOS native mechanisms

Current mechanisms are:

- `NSPasteboard.general` + content-blind `changeCount` for current text/revision evidence;
- Core Graphics `CGEvent` for bounded Unicode/key input and balanced Command+V;
- frontmost-process plus Accessibility focused-element/window identity without reading field values/titles;
- explicit `AXIsProcessTrustedWithOptions` onboarding/remediation only;
- OS global-shortcut registration with candidate probing and transactional replacement/rollback;
- AppKit `NSStatusItem` / `NSMenu` lifecycle;
- `SMAppService.mainApp` for supported start-at-login behavior.

Synthetic event state is kept separate from physical modifier evidence. Current Code-mode navigation and focus policies are documented in the accepted/superseding ADR chain and native contract tests.

## Current macOS distribution boundary

`v0.2.0-beta.8` publishes an additive arm64 testing preview built from the exact release commit. The P4 workflow:

- runs native Code/Swift contracts, full Rust quality gates, Flutter format/analyze/test/build;
- verifies arm64-only Mach-O slices;
- ad-hoc signs, installs, and launch-smokes `/Applications/ClipType.app`;
- produces ZIP/DMG plus checksum/build metadata;
- after the Windows prerelease/tag exists at the same exact SHA, uploads assets additively;
- re-downloads published files and verifies bytes/checksums.

Developer ID, Hardened Runtime promotion policy, notarization, stapling, and clean-machine Gatekeeper verification are not configured/proven by this testing-preview path. Those remain #61 if a normal trusted macOS beta is pursued.

No Intel/Rosetta/Universal 2 artifact is planned or claimed for the current P4 line.

## Settings surfaces

### Windows

The Windows product exposes enabled state, notifications, start at login, mode, exact characters-per-second, jitter, corrected-typo probability, Auto threshold, and validated Trigger/Cancel shortcuts through its native-compiled settings/tray surfaces.

### macOS

The Flutter shell uses task-oriented Overview, Input, Shortcuts, System, and About surfaces. Shortcut candidates are local UI state until native validation/probing/replacement succeeds. Accessibility remediation is explicit. Runtime build identity is content-free.

## OS binding policy

Prefer official APIs and maintained bindings:

- Windows: `windows-sys`/windows-rs style bindings to Win32 APIs;
- macOS: Swift/AppKit/Foundation/CoreGraphics/Accessibility/ServiceManagement plus narrow Rust FFI where needed;
- Linux (future): maintained X11/Wayland/DBus/libevdev bindings appropriate to actual capability decisions.

Shelling out to `xdotool`, `wtype`, `wl-paste`, `ydotool`, PowerShell, AppleScript, etc. may be useful for research/packaging diagnostics but is not the default production input architecture.

## Async/runtime policy

Do not introduce a full async runtime until a concrete need exists. Native event loops, bounded workers, atomics/channels/condition variables, and explicit main-thread ownership are the current design. Clipboard/input work remains off UI/message-loop owners.

## Configuration format

Human-editable product configuration uses a strict versioned semantic model. Unknown/invalid security-sensitive values fail explicitly without echoing plaintext. Shortcut settings use native-neutral canonical Trigger/Cancel specifications.

## Logging

Use structured, content-free logging with strict field allowlists. No log API may accept clipboard plaintext, typed plaintext, focused content, window titles, recorded-key history, or persistent content fingerprints as a normal field.

## Packaging direction

- Windows: versioned ZIP + portable EXE; SHA-256, Sigstore keyless signatures, GitHub attestations; trusted Authenticode remains a separate credential boundary.
- macOS: arm64-only ad-hoc-signed testing-preview ZIP/DMG today; any trusted public promotion requires separate Developer ID/notarization evidence in #61.
- Linux: no shipped backend yet; packaging must follow actual capability evidence and must not imply unsupported Wayland behavior.

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
