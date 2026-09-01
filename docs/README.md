# ClipType Documentation

This directory is the design source of truth for ClipType. P0 established the architecture and product foundation; P1 is now the current implementation phase.

## Reading order

1. [`PRODUCT.md`](PRODUCT.md) — product definition, users, goals, non-goals, UX contract.
2. [`ARCHITECTURE.md`](ARCHITECTURE.md) — system boundaries, components, event flow, process model.
3. [`TECHNOLOGY.md`](TECHNOLOGY.md) — language, UI, dependency, packaging, and native-integration choices.
4. [`INJECTION_ENGINE.md`](INJECTION_ENGINE.md) — injection modes, planner, state machine, cancellation, restoration.
5. [`PLATFORMS.md`](PLATFORMS.md) — Windows/macOS/X11/Wayland backend strategy.
6. [`COMPATIBILITY.md`](COMPATIBILITY.md) — capability matrix and support vocabulary.
7. [`SECURITY_PRIVACY.md`](SECURITY_PRIVACY.md) — threat model and non-negotiable data rules.
8. [`DEPENDENCY_POLICY.md`](DEPENDENCY_POLICY.md) — licensing, dependency review, copied-code, and attribution rules.
9. [`CONFIGURATION.md`](CONFIGURATION.md) — planned configuration model and stable semantics.
10. [`TESTING.md`](TESTING.md) — test pyramid, platform matrix, release gates.
11. [`ROADMAP.md`](ROADMAP.md) — staged implementation with entry/exit criteria.
12. [`phases/P1_WINDOWS_VERTICAL_SLICE.md`](phases/P1_WINDOWS_VERTICAL_SLICE.md) — current sequencing, ownership, and evidence gates.
13. [`research/P1_WINDOWS_NATIVE_SPIKE.md`](research/P1_WINDOWS_NATIVE_SPIKE.md) — official and automated Windows contract evidence plus interactive gaps.
14. [`DEVELOPMENT_WORKFLOW.md`](DEVELOPMENT_WORKFLOW.md) — branch/PR/design/review/merge workflow.
15. [`AGENT_DESIGN.md`](AGENT_DESIGN.md) — AI-agent roles, task packets, risk levels, handoffs, and multi-agent rules.
16. [`RELEASE.md`](RELEASE.md) — versioning, artifacts, signing, promotion gates.
17. [`GLOSSARY.md`](GLOSSARY.md) — shared terminology.
18. [`REFERENCES.md`](REFERENCES.md) — official APIs and reference projects.
19. [`adr/`](adr/) — immutable architecture decision records.
20. [`research/`](research/) — bounded research evidence.

Repository-level [`../AGENTS.md`](../AGENTS.md) is the mandatory execution contract.

## Normative language

`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` are used in the RFC sense. A `MUST` violation requires a design change or ADR rather than silent implementation drift.

## Source-of-truth boundaries

- Product behavior: `PRODUCT.md` and `INJECTION_ENGINE.md`.
- Component ownership/boundaries: `ARCHITECTURE.md`.
- OS mechanisms: `PLATFORMS.md`.
- Support claims: `COMPATIBILITY.md`.
- Privacy/security: `SECURITY_PRIVACY.md` overrides convenience/performance.
- Licensing and third-party provenance: `DEPENDENCY_POLICY.md`.
- Phase scope: `ROADMAP.md`.
- Current P1 sequencing and gates: `phases/P1_WINDOWS_VERTICAL_SLICE.md`.
- Agent execution: repository `AGENTS.md`; detailed model: `AGENT_DESIGN.md`.
- Development lifecycle: `DEVELOPMENT_WORKFLOW.md`.
- Accepted ADRs override older descriptive text; dependent docs must then be updated.
- Research reports supply evidence but do not create support claims or bypass normative decisions.

## Documentation change policy

Public behavior, permissions, compatibility, architecture, persistence, injection semantics, dependency/contribution licensing, or release gates must be documented in the same PR that changes them.

Accepted ADRs are historical records. Add a superseding ADR rather than rewriting an accepted decision.

Phase execution documents may refine sequencing and ownership without an ADR. If they change architecture, security posture, public behavior, or support promises, update the normative document and add an ADR when required.
