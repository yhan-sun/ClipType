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

## Initial ADRs

- ADR-0001 — Rust as the primary systems language.
- ADR-0002 — Ports and adapters around a platform-independent core.
- ADR-0003 — Keyboard, clipboard, and auto injection modes.
- ADR-0004 — Clipboard plaintext is ephemeral and not persisted by default.
- ADR-0005 — Single unprivileged user process by default; privileged helpers only when required.