# ADR-0021: Isolate macOS synthetic input from physical modifier evidence

- Status: Accepted
- Date: 2026-09-05
- Base: `683f7ab54f21d0881f2ef1c01ce7edd920a8eaa8`
- Related: ADR-0004, ADR-0010, ADR-0012, ADR-0017, ADR-0019, ADR-0020

## Context

Two user reports stop near the first line-leading Code-mode closer. This is
where the macOS adapter first emits Right followed by Command+Right. The
baseline observes `kCGEventSourceStateCombinedSessionState`, creates navigation
events using a default source, leaves some event flags implicit, and ends the
chord's Right key-up with Command flags still set. Ordinary Unicode events also
use a combined-session source without explicit flag clearing.

These are unsafe dependencies on synthetic session state. They make a
self-induced modifier conflict a plausible explanation for the reports; a
physical reproduction with an exact terminal outcome is still needed. Source
inspection and a mock Quartz test must not be described as that reproduction.

## Decision

1. Observe hardware modifier evidence with
   `CGEventSourceFlagsState(kCGEventSourceStateHIDSystemState)`.
2. Create each bounded keyboard action with a private event-source state table.
3. Set flags explicitly on every event. Plain text/control events use zero;
   Command+Right/Command+V use Command on the action's key-down and zero on its
   key-up. Do not synthesize a key-up for a physical Command, Shift, Option, or
   Control key.
4. Allocate the entire sequence before posting. Failure releases every
   allocated event/source and posts nothing. Never retry a partial dispatch.
5. Preserve ADR-0017's Right -> Command+Right navigation, event counts, FIFO,
   delay policy, and the coordinator's target/cancellation/modifier checks.
6. Add distinct completion codes 7..15 to the existing bridge enum without
   changing codes 0..6 or the CTBridgeState layout. Map all terminal outcomes
   exhaustively and show fixed English/Chinese remediation text in Flutter.

The keyboard helper is shared by macOS Keyboard and Paste, so their event-source
construction also changes. Code never invokes Paste. No clipboard storage,
clipboard writes, target text reads, global event tap, or physical key release
is introduced. Windows adapters and the Code planner are unchanged.

## Alternatives

- Adding another delay does not establish correct event state; rejected.
- Ignoring Command conflicts or releasing every modifier weakens user safety;
  rejected.
- Replacing navigation with Paste violates Code-mode scope; rejected.
- Keeping combined-session observation but filtering self-generated flags would
  require separately proven event ownership and timing; deferred.

## Consequences and evidence boundary

Private sources and hardware observation separate intended input from physical
modifier evidence, but actual Quartz/HID behavior, Accessibility, keyboard
remappers, remote input, and editor delivery must be verified on macOS. The
hardware table is not a promise to detect every third-party synthetic modifier.
The product remains fail-closed on observed physical modifiers and target
changes. CGEventPost does not prove target-editor text acceptance.

Portable tests compile the production input functions against a mock Quartz API
and compile the production Swift snapshot mapping against the real bridge
header. They establish event construction, resource cleanup and ABI mapping;
they do not establish real keyboard-state behavior or editor compatibility.

## Follow-up

Run pinned Rust/Flutter gates and continue collecting the physical matrix in
`../testing/CODE_NAVIGATION_FIX.md`. Do not claim all editor compatibility or
close the user report solely from the portable tests. The grouped-closer and
language/indentation limitations are separate and intentionally not rewritten
by this focused native-input patch.
