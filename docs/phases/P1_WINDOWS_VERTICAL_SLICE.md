# P1 Windows Vertical Slice Execution Plan

**Status:** Current implementation phase  
**Tracking issue:** [#1](../../issues/1)  
**Native discovery gate:** [#13](../../issues/13)  
**Governance gate:** [#14](../../issues/14)

This document turns the high-level P1 roadmap into an executable delivery plan. Product behavior remains normative in the top-level documents under `docs/`; this file owns P1 sequencing, integration checkpoints, issue boundaries, and evidence required to leave the phase.

## 1. P1 outcome

P1 must prove one complete, safe Windows path:

```text
explicit global hotkey
  -> reserve the single session slot
  -> capture destination evidence immediately
  -> wait for trigger modifiers to be released
  -> read current CF_UNICODETEXT
  -> normalize/validate text
  -> create immutable keyboard plan
  -> revalidate destination
  -> dispatch bounded SendInput batches
  -> recheck cancellation, focus evidence, and modifier safety
  -> publish a content-free result
  -> release the session slot
```

The output is a development-quality Windows executable and reproducible evidence. P1 is not a public beta or polished Windows product.

## 2. Evidence-before-freeze rule

P1-S01 validates native facts that constrain shared contracts before those contracts are frozen. It may combine:

- official Microsoft API contracts;
- content-free automated Windows runner probes;
- interactive desktop observations when available.

A conservative contract may proceed from official and automated evidence only when it explicitly represents unknown/degraded states and does not convert unverified target behavior into a support claim. Controlled interactive-desktop evidence remains mandatory in P1-10 before P1 can complete.

## 3. Fixed P1 scope

### In scope

- a minimal Rust workspace and CI baseline;
- a Windows-only vertical slice;
- current plain Unicode clipboard text read at trigger time;
- one global trigger plus an explicit cancellation path;
- target/focus evidence and fail-safe revalidation;
- Unicode-oriented native keyboard injection;
- ASCII, CJK, combining marks, supplementary Unicode, multiline, and explicit Tab/control handling;
- one active session maximum;
- bounded dispatch/cancellation/focus checkpoints;
- typed, content-free status and errors;
- controlled E2E harness and representative target evidence;
- independent architecture/security/privacy gate review.

### Out of scope

- clipboard listener/history;
- clipboard-paste backend and restore transaction;
- automatic backend selection;
- polished tray/settings UI;
- installer, auto-start, signing, or release distribution;
- macOS, X11, or Wayland crates/stubs;
- app profiles;
- low-level global keyboard hooks;
- exact-caret guarantees through UI Automation;
- retrying partial or unknown synthetic input;
- bypassing Windows security boundaries.

## 4. P1 repository and dependency graph

Only these implementation packages are created:

```text
crates/cliptype-core
crates/cliptype-platform
crates/cliptype-app
crates/cliptype-windows
apps/cliptype
```

Dependency direction is a diamond, not a linear chain:

```text
                  cliptype-core
                    ^       ^
                    |       |
          cliptype-platform |
             ^          ^   |
             |          |   |
      cliptype-app   cliptype-windows
             ^          ^
              \        /
               apps/cliptype
```

Normative edges:

- `cliptype-core`: platform-independent values and pure policy; no Win32 imports.
- `cliptype-platform -> cliptype-core`: ports, capability/evidence contracts, native-neutral results.
- `cliptype-app -> cliptype-core + cliptype-platform`: use cases and live session coordination.
- `cliptype-windows -> cliptype-core + cliptype-platform`: safe Windows implementations; it does not depend on `cliptype-app`.
- `apps/cliptype -> cliptype-app + cliptype-windows`: composition root and Windows message-loop host.

No generic `utils`, cross-platform placeholder, GUI-framework, or helper-process crate is created in P1.

## 5. Runtime ownership model

```text
Windows message-loop owner
  - owns RegisterHotKey / UnregisterHotKey
  - receives trigger, cancel, and shutdown events
  - never performs clipboard retries or native text dispatch
  - calls a narrow application command surface
                |
                v
Application session coordinator
  - atomically reserves one session slot
  - captures target evidence immediately for a trigger
  - creates cancellation token / status channel
  - starts one injection worker
                |
                v
Injection worker
  - waits for trigger modifiers to settle within a bound
  - reads clipboard with bounded contention handling
  - plans and dispatches bounded batches
  - checks cancellation/focus/modifier state between batches
  - reports typed terminal result
                |
                v
Message-loop/status surface
  - remains responsive
  - renders content-free development status
```

Required properties:

1. The session slot is reserved before worker creation.
2. The hotkey/message-loop owner remains responsive while injection is active.
3. Cancellation reaches the active token without waiting for the worker.
4. Every native dispatch batch is bounded.
5. Session state returns to idle on every contained terminal path.
6. Platform-thread affinity is explicit and tested.

## 6. Trigger-to-dispatch ordering

Required order:

1. receive explicit trigger;
2. reserve session slot or return `busy`;
3. capture initial target evidence immediately;
4. start worker;
5. wait for trigger modifiers to be released within a bounded timeout;
6. acquire current clipboard text using bounded retry on transient busy state;
7. validate and normalize text;
8. acquire capability/integrity evidence;
9. create immutable plan;
10. recapture target and abort if evidence changed or became unsafe;
11. dispatch one bounded batch;
12. before every following batch, check cancellation, target evidence, and conflicting modifier state;
13. classify complete/partial/cancelled/target-changed/blocked/native failure;
14. publish content-free status and release the session slot.

The application never adopts a new destination after a change merely to finish the payload.

## 7. Text and native dispatch contract

Core policy models validated text as semantic atoms:

- printable Unicode scalar;
- normalized line break;
- Tab when enabled;
- rejected unsupported control character.

Requirements:

- preserve Unicode scalar order and combining marks;
- normalize CRLF and lone CR/LF according to one documented policy;
- do not convert arbitrary controls into commands;
- translate semantic atoms to Win32 events only inside `cliptype-windows`;
- supplementary characters remain one semantic element at the core boundary;
- a partial native event count may be `progress unknown` rather than an invented text prefix;
- partial/unknown dispatch is never automatically retried.

## 8. Target evidence and focus guarantee

A Windows target fingerprint uses non-content evidence such as foreground top-level window, owning process and GUI thread, active/focused native window, and optional integrity relation.

Window titles, focused text, and accessibility-tree contents are excluded.

Fail-safe rules:

- known target change aborts before the next batch;
- target disappearance aborts;
- under strict P1 policy, evidence that becomes unavailable after dispatch begins aborts;
- same-render-host logical field changes may be undetectable and must be recorded as a limitation.

P1 promises bounded revalidation against available Windows window/thread focus evidence, not proof that an exact logical caret never moved inside one render host.

## 9. Modifier safety

- never release the user's physical modifier keys;
- use a bounded pre-dispatch modifier-release gate;
- fail with a typed conflict/timeout result if state does not become safe;
- inspect modifier state at later boundaries where required;
- use no-repeat semantics for development hotkeys;
- do not add a global keylogger-style hook.

## 10. UIPI and blocked-input reporting

- known higher-integrity relation is a security-boundary restriction;
- unknown relation plus zero inserted events is blocked/native-cause-unknown;
- zero return is not always asserted to be UIPI;
- ClipType never elevates automatically or bypasses UIPI.

## 11. Work waves and merge order

### Wave 0 — governance and foundation

- [#14 P1-G01](../../issues/14): license/contribution decision.
- [#2 P1-01](../../issues/2): workspace, dependency graph, toolchain, baseline CI.

### Wave 1 — native discovery

- [#13 P1-S01](../../issues/13): official API analysis plus bounded Windows probe and contract recommendations.

### Wave 2 — contract freeze

- [#3 P1-02](../../issues/3): native-neutral contracts informed by P1-S01.

The merged P1-02 commit becomes the common base for parallel implementation work.

### Wave 3 — pure policy and adapters

- [#4 P1-03](../../issues/4): pure text/plan/transition policy.
- [#5 P1-04](../../issues/5): Windows current Unicode clipboard read.
- [#6 P1-05](../../issues/6): Windows target/focus and integrity evidence.
- [#7 P1-06](../../issues/7): bounded Windows Unicode `SendInput` backend.
- [#8 P1-07](../../issues/8): trigger/cancel event source and message-loop lifecycle.

### Wave 4 — runtime integration

- [#9 P1-08](../../issues/9): live session coordinator and fake-port integration tests.
- [#10 P1-09](../../issues/10): Windows composition root, message loop, worker hosting, and minimal development status.

### Wave 5 — evidence and gate review

- [#11 P1-10](../../issues/11): controlled E2E harness, interactive matrix, privacy sentinel, compatibility evidence.
- [#12 P1-11](../../issues/12): independent architecture/security/privacy gate review.

Critical path:

```text
P1-01 -> P1-S01 -> P1-02
                    |
                    +-> P1-03..P1-07
                              |
                              v
                           P1-08
                              |
                              v
                           P1-09
                              |
                              v
                           P1-10
                              |
                              v
                           P1-11
```

## 12. Issue responsibility boundaries

| Task | Owns | Must not own |
|---|---|---|
| P1-01 | workspace, crate graph, toolchain/build/CI | product behavior or Win32 mechanisms |
| P1-S01 | native evidence and conservative recommendations | production adapters or support claims |
| P1-02 | traits/value types/results/capabilities | runtime loop or Win32 handles in policy |
| P1-03 | normalization, planning, pure transition reducer | threads, channels, live ports, Win32 |
| P1-04 | clipboard acquisition mechanism | history/listener/write/restore |
| P1-05 | target/focus/integrity evidence | focused text, refocusing, UI Automation |
| P1-06 | semantic atom to bounded native dispatch | session orchestration, retries, policy |
| P1-07 | hotkey registration/event/lifecycle | injection policy or arbitrary key capture |
| P1-08 | one-session runtime coordinator | Windows message-pump presentation |
| P1-09 | composition root and responsive host | polished UI/installer/settings |
| P1-10 | evidence, fixtures, compatibility report | hiding failures by changing behavior |
| P1-11 | independent gate findings | feature implementation bundled into review |

## 13. Verification strategy

Automated checks cover formatting, lint, deterministic tests, Windows build/test, and non-Windows boundary checks. Headless or hosted CI is build/contract evidence, not proof of native input into a user's application.

P1-10 must still record controlled interactive evidence for the complete trigger-to-text path, cancellation, focus changes, modifiers, Unicode/line-break behavior, target categories, UIPI limitations, and privacy sentinel.

## 14. PR and agent protocol

- one task packet normally produces one focused PR;
- every PR records base/head SHA, docs/ADRs/handoffs read, changes, exact checks, unverified paths, and privacy/security review;
- parallel adapter work may not fork shared contracts;
- no agent merges, force-pushes shared work, tags, publishes, or declares P1 complete without explicit authority;
- P1-11 should be independent from the primary implementation path where practical.

## 15. P1 exit gate

P1 completes only when:

- required build/unit/native checks pass;
- the complete path works in a controlled interactive Windows session;
- one-session, cancellation, focus, modifier, partial-result, payload-bound, and shutdown behavior is evidenced;
- plaintext is absent from ordinary logs and persistent artifacts;
- compatibility wording matches observed evidence;
- P1-11 returns `PASS` or `PASS WITH NON-BLOCKING FOLLOW-UPS`;
- no unresolved privacy, destination-safety, privilege, or unbounded-work blocker remains.

Closing P1-S01 authorizes contract freeze only. Closing P1-00 still requires the interactive and independent final gates. No issue closure authorizes a tag, release, or broad support claim.
