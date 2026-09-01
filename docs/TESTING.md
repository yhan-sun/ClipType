# Testing Strategy

## Objectives

Testing must prove not only that text appears, but that ClipType preserves safety invariants under cancellation, focus changes, clipboard races, Unicode, permissions, and platform restrictions.

## Evidence classes

### 1. Pure deterministic tests

Platform-independent tests cover text normalization, planning, capability gating, transition rules, cancellation decisions, focus-change decisions, retry/idempotency rules, configuration validation, error categorization, and content-free diagnostics.

### 2. Native contract and runner probes

Native adapters and bounded research probes run on the matching OS to verify compilation, ownership/cleanup, message-loop/thread assumptions, error mapping, and non-interactive mechanisms that the runner can genuinely exercise.

A hosted runner is not automatically an unlocked representative desktop. Its result must not be used as proof that a global hotkey or `SendInput` reached another foreground application.

### 3. Controlled interactive E2E

A purpose-built target in an unlocked Windows desktop verifies delivered fixture text, focus changes, cancellation, modifiers, one-session arbitration, and lifecycle outcomes. Only generated fixture text may be retained inside the controlled assertion buffer.

### 4. Representative target observations

Manually exercise named application/version categories. These observations inform compatibility wording but do not imply universal application support.

## Required text fixtures

Use generated, non-sensitive fixtures covering:

- ASCII and punctuation;
- LF, CRLF, and lone CR normalization;
- Tab policy;
- CJK;
- emoji/supplementary Unicode;
- combining marks;
- long multi-batch text;
- maximum payload boundary and over-limit input;
- unsupported controls with expected rejection.

## Required P1 behavioral cases

### Keyboard path

- complete short injection;
- multi-batch long injection;
- cancel before and during dispatch;
- focus switch and target closure;
- target evidence unavailable under strict policy;
- trigger modifier still held;
- later conflicting modifier;
- partial/progress-unknown native dispatch with no retry;
- higher-integrity known restriction versus blocked cause unknown;
- rapid second trigger -> busy;
- shutdown while idle and active.

### Clipboard acquisition

- current ASCII/Unicode text;
- empty and non-text clipboard;
- transient busy then success within budget;
- busy budget exhausted;
- malformed/missing terminator where safely testable;
- configured hard bound exceeded;
- native lock/handle cleanup;
- no listener/history/write/restore in P1.

## Compatibility matrix

For every candidate observation record:

- exact commit SHA;
- OS edition/version/build/architecture;
- session/evidence class;
- backend and configured bounds;
- application/version/category;
- text and safety cases exercised;
- result and limitation;
- skipped/unverified paths.

Update `COMPATIBILITY.md` only from observed evidence.

## CI stages

As implementation arrives:

1. formatting;
2. deterministic tests;
3. non-Windows checks for native-neutral crates;
4. Windows workspace checks/tests/lints;
5. focused Windows contract probes;
6. dependency/license/security scanning;
7. packaging smoke tests in later phases;
8. release-only signing/notarization verification.

## Performance and boundedness

Track:

- startup and idle footprint;
- characters/elements per second;
- clipboard acquisition/retry time;
- modifier settle time;
- cancellation and focus-check latency;
- per-dispatch native event count;
- large-payload memory use;
- shutdown/join time.

An optimization is rejected if it weakens target, cancellation, modifier, clipboard-bound, or partial-result safety.

## Privacy sentinel

P1 interactive tests use a distinctive synthetic marker. Search ordinary logs, status output, snapshots, crash/debug artifacts, and persistent files. The marker may appear only in the controlled target/assertion data.

Also inspect for prefixes/suffixes, content fingerprints, window titles, focused-field text, and unrelated keystroke capture.

## Release and phase evidence

A P1 gate report records:

- reviewed commit and merge order;
- automated commands/jobs;
- controlled interactive evidence;
- representative target observations;
- measured timing/bounds;
- privacy sentinel result;
- skipped/unverified paths;
- compatibility wording changes;
- blocking issues and final recommendation.
