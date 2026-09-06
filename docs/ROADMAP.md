# Roadmap

## Current release

`v0.2.0-beta.8` is the current public prerelease.

- Windows x86_64 is the primary beta channel.
- macOS Apple Silicon arm64 is an additive testing preview.
- Windows assets are SHA-256 manifested, Sigstore keyless signed, and covered by GitHub artifact attestations.
- macOS preview assets are ad-hoc signed, checksum verified, and re-downloaded after publication; they are not Developer ID signed, notarized, stapled, Intel/Rosetta, or Universal 2.
- Published tags/assets are immutable; remediation uses a newer beta.

## Delivered

### P0 — foundations

- Rust workspace and ports/adapters boundaries;
- platform-independent normalization, limits, state, and content-free diagnostics;
- pinned toolchain, formatting, checks, tests, Clippy, dependency and documentation policy.

### P1 — Windows vertical slice

- bounded `CF_UNICODETEXT` acquisition;
- Unicode-oriented `SendInput` adapter;
- foreground destination and integrity evidence;
- global trigger and independent cancel hotkeys;
- single-session coordinator, cancellation, target revalidation, and conservative native result handling;
- controlled Windows native edit-target E2E;
- fail-closed regression for detailed target evidence that later degrades;
- host/message-loop lifecycle and automated evidence merged into `main`.

The old P1 phase tracker is complete. Representative physical Windows application validation is deliberately preserved in #33 rather than inferred from hosted CI.

### P2 — Windows productization

- explicit keyboard, clipboard, code, and auto injection modes;
- revision-guarded current-clipboard paste without clipboard rewrite/restore;
- immutable per-session backend and configuration selection;
- true characters-per-second pacing, bounded per-action jitter, and opt-in corrected adjacent-key typos;
- per-action cancellation, target, and modifier checks;
- strict recoverable per-user settings and schema migration;
- native Win32 tray/menu/notifications and controlled shutdown;
- validated custom Trigger/Cancel shortcuts with OS probing and transactional replacement;
- start-at-login integration;
- backend benchmarks and repeated controlled E2E;
- per-user portable/install/uninstall package smoke;
- Windows Server 2022 and 2025 x86_64 compatibility matrix;
- application/tray branding;
- public GitHub prerelease pipeline.

The implementation work previously tracked in #41 is complete. Its remaining named-application/physical validation obligation is consolidated into #33.

### P3 — cross-platform implementation history

P3 introduced the shared settings/hotkey contracts, Windows live hotkey replacement, macOS native adapters, menu-bar/product-shell work, and release/evidence tooling. The original target of a shared macOS Slint UI plus Universal 2 distribution was later superseded by the maintainer-directed P4 architecture.

Historical P3 implementation issues are closed when their shipped mechanism remains in the current product. Universal 2 and old-candidate validation tasks are closed as superseded rather than represented as completed support claims.

### P4 — macOS Apple Silicon Flutter rebuild

Delivered in the current product line:

- Flutter is the sole macOS settings/front-end composition root;
- one Apple Silicon arm64 product path; no Intel/Rosetta/Universal 2 claim;
- Rust remains authoritative for settings, injection policy, target safety, pacing, cancellation, and content-free outcomes;
- Swift/AppKit owns menu bar, Accessibility remediation, global hotkeys, startup integration, lifecycle, and the bounded Flutter/Rust bridge;
- task-oriented Overview / Input / Shortcuts / System / About UI;
- transactional shortcut validation/probing/replacement;
- keyboard/clipboard/code/auto behavior with corrected-typo controls;
- native Code-mode and Swift bridge contract gates;
- Apple Silicon CI build, Flutter analyze/test, Rust quality gate, ad-hoc signing, `/Applications` install/launch smoke, ZIP/DMG packaging, checksum verification, additive release upload, and post-publication re-download verification;
- continuous Code-mode regression coverage and Windows 2022/2025 controlled-host reliability coverage.

The implementation/release portion of P4 is complete for the testing-preview scope. Remaining physical Apple Silicon behavior and trusted Apple distribution promotion are consolidated into #61.

## Remaining acceptance gates

### #33 — representative Windows interactive validation

Required before stronger named-application or universal Windows behavior claims:

- exact-release physical/unlocked Windows environment record;
- real Trigger/Cancel behavior;
- representative native, Chromium, VS Code/Electron, terminal, and elevated/security-boundary paths;
- CJK/Unicode/newline/Tab behavior;
- measured chars/s, jitter, cancellation, focus-switch behavior, and typo/Backspace pacing;
- privacy sentinel;
- final evidence-backed `WINDOWS BETA READY` or `NOT READY` recommendation.

### #61 — physical macOS acceptance and trusted distribution promotion

Required before promoting the arm64 testing preview to a normal trusted macOS beta:

- downloaded exact-release Apple Silicon application behavior;
- persistent Accessibility grant/revoke and remediation;
- real named-application Unicode/Code-mode behavior;
- physical shortcut conflict/replacement, focus/cancel, modifiers, Secure Event Input, menu/status lifecycle, and Login Item behavior;
- privacy sentinel;
- Developer ID signature, Hardened Runtime, notarization, stapling, `codesign`/`spctl`, and exact-release provenance if trusted distribution is pursued.

The Intel/Rosetta boundary is explicit: P4 does not ship or claim Intel/Rosetta/Universal 2 support.

## Engineering follow-ups

These are useful improvements but are not represented as completed compatibility evidence:

- configure server-enforced branch protection / required checks when repository-administration access is available;
- configure a trusted Windows Authenticode certificate or managed signing service if desired;
- remove non-blocking CI/tooling deprecation warnings as upstream actions/toolchains evolve;
- measure long-running tray/menu-bar lifecycle, startup, settings migration, and repeated-session resource behavior on physical clients;
- evaluate package-manager publication after install/update semantics stabilize.

## Later platform expansion

- evaluate Windows ARM64 with a dedicated artifact and compatibility matrix;
- implement Linux X11 without weakening platform-independent core policy;
- treat Wayland as independent capabilities rather than a boolean platform claim;
- add a Linux UI only after backend/evidence decisions are accepted;
- consider transformed/generated text only through a separately reviewed clipboard-transaction design that cannot overwrite external changes.

## Stable 1.0 gate

A stable release requires more than green prerelease pipelines:

- sufficient physical-client and named-application evidence on every claimed platform;
- no unresolved high-severity data-loss, privacy, destination-safety, permission, privilege, shortcut, packaging, signing, or migration defect;
- stable configuration migration and uninstall behavior;
- an explicit support policy and security-maintenance plan;
- trusted platform signing decisions appropriate to the supported platforms;
- server/repository governance appropriate for stable release;
- a separately reviewed release decision and version bump.
