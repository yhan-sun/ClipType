# ADR-0012: Flutter is the sole macOS front end

- Status: Accepted
- Date: 2026-09-04
- Scope: macOS application composition and settings presentation
- Supersedes: ADR-0010's coexistence of the legacy macOS Slint composition root

## Context

The P4 implementation added a Flutter macOS settings/menu-bar application, but
the repository also retained the older Rust/Slint `apps/cliptype-macos`
composition root. Both could be built or launched as macOS applications. That
made the user-facing entry point ambiguous and could leave a user authorizing
or testing the wrong application bundle. The older path also did not provide
the current English/Simplified Chinese Flutter interface.

## Decision

Remove the legacy `apps/cliptype-macos` Rust/Slint composition root and its
macOS-specific release/package automation. Keep
`apps/cliptype-flutter` as the only macOS settings/front-end entry point.

Keep `crates/cliptype-macos` because it owns the macOS clipboard, target,
keyboard, paste, focus, permission, hot-key, and startup adapters used behind
the Flutter shell. Keep `crates/cliptype-ui` only for the existing Windows
composition root; removing that Windows presentation dependency requires a
separate migration decision.

Keep the existing product bundle identifier,
`io.github.yhan-sun.ClipType`, for the Flutter replacement. This preserves the
macOS application identity used by Accessibility consent while removing the
legacy frontend itself.

The Flutter shell remains one ordinary application process. It does not bypass
macOS Accessibility consent, move clipboard or injected plaintext into the UI
boundary, or add a second runtime process.

## Alternatives considered

### Keep both macOS front ends and document the preferred one

Rejected because two launchable macOS composition roots continue to create
bundle/permission ambiguity and permit validation of the wrong UI.

### Make the old Rust app launch the Flutter app

Rejected because it preserves a redundant process/composition root and adds a
second launcher path without improving the runtime boundary.

### Remove the shared Slint crate immediately

Rejected for this change because the Windows product still depends on it. A
Windows UI migration needs its own compatibility, licensing, and release
evidence.

## Consequences

### Positive

- There is one documented macOS front end and one application bundle to
  authorize in System Settings.
- macOS users always reach the Flutter settings surface, including its
  English/Simplified Chinese display mode.
- The Rust core and macOS adapters remain reusable and policy ownership does
  not move into Flutter.
- Old macOS release jobs cannot accidentally publish the removed UI.

### Negative / trade-offs

- The current macOS candidate remains arm64-only, local, unsigned, and outside
  the public Universal 2 release channel.
- The Windows product continues to carry the Slint presentation dependency
  until a separately approved migration.
- Historical P3 documents and release notes retain references to the former
  Universal 2 Slint path as historical context.

## Follow-up

1. Use `apps/cliptype-flutter` and `flutter build macos --release` for macOS
   development and local validation.
2. Keep Accessibility evidence tied to the current Flutter bundle identifier
   and path; do not claim authorization from a stale legacy bundle entry.
3. Revisit public macOS packaging only after Flutter Universal 2, signing,
   notarization, and physical-client evidence are separately complete.
