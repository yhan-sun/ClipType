# ADR-0001: Rust as the Primary Systems Language

- Status: Accepted
- Date: 2026-09-01

## Context

ClipType is a long-running cross-platform desktop systems utility that interacts with native clipboard, input, focus, event-loop, permission, and potentially privileged-helper APIs. It handles sensitive text and requires predictable resource use plus good FFI.

Alternatives considered include Go and C++.

## Decision

Use Rust as the primary language for the core, application orchestration, and platform adapters.

Small platform-native auxiliary components may use another language only when they materially improve OS integration and expose a narrow, documented boundary.

## Alternatives considered

### Go
Strong productivity and cross-platform tooling, but the project would still need substantial cgo/syscall/Objective-C/native GUI integration in its hardest platform areas.

### C++
Excellent native API access, but larger memory-safety and maintenance burden for a sensitive long-running utility.

### Electron/web stack
Rejected as the core architecture because ClipType's primary work is native system integration and the product aims for a small runtime footprint.

## Consequences

### Positive
- strong memory-safety baseline;
- explicit unsafe/FFI boundaries;
- native binary distribution;
- good fit for Windows/Linux systems APIs;
- viable macOS FFI.

### Negative / trade-offs
- Rust desktop GUI ecosystem is less unified than web stacks;
- macOS/native presentation may require additional bindings or a thin Swift shell;
- contributor learning curve is higher than some managed languages.

## Follow-up

Exact crates/runtimes are implementation decisions unless they materially change architecture or security.