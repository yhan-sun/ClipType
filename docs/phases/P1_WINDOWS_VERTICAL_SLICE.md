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

## 2. Re-analysis: changes from the original decomposition

The first issue set was directionally correct but had four execution risks.

### 2.1 Do not freeze ports before native evidence

Win32 message-loop ownership, `SendInput` progress, Unicode/newline behavior, foreground evidence, modifier contamination, and UIPI diagnostics constrain the contracts. Therefore [P1-S01 #13](../../issues/13) runs after workspace bootstrap and before P1-02 contract freeze.

### 2.2 Separate pure policy from runtime orchestration

- **P1-03** owns pure plan/text/state-transition policy only.
- **P1-08** owns the live session coordinator, concurrency, ports, cancellation token, focus rechecks, and lifecycle.

P1-03 must not create a second orchestration layer, and P1-08 must not reimplement policy in callbacks.

### 2.3 Keep the Win32 event loop responsive

A hotkey is delivered through the owning thread's message queue. Injection must not block that queue. P1 uses a responsive message-loop owner plus a bounded worker/session coordinator; a full async runtime is not introduced unless the native spike proves a concrete need.

### 2.4 Make support claims match actual evidence

Windows focus evidence cannot automatically be equated with an exact logical text field. P1 can detect top-level window/thread/focused-native-window changes where exposed, but some applications place many logical fields inside one render host. Compatibility wording must report the actual guarantee.

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

P1-S01 must validate the final details, but the intended ownership is:

```text
Windows message-loop owner
  - owns RegisterHotKey / UnregisterHotKey
  - receives trigger, cancel, and shutdown events
  - never performs long text dispatch
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

### Required concurrency properties

1. The session slot is reserved before worker creation, so two rapid triggers cannot create two sessions.
2. The hotkey/message-loop owner remains responsive while injection is active.
3. Cancellation reaches the active token without waiting for the injection worker to finish.
4. Every native dispatch batch is bounded; an entire unbounded payload is never one uncancellable operation.
5. Session state returns to idle on every contained terminal path.
6. Platform-thread affinity is explicit and tested.

## 6. Trigger-to-dispatch ordering

Destination evidence is captured before potentially contended clipboard acquisition. This preserves the target intended at trigger time.

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

The application must never recapture a new destination after a change and continue there merely to finish the payload.

## 7. Text and native dispatch contract

P1 should model validated text as semantic atoms rather than exposing Win32 event arrays to core policy. The exact names are implementation choices, but the semantics must distinguish:

- printable Unicode scalar;
- normalized line break;
- Tab when enabled by P1 policy;
- rejected unsupported control character.

Requirements:

- preserve Unicode scalar order and combining marks;
- convert CRLF and lone CR/LF according to one documented normalization policy;
- do not silently convert arbitrary controls into key commands;
- translate semantic atoms to Win32 events only inside `cliptype-windows`;
- supplementary characters may require multiple UTF-16 units but remain one semantic input element at the core boundary;
- a partial native event count may not map to a trustworthy Unicode-text prefix, so the result may be `partial progress unknown` rather than inventing an exact typed prefix;
- partial/unknown dispatch is never automatically retried.

Newline/Tab behavior must be validated by target category in P1-S01 and P1-10. A multiline payload may be interpreted as commands by terminals; terminal testing must use benign controlled fixtures and must not be described as universally safe.

## 8. Target evidence and focus guarantee

A Windows target fingerprint should use only non-content evidence, expected to include the strongest practical combination of:

- foreground top-level window identity;
- owning process and GUI thread identity;
- active/focused native window identity when available;
- optional integrity relation needed for security-boundary reporting.

Window titles, focused text, and accessibility-tree contents are excluded.

### Fail-safe rules

- a known target change aborts before the next batch;
- target disappearance aborts;
- under strict P1 policy, evidence that becomes unavailable/ambiguous after dispatch begins aborts rather than continuing blindly;
- transient native activation races are classified explicitly;
- same-render-host logical field changes may be undetectable and must be recorded as a compatibility limitation.

P1 therefore promises **bounded revalidation against available Windows window/thread focus evidence**, not proof that the exact logical caret never moved inside one render host.

## 9. Modifier safety

The trigger's Ctrl/Alt/Shift/Win keys may still be physically down when the hotkey event arrives. Existing physical key state can interfere with synthetic input.

P1 rules:

- never release the user's physical modifier keys;
- use a bounded pre-dispatch modifier-release gate;
- fail with a typed modifier-conflict/timeout result if the state does not become safe;
- inspect conflicting modifier state before subsequent batches where the chosen mechanism requires it;
- use `MOD_NOREPEAT` for the development trigger where supported;
- do not add a global keylogger-style hook merely to improve this behavior.

## 10. UIPI and blocked-input reporting

P1 distinguishes evidence from diagnosis:

- if integrity comparison reliably proves the target is higher integrity, report a security-boundary restriction before dispatch;
- if the relationship is unknown and `SendInput` inserts no events, report a blocked/native-unknown result with a possible elevated-target remediation hint;
- do not label every zero-return as definitely caused by UIPI;
- never elevate automatically or bypass UIPI.

## 11. Work waves and merge order

### Wave 0 — governance and foundation

- [#14 P1-G01](../../issues/14): license/contribution decision. May proceed in parallel but must close before the first production-code PR merges.
- [#2 P1-01](../../issues/2): workspace, dependency graph, pinned toolchain policy, baseline CI.

### Wave 1 — native discovery

- [#13 P1-S01](../../issues/13): interactive Windows mechanism and runtime spike.

P1-02 may not freeze contracts until the spike handoff is reviewed.

### Wave 2 — contract freeze

- [#3 P1-02](../../issues/3): native-neutral contracts informed by P1-S01.

The merged P1-02 commit becomes the common base for parallel implementation PRs.

### Wave 3 — parallel pure policy and adapters

- [#4 P1-03](../../issues/4): pure text/plan/state-transition policy.
- [#5 P1-04](../../issues/5): Windows current Unicode clipboard read.
- [#6 P1-05](../../issues/6): Windows target/focus and integrity evidence.
- [#7 P1-06](../../issues/7): bounded Windows Unicode `SendInput` backend.
- [#8 P1-07](../../issues/8): trigger/cancel event source and message-loop lifecycle.

These tasks may proceed in parallel only from the same accepted P1-02 contract base. Agents must not independently fork shared contracts. Contract pressure returns to P1-02 through a focused change/review.

### Wave 4 — runtime integration

- [#9 P1-08](../../issues/9): live session coordinator and fake-port integration tests.
- [#10 P1-09](../../issues/10): Windows composition root, message loop, worker hosting, and minimal development status.

P1-09 does not require a tray. A console/development surface is preferred over introducing product UI complexity.

### Wave 5 — evidence and gate review

- [#11 P1-10](../../issues/11): controlled E2E harness, interactive matrix, privacy sentinel scan, compatibility evidence.
- [#12 P1-11](../../issues/12): independent architecture/security/privacy gate review.

### Critical path

```text
P1-01 -> P1-S01 -> P1-02
                    |
                    +-> longest of P1-03..P1-07
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

P1-G01 is a parallel governance gate but blocks merging production implementation.

## 12. Issue responsibility boundaries

| Task | Owns | Must not own |
|---|---|---|
| P1-01 | workspace, crate graph, toolchain/build/CI baseline | product behavior or Win32 mechanisms |
| P1-S01 | empirical native evidence and recommendations | production adapters |
| P1-02 | traits/value types/results/capabilities | runtime loop or Windows handles in public policy |
| P1-03 | normalization, planning, pure transition reducer | threads, channels, live ports, Win32 |
| P1-04 | clipboard acquisition mechanism | history/listener/write/restore |
| P1-05 | target/focus/integrity evidence | focused text, refocusing, UI Automation |
| P1-06 | semantic-atom to bounded native input dispatch | session orchestration, retries, policy |
| P1-07 | hotkey registration/event/lifecycle | injection policy or arbitrary key capture |
| P1-08 | one-session runtime coordinator | Windows message pump presentation |
| P1-09 | composition root and responsive host | polished UI/installer/settings |
| P1-10 | evidence, fixtures, compatibility report | hiding failures by changing product behavior |
| P1-11 | independent gate findings | feature implementation bundled into review |

## 13. Risk-level alignment

Use the repository risk taxonomy consistently:

- P1-01, P1-02, P1-03: R1 unless an architecture decision expands scope.
- P1-S01, P1-04, P1-05, P1-06, P1-07, P1-08, P1-09: R2 native/integration; P1-06 additionally requires focused security review.
- Any new persistence, crash-memory upload, privileged process, IPC authorization, or broad input hook: R3 and outside the current plan without review.
- P1-G01, P1-10 compatibility recommendation, and P1-11 phase/support gate: R4.

Safety-critical does not mean inventing a different risk number; add the required reviewer while preserving the taxonomy.

## 14. PR and Agent execution protocol

1. One task packet normally produces one focused PR.
2. Every PR names the issue and uses `Closes #N` only when all acceptance criteria are met.
3. Each PR records:
   - base/head commit;
   - source-of-truth docs read;
   - files/subsystems changed;
   - contract/API changes;
   - exact checks run;
   - interactive Windows evidence, if any;
   - unverified paths;
   - privacy/security self-review;
   - intentionally deferred work.
4. Parallel Wave 3 PRs branch from the same merged P1-02 base.
5. Stacked PRs are avoided unless dependencies are explicit and the base PR is linked.
6. No agent merges, force-pushes shared work, tags, publishes, or declares P1 complete without explicit maintainer authority.
7. P1-11 should be reviewed by an agent/person independent of the primary P1 implementation path where practical.

## 15. Verification strategy

### Automated baseline

P1-01 establishes CI that can evolve with implementation:

- formatting check;
- clippy/lint policy;
- deterministic core/app tests;
- Windows workspace build/test;
- non-Windows verification that platform-neutral crates remain platform-neutral;
- dependency lockfile and dependency/license inspection appropriate to the selected license policy.

### Interactive Windows evidence

Native input behavior must be tested in a real interactive desktop session. Headless CI is compile/unit evidence, not proof of global hotkeys, foreground focus, or `SendInput` delivery.

### Required safety evidence

- two rapid triggers cannot create two sessions;
- message loop still receives cancel while worker injects;
- cancel stops future batches within a measured bound established from P1-S01;
- top-level target switch stops future batches;
- target closure stops future batches;
- modifier conflict is handled without releasing physical keys;
- partial/unknown native result is not retried;
- session always returns to idle;
- high-integrity behavior is reported without false certainty.

### Privacy sentinel

P1-10 uses a distinctive synthetic fixture and searches logs/test artifacts. The fixture must appear only in explicitly controlled target/assertion buffers, never in ordinary logs, errors, status snapshots, or persistent diagnostics.

## 16. Representative target matrix

Minimum categories:

- controlled native Win32 edit target;
- Chromium-family text field;
- VS Code/editor field;
- Windows Terminal or equivalent terminal input;
- elevated/high-integrity target for expected restriction evidence.

Rules:

- compatibility is recorded per tested application/version/environment;
- one success does not establish universal support;
- terminal multiline tests use benign, controlled input and explicitly acknowledge command interpretation risk;
- same-window logical field switching is tested where practical and limitations are recorded.

## 17. P1 exit gate

P1 may be recommended complete only when:

### Architecture/build

- [ ] P1-G01 is resolved before production implementation merges;
- [ ] only P1 package boundaries exist;
- [ ] crate dependency direction matches Section 4;
- [ ] platform-neutral crates compile/test without Win32 leakage;
- [ ] Windows workspace checks pass on the recorded toolchain;
- [ ] no unnecessary async/UI/helper framework was introduced.

### Functional

- [ ] global trigger reaches the application command surface;
- [ ] destination is captured at trigger time;
- [ ] current `CF_UNICODETEXT` is acquired safely;
- [ ] ASCII, CJK, combining, supplementary Unicode, newline, and Tab/control policy have evidence;
- [ ] bounded `SendInput` batches reach controlled and representative targets;
- [ ] hotkey conflict and shutdown lifecycle are surfaced.

### Safety

- [ ] one active session is enforced atomically;
- [ ] message loop remains responsive during injection;
- [ ] trigger modifiers are not allowed to contaminate first dispatch;
- [ ] cancellation and focus checks are bounded and measured;
- [ ] known target change prevents subsequent batches;
- [ ] partial/unknown dispatch is never retried;
- [ ] UIPI is not bypassed or falsely diagnosed;
- [ ] exact focus-evidence limitations are documented.

### Privacy/evidence

- [ ] clipboard/injected plaintext is absent from ordinary logs and persistent artifacts;
- [ ] clipboard contents are not persisted as history/cache;
- [ ] P1-10 records CI, interactive, and manual evidence separately;
- [ ] compatibility statements are evidence-backed;
- [ ] P1-11 returns `PASS` or `PASS WITH NON-BLOCKING FOLLOW-UPS`;
- [ ] remaining work is classified as P2 or a visible blocker.

Closing issues alone does not authorize a tag, release, or public compatibility promise.

## 18. Expected P1 handoff

The final phase handoff contains:

1. exact reviewed commit SHA;
2. completed issues/PRs and merge order;
3. Windows edition/version/build/architecture/session;
4. Rust toolchain and dependency summary;
5. crate graph;
6. runtime thread/message-loop diagram;
7. automated commands and results;
8. interactive E2E matrix;
9. measured cancellation/focus checkpoint behavior;
10. privacy sentinel method/result;
11. unsafe/FFI inventory;
12. known compatibility limitations;
13. P1-11 recommendation;
14. P2 entry recommendation.
