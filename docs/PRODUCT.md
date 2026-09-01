# Product Specification

## Product statement

**ClipType lets a user type the current clipboard text into the currently focused application using OS-native input facilities.**

Tagline: **Type your clipboard anywhere.**

ClipType is not primarily a clipboard manager. It is a local text-delivery tool for situations where normal paste is inconvenient, blocked, unreliable, or semantically different from typing.

## Primary use cases

- Enter clipboard text into applications that reject or mishandle paste.
- Type into remote desktops, virtual machines, terminal-like surfaces, forms, IDEs, and other focused controls.
- Deliver Unicode and multiline clipboard content with a predictable, cancellable mechanism.
- Choose between simulated typing and normal clipboard paste depending on target and payload.

## Core UX contract

1. The user copies text using normal OS/application behavior.
2. The user focuses the destination control.
3. The user invokes ClipType through a global hotkey or explicit tray action.
4. ClipType reads the current clipboard snapshot.
5. ClipType captures target/focus metadata and builds an injection plan.
6. ClipType injects the content using the selected backend.
7. The user can cancel immediately.
8. If the target changes in a way that violates the plan's focus policy, ClipType aborts rather than continuing into an unintended application.

ClipType MUST NOT require locating a visual caret on screen. The OS focus/input-routing model is the target mechanism.

## Default safety posture

- Manual invocation is the default.
- Clipboard-change-triggered auto-typing is not a V1 feature.
- Empty/non-text clipboard content is a no-op with explicit user feedback.
- Injection is cancellable.
- Focus changes are guarded.
- Clipboard content is not persisted by default.
- The application does not transmit clipboard text.

## V1 feature set

- Current plain-text clipboard read.
- Global trigger.
- Keyboard injection mode.
- Clipboard-paste mode.
- Auto planner that selects an appropriate mode.
- Configurable keyboard delay/throughput bounds.
- Unicode and multiline support within each backend's documented capabilities.
- Cancellation.
- Focus guard.
- Tray/status shell and permission onboarding.
- Clear error/reporting for unsupported permissions/capabilities.

## Explicit V1 non-goals

- Clipboard history.
- Background capture of every copied item.
- Cloud sync or accounts.
- AI text rewriting.
- OCR/image/rich-media typing.
- General-purpose macros or arbitrary scripting.
- Hidden automation designed to evade application controls.
- Security-boundary bypass.
- Mobile platforms.

## Product principles

### User control over automation
Synthetic input can be dangerous. Every default favors an explicit trigger, visible state, immediate cancellation, and fail-safe behavior.

### Capability honesty
The UI and documentation MUST distinguish supported, degraded, experimental, and unavailable behavior. This is especially important on Wayland.

### Native integration
Use documented OS facilities and platform permissions instead of browser automation or accessibility hacks when a direct API exists.

### Privacy by architecture
Clipboard content should spend as little time as practical in memory, should not enter logs, and should not cross the network.

### Small surface area
ClipType should remain a focused utility rather than growing into a general desktop automation framework.

## Success metrics

V1 success is measured primarily by correctness, safety, compatibility, and startup/runtime footprint—not feature count.

Engineering metrics should include:
- injection completion/failure reason counts without content;
- cancellation latency;
- focus-change abort correctness;
- clipboard restoration success for paste mode;
- Unicode/multiline compatibility matrix;
- CPU/memory idle footprint;
- platform permission/setup failure rate measured locally unless privacy-preserving telemetry is later explicitly designed.

No metric may require recording clipboard plaintext.