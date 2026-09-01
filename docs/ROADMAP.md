# Roadmap and Phase Gates

Roadmap phases are **gates**, not dates. A later phase does not start merely because time passed; the current phase must meet exit criteria or have an explicit maintainer waiver documented in the relevant PR/ADR.

## P0 — Documentation Foundation (complete)

### Deliverables
- product scope/non-goals;
- architecture and technology choices;
- injection semantics;
- per-platform strategy;
- privacy/security model;
- compatibility vocabulary;
- test/release model;
- ADR foundation;
- agent/contribution rules.

### Exit criteria
- foundational docs committed;
- contradictions resolved sufficiently to enter implementation discovery;
- initial ADRs accepted;
- P1 scope and authority order defined.

P0 completed when the documentation foundation and initial ADR set were committed. Later evidence may still refine documents through normal ADR/document change rules.

## P1 — Windows Vertical Slice (current)

Goal: prove the entire product loop on one platform with minimal presentation and strong safety evidence.

Detailed sequencing, issue boundaries, concurrency model, and gate evidence are defined in [`phases/P1_WINDOWS_VERTICAL_SLICE.md`](phases/P1_WINDOWS_VERTICAL_SLICE.md).

### Entry sequence

P1 does not jump directly from docs to production adapters:

1. establish the minimal Rust workspace and CI baseline;
2. run a bounded interactive Windows native-mechanism/runtime spike;
3. freeze native-neutral P1 contracts from evidence;
4. implement pure policy and Windows adapters in parallel from the same contract base;
5. integrate the live session coordinator and Windows host;
6. execute interactive E2E/privacy/compatibility evidence;
7. complete an independent gate review.

The project license/contribution boundary must be decided before the first production implementation PR is merged.

### In scope
- Rust workspace/core boundaries actually needed for Windows;
- read current Unicode clipboard text at trigger time;
- global trigger and explicit cancellation path;
- capture foreground/keyboard-focus evidence without reading content;
- keyboard injection through native Windows API;
- Unicode/multiline/Tab/control-character behavior;
- one active session;
- responsive Win32 message loop plus bounded injection worker;
- modifier-release safety;
- cancellation and focus guard between bounded native batches;
- structured content-free diagnostics;
- minimal development status surface;
- deterministic core tests and Windows integration/E2E evidence.

### Out of scope
- macOS/Linux code stubs;
- clipboard listener/history;
- clipboard-paste backend and automatic threshold selection;
- app profiles;
- polished tray/settings UI;
- installer, auto-start, signing, or public release;
- exact logical-field/caret guarantees through UI Automation;
- low-level global keyboard hooks;
- automatic privilege elevation or UIPI bypass.

### Exit criteria
- repeatable end-to-end typing into controlled and representative Windows targets;
- message loop remains responsive while injection is active;
- one-session, modifier-release, cancellation, focus-change, partial-dispatch, and cleanup behavior is evidenced;
- no clipboard plaintext in ordinary logs or persistent artifacts;
- UIPI limitation is handled without bypass or false certainty;
- exact focus-evidence limitations are documented;
- compatibility claims come from recorded evidence;
- architecture matches docs or ADRs update it;
- independent P1 review returns pass or pass with non-blocking follow-ups.

## P2 — Windows Productization

### In scope
- clipboard-paste backend with safe restoration;
- auto planner and benchmark-derived threshold;
- polished tray/settings and permission/error UX;
- startup-at-login option;
- expanded target compatibility matrix;
- packaging/installer;
- CI hardening and release artifacts;
- crash/diagnostic privacy review.

### Exit criteria
A Windows beta can be used daily without hidden developer steps and has documented compatibility/known limitations.

## P3 — macOS Native Backend

### In scope
- NSPasteboard current text + transaction support;
- CGEvent keyboard/text injection;
- Accessibility permission onboarding/detection;
- focus guard;
- native tray/settings quality;
- signing/notarization path;
- macOS compatibility suite.

### Exit criteria
Feature semantics match Windows where macOS capabilities allow; deviations are in compatibility docs.

## P4 — Linux X11 Backend

### In scope
- X11 clipboard selection access;
- XTEST/native input injection;
- global trigger/focus evidence;
- Linux tray/settings integration as practical;
- distro/dependency packaging baseline;
- Unicode/keymap compatibility evidence.

### Exit criteria
A documented X11 beta works on representative desktops and target applications.

## P5 — Wayland Capability Backends

Goal: support real capability combinations without false portability claims.

### Research first
- `ext-data-control-v1` availability;
- portal clipboard session feasibility;
- compositor exposure of virtual keyboard protocol;
- uinput permission/helper design;
- GNOME/Mutter, KDE/KWin, wlroots-family behavior;
- global hotkey options/portals;
- focus evidence.

### Implementation
Add only capabilities with a clear permission/security model and reproducible tests. A Linux uinput helper requires a dedicated ADR before implementation.

### Exit criteria
Compatibility is published per compositor/environment and every support claim maps to evidence. Unsupported combinations fail clearly.

## P6 — 1.0 Hardening

### In scope
- stable configuration schema/migrations;
- accessibility/onboarding polish;
- signed/notarized release pipeline where applicable;
- dependency/security/license review;
- recovery/diagnostics UX;
- documentation completeness;
- stable compatibility promise;
- performance/footprint tuning;
- license finalized before broad distribution.

### 1.0 gate
No open release-blocking privacy, data-loss, focus-safety, or privilege issues.

## Post-1.0 candidates (not commitments)

Only after V1 is stable:
- target application profiles;
- richer hotkey customization;
- optional CLI control surface;
- plugin/transform system with a new security model;
- opt-in clipboard history only if product direction explicitly changes and a privacy ADR is accepted.

AI rewriting, cloud sync, and arbitrary macros remain separate product decisions, not assumed roadmap items.
