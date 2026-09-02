# Product

## Purpose

ClipType is a local, privacy-first Windows tray utility for one explicit action:

> deliver the text currently on the user's clipboard to the destination that was active when the user triggered ClipType.

It is not a clipboard manager, key logger, macro recorder, remote automation service, or privilege-bypass tool.

## User loop

1. The user copies text in any application.
2. The user focuses the intended destination.
3. The user invokes the reviewed global trigger hotkey or tray command.
4. ClipType captures content-free destination evidence, waits for physical trigger modifiers to clear, reads the current clipboard once within configured bounds, and freezes one injection backend.
5. ClipType revalidates destination, integrity, modifier, cancellation, and—when pasting—clipboard revision evidence.
6. ClipType performs bounded native input and reports a content-free outcome.
7. An independent cancel hotkey or tray command can stop remaining work between safe boundaries.

## Injection modes

### Keyboard

`keyboard` normalizes Unicode text into semantic atoms and emits bounded `SendInput` batches. It is appropriate when text-event semantics are preferred over application paste behavior.

The path supports ASCII, CJK, supplementary Unicode scalars, combining marks, normalized line breaks, and the configured Tab policy. It stops on target change/evidence loss, conflicting modifiers, cancellation, partial input, or unknown native progress.

### Clipboard

`clipboard` uses the user's already-current clipboard. It captures a content-blind revision, verifies that revision immediately before dispatch, and sends exactly one bounded `Ctrl+V` chord.

ClipType does not write, clear, own, restore, cache, or retain clipboard contents. The destination application chooses its ordinary paste behavior and may select an existing rich-text format.

### Auto

`auto` chooses one backend from payload size and proven capabilities. The choice is immutable for the session. It can select clipboard only when both paste dispatch and revision guarding are fully available. Explicit modes never silently fall back.

## Product surface

The Windows beta provides:

- native notification-area icon and context menu;
- trigger, cancel, enable/disable, and quit commands;
- keyboard/clipboard/auto selection;
- slow/normal/fast keyboard pacing;
- notification control;
- reviewed trigger/cancel hotkey presets;
- per-user start-at-login control;
- strict versioned per-user configuration with backup recovery;
- portable and per-user installation options;
- content-free status and remediation categories.

## Safety rules

- The destination is captured before clipboard acquisition and revalidated before every dispatch boundary.
- ClipType never redirects remaining input to a newly focused target.
- Detailed target evidence that later degrades fails closed.
- A normal-integrity process does not inject into a higher-integrity target.
- ClipType observes physical modifiers and never releases keys owned by the user.
- Clipboard retries, modifier settling, batch sizes, waits, worker lifetime, and shutdown are bounded.
- Partial or progress-unknown native input is terminal and is never blindly retried.
- Only one injection session can be active; a second trigger returns Busy.
- Active sessions retain an immutable configuration snapshot; settings changes affect future sessions.

## Privacy rules

- No clipboard history or continuous plaintext watcher.
- No network transmission of clipboard content.
- No persistence of clipboard/injected text, prefixes, suffixes, hashes, fingerprints, or samples.
- No focused-field content or window-title collection.
- Normal logs, status, notifications, evidence, and package metadata contain categories and counts only.
- Generated privacy sentinels are scanned out of distributable files and ordinary workflow output.

## Compatibility promise

The first public beta is Windows x86_64. Windows 11 x64 is the recommended client. Windows 10 22H2 x64 is best-effort API-compatible with an operating-system support/security caveat. Windows Server 2022 and 2025 Desktop Experience are mechanism-compatible CI reference environments.

ClipType supports ordinary editable desktop targets that accept Unicode-oriented `SendInput` or standard `Ctrl+V`. This is a mechanism-level support contract, not a universal guarantee for every application, logical field, remote session, or security boundary. See `docs/COMPATIBILITY.md`.

## Explicit non-goals

- clipboard history, search, synchronization, or analytics;
- arbitrary global keyboard capture;
- macros, scripting, or unattended command execution;
- automatic elevation or Windows integrity-boundary bypass;
- forced focus restoration or destination switching;
- transformed/generated text through a clipboard rewrite/restore transaction;
- ARM64, 32-bit, service, Server Core, or non-interactive support in the first beta;
- claiming Authenticode trusted-publisher identity without a trusted certificate.

## Public release

`v0.1.0-beta.1` is distributed as a GitHub prerelease with a ZIP package, portable executable, SHA-256 manifest, dependency inventory, build metadata, Sigstore keyless signatures, and GitHub artifact attestations.

The first beta is not Authenticode publisher-signed. This boundary is shown in release notes and build metadata rather than hidden.
