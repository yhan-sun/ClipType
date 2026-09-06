# Product

## Purpose

ClipType is a local, privacy-first desktop utility for one explicit action:

> deliver the text currently on the user's clipboard to the destination that was active when the user triggered ClipType.

Windows x86_64 is the primary beta channel. macOS is currently an Apple Silicon arm64 testing preview. ClipType is not a clipboard manager, key logger, macro recorder, remote automation service, or privilege-bypass tool.

## User loop

1. The user copies text in any application.
2. The user focuses the intended destination.
3. The user invokes the reviewed global trigger shortcut or product command.
4. ClipType captures content-free destination evidence, waits for physical trigger modifiers to clear, reads the current clipboard once within configured bounds, and freezes one injection backend.
5. ClipType revalidates destination, integrity, modifier, cancellation, and—when pasting—clipboard revision evidence.
6. ClipType performs bounded native input and reports a content-free outcome.
7. An independent cancel shortcut or product command can stop remaining work between safe boundaries.

## Injection modes

### Keyboard

`keyboard` normalizes Unicode text into semantic atoms and emits one bounded native action at a time. It is appropriate when text-event semantics are preferred over application paste behavior.

The path supports ASCII, CJK, supplementary Unicode scalars, combining marks, normalized line breaks, and the configured Tab policy. True characters-per-second pacing and bounded per-action jitter apply to each action. It stops on target change/evidence loss, conflicting modifiers, cancellation, partial input, or unknown native progress.

### Clipboard

`clipboard` uses the user's already-current clipboard. It captures a content-blind revision, verifies that revision immediately before dispatch, and sends exactly one bounded paste chord.

ClipType does not write, clear, own, restore, cache, or retain clipboard contents. The destination application chooses its ordinary paste behavior and may select an existing rich-text format.

### Code

`code` is an explicit keyboard-only mode for source code and structured text. It is separate from Keyboard and Clipboard delivery and never invokes Paste.

It skips leading spaces and Tabs at each normal-code line so the editor can supply indentation. Pair-aware behavior is limited to `()`, `{}`, `[]`, `""`, and `''`: openers are typed and a matching source closer uses cursor navigation to pass over the editor-generated closer. Brackets inside strings and recognized comments are typed literally. When a matching closer is the first non-indentation character on its source line, including immediately after a `//` comment, Code mode avoids a redundant line break and crosses the existing editor-generated closing line. Python-style triple-quoted boundaries (`"""` and `'''`) are typed explicitly. Markdown triple-backtick fences and single backticks are literal.

Actions remain strict FIFO with bounded settle barriers so ordinary editor auto-pair and auto-indent updates can complete before dependent navigation. Code mode never reads target content or claims editor-specific formatting semantics.

When corrected-typo probability is non-zero, Code mode applies it only to source Atom actions as `wrong key -> Backspace -> correct source atom`. Temporary wrong keys are restricted so they cannot be brackets, quotes, or `/`, preventing the typo simulation itself from triggering editor auto-pair or comment behavior. Cursor-navigation actions are never typo-simulated, and non-ASCII text has no fabricated QWERTY typo.

### Auto

`auto` chooses one backend from payload shape, payload size, and proven capabilities. For non-ASCII text—including CJK, emoji, combining marks, and mixed Unicode—it prefers one revision-guarded paste even when the payload is short. The choice is immutable for the session. If guarded paste is unavailable, Auto may use the Unicode keyboard path. Explicit modes never silently fall back.

## Product surface

### Windows

The Windows beta provides:

- native notification-area icon and context menu;
- trigger, cancel, enable/disable, and quit commands;
- keyboard/clipboard/code/auto selection;
- exact characters-per-second controls;
- bounded jitter controls;
- opt-in corrected adjacent-key typo probability;
- notification control;
- validated Trigger/Cancel shortcut pairs with OS-level probing and transactional replacement;
- per-user start-at-login control;
- strict versioned per-user configuration with backup recovery;
- portable and per-user installation options;
- content-free status and remediation categories.

### macOS Apple Silicon preview

The macOS Flutter/AppKit shell provides task-oriented Overview, Input, Shortcuts, System, and About surfaces. Overview shows readiness and the next action; Input exposes mode-relevant controls; Shortcuts validates and transactionally applies candidate pairs; System owns Accessibility/startup/application preferences; About reads build identity from the running bundle. Valid settings save automatically.

The preview is arm64-only, ad-hoc signed, and not notarized. Hosted CI cannot substitute for persistent Accessibility consent or real named-application testing.

## Safety rules

- The destination is captured before clipboard acquisition and revalidated before native dispatch.
- ClipType never redirects remaining input to a newly focused target.
- Detailed target evidence that later degrades fails closed.
- A normal-integrity process does not inject into a higher-integrity target.
- ClipType observes physical modifiers and never releases keys owned by the user.
- Clipboard retries, modifier settling, native action sizes, waits, worker lifetime, and shutdown are bounded.
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

`v0.2.0-beta.8` is the current public prerelease. Windows x86_64 is the primary beta channel; Windows 11 x64 is the recommended client. Windows 10 22H2 x64 is best-effort API-compatible with an operating-system support/security caveat. Windows Server 2022 and 2025 Desktop Experience are mechanism-compatible CI reference environments.

The same release carries a clearly labelled macOS Apple Silicon arm64 testing preview. It is not an Intel/Rosetta/Universal 2, Developer ID, notarized, or broad named-application support claim.

ClipType supports ordinary editable desktop targets that accept the selected native mechanism. This is a mechanism-level support contract, not a universal guarantee for every application, logical field, remote session, or security boundary. See `docs/COMPATIBILITY.md`.

Representative Windows physical/named-application evidence remains tracked in #33. Physical Apple Silicon behavior and trusted Apple distribution promotion remain tracked in #61.

## Explicit non-goals

- clipboard history, search, synchronization, or analytics;
- arbitrary global keyboard capture;
- macros, scripting, or unattended command execution;
- automatic elevation or security-boundary bypass;
- target-content inspection to make input appear successful;
- silent fallback from an explicitly selected backend;
- claiming untested architectures, applications, or signing states.
