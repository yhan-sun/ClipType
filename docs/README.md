# ClipType Documentation

This directory is the design source of truth for ClipType. P0 established the architecture and product foundation; P1 is now the current implementation phase.

## Reading order

1. [`PRODUCT.md`](PRODUCT.md) — what ClipType is, users, goals, non-goals, UX contract.
2. [`ARCHITECTURE.md`](ARCHITECTURE.md) — system boundaries, components, event flow, process model.
3. [`TECHNOLOGY.md`](TECHNOLOGY.md) — language, UI, dependency, packaging, and native-integration choices.
4. [`INJECTION_ENGINE.md`](INJECTION_ENGINE.md) — injection modes, planner, state machine, cancellation, restoration.
5. [`PLATFORMS.md`](PLATFORMS.md) — Windows/macOS/X11/Wayland backend strategy.
6. [`COMPATIBILITY.md`](COMPATIBILITY.md) — capability matrix and support vocabulary.
7. [`SECURITY_PRIVACY.md`](SECURITY_PRIVACY.md) — threat model and non-negotiable data rules.
8. [`DEPENDENCY_POLICY.md`](DEPENDENCY_POLICY.md) — project/contribution licensing, dependency review, copied-code, and attribution rules.
9. [`CONFIGURATION.md`](CONFIGURATION.md) — planned configuration model and stable semantics.
10. [`TESTING.md`](TESTING.md) — test pyramid, platform matrix, release gates.
11. [`ROADMAP.md`](ROADMAP.md) — staged implementation with entry/exit criteria.
12. [`phases/P1_WINDOWS_VERTICAL_SLICE.md`](phases/P1_WINDOWS_VERTICAL_SLICE.md) — current P1 sequencing, issue boundaries, runtime model, PR order, and phase evidence.
13. [`DEVELOPMENT_WORKFLOW.md`](DEVELOPMENT_WORKFLOW.md) — branch/PR/design/review/merge workflow.
14. [`AGENT_DESIGN.md`](AGENT_DESIGN.md) — AI-agent roles, task packets, state machine, risk levels, handoffs, and multi-agent rules.
15. [`RELEASE.md`](RELEASE.md) — versioning, artifacts, signing, promotion gates.
16. [`GLOSSARY.md`](GLOSSARY.md) — shared terminology.
17. [`REFERENCES.md`](REFERENCES.md) — official APIs and projects used as design references.
18. [`adr/`](adr/) — immutable architecture decision records.
19. [`research/`](research/) — bounded research/spike evidence; research does not override accepted architecture without an ADR/document update.

Repository-level [`../AGENTS.md`](../AGENTS.md) is the mandatory execution contract for AI agents. `AGENT_DESIGN.md` explains the detailed operating model behind it.

## Normative language

`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` are used in the RFC sense. A `MUST` violation is not a minor implementation detail; it requires a design change or ADR.

## Source-of-truth boundaries

- Product behavior: `PRODUCT.md` and `INJECTION_ENGINE.md`.
- Component ownership/boundaries: `ARCHITECTURE.md`.
- OS-specific mechanisms: `PLATFORMS.md`.
- Support claims: `COMPATIBILITY.md`.
- Privacy/security: `SECURITY_PRIVACY.md` overrides convenience/performance goals.
- Dependency/contribution licensing and third-party provenance: `DEPENDENCY_POLICY.md`.
- Phase scope: `ROADMAP.md`.
- Current P1 sequencing/integration gates: `phases/P1_WINDOWS_VERTICAL_SLICE.md`.
- Agent execution rules: repository `AGENTS.md`; detailed role/workflow design: `AGENT_DESIGN.md`.
- Development lifecycle: `DEVELOPMENT_WORKFLOW.md`.
- Architecture decisions: accepted ADRs override older descriptive text; dependent docs should then be updated.
- Research findings: evidence for decisions, not permission to bypass normative documents.

## Documentation change policy

Public behavior, permissions, compatibility, architecture, persistence, injection semantics, dependency/contribution licensing, or release gates MUST be documented in the same PR that changes them.

Accepted ADRs are historical records. Do not edit their decision into a different decision; add a superseding ADR.

Phase execution documents may refine sequencing and ownership without an ADR. If they change architecture, security posture, public behavior, or support promises, update the normative document and add an ADR when required.
