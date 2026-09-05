# ADR-0022: Task-oriented macOS settings UX

- Status: Accepted
- Date: 2026-09-05
- Scope: Flutter macOS settings information architecture and content-free presentation

## Context

The P4 Flutter front end exposed the correct settings and safety boundaries, but
its navigation mirrored implementation modules: General, Shortcuts, Typing,
Permissions, and About. Readiness was scattered across several pages; Trigger
could be offered before permission or shortcut prerequisites were satisfied;
build identity was hard-coded; and Code-mode corrected-typo support introduced
in beta.6 was not represented by the UI.

The maintainer explicitly requested a complete UI/UX correction while keeping
Flutter as the sole macOS front end and preserving the Rust/Swift safety and
privacy boundaries.

## Decision

The macOS Flutter settings window uses task-oriented presentation:

- **Overview** owns readiness, the next actionable setup step, current mode,
  shortcut summary, and the latest content-free session outcome.
- **Input** owns mode selection and reveals only controls relevant to the
  selected mode. Code mode exposes the safe corrected-typo setting supported by
  the Rust coordinator.
- **Shortcuts** treats recorder values as candidates. A complete pair is
  validated and applied through the existing transactional native replacement;
  a failed candidate never unregisters the previous working pair.
- **System** groups enabled state, notifications, login-item state, interface
  language, and explicit Accessibility onboarding/remediation.
- **About** obtains version/build/channel/signing metadata from the running app
  bundle through a fixed content-free native method instead of hard-coding a
  release string.

The shell derives a content-free readiness state from existing product settings,
permission, native-runtime availability, session phase, and whether the native
hot-key pair is registered. The settings-window Trigger action explains its
focus handoff once per app run before hiding the window. Permission failure does
not automatically open System Settings; the user chooses the remediation action.

The menu-bar shell mirrors the same content-free states and enables Start/Stop
commands only when they are meaningful. A login item awaiting macOS approval is
shown as pending rather than enabled.

This change does not move injection policy into Flutter, add clipboard history,
read destination text, add telemetry/networking, or expose clipboard/input
plaintext through Flutter or Swift channels.

## Alternatives considered

### Keep implementation-oriented pages and only restyle them

Rejected because visual polish would not solve the readiness, prerequisite,
stale-version, or capability-drift problems.

### Add a separate onboarding wizard

Rejected for this beta because Overview can serve both first-run setup and
ongoing status without adding a second state machine or persistence model.

### Let Flutter decide backend or target policy

Rejected. Rust remains the policy authority; Flutter only presents settings and
content-free runtime categories.

## Consequences

### Positive

- The first screen tells the user whether ClipType is usable and what to do next.
- UI capability follows the beta.6 Code-mode corrected-typo contract.
- Shortcut conflicts preserve the previous working pair and are visible at the
  point of editing.
- Build/version information reflects the installed bundle.
- Menu-bar and window states use the same user-facing readiness concepts.

### Negative / trade-offs

- The Flutter shell has more presentation state and responsive behavior.
- New build metadata keys must remain present in the macOS bundle.
- Hosted CI can verify the UI/build contract but still cannot grant persistent
  Accessibility consent or replace physical real-editor evidence.

## Follow-up

- Keep widget tests for Code-mode control availability, temporary numeric edits,
  runtime build information, and Overview readiness.
- Keep physical Accessibility and real VS Code/Monaco evidence separate from
  hosted UI/build gates.
