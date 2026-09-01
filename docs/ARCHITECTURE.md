# Architecture

## Architectural style

ClipType uses **ports and adapters (hexagonal architecture)** around a small Rust application/core layer.

The design separates:

- **policy**: when and how to inject;
- **capabilities/evidence**: what the current platform can prove and perform;
- **mechanism**: native clipboard, hotkey, focus, and input APIs;
- **runtime orchestration**: one-session lifecycle, cancellation, worker ownership;
- **presentation**: tray/settings/onboarding or a minimal development host.

## Logical system

```text
                  +-----------------------+
                  | Native UI / Host      |
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
      | Clipboard   |  | Target/Focus |  | Capability  |
      | Port        |  | Port         |  | Provider    |
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
              | Session /      |
              | Injection Loop |
              +---+---------+--+
                  |         |
          +-------v--+   +--v-------------+
          | Keyboard |   | ClipboardPaste |
          | Port     |   | Mechanism      |
          +----------+   +----------------+
```

Platform adapters implement the ports:

```text
Windows: Win32 clipboard / SendInput / hotkey / foreground and GUI-thread evidence
macOS:   NSPasteboard / CGEvent / Accessibility / app focus APIs
X11:     selections / XFixes / XTest / X11 focus
Wayland: capability-dependent data-control/portal + virtual keyboard/uinput
```

## Repository layout

The target multi-platform layout is:

```text
ClipType/
  crates/
    cliptype-core/         # domain values, normalization, planner, pure policy
    cliptype-platform/     # native-neutral ports/capability/evidence types
    cliptype-app/          # application use cases and live session coordination
    cliptype-windows/      # Win32 adapter
    cliptype-macos/        # macOS adapter, only in its roadmap phase
    cliptype-x11/          # X11 adapter, only in its roadmap phase
    cliptype-wayland/      # Wayland capability/adapters, only in its roadmap phase
    cliptype-ui/           # shell contracts only if later evidence justifies it
  apps/
    cliptype/              # normal user process / platform composition root
  helpers/
    cliptype-uinput/       # optional Linux helper only if required and approved
  docs/
  tests/
```

This is a target boundary, not permission to pre-create empty crates before their roadmap phase.

### P1 dependency graph

P1 creates only `cliptype-core`, `cliptype-platform`, `cliptype-app`, `cliptype-windows`, and `apps/cliptype`.

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

- `cliptype-platform` depends on core domain types.
- `cliptype-app` depends on core policy and native-neutral ports.
- `cliptype-windows` depends on core/native-neutral contracts, not on app orchestration.
- the executable is the composition root that depends on app plus the Windows adapter.
- `cliptype-core` MUST remain free of platform APIs.

## Core ports

Exact Rust signatures are designed during P1 after the Windows native-mechanism spike. The semantic contracts below are fixed unless changed through the repository decision process.

### ClipboardPort

Responsibilities:

- obtain current text and metadata needed for the active operation;
- later, write temporary text and restore a prior clipboard only when clipboard-paste mode enters scope;
- expose capability and typed error information.

It MUST NOT implement clipboard history. P1 uses current-text read only and does not require a clipboard listener.

### HotkeyPort / command event source

Responsibilities:

- register/unregister explicit global commands;
- report conflicts and unsupported combinations;
- deliver typed trigger/cancel/shutdown events to the application layer;
- preserve platform thread/message-loop requirements without invoking policy directly.

It MUST NOT become a general key capture interface.

### FocusPort / TargetEvidencePort

Responsibilities:

- capture the strongest practical non-content destination evidence;
- compare/re-capture evidence to detect meaningful changes;
- report safe target metadata such as process/application identity;
- surface unknown/degraded evidence honestly.

It MUST NOT read focused text. The contract describes evidence, not a universal guarantee of exact logical-field identity.

### KeyboardInjectionPort

Responsibilities:

- accept bounded semantic text batches rather than leaking native event arrays into policy;
- emit native synthetic text/key input;
- expose supported Unicode/control/modifier semantics;
- return typed complete/none/partial/unknown results;
- avoid retry policy, which belongs to application/core and is conservative.

### ModifierStatePort or equivalent capability

The Windows P1 design may expose modifier safety through the keyboard adapter or a separate native-neutral contract. It must support a bounded pre-dispatch safety check without releasing physical user keys.

### CapabilityProvider

Responsibilities:

- detect available APIs, permissions, protocols, integrity/security relation, and degraded states;
- distinguish known facts from unknown evidence;
- drive planner eligibility and user explanations.

## Application services

### TriggerInjection

The destination intended by the trigger is captured before potentially contended clipboard work.

Conceptual sequence:

1. atomically reserve the single session slot or return busy;
2. capture initial target evidence immediately;
3. create cancellation/status state and start the bounded worker;
4. wait for trigger modifiers to settle within a bound;
5. acquire current clipboard text;
6. obtain capabilities/security evidence and create an immutable plan;
7. revalidate target evidence;
8. dispatch bounded batches with cancellation/focus/modifier checks;
9. publish a content-free result and release the session slot.

### CancelInjection

Signals the active cancellation token with bounded delivery latency. It does not wait for the entire payload to finish and does not destroy cleanup state.

### UpdateConfiguration

Validates and applies supported configuration without weakening mandatory security invariants. P1 may expose only an in-memory/development configuration surface.

### QueryStatus

Provides non-sensitive status: idle, ready, preparing, injecting, cancelling, busy, completed, target-changed, modifier-conflict, blocked, permission-required, unsupported, partial, or native failure.

## Pure policy versus runtime coordination

The project intentionally separates:

- **pure core policy**: text validation/normalization, plan construction, state-transition decisions, retry rules;
- **application runtime**: session slot, channels/worker, port calls, cancellation token, focus checks, terminal cleanup;
- **platform mechanism**: Win32 or other native calls.

Do not implement two competing state machines in core and app. Core provides deterministic transition/policy functions; app owns the live execution instance.

## Injection session model

Every attempt gets an in-memory session ID and immutable initial context:

- trigger/start timestamp;
- initial target evidence;
- clipboard snapshot metadata and ephemeral text;
- configuration snapshot;
- capability/security-evidence snapshot;
- selected plan.

The active plan does not silently switch backend mid-session. Clipboard plaintext exists only in memory required for the action and MUST NOT be added to diagnostics.

## Concurrency model

V1 permits at most one active injection session. The session slot must be reserved atomically before a worker is created. A second trigger returns busy; it is not queued.

Cancellation is cooperative but checked between bounded native dispatch batches. Focus and conflicting modifier evidence are also checked at bounded points.

Platform event loops run on platform-required threads and communicate through narrow commands/events. The Windows hotkey/message-loop owner must remain responsive while the injection worker runs. Do not add a full async runtime merely for architectural symmetry; use the smallest concurrency mechanism proven sufficient by platform evidence.

## Process model

Default architecture: **one unprivileged user process** containing core, application services, platform adapter, and shell/host.

An additional process/helper is allowed only when an OS capability requires a privilege or lifecycle boundary. The expected first case is Linux `/dev/uinput`. Such a helper:

- MUST have a minimal, versioned local protocol;
- MUST NOT receive configuration unrelated to input emission;
- MUST NOT store clipboard history;
- SHOULD receive only the minimal current operation plan/events;
- MUST authenticate/limit local callers appropriately for the platform.

Do not introduce a general daemon/client split merely for symmetry.

## Error and outcome model

Errors/outcomes are typed for UI and diagnostics:

- busy;
- unsupported/degraded capability;
- permission required/denied;
- known security-boundary restriction;
- blocked/native cause unknown;
- target changed/disappeared/evidence unavailable;
- modifier conflict or settle timeout;
- clipboard unavailable/busy;
- empty/non-text/malformed clipboard;
- injection complete;
- injection partially completed/progress unknown;
- cancellation;
- temporary backend failure;
- internal invariant failure.

A zero native dispatch result must not automatically be labelled UIPI when the OS evidence cannot prove that cause. Do not collapse outcomes into a generic `failed` message.

## Observability

Allowed diagnostics include backend, platform, safe target application identifier, payload length/count bucket, duration, result category, cancellation/focus flags, and capability state.

Forbidden diagnostics include clipboard/injected plaintext, focused-field contents, raw key capture, window titles by default, secrets, or persistent content fingerprints.

## Architecture invariants

1. Core policy is platform-independent.
2. Platform adapters do not decide product policy.
3. Presentation does not directly implement injection policy.
4. Destination evidence is captured at explicit trigger time before contended preparation.
5. Injection is explicit, one-session, bounded, and cancellable.
6. Focus safety uses the strongest available evidence and reports its limitations.
7. Physical user modifiers are not released by ClipType.
8. Partial/unknown synthetic input is never blindly retried.
9. Clipboard plaintext is ephemeral.
10. Wayland support is capability-driven, not session-name-driven.
11. Privilege is isolated and minimized.
12. Architecture changes require ADRs when defined by repository policy.
