# ADR-0015: Flutter settings auto-save

- Status: Accepted
- Date: 2026-09-04

## Context

The P4 macOS Flutter settings window previously collected changes in local
page drafts and required an explicit Apply action. That creates an avoidable
commit step, makes small settings changes easy to lose, and gives no useful
feedback while a native persistence operation is in progress. The Flutter
front end must remain a presentation shell: Rust continues to validate and
persist the product settings, while Swift continues to own native hot-key and
login-item transactions.

## Decision

The Flutter settings controller accepts complete `AppSettings` snapshots from
the pages and persists every valid change automatically.

- Discrete changes save immediately.
- Text fields and sliders use a short bounded debounce and coalesce rapid
  edits into the latest snapshot.
- Writes are serialized; a newer snapshot is never allowed to be overwritten
  by an older in-flight write.
- Invalid snapshots remain in the page draft, are shown inline, and are not
  sent to the native bridge.
- A native failure is visible, keeps the failed snapshot retryable, and does
  not claim that the setting was saved.
- Reset is scoped to the current page and is itself automatically persisted.
- The UI communicates Pending, Saving, Saved, and failure/review states. It
  does not expose an Apply transaction.

The shared Flutter shell also uses a centered responsive content column,
consistent cards and page headers, and an auto-save footer so the save model is
discoverable without competing with the settings themselves.

## Alternatives considered

### Keep Apply as the only commit

Rejected because it leaves the user responsible for a second action after
every change and was the source of the reported “settings did not save”
behavior.

### Save every keystroke synchronously

Rejected because it creates unnecessary native writes and can race while a
number field is temporarily empty or incomplete. A short debounce preserves
direct-save behavior while coalescing continuous input.

### Let each page call the native bridge independently

Rejected because it would duplicate validation, persistence ordering, failure
handling, and privacy-sensitive platform boundaries in UI code.

## Consequences

### Positive

- Valid changes become durable without an Apply button.
- Rapid edits have deterministic latest-snapshot-wins behavior.
- Save progress and failures are visible and recoverable.
- Existing Rust validation, atomic persistence, and native rollback remain the
  authority.
- The UI is easier to scan and operate at both narrow and wide window sizes.

### Negative / trade-offs

- A valid change may be delayed by the short debounce for continuous controls.
- Each page must maintain a complete local draft while an edit is being
  validated or saved.
- The native bridge can still reject a valid UI snapshot because of external
  hot-key conflicts, permission, startup, or platform failures.

## Follow-up

- Keep Flutter auto-save/coalescing and invalid-snapshot tests in the macOS
  arm64 gate.
- Retain physical Accessibility and named-application evidence as separate
  platform gates; auto-save does not change those consent boundaries.
