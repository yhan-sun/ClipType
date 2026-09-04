# ADR-0013: Code mode uses guarded whole-block paste

- Status: Superseded by ADR-0014
- Date: 2026-09-04
- Scope: shared injection policy and settings surface
- Supersedes: ADR-0003 for the mode vocabulary
- Related: ADR-0006, ADR-0011

## Context

Code editors commonly auto-complete pairs such as `()`, `{}`, `[]`, and quotes,
and may add indentation after a line break. Feeding a copied source file through
the per-character keyboard path can therefore duplicate closing delimiters or
indentation. ClipType cannot portably inspect editor-specific caret, completion,
or indentation state without reading application content or adding editor
integrations.

## Decision

Expose a fourth explicit mode, `code`. Code mode selects the existing
revision-guarded current-clipboard paste path for the entire payload. It does
not emit per-character Unicode, Return, Tab, or typo-correction actions.

This preserves the exact delimiters, quotes, line breaks, and indentation that
are already present in the clipboard and prevents ordinary editor auto-pair and
per-line auto-indent handlers from processing each copied character. The
destination may still apply its own format-on-paste behavior; ClipType does not
claim a universal language formatter or editor-specific integration.

Code mode requires the same content-blind clipboard revision witness and paste
capability as explicit Clipboard mode. It never silently falls back to the
keyboard backend.

## Alternatives considered

### Smart per-character bracket and indentation emulation

Rejected for the cross-platform default. Correct behavior depends on the
editor, language mode, current caret context, whether completion is enabled,
and the editor's actual indentation policy. Those facts are not available from
ClipType's content-free target evidence.

### Treat code as a special Auto heuristic

Rejected because source code is not reliably identifiable from text shape, and
automatic backend changes would make exact-data behavior surprising. The user
must explicitly select Code mode.

## Consequences

### Positive

- ASCII source code no longer takes the short-text keyboard path in Code mode;
- existing `()`, `{}`, `[]`, quotes, newlines, and indentation are preserved;
- keyboard jitter and corrected-typo settings cannot mutate Code-mode payloads;
- no editor-specific API, content inspection, or new native dependency is
  required.

### Trade-offs

- Code mode requires a working guarded paste capability;
- it preserves the clipboard's indentation rather than asking the editor to
  synthesize indentation;
- editor format-on-paste behavior remains application-specific.
