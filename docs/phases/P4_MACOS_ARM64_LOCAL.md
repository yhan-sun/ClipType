# P4 — macOS Apple Silicon local runner

## Status

- Phase: implemented local candidate; physical release gate remains open
- Target: `aarch64-apple-darwin` only
- UI: Flutter macOS desktop with English / Simplified Chinese display mode
- Native shell: Swift/AppKit
- Policy/runtime: Rust core, application coordinator, and macOS adapters
- Release status: unsigned local candidate; not a public beta

## Scope

P4 replaces the Flutter counter scaffold with a real ClipType settings and
menu-bar composition root. It does not claim Universal 2, Intel, Rosetta,
Developer ID, notarization, or named-application compatibility.

The runtime contract is one process, one Flutter engine, one status item, one
settings window, one global hot-key owner, and at most one active session.
Closing Settings hides the window; it does not terminate the menu-bar process.

## Boundaries

```text
Flutter UI
  ├─ General / Shortcuts / Typing / Permissions / About
  └─ fixed channels: io.cliptype/native, io.cliptype/events
        │ content-free state, commands, and event categories
Swift/AppKit shell
  ├─ NSStatusItem and NSMenu
  ├─ Carbon global hot-key registration
  ├─ Accessibility request/status and System Settings link
  ├─ SMAppService login-item adapter
  └─ fixed C ABI ownership of the Rust static library
        │ bounded settings and enum/counter status
Rust
  ├─ ProductSettings validation and persistence
  ├─ Coordinator single-session/cancel/backend policy
  └─ cliptype-macos clipboard, target, keyboard, modifier, and paste adapters
```

The bridge does not carry clipboard or injected plaintext. All UI messages are
fixed categories such as `ok`, `invalid`, `busy`, `permission_required`,
`conflict`, `target_changed`, `clipboard_changed`, and `native_failure`.
When an explicit Trigger action finds Accessibility trust missing, the Flutter
runner opens the macOS Accessibility settings page and continues a bounded
permission-state observation; macOS consent remains user-controlled.

## Local verification gate

The exact commands and results for a local run belong in
`local-evidence/P4_LOCAL_MACOS_ARM64_REPORT.md`. The gate includes Flutter
format/analyze/test, the arm64 Rust workspace check/test/Clippy, a macOS
release build, an arm64-only Mach-O scan, codesign integrity verification, and
interactive UI/process smoke.

The local gate is not complete until the report separately accounts for
Accessibility grant/revoke, recorder behavior, Chinese/Unicode target matrix,
conflict rollback, cancellation, focus changes, and measured trigger/cancel
latency. If a physical case is not run, it remains `NOT RUN`.

## Release boundary

This phase may produce a local `.app` for inspection. It must not be described
as `MACOS ARM64 BETA READY` without the physical and release evidence listed
above. No push, merge, tag, release publication, Apple credential, or
notarization action is part of this local track.
