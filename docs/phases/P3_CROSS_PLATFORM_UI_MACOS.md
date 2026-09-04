# P3 — Cross-platform Settings UI, Custom Hotkeys, and macOS Productization

## Status

- Phase: active architecture/contracts
- Target prerelease: `v0.2.0-beta.1`
- Planned platforms: Windows x86_64 and macOS Universal 2
- Entry commit: `a7132f4175b06567e954e871f96165d4831bc4e3`
- Parent tracker: #43

The existing `v0.1.0-beta.1` remains a Windows-only prerelease. P3 does not retroactively claim macOS support.

The repository now uses a P4 local Apple Silicon Flutter composition root as
the only macOS settings/front-end entry point. The legacy Rust/Slint macOS
composition root was removed; the shared Slint crate remains a Windows-only
presentation dependency. The broader P3 Universal 2, signing/notarization,
and physical evidence gate remains open. See
[P4 macOS Apple Silicon local runner](P4_MACOS_ARM64_LOCAL.md).

## User outcomes

P3 is complete only when a user can:

1. open a graphical Settings window from the Windows tray or macOS menu bar;
2. record distinct Trigger and Cancel shortcuts without editing TOML;
3. see whether a candidate is invalid, reserved, unsupported, already registered, available, or not fully verifiable;
4. apply the complete shortcut pair live without losing the previous working pair on failure;
5. configure mode, exact characters per second, jitter, corrected typo probability, Auto threshold, notifications, and start at login;
6. understand and grant/revoke macOS Accessibility permission through an explicit system-controlled flow;
7. use Keyboard, Clipboard, and Auto modes in a normal macOS desktop application;
8. install a properly bundled macOS application and verify its signing/notarization status.

## Architecture baseline

ADR-0009 records the original UI/process-model decision. ADR-0010 and ADR-0012
define the current macOS replacement:

```text
cliptype-core          native-neutral policy and settings
cliptype-platform      native-neutral ports and hotkey outcomes
cliptype-app           coordinator and settings transactions
cliptype-ui            shared Slint settings window/view model (Windows)
cliptype-windows       Win32 tray/hotkey/input adapters
cliptype-macos         AppKit/CoreGraphics/AX adapters
apps/cliptype          Windows composition root
apps/cliptype-flutter  macOS Flutter settings/menu-bar composition root
```

The Windows UI is native-compiled and the macOS UI is Flutter desktop; neither
contains Electron or a WebView. Native shell, permission, global-hotkey, input,
clipboard, target, startup/login-item, and packaging behavior remains
platform-specific.

## Custom shortcut contract

### Recorder

- receives only local key events while the recorder control is focused;
- Escape cancels recording;
- Backspace/Delete clears a candidate;
- never installs a general keyboard hook or stores key history;
- renders platform-appropriate labels from native-neutral values.

### Validation

- requires at least one primary modifier and one non-modifier key;
- rejects bare and Shift-only values;
- rejects identical Trigger and Cancel values;
- rejects known OS-reserved/unsafe combinations;
- rejects Windows F12 as a custom/recommended shortcut;
- keeps configuration independent from current keyboard layout and native keycodes.

### Probe and apply

```text
validate pair
  -> probe/register candidate Trigger temporarily
  -> probe/register candidate Cancel temporarily
  -> on failure: remove candidates, keep old pair, do not persist
  -> on success: commit new pair, release old pair, persist settings
```

Status values are:

- Available;
- Conflict/In use;
- Reserved;
- Unsupported;
- Unknown/Cannot fully verify.

A successful OS registration probe does not prove that every application-local shortcut or hook-based tool will remain silent.

## Settings window

### General

- Enabled;
- Notifications;
- Start at login.

### Shortcuts

- Trigger recorder;
- Cancel recorder;
- static and OS probe status;
- Reset;
- automatic persistence status;
- visible rollback/error state.

### Typing

- Keyboard / Clipboard / Auto;
- characters per second;
- jitter percentage;
- corrected typo probability;
- Auto clipboard threshold;
- safety warning for typo mode and terminal/exact-data use.

### Permissions

- macOS Accessibility state;
- explicit Request/Open System Settings action;
- fixed content-free remediation;
- platform capability summary.

### About & Updates

- current version and channel;
- release notes and issue tracker;
- project licenses and dependency notices;
- required Slint attribution for the Windows UI; the macOS Flutter UI has its
  own dependency and license surface.

## macOS product mechanisms

### Clipboard and paste

- bounded current plain-text acquisition through `NSPasteboard.general`;
- content-blind `changeCount` revision witness;
- changed-during-read detection;
- one balanced Command+V chord only after revision and destination checks;
- no pasteboard history, clear, rewrite, ownership transaction, or restore.

### Keyboard and modifiers

- bounded Core Graphics Unicode/key events;
- ASCII, CJK, Japanese, Korean, emoji, combining marks, newline, Tab, Backspace;
- the same per-action pacing, jitter, typo correction, cancellation, focus, and partial/unknown semantics as Windows;
- no release of physical user modifiers.

### Destination evidence

- frontmost process identity;
- focused Accessibility element identity when permission is available;
- compare same/changed/disappeared/ambiguous/degraded;
- no focused value, selection, document content, or window-title reads;
- fail closed when strict evidence is lost.

### Permissions

- use system Accessibility trust APIs;
- prompt only after explicit user action;
- represent not requested, not granted, granted, and revoked while running;
- no prompt loop, synthetic consent, or bypass.

### Shell and login item

- retained AppKit `NSStatusItem` and native `NSMenu`;
- one settings window, hidden on close;
- menu-bar-first activation with no mandatory Dock presence while idle;
- `SMAppService.mainApp` status/register/unregister on supported macOS versions;
- bounded quit/restart and no duplicate status items or hotkeys.

## Execution order

### Wave 0 — architecture and contracts

1. #44 — UI/process-model and license ADR.
2. #45 — native-neutral hotkey values, availability/apply results, and configuration migration.

### Wave 1 — Windows proof path

3. #46 — shared graphical settings window and recorder/view-model tests.
4. #47 — Windows temporary registration probe and live atomic pair replacement.

#46 and #47 may proceed in parallel after #45 contracts are merged, then integrate on one reviewed base.

### Wave 2 — macOS evidence and adapters

5. #48 — real macOS native mechanism spike.
6. #49 — production adapters only after #48 explicitly returns `P3 macOS production adapters may proceed: YES`.

### Wave 3 — shell and release automation

7. #50 — macOS menu-bar application, permissions onboarding, settings, branding, and login item.
8. #51 — Universal 2 packaging, Developer ID signing, notarization, stapling, and release CI.

### Wave 4 — exact-candidate evidence

- Windows settings/custom-hotkey regression matrix;
- macOS controlled E2E and physical application matrix;
- permission denied/granted/revoked tests;
- free/occupied custom shortcut tests;
- Unicode, Clipboard, Auto, focus, Cancel, and privacy-sentinel evidence;
- app-bundle, signature, notarization, Gatekeeper, install/upgrade/uninstall verification;
- final `CROSS_PLATFORM BETA READY` or `NOT READY` result.

## Release boundaries

### Unsigned candidates

CI may publish clearly labelled unsigned macOS candidate artifacts for development and testing.

### Public macOS prerelease

A public macOS artifact requires:

- exact tested commit;
- Universal 2 arm64 and x86_64 slices;
- hardened runtime;
- Developer ID Application signature;
- successful notarization;
- stapled ticket;
- `codesign`, `spctl`, and `stapler` verification;
- protected release credentials and approval;
- SHA-256 manifest, Sigstore provenance, and GitHub attestations;
- immutable versioned release assets.

Lack of Apple credentials does not justify publishing an unsigned artifact as a general macOS beta.

## Non-negotiable invariants

- no clipboard/injected plaintext in UI, logs, diagnostics, configuration, screenshots, crash output, release metadata, or network transport;
- no clipboard history;
- no focused content or window titles;
- no arbitrary global key capture;
- no settings persistence after a failed shortcut update;
- no destination adoption/refocus;
- no retry after partial or unknown native input progress;
- no Windows elevation or macOS permission bypass;
- active sessions retain immutable settings/backend snapshots;
- compatibility wording does not exceed exact evidence.

## Exit criteria

- [ ] ADR-0009 is accepted and dependency attribution is implemented;
- [ ] legacy preset settings migrate safely to explicit shortcut values;
- [ ] the complete shortcut pair can be recorded, validated, probed, atomically applied, and rolled back;
- [ ] the shared settings window passes accessibility/theme/lifecycle checks on Windows and macOS;
- [ ] macOS Clipboard, Keyboard, and Auto modes pass controlled E2E;
- [ ] Accessibility onboarding and revocation are honest and recoverable;
- [ ] status-item, login-item, quit, and restart lifecycle is clean;
- [ ] Universal 2 artifact structure is verified;
- [ ] public macOS artifacts are Developer ID signed, notarized, and stapled;
- [ ] exact-SHA physical Windows and macOS reports exist;
- [ ] the final result is `CROSS_PLATFORM BETA READY` or `NOT READY`.
