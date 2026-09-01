# Roadmap and Phase Gates

Roadmap phases are **gates**, not dates. A later phase does not start merely because time passed; the current phase must meet exit criteria or have an explicit maintainer waiver documented in the relevant PR/ADR.

## P0 — Documentation Foundation (current)

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
- all foundational docs committed;
- contradictions resolved;
- initial ADRs accepted;
- implementation scope for P1 is unambiguous.

**No production implementation belongs in P0.**

## P1 — Windows Vertical Slice

Goal: prove the entire product loop on one platform with minimal UI.

### In scope
- Rust workspace/core boundaries actually needed for Windows;
- read current Unicode clipboard text;
- global trigger;
- capture foreground target identity;
- keyboard injection through native Windows API;
- Unicode/multiline behavior;
- one active session;
- cancellation;
- focus guard;
- structured content-free diagnostics;
- minimal tray/status surface or development trigger needed to exercise the flow;
- deterministic core tests and Windows integration tests.

### Out of scope
- macOS/Linux code stubs;
- clipboard-paste auto optimization unless required to validate architecture;
- app profiles;
- polished installer.

### Exit criteria
- repeatable end-to-end typing into representative Windows targets;
- no clipboard plaintext in logs;
- UIPI limitation handled explicitly;
- cancel/focus-change tests pass;
- architecture matches docs or ADRs update it.

## P2 — Windows Productization

### In scope
- clipboard-paste backend with safe restoration;
- auto planner and benchmark-derived threshold;
- polished tray/settings and permission/error UX;
- startup-at-login option;
- target compatibility matrix;
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