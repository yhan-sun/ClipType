# Architecture

## Architectural style

ClipType uses **ports and adapters (hexagonal architecture)** around a small Rust application/core layer.

The design separates:

- **policy**: when and how to inject;
- **capabilities**: what the current platform/backend can do;
- **mechanism**: native clipboard, hotkey, focus, and input APIs;
- **presentation**: tray/settings/onboarding.

## Logical system

```text
                  +-----------------------+
                  |   Native UI / Tray    |
                  +-----------+-----------+
                              |
                     Commands / Status
                              |
                  +-----------v-----------+
                  |  Application Service  |
                  | trigger, cancel, cfg  |
                  +-----------+-----------+
                              |
             +----------------+----------------+
             |                |                |
      +------v------+  +------v------+  +------v------+
      | Clipboard   |  | Focus/Target |  | Capability  |
      | Port        |  | Port         |  | Registry    |
      +------+------+  +------+------+  +------+------+
             |                |                |
             +--------+-------+----------------+
                      |
              +-------v--------+
              | Injection      |
              | Planner        |
              +-------+--------+
                      |
              +-------v--------+
              | Injection      |
              | Engine         |
              +---+---------+--+
                  |         |
          +-------v--+   +--v-------------+
          | Keyboard |   | ClipboardPaste |
          | Port     |   | Mechanism      |
          +----------+   +----------------+
```

Platform adapters implement the ports:

```text
Windows: Win32 clipboard / SendInput / hotkey / foreground window
macOS:   NSPasteboard / CGEvent / Accessibility / app focus APIs
X11:     selections / XFixes / XTest / X11 focus
Wayland: capability-dependent data-control/portal + virtual keyboard/uinput
```

## Future repository layout

No code exists yet. The intended structure is:

```text
ClipType/
  crates/
    cliptype-core/         # domain types, planner, state machine, policy
    cliptype-app/          # application orchestration/use cases
    cliptype-platform/     # common adapter traits/capability types
    cliptype-windows/      # Win32 adapter
    cliptype-macos/        # macOS adapter
    cliptype-x11/          # X11 adapter
    cliptype-wayland/      # Wayland capability/adapters
    cliptype-ui/           # shared shell contracts only where useful
  apps/
    cliptype/              # normal user process / tray entry point
  helpers/
    cliptype-uinput/       # optional Linux helper only if required
  docs/
  tests/
```

This is a target boundary, not permission to pre-create empty crates before their roadmap phase.

## Core ports

Exact Rust signatures will be designed during P1, but the semantic contracts are fixed.

### ClipboardPort
Responsibilities:
- obtain current text and metadata needed for safe restore/suppression;
- write temporary text for paste-mode injection;
- restore the prior clipboard when restoration is enabled;
- expose capability/error information.

It MUST NOT implement clipboard history.

### HotkeyPort
Responsibilities:
- register/unregister the global trigger;
- report conflicts and unsupported combinations;
- deliver trigger events to the application layer.

### FocusPort
Responsibilities:
- identify the current target sufficiently to detect meaningful target changes;
- report target metadata that is safe to log (process/app identifier, not focused text);
- support focus-guard checks where the platform permits.

### KeyboardInjectionPort
Responsibilities:
- emit native synthetic keyboard/text input;
- expose supported semantics (Unicode, physical-key events, modifiers, etc.);
- return typed failure categories rather than boolean success.

### CapabilityProvider
Responsibilities:
- detect available platform protocols/APIs/permissions at runtime;
- report degraded/experimental capabilities;
- drive planner eligibility and UI explanations.

## Application services

### TriggerInjection
Reads a clipboard snapshot, captures target identity, obtains runtime capabilities, creates an `InjectionPlan`, and starts the engine.

### CancelInjection
Transitions the active session toward cancellation with bounded latency.

### UpdateConfiguration
Validates and applies supported configuration without weakening mandatory security invariants.

### QueryStatus
Provides non-sensitive status for UI/tray: idle, ready, injecting, cancelling, blocked, permission-required, unsupported.

## Injection session model

Every attempt gets an in-memory session ID and immutable initial context:

- start timestamp;
- clipboard snapshot metadata;
- target identity;
- configuration snapshot;
- capability snapshot;
- selected plan.

Clipboard plaintext exists only in the session memory required to perform the action and MUST NOT be added to diagnostics.

## Concurrency model

V1 permits at most one active injection session. A second trigger while injecting follows an explicit policy (default: reject with busy status; future queueing requires design review).

Cancellation is cooperative but MUST be checked between bounded chunks/events so a large payload cannot make cancellation unresponsive.

Platform event loops may run on platform-required threads; they communicate with the application layer through typed events/channels rather than invoking policy directly.

## Process model

Default architecture: **one unprivileged user process** containing core, platform adapter, and tray/settings shell.

An additional process/helper is allowed only when an OS capability requires a privilege or lifecycle boundary. The expected first case is Linux `/dev/uinput`. Such a helper:

- MUST have a minimal, versioned local protocol;
- MUST NOT receive configuration unrelated to input emission;
- MUST NOT store clipboard history;
- SHOULD receive only the minimal text/key plan needed for the current operation, or preferably already-translated events when practical;
- MUST authenticate/limit local callers appropriately for the platform.

Do not introduce a general daemon/client split merely for architectural symmetry.

## Error model

Errors are typed into categories suitable for UI and diagnostics:

- unsupported capability;
- permission denied / permission required;
- target changed;
- clipboard unavailable/busy;
- empty/non-text clipboard;
- injection partially completed;
- cancellation;
- temporary backend failure;
- security-boundary restriction;
- internal invariant failure.

Do not collapse these into generic `failed` messages.

## Observability

Allowed diagnostics include backend, platform, target application identifier where non-sensitive, payload length bucket, duration, result category, cancellation, and capability state.

Forbidden diagnostics include clipboard/injected plaintext, focused-field contents, raw key capture, secrets, or persistent content fingerprints.

## Architecture invariants

1. Core policy is platform-independent.
2. Platform adapters do not decide product policy.
3. UI does not directly call OS injection APIs.
4. Injection is explicit and cancellable.
5. Focus safety is checked by the engine/application layer using platform evidence.
6. Clipboard plaintext is ephemeral.
7. Wayland support is capability-driven, not session-name-driven.
8. Privilege is isolated and minimized.
9. Architecture changes require ADRs.