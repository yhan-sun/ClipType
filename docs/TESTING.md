# Testing Strategy

## Objectives

Testing must prove not only that text appears, but that ClipType preserves safety invariants under cancellation, target changes, modifier state, clipboard contention, Unicode, permissions, and platform restrictions.

P1-specific sequencing and gate evidence are defined in [`phases/P1_WINDOWS_VERTICAL_SLICE.md`](phases/P1_WINDOWS_VERTICAL_SLICE.md).

## Test layers

### 1. Pure core unit tests

Fast, deterministic, platform-independent tests for:

- text validation/normalization;
- planner mode/capability decisions;
- pure state-transition rules;
- busy/cancellation/focus-change decisions;
- retry/idempotency rules;
- configuration validation;
- error categorization;
- future clipboard-restoration decision logic using fake generations.

Pure core tests do not create threads, GUI loops, or OS handles.

### 2. Application coordinator tests

Use fake ports/event sources to test the live orchestration contract:

- atomic one-session reservation;
- initial target capture before clipboard acquisition;
- preparation/cancel races;
- worker lifecycle and terminal state reset;
- cancellation/focus/modifier checks between batches;
- partial result not retried;
- content-free status publication.

### 3. Adapter contract tests

Each platform adapter is tested against semantic contracts:

- current text read and resource cleanup;
- bounded event dispatch result semantics;
- target-evidence stability/change/unknown states;
- hotkey registration/conflict/teardown;
- permission/integrity/capability evidence;
- modifier-state behavior;
- native error translation.

### 4. Platform integration tests

Run on actual OS runners/machines where native behavior can be exercised. CI limitations must be documented; a headless runner success is not equivalent to an interactive desktop test.

### 5. End-to-end tests

A controlled target application verifies delivered fixture text and behavioral outcomes. It may store only generated fixture data needed for assertions, never arbitrary user clipboard history.

## P1 native discovery tests

Before P1 contracts freeze, the Windows spike must record:

- message-loop owner/thread affinity;
- cancellation delivery while a worker is active;
- hotkey no-repeat and trigger-modifier release behavior;
- Unicode, supplementary Unicode, newline, and Tab behavior by target category;
- native partial/zero-result semantics;
- target evidence and same-render-host limitations;
- clipboard contention behavior;
- UIPI evidence/diagnostic limits.

## Required text fixtures

Use generated, non-sensitive fixtures covering:

- ASCII;
- spaces/punctuation;
- LF, CRLF, and lone CR according to normalization policy;
- Tab when supported;
- CJK;
- emoji/supplementary Unicode;
- combining marks;
- long text that spans many dispatch batches;
- unsupported controls with expected rejection.

## Required P1 behavioral cases

### Keyboard path

- complete short injection;
- long multi-batch injection;
- cancellation before first dispatch;
- cancellation between batches;
- top-level target switch mid-injection;
- target closure/disappearance;
- evidence unavailable/ambiguous under strict policy;
- trigger modifier still held;
- conflicting modifier introduced between batches where detectable;
- partial native dispatch;
- zero/blocked result with known and unknown integrity evidence;
- second trigger while active returns busy;
- message loop still processes cancel/status/shutdown while worker runs.

### Clipboard acquisition

- ASCII and Unicode current text;
- empty clipboard;
- non-text clipboard;
- transient busy/contended clipboard within retry budget;
- contention exceeding retry budget;
- malformed/unusable data handled without panic;
- all native locks/handles closed on success and failure;
- no listener/history/persistence introduced in P1.

### Hotkey/lifecycle

- global trigger from unrelated foreground applications;
- registration conflict;
- key hold with no-repeat semantics;
- cancel command during active injection;
- normal shutdown/unregister;
- restart without stale registration;
- rapid double trigger race.

### Focus evidence

- repeated capture of unchanged target;
- switch between top-level windows/applications;
- target closes between checks;
- transient activation change;
- switch between controls/fields inside one application where observable;
- explicitly record cases where the native render host does not expose a logical-field change.

### Future clipboard mode

When P2 adds clipboard paste, test temporary write/paste/conditional restore, external clipboard races, own-event suppression, and cleanup failures. These are not P1 implementation requirements.

## Compatibility matrix

For every candidate support claim, record:

- OS edition/version/build and architecture;
- interactive session type;
- backend selected;
- application name/version/category;
- text fixture class;
- observed result and limitations;
- whether evidence came from CI, controlled E2E, or manual representative testing.

Update `COMPATIBILITY.md` only from evidence. A successful basic edit control does not imply all Windows applications are supported.

## P1 CI baseline

P1 should establish, then evolve:

1. formatting check;
2. clippy/lint policy;
3. deterministic core/app tests;
4. Windows workspace build/test;
5. non-Windows build/test of platform-neutral crates to detect Win32 leakage;
6. dependency lockfile and license/security inspection appropriate to project policy.

Native desktop input evidence remains interactive even when compilation/unit tests run in CI.

## Responsiveness and performance

Measure rather than assume:

- startup and idle footprint;
- message-loop responsiveness while injecting;
- semantic atoms/chars per second;
- native batch duration;
- cancellation request to last-dispatch latency;
- focus-check cadence;
- clipboard acquisition/retry latency;
- long-fixture memory use.

P1-S01 recommends initial bounds; P1-10 records measured behavior. A performance optimization is rejected if it weakens cancellation, target, modifier, or privacy guarantees.

## Privacy sentinel testing

Use a distinctive generated marker and search ordinary logs, status snapshots, test artifacts, and crash/debug output produced by the test. The marker may appear only in explicitly controlled target/assertion buffers.

Also review for:

- content prefixes/suffixes;
- persistent clipboard fingerprints/hashes;
- window titles/focused-field content;
- accidental test fixture persistence outside the harness.

## Evidence report

A phase/release candidate report records:

- exact commit SHA;
- toolchain/dependency summary;
- commands/checks actually run;
- platform environments;
- controlled E2E results;
- representative target results;
- measured responsiveness bounds;
- privacy sentinel result;
- known skipped/unverified cases;
- compatibility claim changes;
- linked blockers and final gate recommendation.
