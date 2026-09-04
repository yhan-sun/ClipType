# P2-B02 Human-Paced Keyboard Contract

This note records the release-blocking keyboard contract implemented for issue #41.

## Scope

The keyboard backend turns normalized clipboard text into a bounded sequence of semantic typing actions. Each action is dispatched separately and is paced independently. Code mode uses the same keyboard boundary with code-aware indentation and pair actions; Clipboard-paste mode is unchanged and never uses humanized typo simulation.

## Required invariants

Before every action, including a simulated wrong key and its corrective Backspace, the coordinator must:

1. observe cancellation;
2. revalidate the original target;
3. verify that relevant physical modifiers are clear;
4. stop without retry after partial or unknown native progress.

The coordinator never refocuses an old target and never adopts a new target to finish a session.

## Timing

`characters_per_second` defines the base pacing slot. `jitter_percent` applies an independently sampled bounded variation to every slot. Correct characters, simulated wrong characters, Backspace corrections, line breaks, and Tab actions all use the same cancellation-aware pacing path.

Active sessions retain an immutable settings and random-state snapshot. Changes affect later sessions only.

## Typo simulation

Typo simulation is explicit opt-in and defaults to zero probability. Eligible ASCII keys may be replaced temporarily by a documented adjacent US-QWERTY key, followed by Backspace and the intended key. CJK, emoji, combining marks, line breaks, Tab, and unsupported controls are never mutated into typos.

This feature is not suitable for passwords, source code, terminals, shell commands, administrative tools, or exact-data entry. Code mode always disables corrected typo simulation.

## Unicode

Already-composed clipboard text remains Unicode-oriented and does not depend on an active IME or keyboard layout. CJK, Japanese, Korean, emoji, and combining marks are delivered as their original Unicode scalar actions. Application-specific rejection of Unicode packets remains an explicit compatibility limitation rather than a hidden backend fallback.
