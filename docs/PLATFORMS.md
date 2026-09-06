# Platform Backend Design

This document records current native mechanisms and known platform constraints. Exact crate choices may change without an ADR; changing a mechanism, security boundary, or compatibility promise requires the repository decision process.

## Windows

Windows x86_64 is the primary beta platform. The historical P1 sequencing remains in [`phases/P1_WINDOWS_VERTICAL_SLICE.md`](phases/P1_WINDOWS_VERTICAL_SLICE.md); current product behavior includes later P2/P3 hardening.

### Clipboard

Current plain text is read through Win32 clipboard APIs using `CF_UNICODETEXT` semantics. Native clipboard handles are borrowed from the OS; the adapter validates allocation bounds, copies UTF-16 data into owned memory while the clipboard is open, then unlocks/closes promptly.

Rules:

- acquire current text only after an explicit trigger/session reservation and destination capture;
- retry transient busy state only within bounded attempt/time budgets;
- do not persist/cache plaintext;
- do not implement clipboard history;
- use a content-blind sequence number as revision evidence;
- never write, clear, replace, own, or restore the user's clipboard for current Clipboard mode.

### Keyboard injection

Windows uses Win32 `SendInput` with Unicode/text-oriented events and explicit special-key events where required.

The product handles semantic actions covering:

- ASCII and punctuation;
- CJK;
- combining marks;
- supplementary Unicode represented through UTF-16 units;
- normalized line breaks;
- Tab;
- Backspace for explicit corrected-typo behavior;
- Code-mode cursor/navigation actions.

Keyboard/Code delivery is bounded and paced per action. Native event counts may prove complete, none, or partial insertion without proving an exact semantic-text prefix. Partial/progress-unknown results are terminal and not automatically retried.

### Clipboard paste

Clipboard mode uses one balanced `Ctrl+V` chord after target, integrity, modifier, cancellation, and expected revision checks. A changed or unavailable required revision fails closed. Because ClipType never performs a clipboard write/restore transaction, it cannot overwrite a newer external value while trying to restore an older snapshot.

### Modifier state

`SendInput` does not reset physical keyboard state. ClipType uses bounded pre-dispatch settling and later conflicting-modifier checks. It observes Ctrl/Alt/Shift/Windows state but never releases arbitrary physical user keys to make injection succeed.

### UIPI

`SendInput` is subject to Windows User Interface Privilege Isolation. A normal-integrity process cannot inject into a higher-integrity target.

Reporting separates evidence from inference:

- reliably proven higher-integrity relation => known security-boundary restriction;
- unknown integrity plus zero accepted input => blocked/native-cause-unknown;
- zero accepted events are not automatically labelled UIPI;
- ClipType does not auto-elevate or circumvent the boundary.

### Target/focus evidence

Use the strongest practical non-content combination of:

- foreground top-level window;
- owning process and GUI thread;
- active/focused native window evidence exposed by GUI-thread inspection;
- optional integrity relationship.

Do not read focused text or log window titles. Detailed original focus evidence that later degrades fails closed. Some applications host multiple logical fields inside one render surface, so ClipType does not promise exact logical-field/caret identity where the OS does not expose it.

### Hotkeys and message loop

Use `RegisterHotKey` / `WM_HOTKEY` with no-repeat behavior rather than a low-level global keyboard hook.

The registering thread owns registration, message queue, live replacement, and teardown. Candidate Trigger/Cancel pairs are validated and probed through temporary registrations. Replacement is transactional: candidate registrations are secured before old registrations are removed where the API permits; failures clean up candidates and keep/restore the previous working pair.

The message-loop owner remains responsive while injection work runs on a separate bounded worker.

### Tray, settings, and startup

The Win32 product shell provides notification-area menu/status, content-free notifications, settings controls, trigger/cancel, and controlled shutdown. Start-at-login uses one product-owned current-user Run value and requires no service or elevation.

### Windows release support

`v0.2.0-beta.8` publishes Windows x86_64 ZIP and portable EXE assets. Windows Server 2022/2025 hosted jobs are CI mechanism references. Windows 11 x64 is the recommended client; Windows 10 22H2 x64 is best effort. Representative physical/named-application evidence remains #33 and is not inferred from CI.

## macOS

The current P4 product line is Apple Silicon arm64 only. `apps/cliptype-flutter` is the sole macOS settings/front-end composition root, with one Flutter engine and one process. Swift/AppKit owns native shell mechanisms and a fixed content-free bridge to Rust.

No Intel, Rosetta, or Universal 2 support/artifact is claimed by P4.

### Clipboard

Use `NSPasteboard.general` for bounded current text access and `changeCount` as the content-blind revision witness. The Rust-triggered session owns plaintext lifetime; clipboard text never crosses Flutter/Swift status channels.

Clipboard mode uses the user's existing pasteboard plus revision guarding. It does not write/restore the pasteboard or create a second clipboard store.

### Keyboard injection

Use Core Graphics `CGEvent` facilities for bounded synthetic keyboard/text events. Rust owns plan/pacing/safety policy; Swift/AppKit does not independently inject product text.

Code navigation uses explicit bounded key actions. Synthetic event state is separated from physical HID-system modifier observation so ClipType's own generated modifier flags do not contaminate the physical-modifier safety gate. Physical modifiers are never released on the user's behalf.

Native/mock/Swift contract gates cover this mechanism, but a real VS Code/Monaco session remains physical evidence in #61.

### Permissions

Cross-application synthetic input requires user-granted Accessibility permission. The app exposes content-free permission state and explicit remediation. It never bypasses consent or loops hidden prompts. Persistent grant/revoke behavior must be verified on a physical user desktop.

### Focus/target

Use workspace/accessibility/window APIs only for target identity and permission-safe evidence; never read focused text, selected text, DOM values, document content, or window titles.

Native controls use exact focused-element identity. An initial `AXWebArea` render-host classification selects a sticky process + focused-window session policy to tolerate legitimate same-window Monaco focus-node rebuilding. Process/window change, stable-window disappearance, or required evidence loss still stops. Logical-field movement hidden inside one shared render host remains a documented limitation.

### Global shortcuts and shell

Swift/AppKit owns one `NSStatusItem`, native menu commands, system Trigger/Cancel registration, `SMAppService`, and Flutter window lifecycle. The Flutter shortcut recorder captures only while its local control owns focus. Candidate pairs are validated/probed/applied transactionally; there is no general event tap/keylogger.

### Distribution

`v0.2.0-beta.8` carries a clearly labelled additive macOS arm64 testing preview:

- Flutter arm64 `.app` packaged as ZIP and DMG;
- ad-hoc signature only;
- exact-main Apple Silicon CI build/install/launch smoke;
- arm64-only Mach-O verification;
- SHA-256 manifest and build metadata;
- additive upload only after the exact Windows-created prerelease/tag exists;
- public re-download and byte/checksum verification.

The preview is not Developer ID signed, notarized, stapled, Gatekeeper-approved, Intel/Rosetta, Universal 2, or a broad named-application compatibility claim. Physical Apple Silicon behavior and any future trusted distribution promotion remain #61.

## Linux X11

Linux is not shipped in the current beta.

Potential X11 mechanisms remain:

- current selection acquisition without a plaintext history service;
- XTEST/native input with an explicit Unicode/keymap compatibility matrix;
- focus/window identity where available;
- explicit-trigger/privacy policy despite X11's broad client interaction model.

No X11 support claim exists until a dedicated implementation and evidence gate is completed.

## Linux Wayland

Wayland is not one uniform backend and is not shipped in the current beta. Any future implementation must probe capabilities independently rather than setting one global support bit.

Potential capabilities may include:

- clipboard read/write mechanisms exposed by compositor protocols or portals;
- global trigger availability;
- virtual keyboard or carefully scoped `uinput` delivery;
- focus evidence.

A privileged helper, if ever required for a specific Linux capability, must be local, minimal, capability-scoped, and must not become a clipboard store or general root daemon.

## Platform fallback policy

Fallback is planner-visible and capability-safe. Auto may select a proven alternate backend according to policy. An explicit Keyboard, Clipboard, or Code request fails clearly if its required capability is unavailable; it never silently changes the user's requested semantics.

Fallback does not cross a security boundary or automatically launch a privileged external command.

## Research/reference APIs

See `REFERENCES.md` for official API/protocol documentation and reference projects.
