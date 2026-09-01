# Injection Engine Specification

## Purpose

The injection engine converts one explicit user trigger plus a clipboard snapshot into a bounded, cancellable attempt to deliver text to the focused target.

## Supported modes

### `keyboard`
Emit synthetic text/keyboard events through the platform keyboard backend.

Use when:
- normal paste is unavailable or undesirable;
- target environment handles synthetic input better than clipboard transfer;
- payload size and backend capabilities make per-event injection reasonable.

### `clipboard`
Temporarily place the desired text on the system clipboard, issue the platform paste chord/action, then restore the prior clipboard when safe and configured.

Use when:
- large payloads make per-event injection slow;
- the target supports normal paste reliably;
- clipboard write/restore is available.

### `auto`
The planner selects an eligible mode based on capabilities, target profile, payload properties, and configurable thresholds.

The conceptual design follows a proven pattern used by text expansion tools: separate simulated-input and clipboard backends, with automatic selection for long text. The exact ClipType policy is independently specified here and MUST be tested rather than copied from another project.

## Planner inputs

An `InjectionPlan` is derived from:

- requested mode;
- payload length and content classes (Unicode/newlines/control characters);
- platform/backend capabilities;
- target application identity/profile when available;
- clipboard write/restore availability;
- configured keyboard interval/chunk size;
- permission state;
- safety policy.

## Planner output

Conceptual fields:

```text
backend: keyboard | clipboard
focus_policy: strict | supported-best-effort
keyboard_interval
chunk_size
restore_clipboard
paste_chord/platform action
capability assumptions
```

The plan is immutable for an active session. If assumptions become false, abort or fail; do not silently switch mechanisms mid-stream unless a future ADR explicitly defines safe recovery.

## Default auto policy

Initial implementation should start simple and data-driven:

- prefer keyboard mode for short payloads when Unicode/text injection is supported;
- prefer clipboard mode above a conservative length threshold when safe clipboard write/restore and paste are supported;
- allow explicit `keyboard`/`clipboard` override;
- target-specific profiles may override thresholds only after compatibility evidence exists.

The exact numeric default threshold is not frozen in documentation before benchmarks. It is a configuration default, not an architectural constant.

## State machine

```text
Idle
  | trigger
  v
AcquireClipboard
  | text available
  v
CaptureTarget
  v
Plan
  | eligible
  v
Injecting ------------------+
  |                         |
  | cancel                  | target/capability failure
  v                         v
Cancelling                Failed
  |
  v
Cleanup/Restore
  |
  +---- success ----------> Completed
  +---- cleanup failure --> FailedWithCleanupError
  |
  v
Idle
```

Failures in acquisition/planning return to Idle after emitting non-sensitive status.

## Focus guard

Before injection, capture a stable target identity using the strongest safe evidence the platform exposes.

During multi-chunk injection:
- re-check target identity at bounded intervals;
- abort if the target materially changes under `strict` policy;
- never continue typing into a newly focused application merely to finish the payload.

Where a platform cannot provide a reliable global target identity, mark the capability degraded and document the actual guarantee.

## Cancellation

Cancellation is a first-class control path, not an error afterthought.

Requirements:
- UI/hotkey cancellation request is accepted while injection is active;
- keyboard backend checks cancellation between bounded event groups;
- clipboard backend avoids issuing paste if cancellation arrives before paste dispatch;
- cleanup/clipboard restoration still runs after cancellation where safe;
- cancellation latency gets a testable upper bound once backend timing is implemented.

## Modifier contamination

Synthetic input can be affected by physical modifiers already pressed by the user. Each platform backend MUST define how it detects/handles modifier state.

The engine MUST NOT blindly release arbitrary physical user keys. Strategies may include delaying, rejecting while conflicting modifiers are down, or using Unicode/text event facilities that minimize layout/modifier dependence.

## Unicode

Text semantics, not US-keyboard scancode emulation, are the primary requirement.

Backends SHOULD use native Unicode/text injection facilities where available. When a backend can only express physical keys/keymaps, its Unicode limitations must be surfaced in capabilities and compatibility docs.

## Newlines and control characters

The planner/backend MUST explicitly map:
- LF/CRLF normalization;
- Tab;
- supported printable Unicode;
- unsupported control characters.

No backend should accidentally convert arbitrary control bytes into commands.

## Clipboard-paste transaction

Conceptual transaction:

1. snapshot prior clipboard ownership/content needed for restoration;
2. mark an internal self-write generation/token;
3. write desired text;
4. verify write when the platform supports a reliable check;
5. verify focus guard;
6. issue paste action;
7. wait only as required for ownership/consumer timing;
8. restore prior clipboard if configured and still safe to do so;
9. suppress/identify ClipType's own clipboard-change notifications.

Race rule: if another external actor changes the clipboard during the transaction, ClipType MUST NOT overwrite the newer external clipboard merely to restore an old snapshot. Restoration requires ownership/generation evidence.

## Partial injection

A keyboard backend may fail after typing a prefix. The result MUST distinguish `partial` from `none` and `complete` when knowable. ClipType must not auto-retry a partial injection because that could duplicate text.

## Retries

Automatic retries are conservative:
- clipboard acquisition may retry bounded transient busy failures;
- synthetic input dispatch may not retry an unknown/partial result as if idempotent;
- permission or security-boundary failures do not retry blindly.

## Performance

Correctness and cancellation dominate raw typing speed. Benchmarks should measure:
- chars/sec by backend/platform;
- CPU while injecting;
- cancellation latency;
- planner overhead;
- clipboard transaction latency.

Performance tuning must not weaken focus or restoration safety.