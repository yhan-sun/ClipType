# Roadmap and Phase Gates

Roadmap phases are **gates**, not dates. A later phase starts only after the current phase meets its exit criteria or an explicit scoped decision records what remains unknown.

## P0 — Documentation Foundation

### Deliverables

- product scope and non-goals;
- architecture and technology choices;
- injection semantics;
- per-platform strategy;
- privacy/security model;
- compatibility vocabulary;
- test/release model;
- ADR foundation;
- agent/contribution rules.

### Exit criteria

- foundational documents committed;
- contradictions resolved;
- initial ADRs accepted;
- P1 scope unambiguous.

P0 is complete. No production implementation belonged in P0.

## P1 — Windows Vertical Slice (current)

Goal: prove the complete product loop on Windows with the smallest safe host.

### In scope

- P1 Rust workspace and native-neutral boundaries;
- official/API and automated native research before contract freeze;
- current bounded Unicode clipboard text acquisition;
- explicit global trigger and cancellation command;
- target/focus/integrity evidence;
- bounded Unicode-oriented native keyboard injection;
- Unicode, multiline, Tab, and control-character policy;
- one active session;
- cancellation, modifier settle, and focus guard;
- content-free diagnostics;
- minimal development host;
- deterministic tests, Windows build/integration probes, and controlled interactive evidence;
- independent architecture/security/privacy review.

### Out of scope

- macOS/Linux stubs;
- clipboard history/listener;
- clipboard-paste and auto-mode optimization;
- app profiles;
- polished tray/settings/installer;
- signing or public release.

### Evidence rule

Official API contracts and automated Windows probes may enable conservative native-neutral contract design when uncertainty is represented explicitly. They do not replace unlocked interactive-desktop evidence for actual hotkey/input/focus compatibility.

### Exit criteria

- repeatable end-to-end typing into a controlled target and representative Windows categories;
- no clipboard plaintext in ordinary logs or persistent artifacts;
- UIPI limitation handled without bypass or false certainty;
- cancel/focus/modifier and one-session safety evidenced;
- all native scanning, retries, waits, and dispatch batches bounded;
- compatibility wording matches observed evidence;
- independent P1 review passes;
- architecture and docs match implementation.

Detailed sequencing is in [`phases/P1_WINDOWS_VERTICAL_SLICE.md`](phases/P1_WINDOWS_VERTICAL_SLICE.md).

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

A Windows beta can be used daily without hidden developer steps and has documented compatibility and limitations.

## P3 — macOS Native Backend

### In scope

- NSPasteboard current text and transaction support;
- CGEvent keyboard/text injection;
- Accessibility permission onboarding/detection;
- focus guard;
- native tray/settings quality;
- signing/notarization path;
- macOS compatibility suite.

### Exit criteria

Feature semantics match Windows where macOS capabilities allow; deviations are documented.

## P4 — Linux X11 Backend

### In scope

- X11 clipboard selection access;
- XTEST/native input injection;
- global trigger and focus evidence;
- practical Linux shell integration;
- packaging baseline;
- Unicode/keymap compatibility evidence.

### Exit criteria

A documented X11 beta works on representative desktops and targets.

## P5 — Wayland Capability Backends

Goal: support real capability combinations without false portability claims.

### Research first

- `ext-data-control-v1` availability;
- portal clipboard-session feasibility;
- compositor exposure of virtual-keyboard protocols;
- uinput permission/helper design;
- GNOME/Mutter, KDE/KWin, and wlroots-family behavior;
- global hotkey options;
- focus evidence.

### Implementation

Add only capabilities with a clear permission/security model and reproducible tests. A Linux uinput helper requires a dedicated ADR.

### Exit criteria

Compatibility is published per environment and every support claim maps to evidence. Unsupported combinations fail clearly.

## P6 — 1.0 Hardening

### In scope

- stable configuration schema and migrations;
- onboarding/accessibility polish;
- signed/notarized release pipeline where applicable;
- dependency/security/license review;
- recovery/diagnostics UX;
- documentation completeness;
- stable compatibility promise;
- performance/footprint tuning.

### 1.0 gate

No open release-blocking privacy, data-loss, focus-safety, or privilege issue.

## Post-1.0 candidates

Only after V1 is stable:

- target application profiles;
- richer hotkey customization;
- optional CLI control surface;
- plugin/transform system with a new security model;
- opt-in clipboard history only if product direction changes and a privacy ADR is accepted.

AI rewriting, cloud sync, and arbitrary macros remain separate product decisions, not assumed roadmap items.
