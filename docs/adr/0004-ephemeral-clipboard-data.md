# ADR-0004: Clipboard Plaintext Is Ephemeral and Not Persisted by Default

- Status: Accepted
- Date: 2026-09-01

## Context

Clipboard text may contain credentials, tokens, private communications, source code, and personal information. ClipType does not need history to satisfy its core product goal.

## Decision

V1 processes clipboard plaintext only for the active operation and does not intentionally persist it to disk, logs, telemetry, analytics, configuration, or history.

Diagnostics use a strict non-sensitive allowlist. Network functionality must remain content-blind.

## Alternatives considered

### Clipboard history by default
Potentially convenient but creates a persistent sensitive-data store unrelated to the core value proposition.

### Opt-out history
Still violates privacy-first defaults and increases breach/debugging risk.

## Consequences

### Positive
- smaller privacy/security surface;
- simpler data lifecycle;
- stronger user trust story;
- fewer migration/storage concerns.

### Negative / trade-offs
- no history-based recovery/reuse feature;
- debugging cannot rely on capturing real user payloads.

## Follow-up

Any future clipboard-history feature requires a new ADR, explicit product scope, retention/encryption design, and migration/privacy review.