# ADR-0003: Keyboard, Clipboard, and Auto Injection Modes

- Status: Accepted
- Date: 2026-09-01

## Context

Per-event keyboard injection best matches the product's core use case but can be slow or limited for large/complex text. Normal clipboard paste is fast but depends on target paste behavior and temporarily changes the clipboard. Mature text-injection tools demonstrate value in supporting both mechanisms.

## Decision

ClipType exposes three semantic modes:

- `keyboard`: require synthetic keyboard/text injection;
- `clipboard`: require safe clipboard write + paste transaction;
- `auto`: choose an eligible mode through an explicit planner.

Explicit user mode choices are not silently replaced by another backend. Auto-mode selection is based on runtime capabilities, payload, target evidence, and benchmark-derived policy.

## Alternatives considered

### Keyboard only
Simple and aligned with the name, but inefficient and less compatible for long payloads.

### Clipboard paste only
Fails the main use case where paste is blocked/different from typing.

### Hidden fallback without exposed modes
Simpler UI but makes debugging/compatibility surprising and can violate user intent.

## Consequences

### Positive
- compatibility and performance flexibility;
- explicit user control;
- planner can evolve without platform adapters owning policy.

### Negative / trade-offs
- clipboard mode requires careful restoration/race handling;
- compatibility testing doubles across mechanisms;
- auto policy needs benchmark/evidence rather than arbitrary thresholds.

## Follow-up

Initial threshold values are implementation/config defaults and are not frozen by this ADR.