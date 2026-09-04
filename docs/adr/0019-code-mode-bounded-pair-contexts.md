# ADR-0019: Restrict Code-mode pairing to five source-context-safe pair families

- Status: Accepted
- Date: 2026-09-05

## Context

Code mode types source text into editors that may synthesize matching closing
characters and indentation. Treating every delimiter-like character as an
automatic pair is unsafe. In particular, editors normally suppress bracket
auto-completion inside strings and comments, while a single backtick has
language-dependent meaning. Tracking those characters as generated pairs can
move the caret over source text, duplicate characters, or leave generated
closers at the end of the document.

Python-style triple-quoted strings are also not a three-character variant of an
ordinary editor pair. Editors do not consistently synthesize their closing
boundary after the first quote.

## Decision

Code mode remains a separate keyboard-only injection backend. Its pair-aware
grammar is restricted to these single-character pairs:

- `()`
- `{}`
- `[]`
- `""`
- `''`

Bracket openers enter the pair stack only in normal source context. Brackets in
ordinary strings, triple-quoted strings, line comments, and block comments are
literal keyboard atoms. For an ordinary single- or double-quoted string, only
the editor-generated closing quote is skipped with Right Arrow.

`"""` and `'''` boundaries are emitted explicitly. Single backticks, Markdown
triple-backtick fences, angle brackets, and every other character outside the
five families are literal.

The existing line-leading closer rule also runs at a `//` line-comment boundary
before emitting Return. This lets the planner cross an already generated
closing line without creating an extra blank line.

## Consequences

The planner no longer assumes editor behavior that depends on language tokens
inside strings or comments. The exact Rust fixture, C/C++ multi-function text,
Chinese comments, nested arrays, quoted brackets, triple quotes, and long-tail
completion can be covered by a deterministic editor model without reading a
real document.

Code mode still assumes ordinary auto-pair and auto-indent are enabled for the
five supported families. It never invokes Paste and never reads destination
text to decide whether a closer exists. Editors whose behavior differs from
this bounded contract remain outside the compatibility claim.
