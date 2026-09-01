# Development Workflow

This document defines the normal path from idea to merged implementation once P0 documentation is complete.

## 1. Work starts from a concrete unit

Every change should begin with one of:
- roadmap deliverable;
- issue/bug;
- accepted ADR follow-up;
- explicitly requested maintenance task.

Do not create work solely because a future abstraction might be useful.

## 2. Scope against the roadmap

Identify the active phase in `ROADMAP.md`. Later-platform scaffolding does not belong in an earlier phase unless it is required to prove a shared contract.

Example: during P1, build the Windows vertical slice and reusable core contracts. Do not create empty macOS/X11/Wayland crates just to make the tree symmetrical.

## 3. Design gate

Ask whether the task changes any accepted decision:
- architecture boundary;
- process/privilege model;
- injection mode semantics;
- privacy/persistence;
- platform mechanism;
- UI architecture;
- compatibility promise.

If yes, write a proposed ADR before or with implementation. If no, reference the existing ADR in the PR.

## 4. Branching

Recommended branch names:

```text
feat/<short-scope>
fix/<short-scope>
docs/<short-scope>
refactor/<short-scope>
test/<short-scope>
```

Keep branch lifetime short and rebase/update against `main` before final review when required by repository policy.

## 5. Implementation order

For a vertical feature:

1. define/adjust core semantic contract;
2. add deterministic core tests;
3. implement native adapter behavior;
4. add platform integration tests;
5. wire application service;
6. add/update minimal UI/status surface if required;
7. update compatibility/docs;
8. run phase gate;
9. self-review.

This order prevents UI/native glue from defining product policy accidentally.

## 6. Commit discipline

Prefer a small number of meaningful commits over both extremes: one enormous opaque commit or dozens of mechanical commits.

A useful sequence for larger work can be:
- `docs:` ADR/contract change;
- `feat:` core behavior and tests;
- `feat:` platform adapter;
- `test:` integration/compatibility evidence.

Do not rewrite accepted ADR history merely to keep a branch tidy.

## 7. Pull request gate

Every substantial PR states:
- goal and scope;
- roadmap phase;
- risk level from `AGENT_DESIGN.md`;
- architecture/ADR impact;
- security/privacy impact;
- platform impact;
- checks actually run;
- paths not verified;
- known risks;
- docs changed;
- rollback notes where relevant.

Use the repository PR template.

## 8. Review depth by risk

- R0 docs: consistency/link/content review.
- R1 core: correctness + unit test review.
- R2 native: R1 plus platform/API/permission/integration review.
- R3 sensitive boundary: architecture + security review before merge.
- R4 release: full release gate and explicit maintainer action.

## 9. Compatibility updates

Do not update `COMPATIBILITY.md` from intention. Update it from evidence.

A compatibility claim should identify environment, backend, target category, and tested semantics. Wayland evidence is compositor/protocol-specific.

## 10. Bug workflow

For a bug:
1. reproduce with non-sensitive fixture data;
2. classify core vs adapter vs environment limitation;
3. add regression test when possible;
4. fix smallest responsible layer;
5. verify no new fallback/safety regression;
6. update compatibility/known limitations when behavior cannot be universally fixed.

Never capture a user's real clipboard contents in a bug artifact.

## 11. Dependency workflow

Before adding a dependency:
- prove current-phase need;
- review license and maintenance;
- inspect transitive/unsafe footprint;
- compare official/native API option;
- document rationale in PR for native/security-sensitive dependencies.

Dependency upgrades that change native semantics are behavior changes, not mechanical housekeeping.

## 12. Merge policy

A PR is mergeable when:
- requested behavior/acceptance criteria are met;
- required reviews are complete;
- applicable tests/gates pass;
- docs and compatibility are current;
- no unresolved release-blocking privacy/security issue remains.

Agents never infer permission to merge from green CI alone.

## 13. Release flow

Release work follows `RELEASE.md`:

```text
Candidate commit
 -> full required tests
 -> compatibility evidence
 -> packaging smoke tests
 -> security/privacy gate
 -> signing/notarization where applicable
 -> release notes
 -> explicit maintainer promotion
```

## 14. Post-merge discipline

If implementation disproves a documented assumption, fix the documentation/ADR promptly rather than allowing code and design source-of-truth to diverge.