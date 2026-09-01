# Agent Design and Collaboration Specification

This document defines how AI agents should be used to develop ClipType safely and predictably. `AGENTS.md` is the mandatory repository contract; this document expands it into an operating model for single-agent and multi-agent work.

## 1. Design goals

Agentic development should improve throughput without weakening architectural discipline. The system is designed around four goals:

1. **Durable state lives in the repository.** Important decisions must be in docs, ADRs, issues, tests, or code—not only in an agent's hidden context.
2. **Tasks are bounded.** An agent owns a clearly scoped task packet and must not opportunistically redesign unrelated areas.
3. **Evidence beats confidence.** Agents report checks actually run and platform paths actually verified.
4. **Privilege and release authority remain narrow.** Agents do not merge, tag, release, or weaken security boundaries unless explicitly instructed.

## 2. Durable context model

Agents may have different context windows or start from fresh sessions. Therefore the repository is the durable shared memory.

Before work, reconstruct context from:

```text
README.md
  -> AGENTS.md
  -> docs/README.md
  -> relevant normative docs
  -> relevant accepted ADRs
  -> current task/issue/PR
  -> current implementation/tests
```

An agent MUST NOT rely on statements such as 'the previous agent decided X' unless X is present in a durable artifact or explicitly supplied by the maintainer.

## 3. Roles

Roles describe responsibilities, not permanently separate processes. One agent may execute several roles sequentially for a small task; for risky tasks, roles should be separated so review is independent.

### 3.1 Maintainer / Product Owner
Human authority for:
- product scope and roadmap changes;
- accepting/superseding major ADRs;
- permission/security trade-offs;
- merge/release decisions;
- license selection;
- support claims.

Agents may prepare proposals but do not invent maintainer approval.

### 3.2 Architecture Agent
Use for changes that alter boundaries or important semantics.

Responsibilities:
- identify affected invariants;
- compare alternatives;
- draft/update ADRs;
- ensure docs remain internally consistent;
- prevent premature abstractions for future phases.

Output should be design/ADR first. Implementation follows only after the task explicitly includes implementation.

### 3.3 Core Agent
Owns platform-independent policy:
- domain types;
- injection planner;
- state machine;
- cancellation policy;
- focus-safety policy;
- configuration semantics;
- error model;
- fake-port/unit tests.

Core Agent MUST NOT import platform APIs or encode compositor/OS branching into product policy when capability types can express it.

### 3.4 Platform Agent
Owns exactly one platform/backend scope per task where practical:
- Windows;
- macOS;
- X11;
- Wayland capability path;
- optional Linux helper.

Responsibilities:
- implement port contracts;
- localize FFI/unsafe code;
- document native API assumptions and permission behavior;
- add integration tests/evidence;
- report unsupported semantics accurately.

A Platform Agent does not change core policy merely to make one backend easy. If a port is insufficient, escalate through an architecture change.

### 3.5 UI/Shell Agent
Owns tray/settings/onboarding presentation.

Responsibilities:
- call application services rather than native injection APIs directly;
- present capability/permission/error states honestly;
- avoid storing clipboard content in UI models longer than required;
- preserve native platform conventions.

### 3.6 QA / Compatibility Agent
Responsibilities:
- derive test cases from docs and bug reports;
- exercise safety cases, not only happy paths;
- maintain compatibility evidence;
- distinguish environment limitations from product bugs;
- verify logs/artifacts do not contain fixture plaintext unexpectedly.

QA may reject a support claim even when a feature 'worked once'.

### 3.7 Security Reviewer
Required for changes involving:
- clipboard persistence/lifecycle;
- native input permissions;
- privileged helpers;
- IPC/authentication;
- telemetry/networking;
- crash dump handling;
- unsafe/FFI expansion.

Responsibilities:
- review threat-model deltas;
- verify least privilege;
- verify sensitive-data rules;
- identify abuse/race/focus-loss paths.

### 3.8 Release Agent
May prepare release notes, verify gates, assemble evidence, and inspect artifacts.

It MUST NOT publish, tag, promote, or merge a release without explicit maintainer instruction.

### 3.9 Reviewer Agent
An independent Reviewer Agent should focus on the diff and requirements rather than reimplementing the feature.

Review dimensions:
1. correctness;
2. architecture;
3. security/privacy;
4. platform semantics;
5. tests/evidence;
6. documentation drift;
7. scope control.

## 4. Task packet

Every substantial agent task should be expressible as this packet:

```yaml
goal: <one concrete outcome>
phase: P0|P1|P2|P3|P4|P5|P6
scope:
  in:
    - <required behavior/file area>
  out:
    - <explicit non-goal>
inputs:
  - <issue/PR/docs/ADR>
constraints:
  - <security/platform/compatibility constraints>
acceptance:
  - <observable result>
  - <tests/checks required>
allowed_writes:
  - <paths or subsystem>
release_actions: forbidden
```

For simple tasks this may exist only in the agent's plan. For multi-agent/risky work it should be placed in the issue/PR description so all agents share the same contract.

## 5. Agent state machine

```text
Assigned
   |
   v
Discovering
   |
   v
Scoped -----> Blocked/Needs Decision
   |
   v
Planned
   |
   v
Implementing
   |
   v
Verifying ----> Fixing ----+
   |                       |
   v                       |
SelfReview <---------------+
   |
   v
ReadyForReview
   |
   v
Handoff
```

An agent should move to `Blocked/Needs Decision` only for a genuine maintainer decision (e.g. incompatible product choices), not for ordinary implementation uncertainty that can be resolved by repository inspection or documented best judgment.

## 6. Work decomposition

Prefer vertical tasks with one user-visible or contract-visible result.

Good:
- 'Windows: implement current Unicode clipboard read behind ClipboardPort with tests.'
- 'Core: add cancellation transitions and deterministic tests.'
- 'Docs: ADR for uinput helper privilege boundary.'

Bad:
- 'Implement Windows/macOS/Linux support.'
- 'Refactor architecture and also add UI and packaging.'
- 'Clean up anything you notice.'

A task should normally fit one reviewable PR and one roadmap phase.

## 7. Multi-agent coordination

### 7.1 Ownership
Avoid concurrent edits to the same core files. Parallelize along stable boundaries:

```text
Architecture/specification
        |
        v
Core contract implementation
        |
   +----+----------------+
   |                     |
Windows adapter      QA fixtures/docs
```

Platform adapters may proceed in parallel only after the shared port contract is stable.

### 7.2 Handoff packet
When one agent hands work to another, provide:

```text
Task/result
Base/head commit or PR
Files changed
Contracts introduced/changed
Tests run + results
Unverified paths
Known risks
Next exact task
```

Do not hand off only 'it should work'.

### 7.3 Conflict handling
If parallel work reveals an incompatible interface:
- stop duplicating local workarounds;
- identify the shared contract conflict;
- resolve in core/ADR first;
- rebase/adapt platform work afterward.

## 8. Change-risk levels

### R0 — Documentation only
No behavior change. Still requires internal link/consistency review.

### R1 — Core deterministic behavior
Planner/config/state changes with no native API expansion. Unit tests mandatory.

### R2 — Native integration
Clipboard/input/focus/hotkey/permission behavior. Platform integration evidence mandatory.

### R3 — Sensitive boundary
Privileged helper, IPC auth, persistence, network/telemetry, crash memory, installer privilege. Architecture + security review mandatory.

### R4 — Release/support contract
Tagging, stable support claims, signing, distribution. Maintainer approval mandatory.

Agents should state the risk level in substantial PRs.

## 9. Agent write boundaries

Agents should minimize write scope.

- Architecture-only task: docs/ADRs; no production code unless asked.
- Core task: core/app + tests + related docs; avoid platform rewrites.
- Platform task: one adapter + tests + compatibility docs; shared contract changes only when necessary and explicit.
- QA task: tests/evidence/docs; avoid silently changing behavior to make tests pass.

Generated files, lockfiles, or formatting changes outside scope should not be committed without cause.

## 10. Native/unsafe checklist for agents

Any new unsafe/FFI block should answer:
- which native function/protocol is called;
- ownership/lifetime rules;
- pointer/buffer validity;
- thread/event-loop requirement;
- encoding assumptions;
- error retrieval/translation;
- cleanup requirements;
- what safe wrapper invariant callers may rely on.

The unsafe boundary should be the smallest practical region.

## 11. Verification reporting language

Use exact language:

- `Verified:` followed by commands/environments actually exercised.
- `Not verified:` for unavailable OS/session/application paths.
- `Expected by contract:` only when a behavior follows from docs/tests but has not been executed end-to-end.

Do not use 'all tests pass' when only a subset was run.

## 12. Review checklist for agent-produced PRs

- Does the diff solve only the task packet?
- Does it follow current roadmap phase?
- Does it preserve accepted ADRs?
- Are platform APIs behind adapters?
- Is clipboard plaintext absent from logs/persistence?
- Are cancellation/focus races covered?
- Are native errors surfaced accurately?
- Are unsafe invariants documented?
- Are tests meaningful and actually run?
- Are compatibility claims evidence-based?
- Are docs updated?
- Are merge/release actions left to explicit maintainer instruction?

## 13. Recommended agent prompt template

```text
Repository: yhan-sun/ClipType
Read AGENTS.md and the relevant docs/ADRs first.

Goal:
<one concrete outcome>

Roadmap phase:
<Px>

In scope:
- ...

Out of scope:
- ...

Acceptance criteria:
- ...

Constraints:
- preserve privacy/security invariants
- do not silently change architecture
- do not merge/tag/release
- report tests actually run and unverified platform paths

Deliver a focused PR/commit and a handoff containing changed files,
verification, risks, and intentionally deferred work.
```

## 14. Completion rule

Agent speed is never the success criterion. The successful agent leaves the repository in a state where a fresh agent or human can reconstruct why the change exists, what guarantees it provides, how it was verified, and what remains unknown.