# ADR-0016: Code mode types triple-quoted boundaries explicitly

- Status: Superseded by ADR-0017
- Date: 2026-09-04
- Scope: shared Code-mode lexer and keyboard action planning
- Supersedes: ADR-0014
- Related: ADR-0003, ADR-0006

## Context

Code mode delegates ordinary single-character pair completion to the
destination editor. That contract does not extend reliably to Python-style
triple-quoted strings such as `"""` and `'''`: many editors do not create a
three-character closing pair. Treating each character as an independent quote
therefore makes the second quote look like a single-quote closer and causes a
spurious cursor-right action.

ClipType cannot inspect the destination editor's content, caret, completion
setting, or generated text without crossing the content-free target boundary.

## Decision

Code mode recognizes a consecutive run of three identical single or double
quote atoms as one triple-quoted boundary. It:

1. emits all three opening quote atoms explicitly;
2. keeps the lexer in the triple-quoted string state while delivering the
   body, including its line breaks and indentation;
3. emits all three closing quote atoms explicitly; and
4. never emits `CursorRight` for either triple-quote boundary.

Ordinary single quotes and double quotes retain the ADR-0014 auto-pair
contract. Markdown runs of three or more backticks are recognized as literal
fence markers, emit the complete run, and leave the lexer in normal code state
so pair handling continues inside the fence. A backtick outside such a run
remains a single-character delimiter. Comments remain literal, and Code mode
remains a separate keyboard backend with no clipboard fallback or corrected
typo simulation.

## Alternatives considered

### Continue treating each quote independently

Rejected because `"""` is split into single-quote open/close operations and
the resulting cursor-right actions can omit source characters or move beyond
the intended insertion point.

### Emit cursor-right for the closing triple run

Rejected because the destination editor is not required to synthesize a
three-character closer for a triple-quoted string.

### Read editor content or add editor-specific integrations

Rejected for the shared product path because it expands privacy scope and
platform dependencies. A future editor-specific adapter would require a new
accepted decision.

## Consequences

### Positive

- `"""..."""` and `'''...'''` preserve all source quote atoms in Code mode;
- multiline triple-quoted bodies do not lose their intentional indentation;
- Markdown fences do not accidentally turn the following code into a
  backtick string, so its ordinary pairs remain balanced;
- ordinary editor-generated single-character pairs keep their existing
  cursor-right behavior;
- the plan remains bounded, cancellable, and content-free at the target
  boundary.

### Negative / trade-offs

- Code mode still cannot cancel conflicting editor-generated characters or
  guarantee every editor's handling of individual quote keystrokes;
- unusual raw-string and language-specific delimiter rules may require
  Keyboard or Clipboard mode;
- as with ADR-0014, Code mode must be used with a compatible destination
  editor and keyboard capability set.
