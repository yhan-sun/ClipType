# ADR-0011: Unicode-first Auto backend selection

- Status: Accepted
- Date: 2026-09-04
- Scope: shared Auto planner, including the P4 macOS arm64 runner
- Related: ADR-0003, ADR-0006, ADR-0010

## Context

The original Auto crossover used payload size as its main preference: short
text used the keyboard backend and long text used revision-guarded paste when
both capabilities were available. That is a poor default for Chinese and
other non-ASCII text. A short CJK, emoji, combining-mark, or mixed-Unicode
payload is still more reliably delivered as the user's already-current
clipboard through one guarded Paste command.

## Decision

Auto selects the revision-guarded clipboard backend before the size threshold
whenever the current text contains any non-ASCII byte and both paste and
clipboard-revision capabilities are fully available. This covers CJK,
Japanese, Korean, emoji, combining marks, and mixed Unicode text without
normalizing or transforming the text.

If guarded paste is unavailable, Auto may use the proven Unicode keyboard
backend. Explicit Keyboard and Clipboard modes retain their requested
semantics and never silently switch backends. ASCII-safe text continues to use
the configured semantic-element threshold as the size crossover.

The planner still receives owned text only inside the Rust session lifetime;
the selected backend, element count, and outcomes remain content-free across
the Swift/Flutter boundary. The clipboard adapter continues to verify the
revision immediately before one Paste chord and never writes or restores the
clipboard.

## Alternatives considered

### Keep payload size as the only Auto preference

Rejected because short CJK and other Unicode text would unnecessarily depend
on target-specific synthetic Unicode keyboard delivery when a guarded paste
is available.

### Always use Clipboard for all Auto text

Rejected because ASCII keyboard delivery remains useful for short text and
because Auto must preserve the existing size/capability crossover.

### Silently fall back from explicit Keyboard to Clipboard

Rejected because it changes the user's requested injection semantics and
would make keyboard-mode failures difficult to reason about.

## Consequences

### Positive

- Short Chinese and other non-ASCII text gets the safer paste-first Auto path.
- The rule is deterministic, platform-neutral, and testable before native I/O.
- No new data crosses the Flutter bridge and no clipboard transaction is added.

### Negative / trade-offs

- Auto Unicode behavior depends on the destination's ordinary paste support.
- Physical target applications and Accessibility permission still require
  separate end-to-end evidence.
- The shared planner changes Auto behavior for non-ASCII text on every
  composition root that uses it.

## Follow-up

- Run the controlled macOS TextEdit/browser/editor matrix with Accessibility
  permission on an approved non-sensitive fixture.
- Keep the physical results separate from deterministic planner tests and do
  not promote the macOS beta claim until the full matrix passes.
