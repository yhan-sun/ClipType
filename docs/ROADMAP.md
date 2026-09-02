# Roadmap

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
- P1 automated evidence and independent review dependencies merged into `main`.

### P2 — Windows productization

- explicit keyboard, clipboard, and auto injection modes;
- revision-guarded current-clipboard paste without clipboard rewrite/restore;
- immutable per-session backend and configuration selection;
- strict recoverable per-user settings;
- native Win32 tray/menu/notifications and controlled shutdown;
- reviewed hotkey presets and start-at-login integration;
- backend benchmarks and repeated P1/P2 controlled E2E;
- per-user portable/install/uninstall package smoke;
- Windows Server 2022 and 2025 x86_64 compatibility matrix;
- scoped Windows 11/Windows 10 beta compatibility declaration;
- SHA-256 manifests, Sigstore keyless signatures, GitHub artifact attestations, dependency inventory, and build metadata;
- automated public GitHub prerelease pipeline for `v0.1.0-beta.1`.

## Current release milestone

`v0.1.0-beta.1` is the first public Windows x86_64 beta. It is intentionally a prerelease and retains documented boundaries for elevated targets, shared render hosts, terminal semantics, rich clipboard formats, remote/virtual desktops, ARM64, 32-bit Windows, and Authenticode trusted-publisher signing.

## Next milestones

### P3 — field compatibility and operational hardening

- collect content-free named-application reports from physical Windows 11 desktops;
- prioritize browsers, editors, office applications, chat clients, and terminal front ends;
- add deterministic regression fixtures for every confirmed application-specific failure that can be reproduced safely;
- improve hotkey conflict remediation and optionally support live preset re-registration with rollback;
- add structured crash-category guidance without collecting dumps or clipboard content;
- measure long-running tray lifecycle, startup, and repeated-session resource behavior.

### P4 — trusted Windows publisher and richer distribution

- procure or configure a trusted Authenticode certificate/managed signing service;
- isolate signing credentials, timestamp and verify the signed executable before packaging;
- retain Sigstore signatures and GitHub attestations as additional provenance layers;
- evaluate MSIX or package-manager publication only after installer/uninstaller and migration behavior are stable;
- add update discovery only with an explicit privacy and rollback design.

### P5 — architecture expansion

- evaluate Windows ARM64 with a dedicated artifact and compatibility matrix;
- evaluate macOS and Linux adapters without weakening the platform-independent core;
- add a dedicated accessible settings window only when it improves the bounded tray experience;
- consider transformed/generated text only through a new clipboard transaction ADR that cannot overwrite external changes.

## Stable 1.0 gate

A stable release requires more than a green beta pipeline:

- sufficient physical-client and named-application evidence;
- no unresolved high-severity data-loss, privacy, target-safety, privilege, or packaging defect;
- stable configuration migration and uninstall behavior;
- an explicit support policy and security-maintenance plan;
- a maintainer decision on Authenticode trusted-publisher signing;
- a separately reviewed release decision and version bump.
