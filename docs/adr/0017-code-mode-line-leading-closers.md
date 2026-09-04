# ADR-0017: Code mode navigates editor-generated closing lines

- Status: Accepted
- Date: 2026-09-05
- Scope: shared Code-mode planning and keyboard adapters
- Supersedes: ADR-0016
- Related: ADR-0003, ADR-0004, ADR-0014

## Context

An editor with auto-pair and auto-indent enabled does more than place a closer
beside an opener. After an opening brace and Return, editors such as VS Code
move the generated closer to its own indented line and leave the caret on a
body line above it.

The previous Code plan treated every matching source closer identically. For a
source sequence such as a line break followed by `}`, it emitted Return and
then one CursorRight. Return created an extra blank line, and CursorRight moved
within that blank line instead of passing the generated `}`. Repeated nested
closers therefore remained at the end of the document. A short settle delay
could not correct this positional error.

ClipType must solve this without reading the destination's text, caret, editor
settings, or generated content.

## Decision

Code mode keeps a source-derived generated-pair stack and distinguishes two
closing cases:

1. A matching closer on the current source line remains one `CursorRight`.
2. A matching closer that is the first non-indentation character after a source
   line break becomes `CursorRightToLineEnd`.

Consecutive matching closers at that boundary, such as `});`, are consumed as
one generated closing-line group so line-end navigation is not followed by
redundant per-closer Right actions.

For the second case, the planner records whether a prior dispatched line break
has already moved that generated pair to a following line. It suppresses the
redundant source line break when the generated closer is already there. For an
empty multiline pair, it retains the first Return so the editor can create its
body and closing lines. The navigation action then:

- sends Right followed by Command+Right on macOS; and
- sends Right followed by End on Windows.

This crosses the editor-created line boundary and reaches the end of the
generated closing line without depending on its indentation width. It is a
bounded keyboard action and does not inspect destination content.

Code mode also applies a 40 ms cancellable settle barrier immediately before
pair-navigation actions so asynchronous ordinary pair completion cannot be
overtaken. The existing per-action focus, modifier, and cancellation checks
still run. Keyboard, Clipboard, and Auto behavior is unchanged.

Ordinary `()`, `{}`, `[]`, `""`, and `''` keep pair-aware behavior. Triple
quoted boundaries (`"""` and `'''`) remain explicit and never use pair
navigation, as decided by ADR-0016.

## Alternatives considered

### Emit many plain Right actions

Rejected because the editor-generated closing line may use spaces, Tabs, or a
different indentation width from the source. A fixed count can stop before the
closer or move into later text.

### Type the source closer

Rejected because it can duplicate the editor-generated closer and does not
meet Code mode's pair-skipping contract.

### Read the current character through Accessibility

Rejected because it crosses the content-free destination boundary and would
expose focused-field content to the application.

## Consequences

### Positive

- nested line-leading closers remain in source order instead of accumulating at
  the document end;
- generated indentation width does not need to match source indentation;
- empty and non-empty multiline pairs have deterministic source-derived plans;
- all navigation remains bounded, cancellable, focus-checked, and content-free;
- only explicit Code mode changes.

### Negative / trade-offs

- Code mode still requires a compatible editor with ordinary auto-pair,
  auto-indent, and conventional line-navigation behavior enabled;
- pair navigation adds a small delay at closing boundaries;
- language-specific constructs outside the bounded lexer may still require
  Keyboard or Clipboard mode.

## Follow-up

Keep controlled editor regression fixtures for nested blocks, line-leading
parentheses/brackets, ordinary quoted braces, and explicit triple quotes.
