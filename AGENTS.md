# ClipType Agent Engineering Contract

This file defines mandatory repository rules for AI coding agents and human contributors using agentic workflows.

## 1. Authority order

When instructions conflict, use this order:

1. Explicit maintainer/user instruction for the current task.
2. This `AGENTS.md`.
3. Accepted ADRs in `docs/adr/`.
4. Normative documents in `docs/`.
5. Existing implementation and tests.
6. Personal preference or inferred convention.

Never silently violate a higher-level source. If an implementation reveals that an ADR is wrong, update/supersede the ADR in the same change.

## 2. Mandatory reading before implementation

Before changing production code, read at minimum:

- `README.md`
- `docs/README.md`
- `docs/PRODUCT.md`
- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/SECURITY_PRIVACY.md`
- every accepted ADR relevant to the requested area

Platform work must also read `docs/PLATFORMS.md` and `docs/COMPATIBILITY.md`. Injection work must read `docs/INJECTION_ENGINE.md`. Release work must read `docs/TESTING.md` and `docs/RELEASE.md`.

## 3. Agent execution lifecycle

Every non-trivial task follows this lifecycle:

`Discover -> Scope -> Plan -> Implement -> Verify -> Self-review -> Handoff`

### Discover
- Inspect the current branch, relevant files, open task/PR context, and applicable docs.
- Do not assume a file, API, behavior, dependency, or CI job exists.

### Scope
- State the concrete behavior being changed and the behaviors explicitly not being changed.
- Stay inside the current roadmap phase unless the maintainer explicitly expands scope.

### Plan
- Identify affected modules, platform adapters, invariants, tests, docs, and failure modes.
- Prefer the smallest architecture-consistent change over broad refactoring.

### Implement
- Keep platform APIs inside platform adapters.
- Keep policy/decision logic in the core, not in UI or platform glue.
- Avoid speculative abstractions for unimplemented future phases.

### Verify
- Run the narrowest relevant tests first, then the required phase gate.
- Never claim a test passed unless it was actually executed.
- If a platform cannot be tested locally, state that limitation and provide the exact remaining verification.

### Self-review
- Review the diff for scope creep, logging of sensitive text, hidden persistence, unsafe/FFI invariants, error handling, cancellation behavior, and documentation drift.

### Handoff
Report:
- changed files/areas;
- behavioral result;
- tests/checks actually run;
- untested platform paths;
- risks/known limitations;
- follow-up items that are intentionally out of scope.

## 4. Architectural rules

- The project uses a Rust core with ports-and-adapters boundaries.
- Core code must not import OS-specific APIs.
- OS-specific behavior belongs in explicit Windows, macOS, X11, or Wayland adapters.
- UI is a shell over application services; business policy does not live in UI callbacks.
- Wayland is capability-driven. Never infer support from `XDG_SESSION_TYPE=wayland` alone.
- A privileged helper may exist only when a platform capability requires it; it must have a minimal protocol and no clipboard-history role.
- Unsafe Rust/FFI must be localized, justified with safety invariants, and wrapped by safe interfaces.

## 5. Privacy and security invariants

These are release-blocking:

- Never log clipboard plaintext, typed plaintext, credentials, hashes intended to fingerprint clipboard contents across sessions, or captured keystrokes.
- Clipboard content is ephemeral and memory-only by default.
- No telemetry or network traffic may contain clipboard content.
- Do not add clipboard history, cloud sync, remote execution, or arbitrary scripting without a new accepted ADR and explicit maintainer scope.
- Injection must be explicitly user-triggered by default.
- Cancellation and focus-change protections must fail safe.
- Do not bypass OS security boundaries such as Windows UIPI or macOS Accessibility consent.

## 6. Dependency policy

A new dependency requires all of:

- a concrete current-phase need;
- active maintenance or a justified exception;
- compatible license;
- bounded platform impact;
- explanation in the PR when it touches security-sensitive/FFI/platform code.

Prefer official OS APIs and small focused crates. Do not add a framework to solve a one-function problem.

## 7. Testing rules

- Core policy requires deterministic unit tests.
- Platform adapters require integration tests where CI/platform automation can exercise them.
- Every bug fix should add a regression test when technically possible.
- Injection tests must include Unicode, multiline text, cancellation, focus change, and modifier-key contamination cases appropriate to the backend.
- Clipboard-paste tests must verify restoration and self-generated clipboard-event suppression.

See `docs/TESTING.md` for the full matrix.

## 8. Documentation and ADR rules

Update docs in the same change when public behavior, architecture, permissions, compatibility, configuration, or release requirements change.

Create an ADR when changing:

- language/runtime strategy;
- process boundaries;
- platform integration mechanism;
- injection semantics/backends;
- persistence/privacy posture;
- UI architecture;
- privileged-helper design;
- compatibility promise.

Never rewrite the history of an accepted ADR to make a new decision appear old. Add a superseding ADR.

## 9. Git/PR discipline

- Keep commits scoped and reviewable.
- Do not mix opportunistic refactors with feature work.
- Do not merge, tag, publish, or create a release unless explicitly instructed.
- Do not force-push shared branches unless explicitly instructed.
- PR descriptions must include scope, architecture impact, security/privacy impact, verification, platform matrix, and rollback notes.

## 10. Prohibited shortcuts

Agents must not:

- implement platform behavior with shelling out to arbitrary external utilities as the default architecture unless an ADR explicitly permits it;
- mark Wayland globally supported based on one compositor;
- fake native input by writing directly into application internals;
- disable permission checks to make tests pass;
- swallow injection errors and report success;
- persist clipboard content for debugging;
- copy source from reference projects without license review and attribution handling;
- claim completion when required gates remain unverified.

## 11. Definition of done

A task is done only when its requested behavior is implemented, architecture remains consistent, applicable tests pass, docs are current, sensitive-data invariants hold, and the handoff accurately states remaining limitations.