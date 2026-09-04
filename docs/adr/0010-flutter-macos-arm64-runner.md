# ADR-0010: Flutter macOS arm64 runner over the Rust application core

- Status: Accepted
- Date: 2026-09-04
- Scope: P4 local macOS Apple Silicon runner
- Supersedes: ADR-0009 for the macOS composition root only
- Related: #63, #64
- Subsequent decision: ADR-0012 removes the legacy macOS Slint composition root.

## Context

The P4 macOS work starts from the existing Rust core, coordinator, and macOS
adapters, but the Flutter target was still the stock counter scaffold. The
local deliverable needs a real settings surface and a runnable menu-bar
application on an Apple Silicon Mac without moving product policy into Dart or
creating a second long-lived native process.

The public P3 plan still describes a Universal 2 and signed/notarized release.
This decision is narrower: it defines an arm64-only local candidate. It does
not expand the public compatibility or release promise.

## Decision

### Runtime ownership

The macOS candidate is one process with one Flutter engine, one AppKit status
item, one settings window, one global-hotkey owner, and at most one active
injection session.

Flutter owns presentation and user interaction only. Swift/AppKit owns the
application shell, Flutter channels, status item/menu, global shortcut
registration, Accessibility onboarding/state, login-item integration, window
lifecycle, and the C ABI boundary. Rust owns settings validation and
persistence, coordinator/session policy, clipboard and revision evidence,
backend selection, timing, cancellation, target/focus/modifier safety, and
content-free outcomes.

### Channel and ABI contract

The fixed channels are:

- `io.cliptype/native` — bounded commands and content-free state;
- `io.cliptype/events` — fixed event categories only.

The presentation layer also exposes an English/Simplified Chinese display
language. This non-sensitive UI preference is mirrored between the Flutter
window and the native status menu; it is kept separate from Rust product
settings and injection policy.

The native side exports a small `#[repr(C)]` integer/status ABI through the
`cliptype-flutter-bridge` static library. It exchanges settings, bounded enum
codes, counters, and opaque runtime ownership only. Clipboard text, injected
text, focused values, window titles, key history, user identity, and content
fingerprints never cross into Dart or the bridge status structures. Panics are
caught before crossing the FFI boundary, and the Swift side destroys the
runtime only during application shutdown.

### Native mechanisms

The candidate reuses `cliptype-app`, `cliptype-core`, and `cliptype-macos`.
Swift registers Trigger and Cancel through the system Carbon hot-key API and
keeps the registration transactional: validate, probe both new shortcuts,
commit the pair, release the old pair, then persist the Rust settings. Failure
removes the new registrations and restores the previous pair.

Accessibility is queried honestly and requested only after an explicit user
action. The application does not install an event tap, a global key logger, or
a broad keyboard monitor. Observation timers are bounded to an active session
or to the short permission-request observation window; there is no permanent
idle 40 ms poller.

The local candidate is non-sandboxed because the documented user-controlled
macOS Application Support configuration, menu-bar lifecycle, pasteboard access,
and Accessibility-mediated cross-application input must remain available to
the normal per-user app. It does not add privileged entitlements or bypass
macOS consent. This is a local packaging choice, not a distribution approval.

### Architecture target

```text
Flutter settings UI
        │ fixed MethodChannel/EventChannel
Swift/AppKit shell and adapters
        │ fixed content-free C ABI
Rust bridge → cliptype-app → cliptype-core/platform/macos
```

The legacy Rust/Slint `apps/cliptype` composition root remains the Windows
path. The former Rust/Slint macOS composition root has since been removed by
ADR-0012; this ADR's Flutter/native-shell ownership remains the current P4
macOS architecture.

## Alternatives considered

### Put session and clipboard policy in Dart

Rejected because it would duplicate or weaken the Rust coordinator's safety
invariants and could expose sensitive data through the UI/runtime boundary.

### Keep the stock Flutter scaffold and validate only startup

Rejected because a counter page is not evidence that ClipType's settings,
status item, hotkeys, permissions, or Rust runtime are integrated.

### Use a global event tap or broad native keyboard monitor

Rejected because the product requires system-registered commands and a
focused, local recorder, not arbitrary key capture or keyboard history.

### Build a Universal 2 or signed/notarized artifact in this local track

Rejected as out of scope for an arm64-only local run. Universal 2 assembly,
Developer ID signing, notarization, and Gatekeeper evidence remain release
gates in the broader P3/public path.

## Consequences

### Positive

- The real Flutter settings UI can be exercised on the target Apple Silicon
  host while Rust remains the policy owner.
- Native macOS responsibilities have one owner each, reducing duplicate
  status items, hotkeys, and shutdown paths.
- The ABI and channel surface are small, auditable, and content-free.
- The local release candidate can be architecture-scanned independently.

### Negative / trade-offs

- The candidate is arm64-only and unsigned locally; it is not a public beta.
- CUA/UI smoke and deterministic tests cannot substitute for physical
  permission, target-application, Chinese input, conflict, and latency evidence.
- The Flutter engine adds a larger resident footprint than a native-only
  settings window.
- Slint remains a Windows presentation dependency, while Flutter is the sole
  macOS settings/front-end composition root.

## Follow-up

1. Keep the local evidence report explicit about PASS, FAIL, BLOCKED, and NOT
   RUN rather than upgrading the compatibility claim.
2. Add controlled TextEdit/Chrome/VS Code and permission-revocation evidence
   before any macOS beta wording.
3. Revisit the public macOS release architecture only with a new exact-SHA,
   Universal 2, signing, notarization, and physical-client gate.
