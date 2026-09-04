# ADR-0018: macOS render hosts use stable window target evidence

- Status: Accepted
- Date: 2026-09-05
- Scope: macOS target capture and focus-change protection
- Related: ADR-0002, ADR-0004, ADR-0010, ADR-0017

## Context

The macOS adapter captured the frontmost process and the `CFHash` of the
focused Accessibility element before injection, then required that element
hash to remain identical before every action. That is strong evidence for
stable native controls.

Web and Electron editors expose text through an `AXWebArea` render host. Monaco
may replace its focused Accessibility text node while the caret remains in the
same editor, including during ordinary typing and Command+Right navigation.
The element hash therefore changed even though the user had not changed the
target. ClipType reported `TargetChanged` and safely stopped, often after a
closing brace and occasionally in the middle of a word.

Disabling target checks or reading editor content would violate the product's
safety and privacy boundaries.

## Decision

The macOS target adapter captures only content-free identity evidence:

- frontmost process identifier;
- focused top-level Accessibility window hash when available;
- focused Accessibility element hash when available; and
- whether a bounded parent-role walk finds an `AXWebArea` ancestor or the
  focused element exposes web-only Accessibility attribute names.

It never requests a window title, focused value, selected text, document text,
or developer-provided content identifier.

For a focused element under `AXWebArea`, evidence is classified as
`RenderHostLimited`. Chromium-derived editors do not always expose a traversable
`AXParent` chain for a transient focused node, so the adapter also recognizes a
render-host element when its supported Accessibility attribute names include
`AXDOMIdentifier` or `AXDOMClassList`. It does not read either attribute value.
During one session, a rebuilt focused element remains the same target only when
both captures remain render-host-limited and the process and focused-window
hashes are unchanged. A process or window change is still `Changed`; loss or
disagreement in evidence is `UnavailableOrAmbiguous` and stops safely.

For native controls outside an `AXWebArea`, the existing exact focused-element
comparison remains required. The parent walk is bounded to 32 elements, and
only attribute names—not identifier/class-list values—are inspected.

## Alternatives considered

### Compare only the process for every macOS target

Rejected because moving between windows or native fields in one application
would no longer be detected.

### Keep exact element identity for render hosts

Rejected because observed Accessibility node replacement makes long editor
sessions nondeterministically stop despite an unchanged destination.

### Read editor text, selection, or caret content

Rejected because it expands plaintext access and violates the content-free
target boundary.

### Maintain an application-specific editor whitelist

Rejected because Accessibility structure provides a platform-level capability
signal and avoids coupling safety policy to bundle identifiers.

## Consequences

### Positive

- VS Code/Monaco can rebuild focused AX nodes without aborting an unchanged
  editor session;
- switching applications or top-level windows still stops immediately;
- native controls retain exact focused-element protection;
- no destination content or titles are read, logged, persisted, or bridged.

### Negative / trade-offs

- within one `AXWebArea` and one top-level window, switching between logical
  fields may be indistinguishable; this remains an explicit render-host
  limitation;
- a host that does not expose `AXWebArea` but still rebuilds focus nodes will
  continue to fail safe until separately characterized.

## Follow-up

Keep regression tests for rebuilt render-host focus, changed windows, changed
processes, and exact native-control focus changes. Validate representative
Electron editors on physical macOS.
