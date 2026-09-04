# Architecture Decision Records

ADRs capture decisions that should not be changed silently by implementation work.

## Status values

- `Proposed`
- `Accepted`
- `Deprecated`
- `Superseded by ADR-XXXX`

## Process

1. Copy the template below to the next numbered file.
2. Describe context and constraints, not only the preferred answer.
3. List meaningful alternatives.
4. Record consequences/trade-offs.
5. Mark accepted only when maintainers agree.
6. If a decision changes later, add a new ADR and mark the old one superseded.

## Template

```markdown
# ADR-XXXX: Title

- Status: Proposed
- Date: YYYY-MM-DD

## Context

## Decision

## Alternatives considered

## Consequences

### Positive

### Negative / trade-offs

## Follow-up
```

## Accepted ADRs

- [`ADR-0001`](0001-rust-primary-language.md) — Rust as the primary systems language.
- [`ADR-0002`](0002-ports-and-adapters.md) — Ports and adapters around a platform-independent core.
- [`ADR-0003`](0003-injection-modes.md) — Keyboard, clipboard, and auto injection modes.
- [`ADR-0004`](0004-ephemeral-clipboard-data.md) — Clipboard plaintext is ephemeral and not persisted by default.
- [`ADR-0005`](0005-process-and-privilege-boundary.md) — Single unprivileged user process by default.
- [`ADR-0006`](0006-current-clipboard-paste-with-revision-guard.md) — Revision-guarded current-clipboard paste without rewrite/restore.
- [`ADR-0007`](0007-native-win32-tray-shell.md) — Native Win32 tray-first product shell and persistent settings boundary.
- [`ADR-0008`](0008-windows-beta-compatibility-and-release-provenance.md) — Windows beta compatibility, Sigstore signing, and GitHub provenance.
- [`ADR-0009`](0009-shared-slint-settings-ui-native-shells.md) — Original shared native-compiled Slint settings window with native Windows/macOS shells; superseded for macOS by ADR-0010 and ADR-0012.
- [`ADR-0010`](0010-flutter-macos-arm64-runner.md) — Flutter macOS arm64 runner over the Rust application core with a Swift/AppKit shell.
- [`ADR-0011`](0011-unicode-first-auto-selection.md) — Non-ASCII Auto selection prefers revision-guarded paste before the size threshold.
- [`ADR-0012`](0012-flutter-sole-macos-frontend.md) — Remove the legacy macOS Slint composition root and keep Flutter as the sole macOS front end.
- [`ADR-0013`](0013-code-mode-guarded-paste.md) — Superseded whole-block paste decision for Code mode.
- [`ADR-0014`](0014-code-mode-keyboard-pair-aware.md) — Superseded keyboard code-action decision that delegates indentation and auto-pairing to the destination editor.
- [`ADR-0015`](0015-flutter-settings-auto-save.md) — Automatically persist Flutter settings with responsive save-state feedback.
- [`ADR-0016`](0016-code-mode-triple-quote-boundaries.md) — Superseded triple-quote boundary decision for Code mode.
- [`ADR-0017`](0017-code-mode-line-leading-closers.md) — Navigate editor-generated closing lines without reading target content.
- [`ADR-0018`](0018-macos-render-host-target-evidence.md) — Use stable process/window evidence for macOS `AXWebArea` render hosts.
