# ADR-0006: Guarded Current-Clipboard Paste Without Temporary Rewrites

- Status: Accepted
- Date: 2026-09-02
- Supersedes: the temporary-write/restoration mechanism in ADR-0003; the three public injection modes remain unchanged

## Context

ADR-0003 established explicit `keyboard`, `clipboard`, and `auto` modes and assumed that clipboard mode would temporarily replace the clipboard, paste, and conditionally restore the previous value.

The P1 implementation and P2 design review exposed an important simplification: ClipType's source payload is already the current system clipboard. Replacing that same clipboard with another copy of the text creates extra ownership, delayed-rendering, format-preservation, acknowledgement, and restore races without creating a new source of truth.

A temporary write is especially risky when the clipboard also carries HTML, RTF, images, application-private formats, delayed-rendered data, or a newer value written by another actor. Preserving every arbitrary format safely requires a much broader clipboard-manager/data-object surface than the focused V1 product needs.

## Decision

P2 clipboard mode performs a guarded **normal paste of the current clipboard** and does not rewrite, clear, snapshot for restoration, or restore clipboard data.

The operation is:

1. atomically reserve the one-session slot;
2. capture destination evidence before clipboard access;
3. wait for trigger modifiers to settle without releasing physical keys;
4. perform one bounded `CF_UNICODETEXT` read for validation, payload bounds, and user feedback;
5. capture non-content clipboard revision evidence with that read where the platform exposes it;
6. freeze the selected backend for the session;
7. revalidate cancellation, destination, integrity, modifiers, and clipboard revision immediately before dispatch;
8. submit one bounded native paste chord;
9. inspect native acceptance and clipboard revision again;
10. stop on partial, unknown, target, modifier, integrity, or clipboard-change uncertainty and never retry the chord automatically.

Because ClipType never becomes clipboard owner in this path:

- no restoration operation exists;
- no newer external clipboard value can be overwritten by ClipType;
- no self-generated clipboard update event needs suppression;
- arbitrary non-text/rich formats remain owned by their original producer;
- clipboard plaintext is not copied into persistent storage.

## Semantic consequence

Clipboard mode intentionally follows the destination application's normal paste semantics. If the current clipboard contains both plain text and richer formats, a target may choose a richer format exactly as it would for the user's own paste command.

Keyboard mode remains the backend for validated plain-text semantic delivery. The UI and compatibility documentation must not describe current-clipboard paste as a universal plain-text conversion mechanism.

`auto` may choose clipboard mode only through explicit capability and benchmark policy. An explicit `keyboard` or `clipboard` request is never silently replaced by another backend.

## Revision evidence

A platform may expose a monotonically changing clipboard sequence/revision. Revision evidence is content-free and may be retained only for the active operation.

- A known pre-dispatch mismatch aborts before the paste chord.
- A change observed during or immediately after dispatch produces a conservative clipboard-changed/progress-unknown result.
- An unavailable revision is a capability fact. Strict clipboard mode fails clearly rather than claiming a guarded paste that cannot be evidenced.
- Revision checks minimize but cannot make the user's normal clipboard inherently atomic with a target application's later paste handling. This limitation is documented and tested by target category.

## Alternatives considered

### Temporary Unicode-text write and text-only restore

Rejected for V1 because it can destroy richer/private formats and cannot safely reconstruct arbitrary delayed-rendered clipboard state.

### Retain and restore an arbitrary OLE `IDataObject`

Potentially preserves more formats, but introduces COM apartment, lifetime, delayed-rendering, cross-process ownership, and acknowledgement complexity disproportionate to the current product requirement. It remains a future option only with a new ADR and dedicated evidence.

### Clipboard mode unavailable

Safe but unnecessarily removes fast normal paste, which is already a core V1 mode and requires no mutation when the source is the current clipboard.

## Consequences

### Positive

- eliminates restore-over-newer-value data-loss risk by construction;
- preserves all existing clipboard formats and their original ownership;
- keeps V1 out of clipboard-manager/history territory;
- materially reduces plaintext copies and unsafe FFI;
- supports fast constant-size native dispatch for long payloads;
- provides a clear semantic distinction between typing and normal paste.

### Negative / trade-offs

- clipboard mode uses target-native paste format selection rather than guaranteed plain text;
- an external clipboard change in the narrow check-to-consume interval remains possible and must be reported conservatively when observed;
- applications that block normal paste still require keyboard mode;
- target-category compatibility evidence is still required.

## Follow-up

A future feature that transforms text, forces plain-text paste, or temporarily owns the clipboard requires a new ADR covering format preservation, delayed rendering, ownership markers, acknowledgement, conditional restoration, external-change races, privacy, and platform-specific tests.