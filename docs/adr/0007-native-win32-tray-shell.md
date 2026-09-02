# ADR-0007: Native Win32 Tray-First Product Shell

- Status: Accepted
- Date: 2026-09-02

## Context

P1 proved the Windows clipboard, target, keyboard, hotkey, coordinator, and controlled-host lifecycle with a minimal console process. P2 must turn that vertical slice into a daily-usable Windows utility with persistent settings, visible status, startup-at-login, and packaging.

The user surface is intentionally small. A rendered cross-platform UI framework or daemon/client split would add runtime size, network-like IPC, lifecycle, dependency, and privacy surface before another platform needs it. The existing application already requires a Windows message-loop owner for global hotkeys.

## Decision

The Windows product uses one unprivileged Rust process with a native Win32 tray-first shell.

A hidden top-level/message window owned by the main thread is responsible for:

- the Windows message queue;
- global trigger and cancellation hotkey registration;
- taskbar/tray icon lifecycle through `Shell_NotifyIconW`;
- a native context menu;
- a small native settings window;
- content-free status notifications;
- session-end and controlled shutdown messages.

UI callbacks translate native commands into narrow application operations such as trigger, cancel, query status, validate/apply configuration, and shutdown. They do not implement injection policy or call clipboard/input FFI directly.

The injection coordinator and worker model remains the single live state machine. Clipboard reads and input dispatch never execute on the tray/message-loop owner.

## Settings model

P2 introduces one versioned human-editable TOML file at the platform-appropriate per-user local application-data directory, normally:

```text
%LOCALAPPDATA%\ClipType\config.toml
```

The initial implementation uses a strict, fixed-schema TOML subset implemented in the application crate rather than adding a serialization framework. It accepts only documented scalar keys and sections, rejects duplicate or unknown security-sensitive fields, validates all numeric bounds, and never stores clipboard text, target text, window titles, content fingerprints, or session history.

Writes use a temporary file plus flush/sync and a recoverable backup/replace sequence. On startup, a valid primary file wins; otherwise a valid backup may be recovered with an explicit content-free status. Invalid settings never produce unbounded or security-weakened defaults.

## Hotkeys

V1 product configuration exposes reviewed hotkey presets rather than recording arbitrary keyboard activity.

- Each preset maps to explicit Win32 modifier and virtual-key values.
- Registration uses `MOD_NOREPEAT` where supported.
- Trigger and cancel must be distinct.
- Reconfiguration unregisters the previous pair on the owning thread before registering the new pair.
- A conflict is visible and leaves the process controllable through the tray.
- No low-level global keyboard hook or general key capture is introduced.

Richer arbitrary hotkey capture remains a post-1.0 candidate unless a bounded native control is designed and reviewed.

## Startup at login

The per-user startup option is implemented through the current-user Windows Run registration. It stores only the quoted executable path and a stable product value name.

- No administrator access is requested.
- Disabling removes only ClipType's own value.
- The setting is reconciled with persistent configuration and surfaced as a content-free error if registry access fails.
- ClipType does not install a service, scheduled task, or elevated helper.

## Notifications and diagnostics

Tray text and notifications use a strict fixed vocabulary derived from typed outcomes. They may include mode, phase, counts, capability category, and remediation class, but never clipboard/injected plaintext, samples, hashes, focused content, or window titles.

The process installs a content-free panic hook for distribution builds. Raw memory dumps are neither collected nor uploaded by ClipType.

## Packaging

P2 initially produces a reproducible per-user Windows archive containing the executable, licenses, configuration reference, and install/uninstall PowerShell scripts. The scripts install under the current user's local programs directory, create/remove product-owned shortcuts and startup state, and require no elevation.

This development/beta artifact does not replace the signed-package requirement. Signing and public promotion remain release gates and require explicit maintainer action and secret configuration.

## Alternatives considered

### Windows App SDK or a broad Rust GUI framework

Deferred. It can improve visual polish, but the current surface does not justify a large runtime/dependency graph or a second event-loop abstraction.

### Electron/web UI

Rejected because it conflicts with the native/small-footprint direction and creates an unnecessary browser runtime for a tray utility.

### Console host as the product UI

Rejected for P2 because startup, status, settings, conflict remediation, and daily use would retain hidden developer steps.

### Background service plus UI client

Rejected under ADR-0005. No demonstrated Windows capability requires a second process or privilege boundary.

## Consequences

### Positive

- reuses the required Windows message loop;
- small dependency and runtime footprint;
- native tray/startup conventions;
- clear separation between shell commands and application policy;
- no new process, IPC, network, or elevated surface;
- auditable fixed-schema settings and diagnostics.

### Negative / trade-offs

- raw Win32 UI code requires localized unsafe wrappers and careful lifecycle tests;
- appearance is intentionally modest rather than framework-rich;
- platform presentation will not share one rendered UI implementation with macOS/Linux;
- arbitrary hotkey capture is deferred.

## Follow-up

P2 must add lifecycle, settings recovery, hotkey conflict, tray command, startup registration, packaging smoke, idle footprint, and privacy-sentinel evidence. Representative desktop/application compatibility remains separately gated before a public beta.