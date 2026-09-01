# Security and Privacy Model

## Security posture

ClipType handles data that may contain passwords, API tokens, private messages, source code, personal information, and one-time codes. Clipboard and injected text are therefore classified as **sensitive by default**.

Privacy and user-control constraints are architecture requirements, not optional settings.

## Data lifecycle

Default flow:

```text
explicit trigger
  -> capture destination evidence
  -> OS clipboard
  -> bounded ephemeral process memory
  -> bounded native input batches
  -> target application
```

Clipboard text MUST NOT enter persistent storage unless a future explicitly scoped feature and accepted decision changes this model. V1 has no clipboard history.

P1 captures the destination before clipboard acquisition so a contended clipboard read cannot silently retarget the operation to whichever application happens to be focused later.

## Data bounds

Clipboard data and native metadata are untrusted inputs.

P1 requires:

- a configured maximum payload size/length;
- scans and copies bounded by the native allocation size;
- bounded clipboard contention/retry;
- bounded native input batches;
- bounded modifier-settle and cancellation/focus checkpoints;
- explicit failure for malformed or excessive data.

The application must not search indefinitely for terminators, allocate from unchecked native lengths, wait forever for clipboard ownership, or submit the whole unbounded payload as one uncancellable native operation.

## Logging rules

### Forbidden

- clipboard/injected plaintext;
- prefixes, suffixes, or samples of content;
- focused field contents;
- window titles by default;
- raw or unrelated user keystroke capture;
- long-lived content hashes/fingerprints intended to correlate clipboard values;
- secrets in panic/crash annotations;
- serialized sensitive-text wrappers in ordinary diagnostics.

### Allowed

- payload length or coarse bucket;
- platform/backend;
- non-sensitive application/process identity;
- timing/duration and batch/checkpoint counts;
- capability/permission/integrity evidence state;
- result/error category;
- cancellation/target/modifier abort flags.

## Network rule

V1 core functionality requires no network access. Clipboard/injected text may not be transmitted. Update checks, telemetry, crash upload, or remote features require separate review and remain content-blind unless a new explicit product/security decision says otherwise.

## Threat model

### Accidental injection into the wrong target

Mitigations:

- explicit manual trigger by default;
- atomically reserve one active session;
- capture destination evidence immediately at trigger time;
- revalidate before first and later bounded batches;
- stop on known change, disappearance, or strict evidence loss;
- never refocus an old target or adopt a new target to finish;
- immediate cancellation path.

Limitation: platform evidence may not distinguish two logical text fields inside one native render host. ClipType documents this limitation and does not make an unsupported exact-caret guarantee.

### Trigger/modifier contamination

A global hotkey's modifiers may still be physically held when injection begins, or a user may press a conflicting modifier mid-session.

Mitigations:

- bounded pre-dispatch modifier-release gate;
- checks at later batch boundaries when needed;
- typed conflict/timeout result;
- ClipType never releases arbitrary physical user keys;
- no broad global keyboard hook merely to observe unrelated input.

### Partial or ambiguous synthetic input

A native input API may accept only part of an event batch, and event counts may not identify a valid Unicode-text prefix.

Mitigations:

- bounded batches;
- complete/none/partial/progress-unknown outcomes;
- no automatic retry after partial or unknown input;
- content-free progress metadata only;
- stop future batches on ambiguous native progress.

### Clipboard acquisition race/contention

Another process may hold or change the clipboard while ClipType prepares an operation.

Mitigations:

- destination captured before acquisition;
- current-text read only after explicit trigger;
- bounded retry budget external to the low-level FFI wrapper so cancellation remains observable;
- copy borrowed native data before releasing the clipboard;
- payload/allocation bounds;
- no continuous plaintext watcher or history in P1.

### Future clipboard-paste restoration race

Another actor may change the clipboard between temporary write and restoration.

Future mitigations:

- generation/ownership evidence;
- never overwrite a newer external clipboard merely to restore an older snapshot;
- typed cleanup errors;
- bounded transaction timing.

This is P2 scope.

### Sensitive-data leakage through diagnostics

Mitigations:

- structured allowlist logging;
- redacted/content-free `Debug` and error representations for sensitive wrappers;
- privacy-sentinel tests;
- content-free crash context;
- independent review of logs, snapshots, and artifacts.

### Privilege abuse

Mitigations:

- main application runs unprivileged;
- do not bypass Windows UIPI, macOS consent, or Wayland policy;
- distinguish known security restriction from blocked cause unknown;
- do not elevate automatically;
- isolate any future Linux input privilege into a minimal helper only if approved;
- minimal local protocol and caller authorization for such a helper.

### Synthetic input surprise or denial of control

Mitigations:

- manual activation;
- visible non-sensitive active state;
- independent cancellation event path;
- responsive message-loop owner;
- bounded event batches;
- one active worker;
- fail closed on target or modifier uncertainty according to policy.

### Malicious clipboard content

Clipboard text is data, not code.

Mitigations:

- no shell evaluation;
- no template/macro evaluation in V1;
- explicit control-character handling;
- no construction of shell command strings from clipboard data;
- native events produced from validated semantic text elements.

Target applications can still assign operational meaning to intentionally emitted keys. For example, a terminal may treat a line break as command submission. Tests use benign fixtures, and product messaging must not claim that arbitrary multiline terminal injection is inert or universally safe.

## OS permission behavior

Permissions are explained and requested only when needed. Denial is a valid state. ClipType remains controllable and provides remediation guidance without coercive repeated prompts.

For Windows, evidence that a target is higher integrity may produce a known restriction before dispatch. A zero native insertion result without sufficient integrity evidence is not automatically labelled UIPI.

## Memory handling

Rust reduces accidental memory-safety bugs but does not guarantee secret erasure. V1 minimizes plaintext lifetime, copies, serialization, and ownership scope. Sensitive text wrappers should avoid ordinary diagnostic output and casual cloning.

Explicit zeroization may be evaluated for bounded owned buffers, but the project must not claim elimination of copies held by OS clipboard APIs, allocator/runtime behavior, GUI frameworks, crash dumps, or target applications.

## Crash handling

Crash reports and minidumps can contain process memory. Distribution builds should configure crash collection conservatively, document captured data, and never automatically upload raw dumps containing process memory without explicit informed consent.

P1 diagnostics do not deliberately include the active sensitive payload in panic annotations or status snapshots.

## Supply chain

Dependencies touching clipboard, input, FFI, serialization, logging, crash handling, or privileged helpers receive heightened review. Lockfiles, license compatibility, reproducible practices where practical, provenance/signing, and vulnerability checks become release requirements as implementation matures.

Project licensing/contribution terms must be decided before P1 production code is merged.

## P1 security invariants checklist

P1 is blocked if any are false:

- [ ] explicit-trigger default preserved;
- [ ] destination captured before clipboard acquisition;
- [ ] only one active session/worker can exist;
- [ ] hotkey/message-loop owner remains responsive;
- [ ] payload, clipboard retry, modifier wait, and native dispatch are bounded;
- [ ] no clipboard/injected plaintext persistence or ordinary logging;
- [ ] no continuous clipboard history/listener/write/restore path;
- [ ] physical user modifiers are never forcibly released;
- [ ] cancellation stops later batches within the measured bound;
- [ ] known target change stops later batches;
- [ ] focus-evidence limitations are documented honestly;
- [ ] partial/unknown native input is never automatically retried;
- [ ] no security-boundary bypass or false UIPI certainty;
- [ ] native data/pointers/lengths/handles are validated and cleaned up;
- [ ] privacy sentinel is absent from ordinary logs/artifacts.

## Long-term release invariants

A release remains blocked if:

- clipboard/injected plaintext is persisted or transmitted by default;
- privileged helpers are broader than necessary;
- future clipboard restoration can overwrite a newer external value;
- compatibility/security claims exceed evidence;
- crash/telemetry paths expose sensitive process memory without informed consent.
