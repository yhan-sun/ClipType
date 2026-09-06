# Architecture

## Style

ClipType uses ports and adapters around a platform-independent Rust core. Policy, native capability/evidence, runtime coordination, platform mechanisms, presentation, and release automation are separate boundaries.

### Windows composition

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

### macOS Apple Silicon composition

```text
Flutter presentation
       |
Swift/AppKit shell
       |
cliptype-flutter-bridge (fixed C ABI)
       |
cliptype-app + cliptype-core + cliptype-platform
       |
cliptype-macos native adapters
```

- `cliptype-core` owns domain values, normalization, limits, state transitions, outcomes, product configuration, Code planning, and pure backend selection.
- `cliptype-platform` owns native-neutral clipboard, target, keyboard, modifier, paste, command, capability, and dispatch-result contracts.
- `cliptype-app` owns the live one-session coordinator, immutable session/configuration snapshots, cancellation, settings parsing, persistence, and recovery.
- `cliptype-windows` owns Win32 clipboard, target/integrity evidence, keyboard dispatch, paste, hotkey/message loop, tray, and startup adapters.
- `apps/cliptype` is the Windows composition root and owns process lifecycle, settings application, content-free status, and user command wiring.
- `apps/cliptype-flutter` is the sole macOS settings/front-end composition root for the current P4 Apple Silicon line. Flutter owns presentation; Swift/AppKit owns channels, menu/status lifecycle, global shortcuts, Accessibility/startup mechanisms, and application lifecycle.
- `crates/cliptype-flutter-bridge` owns the narrow content-free C ABI that keeps Rust coordination and macOS adapters behind the Swift shell.
- `crates/cliptype-ui` remains a Windows presentation dependency; there is no legacy Rust/Slint macOS application composition root.
- packaging/release workflows own reproducible assets, compatibility checks, signatures/attestations, exact-source identity, and publication.

Core never imports platform APIs. Platform adapters do not choose product policy. Presentation does not implement injection policy directly.

## Product runtime

```text
explicit trigger / tray-menu command
  -> atomically reserve one session
  -> snapshot validated product settings
  -> capture initial destination evidence
  -> wait boundedly for physical trigger modifiers to clear
  -> read bounded current clipboard text + revision
  -> build and freeze keyboard / clipboard / code / auto-selected plan
  -> reject known higher-integrity target
  -> revalidate destination, modifiers, cancellation and required revision
  -> dispatch bounded semantic/native actions
  -> classify complete / none / partial / unknown result
  -> publish content-free completion
  -> release session slot
```

The destination is captured before clipboard work. A second trigger is Busy, not queued. Cancellation is cooperative and checked at bounded points. Active sessions keep their original settings/backend snapshot even when future settings change.

## Core plans

### Keyboard plan

Core normalizes owned clipboard text into semantic atoms. Keyboard delivery applies validated per-action pacing, jitter, optional corrected-typo expansion, and immutable safety policy. Platform adapters translate one bounded semantic action into native events.

### Clipboard plan

Clipboard mode refers to the **already-current** operating-system clipboard. The plan requires ordinary Paste capability and a known content-blind revision witness.

ClipType does not create a temporary clipboard transaction: it does not write, clear, replace, own, snapshot-for-restoration, or restore clipboard contents. Immediately before the single native Paste chord, target/integrity/modifier/cancellation/revision requirements are revalidated. A changed revision aborts before paste.

### Code plan

Code mode is a named keyboard-only source plan.

It:

- normalizes current clipboard text;
- strips leading spaces/Tabs at normal-code line starts so the editor may supply indentation;
- types ordinary source atoms;
- recognizes only `()`, `{}`, `[]`, `""`, and `''` as pair-aware families;
- represents matching source closers as bounded cursor-navigation actions when the destination editor is expected to have generated the closer;
- uses a line-closing navigation action for matching closers that begin a source line, including after a `//` comment boundary;
- keeps brackets inside recognized strings/comments literal;
- emits Python-style triple-quoted boundaries explicitly;
- treats Markdown backtick fences and single backticks as literal;
- preserves strict FIFO action order and bounded settle barriers;
- may apply corrected typo simulation to eligible source atoms only, never to cursor navigation or non-ASCII source text;
- requires keyboard + navigation capabilities and never uses Paste/revision fallback.

macOS uses bounded Right/Command+Right-style navigation mechanics behind its adapter; Windows uses the corresponding Right/End path. The plan never reads editor text, selected text, DOM values, caret content, or editor configuration. It is therefore a destination-editor contract, not editor automation.

### Auto selection

Auto uses pure policy and current capability evidence. Non-ASCII text—including CJK, emoji, combining marks, and mixed Unicode—prefers the already-current revision-guarded paste path when available. Otherwise Auto may choose the proven Unicode keyboard path. The configured threshold remains a crossover for otherwise keyboard-friendly payloads.

One backend is frozen before dispatch. Explicit Keyboard, Clipboard, and Code modes never silently fall back.

## Native-neutral ports

### ClipboardPort

Responsibilities:

- perform one bounded current-text acquisition;
- return owned text after releasing native clipboard locks/handles;
- expose a content-blind revision witness;
- reject a known change across snapshot acquisition.

It has no clipboard history/listener/write/clear/restore responsibility in the current product.

### TargetPort

Responsibilities:

- capture the strongest practical non-content destination identity;
- compare current evidence with the original;
- report change, disappearance, ambiguity/degradation, and integrity relation;
- redact opaque handles/tokens from diagnostics.

It never reads focused-field text or window titles.

On macOS, native controls retain exact focused-element comparison. An initial focused element beneath an `AXWebArea` selects a sticky render-host-limited session policy: the adapter compares stable frontmost process + focused top-level window while tolerating transient same-window renderer focus-node replacement. Process/window changes or loss of required stable window evidence stop. Logical-field changes inside one shared render-host window may remain indistinguishable and are documented as such.

### KeyboardPort and ModifierPort

Responsibilities:

- advertise Unicode/line-break/Tab/navigation/modifier capabilities;
- observe conflicting physical modifiers;
- accept bounded semantic actions;
- return complete, none, partial, or progress-unknown native results.

Adapters never release physical keys owned by the user. Partial/unknown native progress is terminal and never blindly retried.

### PastePort

Responsibilities:

- advertise ordinary Paste and revision-guard capabilities;
- verify the expected current-clipboard revision immediately before dispatch;
- send one balanced native Paste chord;
- return conservative native progress.

It never rewrites/restores clipboard contents.

### Command source

Command sources register validated Trigger/Cancel pairs with no-repeat behavior and deliver typed product commands only. They are not general keyboard-capture interfaces.

## Windows adapters

### Clipboard

`CF_UNICODETEXT` is copied from clipboard-owned global memory within configured byte limits. Sequence-number checks are content-blind. Clipboard contention maps to bounded retryable categories; malformed/non-text/empty/oversized data fails clearly.

### Keyboard and paste

`SendInput` is used for bounded Unicode/key events and one balanced `Ctrl+V`. Accepted native event counts are preserved. Zero accepted events are not automatically labelled UIPI unless independent integrity evidence proves a restricted relation.

Keyboard and Code work is paced per semantic action with bounded jitter and optional corrected typo actions. Cancellation, original-target comparison, and modifier checks occur at bounded action checkpoints.

### Destination and integrity

Foreground top-level window, process/thread identity, GUI-thread active/focus evidence, and integrity relation form the destination witness. Detailed original evidence that later weakens fails closed. A normal process does not inject into a known higher-integrity target.

### Hotkeys, tray, and startup

The Win32 command source owns `RegisterHotKey` registrations/message-loop lifetime. Candidate Trigger/Cancel pairs are validated/probed and replaced transactionally with rollback on failure; no `WH_KEYBOARD_LL` keylogger path is used.

A dedicated tray thread owns the hidden window, notification icon, menu, and message loop. Start-at-login uses one product-owned value under the current user's Run key.

## macOS Flutter/AppKit composition

The current macOS product line has one process and one Flutter engine. Swift/AppKit retains one `NSStatusItem`, one native menu, and one Settings window. Closing Settings hides it so native commands remain alive; Quit performs bounded Rust shutdown and removes native state.

The fixed Flutter boundary is:

```text
Flutter MethodChannel: io.cliptype/native
Flutter EventChannel:  io.cliptype/events
             │ bounded settings/commands/content-free events
Swift/AppKit shell -> cliptype-flutter-bridge static library
             │ fixed C ABI: settings, enums, counters, result categories
Rust Coordinator -> cliptype-core + cliptype-platform + cliptype-macos
```

The channel/C ABI do not carry clipboard text, injected text, focused values, window titles, recorded-key history, user identity, or content fingerprints. Flutter does not read the pasteboard or execute product input. Rust owns validation, session reservation, backend selection, target/modifier/revision safety, pacing, cancellation, and outcomes.

### Target and modifier evidence

The macOS target token contains only opaque process/window/focused-element identity plus content-free render-host classification. Bounded Accessibility role/attribute-name inspection may classify the initial web render host without reading titles, values, selections, DOM identifier values, class-list values, or document text.

Synthetic key event state is isolated from physical modifier evidence: physical conflicts are observed from the HID/system state rather than a state table polluted by ClipType's own generated flags. ClipType never releases the user's physical modifiers.

### Settings and shortcuts

Flutter presents task-oriented Overview, Input, Shortcuts, System, and About surfaces. Product-setting changes submit complete validated snapshots; invalid drafts remain local. Writes are serialized/coalesced through the application settings boundary.

Shortcut recorder values are local candidates until native validation/probing succeeds. Swift/AppKit owns system registrations; failure cleans temporary registrations and keeps/restores the previous pair. There is no general event tap, global key monitor, or unrelated key history.

### Observation model

Idle observation is event-driven. A bounded refresh timer may exist while a session is active, and a short-lived permission observation may follow explicit Accessibility onboarding. There is no permanent high-frequency application poll.

## Settings persistence

The versioned schema stores only product configuration. Parsing rejects unknown/duplicate/missing/malformed/unsupported values without echoing sensitive contents.

Saving uses an adjacent temporary file, durable flush, validated backup rotation, and replacement. A missing file loads safe defaults; a corrupt primary may recover from a valid backup. Settings never contain clipboard text or target data.

## Process and concurrency

The default product is one normal-integrity per-user process. Native message-loop/UI owners communicate with application services through typed signals/channels. Bounded input work runs off presentation/message-loop owners.

Poisoned synchronization primitives are recovered without formatting plaintext. Worker panics are caught at the session boundary and mapped to content-free internal-invariant outcomes. Shutdown requests cancellation, waits within a configured grace period, joins completed workers, and unregisters/removes native state.

No Windows service, driver, privileged helper, automatic elevation, or general daemon/client split exists.

## Error and outcome model

Preparation failures and terminal outcomes remain typed/content-free, including:

- disabled, busy, shutting down;
- unsupported/degraded capability;
- empty/non-text/malformed/oversized/unavailable/changed clipboard;
- revision changed/unavailable;
- target changed/disappeared/ambiguous/evidence unavailable;
- modifier conflict/settle timeout;
- known security restriction versus blocked cause unknown;
- complete, cancelled, partial input, progress unknown, native failure, or internal invariant.

UI/logging maps categories to fixed remediation text without clipboard text, window content/title, raw handles, or revision numbers.

## Release architecture

### Windows

The public release workflow:

- rebuilds from the exact `main` commit;
- reruns workspace check/test/Clippy;
- builds optimized versioned ZIP/EXE assets;
- embeds licenses, configuration, release notes, dependency inventory, and build metadata;
- scans distributable privacy boundaries;
- creates SHA-256 checksums;
- signs primary assets/manifests with Sigstore keyless GitHub OIDC identity;
- verifies signatures before publication;
- creates GitHub artifact attestations;
- creates a prerelease/tag only when the version is unused.

Authenticode trusted-publisher signing is a separate credential boundary.

### macOS Apple Silicon testing preview

For a new `release/VERSION`, P4 rebuilds the exact same `main` commit on an Apple Silicon runner, runs native/Swift/Rust/Flutter gates, verifies arm64-only/ad-hoc-signed bundle integrity, installs/launch-smokes `/Applications/ClipType.app`, and creates ZIP/DMG + metadata/checksum assets.

The attachment job waits until the Windows-created prerelease **and exact tag ref** are both visible and bound to the same `GITHUB_SHA`, refuses existing asset names, uploads additively, then re-downloads and byte/checksum-verifies the public files.

The current P4 artifact is not Intel/Rosetta/Universal 2, Developer ID signed, notarized, stapled, or a broad macOS compatibility claim. Physical Apple Silicon acceptance and any trusted distribution promotion remain #61.

## Invariants

1. Core policy is platform-independent.
2. Native adapters do not decide product policy.
3. Destination evidence is captured before clipboard acquisition and revalidated before dispatch.
4. Detailed evidence degradation fails closed within the documented platform policy.
5. Injection is explicit, one-session, bounded, and cancellable.
6. Physical modifiers are observed, never released.
7. Partial/unknown native input is never blindly retried.
8. Clipboard plaintext is ephemeral and absent from persistence/diagnostics/network transport.
9. Clipboard mode uses the already-current clipboard and never writes/clears/restores it.
10. Active plans/settings snapshots are immutable.
11. Privilege is not escalated or bypassed.
12. Compatibility wording cannot exceed evidence.
13. Published assets are versioned, checksummed, provenance-bound, and never silently replaced.
14. Cross-cutting changes require the ADR/document process.
15. The Flutter/native boundary remains fixed, bounded, and content-free.
