# P2 Windows Productization

**Status:** Implementation complete; public beta candidate under final automated gate  
**Tracking issue:** [#36](../../issues/36)  
**Original stacked integration base:** `0e8e2dad96fc82529961dbbabe31d672da194ee7`  
**Current merge target:** `main`  
**Public beta:** `v0.1.0-beta.1`

P2 turns the P1 development vertical slice into a daily-usable native Windows tray utility and defines its scoped public compatibility and provenance contract.

## 1. Product loop

```text
native tray or global trigger
  -> reserve one session
  -> snapshot product configuration
  -> capture destination evidence
  -> wait for physical trigger modifiers
  -> read bounded current clipboard text + revision
  -> select keyboard or current-clipboard paste plan
  -> revalidate destination, integrity, modifiers and clipboard revision
  -> perform bounded backend dispatch
  -> publish content-free status/notification
  -> release session
```

## 2. Implemented scope

- strict versioned and recoverable per-user settings;
- enable/disable, notifications, speed, startup-at-login, and reviewed hotkey presets;
- explicit `keyboard`, `clipboard`, and `auto` modes;
- benchmark-covered auto threshold behavior;
- revision-guarded current-clipboard paste with no clipboard rewrite/restore;
- native Win32 tray, context menu, status notifications, and controlled shutdown;
- per-user start-at-login registration;
- portable executable and per-user ZIP/install/uninstall package;
- deterministic policy, Windows adapter, coordinator, privacy, and boundedness tests;
- P1/P2 controlled native edit-target E2E;
- Windows Server 2022 and 2025 x86_64 compatibility matrix;
- SHA-256 manifests, Sigstore keyless signatures, GitHub artifact attestations, dependency inventory, and build metadata;
- public GitHub prerelease pipeline tied to `release/VERSION`.

## 3. Retained non-goals and boundaries

- no clipboard history or continuous plaintext observation;
- no transformed-text clipboard transaction;
- no arbitrary macros or general key capture;
- no automatic elevation or UIPI bypass;
- no exact logical-caret guarantee inside one shared render host;
- no ARM64 or 32-bit artifact in this release;
- no Server Core/service/non-interactive support;
- no universal per-application claim;
- no Authenticode trusted-publisher claim until a trusted certificate or managed signing service is configured.

## 4. Architecture

The P1 dependency diamond remains intact:

```text
                  cliptype-core
                    ^       ^
                    |       |
          cliptype-platform |
             ^          ^   |
             |          |   |
      cliptype-app   cliptype-windows
             ^          ^
              \        /
               apps/cliptype
```

- core owns product configuration, normalization, limits, state, and pure backend selection;
- platform owns clipboard revision, paste, keyboard, target, command, and native result contracts;
- app owns one live session, immutable configuration/backend snapshots, cancellation, and persistence;
- Windows owns clipboard sequence observation, paste chord, keyboard dispatch, target/integrity evidence, hotkeys, tray, and startup registration;
- the executable owns composition, status, settings application, and lifecycle;
- GitHub Actions owns reproducible compatibility, packaging, signing, attestation, and release publication.

## 5. Completed work waves

### Wave 1 — product contracts and planner

Completed: mode/backend/configuration contracts, clipboard revision/snapshot, current-clipboard paste port, explicit no-fallback policy, auto selection, and ADR-0006.

### Wave 2 — mechanism and coordinator

Completed: Windows revision guard and bounded Paste, unified coordinator, immutable per-session selection, and clipboard/focus/cancellation/modifier/integrity failure coverage.

### Wave 3 — persistence and native shell

Completed: strict settings store, atomic recovery writes, native tray/menu/status, hotkey presets, startup registration, and future-session configuration updates.

### Wave 4 — evidence and packaging

Completed: controlled keyboard/clipboard/auto E2E, Code-mode coordinator coverage, benchmark workflow, package install/startup/uninstall smoke, privacy scanning, and release-subsystem build.

### Wave 5 — compatibility and public release

Completed in the candidate: scoped Windows x86_64 compatibility declaration, Server 2022/2025 matrix, release notes, Sigstore signing, GitHub attestations, checksum/dependency/build metadata, and public prerelease automation.

## 6. Mandatory safety properties

1. Explicit modes do not silently change backend.
2. Auto selects only fully available capabilities.
3. A clipboard revision change stops Paste.
4. ClipType never rewrites or restores the clipboard.
5. Target evidence is captured before clipboard work and revalidated before dispatch.
6. Detailed focus evidence degrading later fails closed.
7. Physical user modifiers are observed, never released.
8. Native work, waits, retries, batches, and shutdown are bounded.
9. Partial/unknown input is never retried.
10. One active worker exists; a second trigger is busy.
11. Configuration, UI, logs, evidence, and distributable metadata contain no clipboard plaintext.
12. The process remains unprivileged and does not bypass platform controls.
13. Compatibility wording never exceeds the evidence class.
14. Public assets are checksummed, signed, attested, and never silently replaced.

## 7. Verification gates

Every product candidate passes:

```text
cargo fmt --all -- --check
cargo check --locked --all-targets -p cliptype-core -p cliptype-platform -p cliptype-app
cargo test --locked -p cliptype-core -p cliptype-platform -p cliptype-app
cargo clippy --locked --all-targets -p cliptype-core -p cliptype-platform -p cliptype-app -- -D warnings
cargo metadata --locked --format-version 1 --no-deps
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Additional jobs cover native hotkey/message-loop behavior, host lifecycle, controlled target delivery, backend selection and clipboard preservation, benchmark evidence, package install/startup/uninstall, dual Windows runner compatibility, release asset assembly, privacy scans, Sigstore verification, and GitHub attestations.

## 8. Merge and release rule

P1-10 and P1-B01 are merged into `main`. P2 may merge only from an exact reviewed head with all applicable checks green. The public prerelease is created only by the release workflow from the resulting exact `main` commit, and only when the declared version/tag does not already exist.

The first beta is a real public release but not a stable or universal compatibility promise. Authenticode trusted-publisher signing remains a separately controlled follow-up because no trusted certificate is configured.
