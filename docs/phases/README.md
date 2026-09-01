# Phase execution plans

Phase-specific implementation sequencing, ownership, merge order, and gate evidence live here. Normative product, architecture, security, and compatibility behavior remains in the top-level documents under `docs/`.

## Current

- [`P1_WINDOWS_VERTICAL_SLICE.md`](P1_WINDOWS_VERTICAL_SLICE.md) — optimized Windows vertical-slice execution plan, including native discovery before contract freeze, crate/runtime boundaries, work waves, Agent Task Packet ownership, and P1 exit evidence.

## Rules

- A phase plan may refine issue sequencing and PR ownership.
- It must not silently change accepted architecture, security posture, public behavior, or support claims.
- Such changes require updates to the relevant normative documents and an ADR where repository policy requires one.
- A completed checklist does not itself authorize merge, tag, publish, or release.
