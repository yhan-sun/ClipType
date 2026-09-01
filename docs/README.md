# ClipType Documentation

This directory is the design source of truth for ClipType. The repository intentionally freezes product and architecture decisions before implementation.

## Reading order

1. [`PRODUCT.md`](PRODUCT.md) — what ClipType is, users, goals, non-goals, UX contract.
2. [`ARCHITECTURE.md`](ARCHITECTURE.md) — system boundaries, components, event flow, process model.
3. [`TECHNOLOGY.md`](TECHNOLOGY.md) — language, UI, dependency, packaging, and native-integration choices.
4. [`INJECTION_ENGINE.md`](INJECTION_ENGINE.md) — injection modes, planner, state machine, cancellation, restoration.
5. [`PLATFORMS.md`](PLATFORMS.md) — Windows/macOS/X11/Wayland backend strategy.
6. [`COMPATIBILITY.md`](COMPATIBILITY.md) — capability matrix and support vocabulary.
7. [`SECURITY_PRIVACY.md`](SECURITY_PRIVACY.md) — threat model and non-negotiable data rules.
8. [`CONFIGURATION.md`](CONFIGURATION.md) — planned configuration model and stable semantics.
9. [`TESTING.md`](TESTING.md) — test pyramid, platform matrix, release gates.
10. [`ROADMAP.md`](ROADMAP.md) — staged implementation with entry/exit criteria.
11. [`DEVELOPMENT_WORKFLOW.md`](DEVELOPMENT_WORKFLOW.md) — branch/PR/design/review/merge workflow.
12. [`AGENT_DESIGN.md`](AGENT_DESIGN.md) — AI-agent roles, task packets, state machine, risk levels, handoffs, and multi-agent rules.
13. [`RELEASE.md`](RELEASE.md) — versioning, artifacts, signing, promotion gates.
14. [`GLOSSARY.md`](GLOSSARY.md) — shared terminology.
15. [`REFERENCES.md`](REFERENCES.md) — official APIs and projects used as design references.
16. [`adr/`](adr/) — immutable architecture decision records.

Repository-level [`../AGENTS.md`](../AGENTS.md) is the mandatory execution contract for AI agents. `AGENT_DESIGN.md` explains the detailed operating model behind it.

## Normative language

`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` are used in the RFC sense. A `MUST` violation is not a minor implementation detail; it requires a design change or ADR.

## Source-of-truth boundaries

- Product behavior: `PRODUCT.md` and `INJECTION_ENGINE.md`.
- Component ownership/boundaries: `ARCHITECTURE.md`.
- OS-specific mechanisms: `PLATFORMS.md`.
- Support claims: `COMPATIBILITY.md`.
- Privacy/security: `SECURITY_PRIVACY.md` overrides convenience/performance goals.
- Schedule/scope: `ROADMAP.md`.
- Agent execution rules: repository `AGENTS.md`; detailed role/workflow design: `AGENT_DESIGN.md`.
- Development lifecycle: `DEVELOPMENT_WORKFLOW.md`.
- Architecture decisions: accepted ADRs override older descriptive text; dependent docs should then be updated.

## Documentation change policy

Public behavior, permissions, compatibility, architecture, persistence, injection semantics, or release gates MUST be documented in the same PR that changes them.

Accepted ADRs are historical records. Do not edit their decision into a different decision; add a superseding ADR.