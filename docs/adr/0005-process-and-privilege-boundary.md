# ADR-0005: Single Unprivileged User Process by Default

- Status: Accepted
- Date: 2026-09-01

## Context

A daemon/UI split can look architecturally clean, but it adds IPC, lifecycle, update, authentication, and debugging complexity. Most Windows/macOS/X11 operations can live in a normal user process. Linux Wayland may require privileged `/dev/uinput` access in some capability paths.

## Decision

Run ClipType as one unprivileged user process by default. Add a separate helper only when a concrete platform capability requires a different privilege/lifecycle boundary.

The likely first helper is a Linux uinput emitter. It must be minimal, local, versioned, and capability-scoped; it is not a general ClipType daemon and does not store clipboard content/history.

## Alternatives considered

### Always-on core daemon + separate UI client
Provides process separation but increases complexity before it solves a demonstrated requirement.

### Run the entire application elevated/root
Rejected because it unnecessarily expands the impact of bugs and conflicts with OS security principles.

### External command tools as helpers
Useful for development research but not a stable product boundary due to dependency/version/permission/semantic variability.

## Consequences

### Positive
- smaller attack and operational surface;
- easier early lifecycle/update behavior;
- least privilege by default;
- isolates privilege only where needed.

### Negative / trade-offs
- if multiple UI/control clients become necessary later, a new process/IPC ADR may be required;
- Linux uinput integration will need careful helper design rather than sharing all application code in a root daemon.

## Follow-up

Before implementing a privileged helper, write a dedicated ADR covering protocol, authorization, installation, upgrades, and threat model.