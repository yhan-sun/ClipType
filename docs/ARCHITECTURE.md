# Architecture

## Style

ClipType uses ports and adapters around a platform-independent Rust core. Policy, native capability/evidence, runtime coordination, platform mechanism, presentation, and release automation are separate boundaries.

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

- `cliptype-core` owns domain values, normalization, limits, state transitions, outcomes, product configuration, and pure backend selection.
- `cliptype-platform` owns native-neutral clipboard, target, keyboard, modifier, paste, command, capability, and dispatch-result contracts.
- `cliptype-app` owns the live one-session coordinator, immutable session/configuration snapshots, cancellation, settings parsing, persistence, and recovery.
- `cliptype-windows` owns Win32 clipboard, target/integrity evidence, keyboard dispatch, paste, hotkey/message loop, tray, and startup adapters.
- `apps/cliptype` is the Windows composition root and owns process lifecycle, settings application, content-free status, and user command wiring.
- `apps/cliptype-flutter` is the P4 macOS arm64 composition root. Flutter owns presentation; its Swift/AppKit shell owns channels, menu-bar lifecycle, global hotkeys, Accessibility/login-item adapters, and the fixed Rust bridge.
- `crates/cliptype-flutter-bridge` owns the narrow content-free C ABI that keeps the Rust coordinator and `cliptype-macos` adapters behind the Swift shell.
- There is no Rust/Slint macOS application composition root. `apps/cliptype-flutter` is the only macOS settings/front-end entry point; `crates/cliptype-ui` remains a Windows presentation dependency.
- packaging/release workflows own reproducible assets, compatibility checks, signatures, attestations, and public publication.

Core never imports platform APIs. Platform adapters do not choose product policy. Presentation does not implement injection policy directly.

## Product runtime

```text
tray / reviewed global trigger
  -> atomically reserve one session
  -> snapshot validated product settings
  -> capture initial destination evidence
  -> wait boundedly for physical trigger modifiers to clear
  -> read bounded current clipboard text and revision
  -> build and freeze keyboard or clipboard plan
  -> reject known higher-integrity target
  -> revalidate destination and modifiers
  -> dispatch bounded keyboard batches or one guarded Paste chord
  -> classify complete / none / partial / unknown result
  -> publish content-free completion
  -> release session slot
```

The destination is captured before clipboard work. A second trigger is Busy, not queued. Cancellation is cooperative and checked at safe bounded points. An active session keeps its original settings/backend snapshot even when future settings change.

## Core plans

### Keyboard plan

The core normalizes the owned clipboard text into semantic atoms. A validated plan contains bounded slices and immutable safety configuration. Windows converts those atoms into Unicode or explicit control-key events.

### Clipboard plan

The clipboard plan is content-free except for element count and backend identity. It refers to the already-current OS clipboard and requires both paste capability and a known revision witness. No text is written into a paste plan for later clipboard restoration.

### Auto selection

Auto uses pure policy and current capability evidence. It can select clipboard only when paste and revision guarding are fully available. It selects one backend before dispatch and never changes backend mid-session. Explicit modes never silently fall back.

For any non-ASCII text—including CJK, emoji, combining marks, and mixed
Unicode—Auto prefers the already-current, revision-guarded paste path even
when the payload is below the size threshold. If that path is unavailable,
Auto may use the proven Unicode keyboard path; explicit Keyboard mode keeps
its requested semantics and never silently switches backends.

## Native-neutral ports

### ClipboardPort

Responsibilities:

- perform one bounded current-text acquisition;
- return owned text after releasing native clipboard locks/handles;
- expose a content-blind revision witness;
- reject a known change across snapshot acquisition.

It does not provide history, continuous observation, write, clear, ownership, or restore operations for the current product.

### TargetPort

Responsibilities:

- capture the strongest practical non-content destination identity;
- compare new evidence with the original;
- report disappearance, ambiguity, degradation, and integrity relation;
- redact opaque handles/tokens from diagnostics.

It never reads focused-field text or window titles. Shared render hosts may not expose exact logical-field identity.

### KeyboardPort and ModifierPort

Responsibilities:

- advertise Unicode/line-break/Tab/modifier capabilities;
- observe conflicting physical modifiers;
- accept bounded semantic batches;
- return complete, none, partial, or progress-unknown native results.

The adapter never releases physical keys owned by the user. Retry policy remains in core/application and forbids retry after partial/unknown progress.

### PastePort

Responsibilities:

- advertise ordinary paste and revision-guard capabilities;
- verify the expected revision immediately before dispatch;
- send one balanced native Paste chord;
- return conservative native progress.

It never rewrites or restores clipboard contents.

### Command source

The command source registers reviewed trigger/cancel hotkey pairs with no-repeat behavior, owns its Windows message queue, and delivers only typed product commands. It is not a general keyboard-capture port.

## Windows adapters

### Clipboard

`CF_UNICODETEXT` is copied from clipboard-owned global memory within configured byte limits. Sequence-number checks are content-blind. Clipboard contention is mapped to bounded retryable categories; malformed, non-text, empty, or oversized data fails clearly.

### Keyboard and paste

`SendInput` is used for bounded Unicode/key events and for one balanced `Ctrl+V` chord. Accepted native event counts are preserved. Zero accepted events are not automatically labelled UIPI unless integrity evidence independently proves that boundary.

### Destination and integrity

Foreground top-level window, process/thread identity, GUI-thread active/focus evidence, and integrity relation form the destination witness. Detailed original evidence that later weakens fails closed. The normal process does not inject into a known higher-integrity target.

### Tray and startup

A dedicated Win32 tray thread owns the hidden window, notification icon, menu, and message loop. The host coordinates tray events with the coordinator and settings store. Start-at-login uses one product-owned value under the current user's Run key and a quoted executable command.

## macOS Flutter composition root (P4 local arm64)

The local macOS candidate has one process and one Flutter engine. Swift/AppKit
retains one `NSStatusItem`, one `NSMenu`, and one Settings window. Closing the
window hides it so the status item and registered commands remain alive; Quit
performs bounded Rust shutdown and removes native state.

The fixed Flutter boundary is:

```text
Flutter MethodChannel: io.cliptype/native
Flutter EventChannel:  io.cliptype/events
             │ bounded settings/commands/content-free events
Swift/AppKit shell → cliptype-flutter-bridge static library
             │ fixed integer status and enum/counter snapshot
Rust Coordinator → cliptype-core + cliptype-platform + cliptype-macos
```

The channel and C ABI do not carry clipboard text, injected text, focused
values, window titles, recorded-key history, user identity, or content
fingerprints. Flutter does not read the pasteboard or execute input. Rust
continues to own validation, session reservation, backend selection, target
and modifier safety, revision guarding, pacing, cancellation, and terminal
outcomes. Swift owns only native shell mechanisms and command registration.

The display language is a separate non-sensitive presentation preference. The
Flutter window and the native status menu synchronize its English/Simplified
Chinese value through the fixed method channel; it is not part of the Rust
product settings or injection policy.

Trigger/Cancel are registered with the system hot-key API as a transactional
pair. A candidate is validated and probed before the old pair is released;
failure removes temporary registrations and leaves the prior pair active.
The local recorder is a focused Flutter control and is not a global event tap,
key logger, or broad keyboard monitor.

Observation is event-driven while idle. A bounded timer may refresh state only
while a session is active, and a separate short-lived observation may follow an
explicit Accessibility onboarding action, including opening System Settings.
There is no permanent 40 ms application poll.

P4 is an Apple Silicon-only local candidate (`aarch64-apple-darwin`). It does
not widen the public P3 Universal 2, signing, notarization, or compatibility
claims. See [ADR-0010](adr/0010-flutter-macos-arm64-runner.md).

## Settings

The fixed versioned schema includes enabled state, mode, auto threshold, speed, notifications, start-at-login, and reviewed hotkey preset. Parsing rejects unknown, duplicate, missing, malformed, or unsupported fields without echoing their values.

Saving uses an adjacent temporary file, durable flush, validated backup rotation, and replacement. A missing file loads safe defaults; a corrupt primary may recover from a valid backup. Settings never contain clipboard contents or target data.

## Process and concurrency

The default product is one normal-integrity per-user process. Native message-loop threads communicate with the application through typed channels/signals; the bounded injection worker is separate so hotkey/tray queues remain responsive.

Poisoned synchronization primitives are recovered without exposing plaintext. Worker panics are caught at the session boundary and mapped to an internal-invariant outcome. Shutdown requests cancellation, waits within a configured grace period, joins completed workers, removes tray state, and unregisters commands.

No service, driver, privileged helper, automatic elevation, or general daemon/client split is used on Windows.

## Error and outcome model

Preparation failures and terminal outcomes remain typed and content-free, including:

- disabled, busy, shutting down;
- unsupported/degraded capability;
- empty, non-text, malformed, oversized, unavailable, or changed clipboard;
- target changed, disappeared, ambiguous, or evidence unavailable;
- modifier conflict/settle timeout;
- known security restriction versus blocked cause unknown;
- complete, cancelled, partial input, progress unknown, native failure, or internal invariant.

UI/logging maps these categories to fixed remediation text. It does not include clipboard text, window title/content, raw handles, or revision numbers.

## Compatibility and release architecture

Compatibility is stated by evidence class and mechanism, not by universal application branding. The matrix runs the complete x86_64 product on Windows Server 2022 and 2025 hosted images. Client support and limitations are defined separately in `docs/COMPATIBILITY.md`.

The public release workflow:

- rebuilds from the exact `main` commit;
- reruns check/test/Clippy;
- creates versioned ZIP and portable executable assets;
- embeds licenses, configuration, release notes, dependency/license inventory, and build metadata;
- generates SHA-256 checksums;
- signs assets with Sigstore keyless GitHub OIDC identity;
- verifies signatures before publication;
- creates GitHub artifact attestations;
- creates an immutable prerelease only when the tag does not already exist.

Authenticode trusted-publisher signing is a separate future boundary because it requires a trusted certificate or managed signing service.

## Invariants

1. Core policy is platform-independent.
2. Native adapters do not decide product policy.
3. Destination evidence is captured before clipboard acquisition and revalidated before dispatch.
4. Detailed evidence degradation fails closed.
5. Injection is explicit, one-session, bounded, and cancellable.
6. Physical modifiers are observed, never released.
7. Partial/unknown native input is never blindly retried.
8. Clipboard plaintext is ephemeral and absent from persistence/diagnostics/network transport.
9. Clipboard mode never writes, clears, owns, or restores the clipboard.
10. Active plans and settings snapshots are immutable.
11. Privilege is not escalated or bypassed.
12. Compatibility wording cannot exceed evidence.
13. Public assets are versioned, checksummed, signed, attested, and never silently replaced.
14. Cross-cutting changes require an ADR.
15. The Flutter/native boundary remains fixed, bounded, and content-free.
