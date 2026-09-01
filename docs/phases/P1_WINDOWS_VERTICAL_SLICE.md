# P1 Windows Vertical Slice Execution Plan

**Status:** Current implementation phase  
**Tracking:** #1  
**Native discovery gate:** #13  
**License/governance gate:** #14

This document owns P1 sequencing, task boundaries, integration checkpoints, merge order, and exit evidence. Product, architecture, security, and compatibility semantics remain normative in the top-level documents under `docs/`.

## 1. Required P1 outcome

P1 proves one complete Windows path:

```text
explicit global trigger
  -> atomically reserve one session
  -> capture destination evidence immediately
  -> wait for trigger modifiers to become safe
  -> read bounded current CF_UNICODETEXT
  -> normalize and validate text
  -> create immutable keyboard plan
  -> revalidate destination
  -> dispatch bounded SendInput batches
  -> check cancel, target evidence, and modifiers between batches
  -> publish a content-free terminal result
  -> release the session slot
```

The output is a development-quality executable plus reproducible evidence. It is not a public beta, productized Windows UI, installer, or release.

## 2. Why the original P1 plan was changed

### 2.1 Native evidence must precede contract freeze

Win32 message-loop ownership, Unicode/line-break behavior, modifier state, focus evidence, clipboard contention, `SendInput` progress, and UIPI diagnostics constrain the shared contracts. Therefore #13 runs after workspace bootstrap and before #3.

### 2.2 Pure policy and live orchestration must not overlap

- #4 owns pure text normalization, plan construction, and transition/outcome rules.
- #9 owns the live session slot, worker, ports, cancellation token, retries, target checks, and cleanup.

There is one live state machine, hosted by the application coordinator and driven by pure policy.

### 2.3 The Win32 message loop must remain responsive

The hotkey registration/message-loop owner only translates native events into application commands. Clipboard contention and text injection execute through the bounded worker/coordinator path. A full async runtime is not introduced unless #13 demonstrates a concrete need.

### 2.4 Compatibility must describe available evidence, not idealized focus

Windows can often compare foreground window, process/thread, and native focused-window evidence. It may not distinguish two logical fields inside one render host. P1 promises bounded revalidation against available evidence, not universal exact-caret identity.

## 3. Scope

### In scope

- minimal Rust workspace and CI baseline;
- Windows-only implementation crates;
- current plain Unicode clipboard read at trigger time;
- one explicit global trigger and cancellation command;
- target/focus/integrity evidence;
- Unicode-oriented keyboard injection;
- ASCII, CJK, combining marks, supplementary Unicode, line breaks, Tab, and unsupported-control behavior;
- configured payload limit;
- one active session maximum;
- bounded clipboard retry, modifier wait, dispatch batches, and safety checkpoints;
- content-free typed diagnostics/status;
- controlled E2E harness and representative-target observations;
- independent architecture/security/privacy/evidence review.

### Out of scope

- clipboard listener, continuous capture, history, cache, write, paste, or restoration;
- `auto` backend selection and thresholds;
- polished tray/settings UI;
- installer, auto-start, signing, update system, or release distribution;
- macOS, X11, or Wayland crates/stubs;
- app profiles;
- low-level global keyboard hooks;
- UI Automation-based exact-caret tracking;
- retries after partial or unknown synthetic input;
- automatic elevation or security-boundary bypass.

## 4. P1 crate graph

Create only:

```text
crates/cliptype-core
crates/cliptype-platform
crates/cliptype-app
crates/cliptype-windows
apps/cliptype
```

Required dependency direction:

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

- `cliptype-core`: sensitive value types, normalization, plans, pure policy.
- `cliptype-platform`: native-neutral ports, capabilities, target/security evidence, results.
- `cliptype-app`: application use cases and live one-session coordinator.
- `cliptype-windows`: Win32 mechanisms behind safe contracts; no dependency on `cliptype-app`.
- `apps/cliptype`: composition root and Windows host.

Do not add generic utility, UI-framework, helper-process, or future-platform crates in P1.

## 5. Runtime ownership

```text
Windows message-loop owner
  - owns hotkey registration/unregistration and native command delivery
  - remains responsive
  - forwards typed trigger/cancel/shutdown commands
                |
                v
Application coordinator
  - atomically reserves one session
  - captures destination immediately
  - creates cancellation/status state
  - starts one bounded worker
                |
                v
Injection worker
  - waits for safe modifier state
  - performs bounded clipboard retry
  - validates/plans text
  - revalidates destination
  - dispatches one bounded native batch at a time
  - checks cancel/target/modifiers at every boundary
                |
                v
Content-free status surface
```

Required properties:

1. The session slot is reserved before worker creation.
2. Two rapid triggers cannot create two workers.
3. Cancellation is signalable without waiting for worker completion.
4. The message-loop owner never runs the whole injection operation.
5. Every wait/retry/native dispatch has a bound.
6. Every contained terminal path releases session state and native resources.

## 6. Trigger-to-dispatch ordering

1. Receive explicit trigger.
2. Reserve session slot or return `busy`.
3. Capture initial target evidence immediately.
4. Create cancellation/status state and start the worker.
5. Wait for trigger modifiers to settle within a bound.
6. Acquire current clipboard text with bounded externally visible retry.
7. Enforce native and configured payload bounds.
8. Normalize/validate semantic text.
9. Obtain capability/integrity evidence and create immutable plan.
10. Revalidate the original target before first dispatch.
11. Before each batch, check cancellation, target evidence, and modifier safety.
12. Dispatch exactly one bounded semantic batch.
13. Stop on cancel, target/evidence change, modifier conflict, partial/unknown progress, or failure.
14. Publish a content-free result and release the session slot.

The coordinator never adopts a newly focused target to finish an operation.

## 7. Text and native dispatch contract

Core policy operates on semantic elements:

- printable Unicode scalar;
- normalized line break;
- Tab when allowed;
- rejected unsupported control character.

Requirements:

- preserve scalar order and combining marks;
- use one documented CRLF/CR/LF normalization policy;
- keep Win32 `INPUT` arrays and UTF-16 event construction inside `cliptype-windows`;
- treat a supplementary scalar as one semantic element even when native translation uses multiple UTF-16 units;
- do not infer an exact typed text prefix when a partial native event count may split a key pair or UTF-16 sequence;
- never automatically retry partial or progress-unknown dispatch.

Terminal-like targets may assign operational meaning to emitted line breaks. E2E fixtures must be benign and compatibility wording must state this limitation.

## 8. Target evidence guarantee

The Windows adapter uses the strongest practical non-content evidence, expected to include:

- foreground top-level window;
- owning process and GUI thread;
- active/focused native window where available;
- optional integrity relationship for security reporting.

It excludes focused text, selection contents, accessibility-tree contents, and window titles by default.

Fail-safe rules:

- known target change aborts before the next batch;
- target disappearance aborts;
- under strict P1 policy, evidence becoming unavailable or ambiguous after dispatch starts aborts;
- target handles are not trusted without the contract-required identity context;
- render-host-limited logical-field changes are recorded as compatibility limitations.

## 9. Modifier safety

- use explicit trigger/cancel combinations with no-repeat behavior where supported;
- wait for trigger modifiers to be physically released within a bounded timeout;
- return a typed conflict/timeout if the state remains unsafe;
- recheck conflicting modifier state between batches where required;
- never release arbitrary physical user keys;
- do not add a broad global hook for this purpose.

## 10. UIPI and blocked-input reporting

- known evidence that the target is higher integrity produces a security-boundary outcome before dispatch;
- zero accepted native events without sufficient evidence produces a blocked/native-cause-unknown outcome;
- do not label every zero result as definitively UIPI;
- do not elevate automatically or bypass the boundary.

## 11. Work waves and merge order

### Wave 0 — foundation and governance

- #2 — workspace, crate graph, toolchain, lockfile policy, baseline CI.
- #14 — project license/contribution boundary; may run in parallel but blocks the first production-code merge.

### Wave 1 — native discovery

- #13 — interactive Windows mechanism/runtime spike.

Do not freeze #3 until #13 evidence is reviewed.

### Wave 2 — contract freeze

- #3 — native-neutral P1 contracts informed by #13.

The merged #3 commit becomes the common base for Wave 3 branches.

### Wave 3 — parallel pure policy and adapters

- #4 — pure normalization, planning, transition policy.
- #5 — bounded current Unicode clipboard acquisition.
- #6 — target/focus/integrity evidence.
- #7 — bounded Unicode `SendInput` dispatch and modifier observations.
- #8 — trigger/cancel hotkey event source and lifecycle.

Shared contract pressure returns to a focused #3 correction. Adapter agents must not fork contracts independently.

### Wave 4 — integration

- #9 — live one-session coordinator and fake-port safety tests.
- #10 — executable composition, responsive message loop/worker, development status.

A tray is optional and not a P1 acceptance condition.

### Wave 5 — evidence and independent gate

- #11 — controlled E2E, privacy sentinel, compatibility evidence.
- #12 — independent architecture, safety, privacy, governance, and evidence review.

Critical path:

```text
#2 -> #13 -> #3 -> longest of #4..#8 -> #9 -> #10 -> #11 -> #12
```

#14 is a parallel governance gate that must close before production implementation merges.

## 12. Task ownership table

| Task | Owns | Must not own |
|---|---|---|
| #2 | workspace, graph, toolchain, baseline CI | product/native behavior |
| #13 | empirical Windows evidence | production adapters |
| #3 | native-neutral contracts | runtime loop or public Win32 handles |
| #4 | normalization, plan, pure transitions | threads, ports, Win32 |
| #5 | one bounded clipboard acquisition | listener/history/write/restore/retry loop |
| #6 | target/focus/integrity evidence | focused content, refocus, universal caret claim |
| #7 | one bounded native input dispatch | session loop, retries, focus policy |
| #8 | hotkey registration and typed events | clipboard/input/session policy |
| #9 | live session worker/coordinator | Win32 message-pump presentation |
| #10 | composition root and responsive host | productized UI/installer |
| #11 | fixtures/evidence/compatibility report | hiding failures by changing behavior |
| #12 | independent findings/recommendation | feature implementation bundled into review |
| #14 | license/contribution decision | implementation/release authorization |

## 13. Risk alignment

- R1: #2, #3, #4 unless architecture scope expands.
- R2: #13, #5, #6, #7, #8, #9, #10; #7 additionally requires focused security review.
- R3: any newly introduced privilege, persistence, network/crash upload, IPC authorization, or broad input hook; these are outside current P1 without review.
- R4: #14 governance decision, #11 support evidence recommendation, #12 and #1 phase gates.

Use the repository risk taxonomy; add specialist review rather than inventing a risk meaning.

## 14. Agent and PR protocol

1. One task packet normally maps to one focused PR.
2. Every PR names the issue, phase/wave, risk, base/head commit, and source-of-truth documents read.
3. `Closes #N` is used only when all acceptance criteria are met.
4. Wave 3 PRs branch from the same merged #3 commit.
5. Stacked PRs require explicit dependency/base links.
6. Reports distinguish executed checks, interactive evidence, expected-by-contract behavior, and unverified paths.
7. Every native PR documents APIs, ownership, buffer/count/encoding invariants, thread requirements, cleanup, and safe-wrapper guarantees.
8. No agent merges, force-pushes shared work, tags, publishes, releases, elevates privilege, or broadens support claims without explicit maintainer authority.
9. #12 should be independent of the main #7/#9/#10 implementation path where practical.

## 15. Verification strategy

### Automated baseline

- formatting and lint;
- deterministic core/application tests;
- Windows workspace build/tests;
- non-Windows checks for platform-neutral crates;
- lockfile/dependency/license/security inspection appropriate to #14.

### Interactive Windows evidence

Headless CI is not proof of global hotkeys, foreground evidence, or native input. #13 and #11 execute on a recorded interactive desktop.

Required evidence includes:

- one worker under rapid/double trigger;
- message-loop responsiveness during long input;
- cancellation received and later batches stopped within a measured bound;
- target change/closure stops later batches;
- modifier settle/conflict without releasing physical keys;
- payload-limit and clipboard-contention behavior;
- partial/progress-unknown path with no retry;
- accurate known-integrity versus blocked-unknown results;
- terminal state returns to idle;
- shutdown/restart cleanup.

### Privacy sentinel

A distinctive generated fixture may appear only in controlled target/assertion buffers. Search ordinary logs, status snapshots, persistent test artifacts, and generated crash/debug output for text samples, prefixes/suffixes, hashes/fingerprints, focused contents, and window titles.

## 16. Representative target matrix

Minimum categories:

- controlled native Win32 edit target;
- Chromium-family text field;
- VS Code/editor field;
- Windows Terminal or equivalent terminal input;
- elevated/high-integrity target.

Record exact Windows/application versions, fixture class, evidence level, result, and limitation. One success is an observation, not universal support.

## 17. P1 exit gate

### Architecture/build

- [ ] #14 resolved before production implementation merge.
- [ ] only P1 packages exist and dependency direction is correct.
- [ ] platform-neutral crates have no Win32 leakage.
- [ ] required format/lint/check/test jobs pass on recorded toolchains.
- [ ] no unjustified async/UI/helper framework.

### Functional

- [ ] explicit global trigger reaches the application coordinator.
- [ ] destination is captured at trigger time.
- [ ] bounded current Unicode clipboard text is acquired.
- [ ] Unicode, line-break, Tab, control, and payload-limit behavior is evidenced.
- [ ] bounded native batches reach controlled and representative targets.
- [ ] hotkey conflict and shutdown lifecycle are surfaced.

### Safety

- [ ] one session/worker under races.
- [ ] message loop remains responsive.
- [ ] trigger modifiers cannot contaminate first dispatch.
- [ ] cancel/target/modifier checkpoints have recorded bounds.
- [ ] target change, disappearance, or strict evidence loss stops later batches.
- [ ] partial/unknown dispatch is not retried.
- [ ] session returns to idle on all contained terminal paths.
- [ ] UIPI is neither bypassed nor falsely diagnosed.
- [ ] focus-evidence limitations are documented.

### Privacy/evidence

- [ ] no clipboard/injected plaintext in ordinary logs or persistent artifacts.
- [ ] no P1 listener/history/write/restore path.
- [ ] CI, controlled E2E, and representative observations are separated.
- [ ] compatibility wording is evidence-backed.
- [ ] #12 returns `PASS` or `PASS WITH NON-BLOCKING FOLLOW-UPS`.
- [ ] remaining work is classified as P2 or linked blockers.

Completing the gate does not authorize a tag, release, or broad support promise.

## 18. Final P1 handoff

Report:

1. exact reviewed commit and PR merge order;
2. Windows/Rust/dependency environment;
3. crate graph and runtime ownership diagram;
4. automated commands/results;
5. interactive controlled and representative-target matrix;
6. batch, cancellation, focus, modifier, clipboard, and payload bounds;
7. privacy sentinel result;
8. unsafe/FFI inventory;
9. known limitations and open blockers;
10. #12 recommendation;
11. recommendation on entering P2.
