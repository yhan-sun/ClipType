# Testing Strategy

## Objectives

Testing must prove not only that text appears, but that ClipType preserves safety invariants under cancellation, focus changes, clipboard races, Unicode, permissions, and platform restrictions.

## Test layers

### 1. Core unit tests
Fast, deterministic, platform-independent tests for:
- planner mode selection;
- capability gating;
- state-machine transitions;
- cancellation;
- focus-change decisions;
- retry/idempotency rules;
- configuration validation;
- error categorization;
- clipboard restoration decision logic using fake generations.

Core policy should be testable with fake ports and no GUI/OS APIs.

### 2. Adapter contract tests
Each platform adapter is tested against semantic contracts:
- current text read;
- temporary write;
- event injection result semantics;
- target identity stability;
- permission/capability detection;
- teardown/resource cleanup.

### 3. Platform integration tests
Run on actual OS runners/machines where synthetic input is possible. CI limitations must be documented; a headless runner success is not automatically equivalent to desktop-session success.

### 4. End-to-end tests
A controlled target application/text field verifies delivered text and records only fixture text, never user clipboard data.

## Required text fixtures

Use generated/non-sensitive fixtures covering:
- ASCII;
- spaces/punctuation;
- LF and CRLF multiline;
- tabs;
- CJK;
- emoji/supplementary Unicode;
- combining marks;
- long text;
- unsupported control characters with expected rejection/normalization.

## Required behavioral cases

### Keyboard mode
- complete short injection;
- long injection;
- cancel during injection;
- focus switch mid-injection;
- physical modifier held at trigger time;
- target closes mid-injection;
- partial native dispatch error.

### Clipboard mode
- successful temporary write/paste/restore;
- external clipboard changes before restore;
- cancellation before paste;
- cancellation after paste but before cleanup;
- clipboard busy/ownership failure;
- own clipboard event suppression;
- restore failure surfaced without overwriting external data.

### Permissions
- permission granted;
- permission denied;
- permission revoked while app runs where platform exposes this;
- Windows high-integrity target restriction;
- Wayland missing protocol/device permission.

## Compatibility matrix

For every release candidate, record capability tests by:
- OS version;
- architecture;
- desktop environment/session/compositor where relevant;
- backend selected;
- representative target app category.

Update `COMPATIBILITY.md` only from evidence.

## CI stages (future)

Expected gates as implementation arrives:

1. formatting/lint;
2. unit tests all platforms where cross-compilation permits;
3. platform compile checks;
4. platform integration suites;
5. dependency/license/security scanning;
6. packaging smoke tests;
7. release-only signing/notarization verification.

## Performance tests

Benchmarks track, not gate prematurely:
- startup time;
- idle CPU/memory;
- chars/sec keyboard backend;
- clipboard transaction latency;
- cancellation latency;
- large-payload memory use.

A performance optimization is rejected if it weakens cancellation/focus/clipboard-race guarantees.

## Privacy testing

Tests and review should search logs/artifacts for known fixture plaintext and verify it does not appear outside explicitly controlled target/test buffers.

## Release evidence

A release candidate has a versioned test report or CI evidence sufficient to reproduce:
- commit SHA;
- required checks;
- platform environments;
- known skipped/unverified cases;
- compatibility claim changes.