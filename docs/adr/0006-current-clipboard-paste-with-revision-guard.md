# ADR-0006: Paste the Current Clipboard Without Rewriting It

- Status: Proposed
- Date: 2026-09-02

## Context

ClipType's primary operation starts from text that is already in the user's current clipboard. A conventional clipboard-paste backend often snapshots the clipboard, writes temporary text, emits Paste, and later tries to restore the snapshot. That transaction is necessary when the application transforms or supplies different text, but it creates a serious race: another application may write a newer clipboard value before restoration.

P2 needs a fast clipboard backend without weakening the project's ephemeral-data and external-change rules.

## Decision

For the P2 `clipboard` backend, the source of truth remains the current OS clipboard. ClipType:

1. captures destination evidence;
2. obtains a bounded current-text snapshot plus a content-blind clipboard revision;
3. waits for trigger modifiers to settle;
4. revalidates destination and integrity evidence;
5. verifies that the clipboard revision is unchanged;
6. emits exactly one bounded native Paste chord;
7. never writes, clears, snapshots for restoration, or restores clipboard contents.

`auto` may select this backend only when paste dispatch and a reliable revision guard are both available. An explicit `clipboard` request fails clearly when either capability is unavailable or degraded. A changed revision aborts; ClipType never pastes a newer value merely because the original session is still active.

The clipboard revision is diagnostic-free evidence. Its numeric value is not logged or persisted.

## Alternatives considered

### Temporary write and conditional restore

This supports transformed or generated text, but adds ownership tracking, delayed cleanup, crash recovery, and a race against newer external clipboard writes. It solves no current P2 requirement because the requested text already is the clipboard.

### Paste without a revision witness

Smallest implementation, but a clipboard change between read/planning and Paste could deliver unintended content. Rejected for the strict destination/data intent model.

### Always use keyboard injection

Avoids clipboard races but is slower for large payloads and does not provide the explicit clipboard mode accepted by ADR-0003.

## Consequences

### Positive

- the user's clipboard is unchanged by ClipType;
- no restoration can overwrite newer external data;
- no additional plaintext copy is retained for rollback;
- the backend is fast and consists of one bounded native command;
- explicit/auto semantics remain visible and testable.

### Negative / trade-offs

- the target decides how Paste handles formatting, newlines, controls, and terminal semantics;
- full native event acceptance does not prove the target consumed the text;
- clipboard mode is unavailable where a stable content-blind revision cannot be observed;
- transformed/generated text would need a separately reviewed transaction design.

## Follow-up

- Add controlled Windows tests that change the clipboard after planning and require an abort.
- Benchmark keyboard and clipboard modes before freezing the default auto threshold.
- Any future feature that writes or restores clipboard data requires a new ADR and must never overwrite a newer external value.
