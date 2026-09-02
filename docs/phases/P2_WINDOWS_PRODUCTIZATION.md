# P2 Windows Productization Execution Plan

**Status:** In progress  
**Tracking issue:** [#36](../../issues/36)  
**Entry integration base:** `0e8e2dad96fc82529961dbbabe31d672da194ee7`  
**Representative compatibility follow-up:** [#33](../../issues/33)

P2 turns the P1 development vertical slice into a daily-usable native Windows utility. This plan owns implementation sequencing and automated evidence. It does not turn hosted-runner evidence into a public compatibility claim.

## 1. Intended product loop

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

## 2. Fixed P2 implementation scope

- versioned recoverable TOML settings;
- enable/disable;
- explicit `keyboard`, `clipboard`, and `auto` modes;
- benchmark-derived auto threshold;
- configurable reviewed hotkey presets and keyboard pacing;
- current-clipboard Paste backend with revision guard and no clipboard rewrite;
- native Win32 tray/settings menu;
- per-user start-at-login control;
- portable and per-user installation artifacts;
- automated keyboard/clipboard/auto controlled target evidence;
- package/config/startup smoke tests;
- privacy sentinel and dependency/license checks.

## 3. Explicitly deferred

- clipboard history or continuous plaintext observation;
- transformed-text clipboard transactions;
- arbitrary macros or scripting;
- automatic elevation or UIPI bypass;
- exact logical-caret guarantees inside one render host;
- macOS/Linux platform crates;
- signing-key use or public release promotion;
- representative interactive application claims tracked by #33.

## 4. Architecture

The P1 dependency diamond remains unchanged. P2 adds contracts and implementations to existing crates rather than creating a generic UI or utility crate prematurely.

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

- core owns product configuration and pure backend selection;
- platform owns clipboard revision and paste ports;
- app owns one live session, configuration snapshots and backend orchestration;
- Windows owns sequence-number observation, paste chord, tray, startup and native shell;
- the executable owns persistence and composition.

## 5. Work waves

### Wave 1 — product contracts and planner

- `InjectionMode`, `InjectionBackend`, `ProductConfig`;
- content-free clipboard snapshot/revision;
- current-clipboard paste port;
- explicit-mode no-fallback and auto selection tests;
- ADR-0006.

### Wave 2 — mechanism and coordinator

- Windows sequence-number implementation;
- bounded Ctrl+V dispatch with conservative native-result mapping;
- one coordinator path for keyboard and clipboard plans;
- immutable configuration/backend snapshot per session;
- clipboard-change, focus, cancellation, modifier and integrity tests.

### Wave 3 — persistence and native shell

- strict versioned TOML parser/serializer;
- recoverable atomic file writes and backup loading;
- native tray/menu/status shell;
- reviewed hotkey presets;
- per-user startup registry management;
- live settings updates affecting only future sessions.

### Wave 4 — evidence and packaging

- controlled E2E for keyboard, clipboard and auto;
- threshold benchmark matrix;
- portable ZIP and per-user install/uninstall scripts;
- package/startup/config smoke tests;
- privacy sentinel, dependency/license and forbidden-content checks;
- P2 automated evidence report.

## 6. Mandatory safety properties

1. Explicit modes do not silently change backend.
2. Auto selects only fully available capabilities.
3. A clipboard revision change stops Paste.
4. ClipType never rewrites or restores the clipboard in P2.
5. Target evidence is captured before clipboard work and revalidated before dispatch.
6. Detailed focus evidence degrading later fails closed.
7. Physical user modifiers are observed, never released.
8. Native work, waits, retries, batches and shutdown are bounded.
9. Partial/unknown input is never retried.
10. One active worker exists; a second trigger is busy.
11. Configuration, UI, logs and artifacts contain no clipboard plaintext.
12. The process remains unprivileged and does not bypass platform controls.

## 7. Verification gates

Every final P2 commit must pass:

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

Additional Windows jobs cover host/tray lifecycle, registry cleanup, controlled target delivery, benchmark evidence, package installation and privacy output scanning.

## 8. Merge and release rule

P2 remains stacked until P1-10 and P1-B01 are independently reviewed and merged. No agent may merge, tag, sign, publish, promote a beta, or expand compatibility wording merely because this automated gate is green.
