# Development Workflow

This document defines the normal path from a scoped task to reviewed implementation. Current phase sequencing may refine this workflow; for P1 see [`phases/P1_WINDOWS_VERTICAL_SLICE.md`](phases/P1_WINDOWS_VERTICAL_SLICE.md).

## 1. Work starts from a concrete unit

Every change begins with one of:

- roadmap/phase deliverable;
- issue or bug;
- accepted ADR follow-up;
- explicitly requested maintenance task;
- bounded research spike required to validate a contract.

Do not create work solely because a future abstraction might be useful.

## 2. Scope against the active phase

Identify the current phase in `ROADMAP.md` and its execution plan. Later-platform scaffolding does not belong in an earlier phase unless it is required to prove a current shared contract and is explicitly approved.

During P1, build only the Windows vertical slice and native-neutral contracts it proves. Do not create empty macOS/X11/Wayland crates for symmetry.

## 3. Evidence-before-freeze rule

A semantic contract should be designed before production adapters, but material native uncertainties should be tested before that contract is declared stable.

Use a bounded spike when a contract depends on uncertain facts such as:

- native thread/message-loop ownership;
- encoding or input-event semantics;
- permission/security-boundary diagnostics;
- focus/target evidence;
- ownership/lifetime or cleanup rules;
- cancellation/checkpoint feasibility.

The sequence is:

```text
documented requirement
  -> bounded native research/spike
  -> recorded observations and limitations
  -> native-neutral contract freeze
  -> production policy/adapters
```

Spike code is disposable by default and must not be relabelled production code without normal review/testing.

## 4. Architecture/decision gate

Ask whether the task changes:

- component or dependency boundary;
- process/thread/privilege model;
- injection semantics;
- privacy/persistence/networking;
- platform mechanism;
- UI architecture;
- compatibility/support promise;
- licensing/contribution terms.

If yes, update the appropriate normative document and add/supersede an ADR when repository policy requires it. Research evidence alone does not silently override accepted decisions.

## 5. Issue and task packet

Every substantial issue specifies:

- one concrete outcome;
- phase and wave;
- dependencies and common base commit where applicable;
- risk level;
- in/out of scope;
- allowed-write boundaries;
- required evidence;
- acceptance criteria;
- handoff format;
- prohibited release actions.

An agent stops and escalates shared-contract pressure rather than creating a local adapter-specific fork.

## 6. Branching and common-base policy

Recommended branch names:

```text
feat/p1-03-core-policy
feat/p1-04-windows-clipboard
research/p1-s01-windows-native
fix/<short-scope>
docs/<short-scope>
test/<short-scope>
```

Keep branch lifetime short. Parallel work begins only after its shared contract/base commit is reviewed and merged. For a parallel wave:

1. record the exact common base commit in each task handoff/PR;
2. avoid concurrent edits to the same shared contract files;
3. route contract changes through one focused PR;
4. rebase/adapt dependent branches after the shared change;
5. do not resolve conflicts by duplicating types or adding hidden compatibility shims.

Stacked PRs are used only when dependency order is explicit and the base PR is linked.

## 7. Implementation order

A typical native vertical slice follows:

1. freeze product/safety requirement;
2. run a bounded native spike for material uncertainty;
3. freeze native-neutral contracts;
4. add pure policy and deterministic tests;
5. implement native adapters behind the contracts;
6. implement the live application coordinator;
7. compose the platform host/message loop;
8. run controlled interactive E2E and privacy checks;
9. update compatibility from evidence;
10. complete independent gate review.

Pure policy, live runtime orchestration, native mechanisms, and presentation remain separate responsibilities.

## 8. Commit discipline

Prefer meaningful review units over one opaque commit or many mechanical commits. A larger task may use:

- `docs:` contract/research/ADR;
- `feat:` pure behavior plus tests;
- `feat:` native adapter;
- `test:` integration/evidence;
- `docs:` compatibility and handoff.

Do not rewrite accepted ADR history to make a new decision appear old. Generated or formatting changes outside scope require explanation.

## 9. Pull request gate

Every substantial PR records:

- issue/task ID, phase/wave, risk;
- base/head commit;
- source-of-truth docs/ADRs/handoffs read;
- goal, in/out of scope, allowed writes;
- architecture and contract impact;
- runtime/thread/ownership impact;
- security/privacy impact;
- unsafe/FFI invariants when applicable;
- dependencies and license impact;
- commands/checks actually run;
- CI, controlled interactive, and manual evidence separately;
- unverified paths;
- rollback, risks, and next dependent task.

Use the repository PR template. `Closes #N` is used only when every acceptance criterion is met.

## 10. Review depth by risk

Use the taxonomy in `AGENT_DESIGN.md`:

- R0 documentation: consistency, links, source-of-truth review;
- R1 deterministic/core: correctness and unit tests;
- R2 native/integration: R1 plus platform/API/thread/permission/integration evidence;
- R3 sensitive boundary: architecture and security review before merge;
- R4 release/support/governance: full gate and explicit maintainer authority.

A safety-critical R2 task may require a specialist security reviewer without being mislabelled R4.

## 11. Verification language

Reports use:

- `Verified:` commands and environments actually exercised;
- `Not verified:` unavailable OS/session/application/race paths;
- `Expected by contract:` behavior established by pure tests/docs but not executed end to end.

Do not call headless CI proof of native interactive input. Do not say `all tests pass` when only a subset ran.

## 12. Compatibility updates

`COMPATIBILITY.md` changes from evidence, not intention.

Each observation records environment, application/version/category, backend, evidence strength, tested semantics, and limitations. A successful observation is not automatically a release support claim.

Focus claims identify whether evidence is top-level, native-control, render-host-limited, degraded, or unavailable. Wayland evidence remains compositor/protocol-specific.

## 13. Security/privacy review during development

Every task self-reviews for:

- clipboard/injected plaintext in logs/errors/snapshots;
- persistent samples or fingerprints;
- unbounded native lengths, payloads, waits, retries, or event batches;
- incorrect cancellation/target/modifier behavior;
- partial/unknown input retry;
- hidden privilege or security-boundary bypass;
- unrelated key/focused-content capture;
- unsafe ownership/lifetime/cleanup mistakes.

High-risk paths use distinctive synthetic privacy sentinels, never real user secrets.

## 14. Dependency and license workflow

Before adding a dependency:

- prove current-phase need;
- verify project licensing/contribution policy is resolved for implementation;
- review license compatibility and attribution;
- inspect maintenance, transitive size, unsafe/FFI surface, and platform scope;
- compare standard-library/official native API options;
- document rationale in the PR.

A native-semantics-changing upgrade is a behavior change, not mechanical housekeeping. Source from reference projects is not copied without compatible licensing and attribution review.

## 15. Bug workflow

1. reproduce with generated non-sensitive fixtures;
2. classify pure policy, coordinator, adapter, host, or environment limitation;
3. add a regression test/test seam where possible;
4. fix the smallest responsible layer;
5. rerun affected controlled evidence;
6. update compatibility/limitations when behavior cannot be universally fixed.

Do not capture a user's real clipboard content in a bug artifact.

## 16. Merge policy

A PR is mergeable only when:

- task acceptance criteria are met;
- dependencies/common-base requirements are satisfied;
- required reviews and evidence are complete;
- docs/compatibility are current;
- no blocking privacy/security/governance issue remains;
- the maintainer explicitly authorizes merge where repository policy requires it.

Green CI alone does not grant merge authority. Agents do not merge, tag, publish, release, or broaden support claims unless explicitly instructed.

## 17. Phase and release gates

A phase gate checks the integrated commit, not merely whether child issues were closed. It reconciles implementation, test evidence, unsafe inventory, compatibility wording, and open blockers.

Release work follows `RELEASE.md`:

```text
candidate commit
  -> required tests
  -> compatibility evidence
  -> packaging/security/privacy gates
  -> signing/notarization where applicable
  -> release notes
  -> explicit maintainer promotion
```

## 18. Post-merge discipline

If implementation disproves an assumption:

- record the evidence;
- correct contracts/docs and add/supersede an ADR when required;
- rerun affected tests/evidence;
- do not leave code and durable design context inconsistent.
