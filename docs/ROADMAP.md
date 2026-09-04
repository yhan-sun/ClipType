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
- true characters-per-second pacing, bounded per-action jitter, and opt-in corrected adjacent-key typos;
- strict recoverable per-user settings and schema migration;
- native Win32 tray/menu/notifications and controlled shutdown;
- reviewed hotkey presets and start-at-login integration;
- backend benchmarks and repeated P1/P2 controlled E2E;
- per-user portable/install/uninstall package smoke;
- Windows Server 2022 and 2025 x86_64 compatibility matrix;
- selected application and tray branding embedded in the executable;
- SHA-256 manifests, Sigstore keyless signatures, GitHub artifact attestations, dependency inventory, and build metadata;
- public GitHub prerelease pipeline for `v0.1.0-beta.1`.

## Current release

`v0.1.0-beta.1` is the first public Windows x86_64 prerelease. It does not include macOS, a graphical settings window, arbitrary user-recorded shortcuts, Authenticode publisher identity, or universal per-application compatibility.

Issues #33 and #41 remain the post-fix Windows interactive-evidence track. Their closure is independent from the architectural start of P3 and is still required before stronger Windows compatibility wording.

## Current milestone

### P3 — cross-platform settings UI, custom hotkeys, and macOS productization

**Target:** `v0.2.0-beta.1`  
**Platforms:** Windows x86_64 and macOS Universal 2

#### Wave 0 — architecture and contracts

- [ ] #44 — accept the desktop UI/process-model and UI-toolkit license ADR;
- [ ] #45 — add native-neutral `HotkeySpec`/`HotkeyPair`, availability/apply outcomes, validation, and settings migration.

#### Wave 1 — shared UI and Windows custom shortcuts

- [ ] #46 — build the native-compiled shared settings window;
- [ ] #47 — implement Windows OS-level hotkey probing and atomic live pair replacement with rollback.

#### Wave 2 — macOS native evidence and adapters

- [ ] #48 — run the macOS permission, Unicode, focus, hotkey, status-item, and event-loop spike;
- [ ] #49 — implement macOS clipboard, keyboard, paste, focus, modifier, permission, and command adapters after the spike returns YES.

#### Wave 3 — macOS product shell and distribution

- [ ] #50 — build the menu-bar `.app`, permission onboarding, settings integration, and login-item lifecycle;
- [ ] #51 — build Universal 2 artifacts and gated Developer ID signing/notarization automation.

#### Wave 4 — cross-platform evidence and prerelease

- Windows regression matrix for the graphical settings window and custom shortcuts;
- physical Apple Silicon and Intel/Universal 2 evidence with named applications;
- exact-SHA Unicode, focus, cancellation, shortcut-conflict, permission-revocation, privacy-sentinel, install/upgrade, and lifecycle evidence;
- signed/notarized macOS release assets when maintainer credentials are configured;
- final `CROSS_PLATFORM BETA READY` or `NOT READY` result.

## P3 product surface

The settings window contains:

- General — enabled, notifications, start at login;
- Shortcuts — local Trigger/Cancel recorders, validation, OS probe status, Reset, Apply, and rollback result;
- Typing — Keyboard/Clipboard/Code/Auto mode, exact characters per second, jitter, corrected typo probability, Auto threshold, and safety guidance;
- Permissions — macOS Accessibility state and explicit remediation;
- About & Updates — version/channel, release notes, project licenses, dependency notices, and UI-toolkit attribution.

Shortcut availability is evidence-based. ClipType can detect many OS-level global-registration conflicts, but it cannot prove that an application-local shortcut or another tool's hook will never also react; the UI reports that boundary as `Unknown` or `Cannot fully verify`.

## Later milestones

### P4 — field compatibility, trusted publisher, and operational hardening

The P4 macOS Apple Silicon local runner is now an implementation track within
this milestone. It provides an arm64-only Flutter/AppKit/Rust candidate and
does not close the public Universal 2 or signing gates.

#### P4-A — Apple Silicon local runner

- [x] Replace the Flutter counter scaffold with the real ClipType settings UI;
- [x] Integrate the fixed Flutter channels, Swift/AppKit shell, and Rust C ABI;
- [x] Build and scan an arm64-only release `.app` locally;
- [x] Record content-free automated and interactive local evidence;
- [ ] Complete physical target-application, permission grant/revoke, conflict,
  cancellation, Unicode, and latency evidence.

- close post-fix Windows and macOS named-application evidence gaps;
- configure a trusted Windows Authenticode certificate or managed signing service;
- retain Sigstore and GitHub attestations as additional provenance;
- add structured crash-category guidance without collecting dumps or clipboard content;
- measure long-running tray/menu-bar lifecycle, startup, settings migration, and repeated-session resource behavior;
- evaluate MSIX, Homebrew Cask, WinGet, or other package-manager publication after install/update semantics stabilize.

### P5 — Linux and architecture expansion

- evaluate Windows ARM64 with a dedicated artifact and compatibility matrix;
- implement Linux X11 without weakening the platform-independent core;
- treat Wayland as independent capabilities rather than a boolean platform claim;
- add GTK/libadwaita or reuse the shared settings UI only after Linux backend evidence;
- consider transformed/generated text only through a new clipboard-transaction ADR that cannot overwrite external changes.

## Stable 1.0 gate

A stable release requires more than green prerelease pipelines:

- sufficient physical-client and named-application evidence on every claimed platform;
- no unresolved high-severity data-loss, privacy, destination-safety, permission, privilege, shortcut, packaging, signing, or migration defect;
- stable configuration migration and uninstall behavior;
- an explicit support policy and security-maintenance plan;
- trusted platform signing decisions for Windows and macOS;
- a separately reviewed release decision and version bump.
