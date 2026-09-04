# ADR-0009: Shared Slint settings UI with native platform shells

- Status: Superseded by ADR-0010
- Date: 2026-09-03
- Scope: P3 cross-platform settings UI, custom hotkeys, and macOS productization
- Related: #43, #44, #45, #46, #47, #48, #49, #50, #51

## Context

The Windows `v0.1.0-beta.1` product is tray-first and exposes only bounded menu controls plus a manually editable configuration file. That surface is insufficient for:

- recording arbitrary Trigger and Cancel shortcuts;
- showing static validation and operating-system registration conflicts;
- applying a complete shortcut pair transactionally with rollback;
- exposing exact characters-per-second, jitter, typo probability, and Auto threshold values;
- onboarding macOS Accessibility permission;
- showing platform capability and release-signature status;
- providing an accessible, keyboard-navigable settings experience on Windows and macOS.

ClipType must remain a single normal user process with a Rust policy/orchestration core. Native clipboard, input, focus, permission, global-hotkey, menu-bar/tray, login-item, packaging, signing, and notarization behavior must remain platform adapters. The UI must not own injection policy or sensitive text.

The project also requires a practical implementation path for one maintainer. Maintaining a full WinUI implementation and a separate SwiftUI implementation would provide the closest platform-widget fidelity, but would duplicate recorder, validation, settings, accessibility metadata, and transaction presentation logic.

## Decision

### 1. Shared settings window

Create `crates/cliptype-ui` in Rust using **Slint `=1.17.1`**, pinned exactly for the P3 release line.

The distributed desktop binaries use the **Slint Royalty-free Desktop, Mobile, and Web Applications License 2.0**, not the GPLv3 option. ClipType will satisfy its attribution condition by:

- exposing a top-level **About ClipType** screen containing the Slint `AboutSlint` attribution widget; and
- recording the selected Slint license and version in the dependency/release inventory.

The settings UI is compiled to native machine code. It does not embed Chromium, Electron, a WebView, HTML, or JavaScript. This decision does **not** describe Slint controls as operating-system-native widgets; platform-native shell responsibilities remain separate.

### 2. Native shell responsibilities

Windows retains the existing native Win32 notification-area implementation. macOS uses an AppKit menu-bar shell with one retained `NSStatusItem` and native `NSMenu`.

Platform shells own only:

- tray/status-item lifecycle and native menus;
- opening, hiding, and reactivating the shared settings window;
- global shortcut registration and command delivery;
- Accessibility permission presentation hooks on macOS;
- start-at-login integration;
- app activation policy, bundle lifecycle, and controlled shutdown;
- platform branding and packaging.

The shared settings window owns presentation and local recorder interaction, but calls typed application services for validation, probing, applying, persistence, rollback, and capability state.

### 3. Process and event-loop model

ClipType remains one process per logged-in user.

#### Windows

- The Slint/winit settings event loop owns the UI thread when the settings window is open or resident.
- The Win32 tray owner and the `RegisterHotKey`/`WM_HOTKEY` command source retain dedicated message-loop ownership.
- Injection work remains on the bounded coordinator worker.
- Typed channels bridge UI, tray, command registration, and application services.
- Opening or closing Settings does not terminate the tray application or create duplicate command registrations.

#### macOS

- The main thread owns `NSApplication`, AppKit status/menu lifecycle, and the Slint settings event loop integration.
- Accessibility prompting and UI state transitions occur on the main thread.
- Global command registration is integrated with the native application run loop according to the evidence from P3-S01.
- Clipboard, target evidence, and synthetic input calls are made through bounded adapters and workers without blocking the main run loop.
- Closing Settings hides the window; Quit performs bounded cancellation and native teardown.

### 4. Crate and composition graph

```text
cliptype-core
  - HotkeySpec/HotkeyPair and validation
  - versioned settings and pure product policy

cliptype-platform
  - hotkey probe/apply contracts
  - clipboard, target, input, modifier, paste, permission capabilities

cliptype-app
  - settings transactions and rollback
  - coordinator and immutable active-session snapshots

cliptype-ui
  - Slint components
  - content-free settings view model and local shortcut recorder state

cliptype-windows                 cliptype-macos
  - Win32 adapters                 - AppKit/CoreGraphics/AX adapters
  - tray/hotkey/startup            - status item/hotkey/login item

apps/cliptype                   apps/cliptype-macos
  - Windows composition root       - macOS `.app` composition root
```

`cliptype-ui` may depend on `cliptype-core` and a narrow UI-facing application contract. It must not depend directly on `cliptype-windows` or `cliptype-macos`. Platform-native keycodes, handles, Accessibility objects, and clipboard objects must not enter UI state or configuration.

### 5. Custom shortcut behavior

The UI exposes separate Trigger and Cancel recorder controls.

- Recording observes only local key events while the recorder control owns focus.
- No low-level global hook, general keylogger, or unrelated-key history is introduced.
- Escape cancels recording; Backspace/Delete clears a candidate.
- Core performs deterministic structural and reserved-combination validation.
- The platform adapter performs an actual temporary OS registration probe.
- The complete candidate pair is applied atomically: both new shortcuts must be secured before the previous pair is released.
- Any failure unregisters temporary candidates, keeps the old pair active, and prevents persistence.
- Availability is reported as `Available`, `Conflict`, `Reserved`, `Unsupported`, or `Unknown`/`Cannot fully verify`.
- Successful OS registration proves only that the operating system granted the global registration. It cannot prove that every foreground application, accessibility tool, or hook-based utility will not also react.

Windows F12 is removed as a recommended default. Legacy preset configurations migrate to reviewed non-F12 values.

### 6. Settings window structure

The initial P3 window contains:

1. **General** — enabled, notifications, start at login.
2. **Shortcuts** — Trigger/Cancel recorders, static validation, OS probe status, Reset and Apply.
3. **Typing** — Keyboard/Clipboard/Auto, characters per second, jitter, corrected typo probability, Auto threshold, safety warnings.
4. **Permissions** — macOS Accessibility state and explicit remediation; platform capability summary.
5. **About & Updates** — version/channel, release notes, project licenses, dependency notices, Slint attribution.

Apply is transactional. An active injection session keeps its immutable settings snapshot. Clipboard text, target content, window titles, raw native handles, and recorded key history never enter UI diagnostics, screenshots, or persistence.

### 7. macOS product boundary

The first macOS release target is `v0.2.0-beta.1` and is not retroactively part of the Windows `v0.1.0-beta.1` claim.

The planned native mechanisms are:

- `NSPasteboard.general` with content-blind `changeCount` revision evidence;
- Core Graphics `CGEvent` for bounded Unicode/key events and one balanced Command+V chord;
- frontmost process plus Accessibility focused-element identity without reading values, selected text, document content, or window titles;
- `AXIsProcessTrustedWithOptions` only after an explicit user action, with denied/granted/revoked states and no consent bypass;
- `SMAppService.mainApp` for user-controlled start at login on supported macOS versions;
- AppKit menu-bar shell, `.icns` app branding, and a monochrome template status-item glyph;
- arm64 and x86_64 slices assembled as Universal 2;
- hardened runtime, Developer ID Application signing, notarization, and ticket stapling for public macOS distribution.

Unsigned macOS CI candidates are clearly separated from signed/notarized public artifacts. Public macOS distribution is blocked until maintainer-owned Apple signing/notarization credentials are configured in a protected release environment.

## Alternatives considered

### Separate WinUI 3 and SwiftUI settings applications

**Benefits:** closest use of platform-native widgets, conventions, and accessibility stacks.

**Rejected for P3:** duplicates substantial view-model and recorder behavior; introduces C#/C++ and Swift build/tooling boundaries; complicates settings transaction parity and release maintenance. This remains a possible future replacement if Slint cannot meet measured accessibility or platform-integration requirements.

### Tauri or another system-WebView shell

**Benefits:** broad UI ecosystem and fast layout iteration.

**Rejected:** introduces HTML/JavaScript and a WebView runtime boundary for a small security-sensitive settings surface; conflicts with the project's native-compiled/no-browser-runtime direction.

### Expand the existing tray menus only

**Benefits:** smallest dependency and implementation footprint.

**Rejected:** cannot provide a usable shortcut recorder, conflict/rollback presentation, exact numeric settings, permission onboarding, accessible validation feedback, or complete About/license surface.

## Consequences

### Positive

- One settings implementation for Windows and macOS.
- Rust remains the dominant implementation language.
- No embedded browser runtime.
- Shortcut recorder, validation, error states, and settings transactions remain behaviorally consistent.
- Platform-native tray/menu-bar, permission, input, and lifecycle behavior remains independently testable.
- The UI dependency and its attribution requirement are explicit rather than hidden.

### Negative / trade-offs

- Slint widgets are rendered cross-platform controls, not native AppKit/WinUI widgets.
- Slint introduces a non-standard royalty-free license obligation and mandatory attribution.
- AppKit/Slint main-run-loop integration must be proven by the macOS spike.
- UI accessibility, keyboard navigation, IME behavior, DPI/Retina, and dark/light rendering require platform tests rather than assumption.
- A future toolkit change requires a superseding ADR.

## Follow-up

1. #45 adds native-neutral hotkey contracts and settings migration.
2. #46 implements `cliptype-ui` and deterministic recorder/view-model tests.
3. #47 implements Windows temporary probing and atomic live re-registration.
4. #48 validates macOS permission, Unicode, focus, global-hotkey, status-item, and event-loop behavior on a real Mac.
5. #49 implements production macOS adapters only after #48 returns an explicit YES.
6. #50 builds the macOS menu-bar app and settings integration.
7. #51 adds Universal 2 packaging and gated Developer ID signing/notarization.
8. The P3 release gate requires exact-SHA Windows and macOS interactive evidence and a `CROSS_PLATFORM BETA READY` or `NOT READY` conclusion.
