# ADR-0014: Code mode uses keyboard-aware editor pairs

- Status: Superseded by ADR-0016
- Date: 2026-09-04
- Scope: shared injection policy and keyboard ports
- Supersedes: ADR-0013
- Related: ADR-0003, ADR-0006

## Context

Code editors commonly auto-indent after a line break and auto-complete pairs
such as `()`, `{}`, `[]`, and quotes. A normal per-character injection path
would send the source indentation and closing delimiters again, producing
duplicated indentation or pairs.

The user explicitly needs Code mode to remain keyboard-based and distinct from
both ordinary Keyboard mode and Clipboard mode. ClipType cannot read target
editor content or completion settings without crossing the content-free target
boundary.

## Decision

Code mode is a separate keyboard plan and backend. It:

1. normalizes the current clipboard text using the existing bounded keyboard
   rules;
2. skips spaces and Tabs at the start of each normal-code line, allowing the
   editor to supply indentation;
3. types opening `()`, `{}`, `[]`, and quote delimiters normally;
4. emits a cursor-right action for a matching closing delimiter or quote that
   the editor is expected to have generated;
5. keeps delimiters inside recognized strings, line comments, and block comments
   literal; and
6. never applies corrected typo simulation, although configured keyboard
   pacing and cancellation/focus checks still apply.

Code mode requires the keyboard capabilities needed by the source and a
cursor-right capability. It does not require Clipboard paste or clipboard
revision evidence and never silently falls back to another backend.

The contract assumes destination-editor auto-pair and auto-indent are enabled.
If an editor does not provide those behaviors, users should choose Keyboard or
Clipboard mode instead.

## Alternatives considered

### Guarded whole-block paste

Rejected for Code mode because the requested behavior is keyboard-based and
must let the editor's own indentation and pair handlers participate. Clipboard
mode remains available for exact rich-format paste.

### Raw keyboard input with no code rules

Rejected because it duplicates editor-generated indentation and closing pairs.

### Read target content or use editor-specific integrations

Rejected for the shared product path because it would expand privacy scope and
platform dependencies. A future editor-specific adapter would require a new
accepted decision.

## Consequences

### Positive

- Code mode is visibly and behaviorally separate from Keyboard and Clipboard;
- editor auto-indent supplies line indentation instead of receiving duplicate
  source indentation;
- matching auto-generated closers and quotes are not typed twice;
- strings and comments do not have their literal brackets removed;
- Code mode remains bounded, cancellable, and content-free at the target
  boundary.

### Negative / trade-offs

- pair skipping is necessarily based on the source lexer and the editor
  contract; unusual raw-string syntaxes and editor-specific completion rules
  may require Keyboard or Clipboard mode;
- if auto-pair is disabled, cursor-right actions can move past real text, so
  Code mode must not be used in that configuration;
- leading indentation is intentionally not reproduced by ClipType.
