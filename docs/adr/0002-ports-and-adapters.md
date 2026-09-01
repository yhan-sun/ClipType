# ADR-0002: Ports and Adapters Around a Platform-Independent Core

- Status: Accepted
- Date: 2026-09-01

## Context

Clipboard access, input injection, focus detection, hotkeys, and permissions differ substantially across Windows, macOS, X11, and Wayland. Mixing OS calls directly with product policy would make compatibility and tests hard to reason about.

## Decision

Separate platform-independent policy/application behavior from native mechanisms using explicit semantic ports and OS-specific adapters.

Core/application logic owns the injection state machine, planner, safety policy, and error semantics. Adapters own native API interaction and runtime capability reporting.

## Alternatives considered

### One cross-platform automation library
Simpler initially but tends to hide platform limitations and makes safety/permission semantics dependent on an external abstraction.

### Platform-specific applications with duplicated logic
Could maximize native behavior but would duplicate policy, tests, and bug fixes.

## Consequences

### Positive
- deterministic core tests with fake ports;
- explicit platform limitations;
- easier staged delivery one platform at a time;
- replacement of one adapter does not rewrite policy.

### Negative / trade-offs
- additional interface/domain design;
- some OS-specific semantics do not fit perfectly into common ports, requiring capability types rather than lowest-common-denominator APIs.

## Follow-up

Do not create adapters/crates for future platforms before their roadmap phase merely to satisfy symmetry.