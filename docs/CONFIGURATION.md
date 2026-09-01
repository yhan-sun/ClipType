# Configuration Model

This document defines configuration semantics and safety constraints, not final Rust names or every default value.

## Principles

- Safe defaults require no configuration.
- Manual explicit trigger remains the default activation model.
- Configuration cannot disable mandatory privacy, security-boundary, one-session, or fail-safe behavior silently.
- Invalid security-sensitive values fail clearly rather than being ignored.
- All timing, payload, retry, and batch values have validated lower/upper bounds.
- Platform-specific options live under explicit namespaces only when truly needed.
- Stable persistence and migration are introduced during productization, not assumed in P1.

## P1 configuration posture

P1 is a Windows development vertical slice and implements the `keyboard` path only. It may use validated compiled defaults, a narrow development CLI, or an in-memory configuration snapshot. A persistent settings file and settings UI are not P1 requirements.

P1 MUST NOT expose working `clipboard` or `auto` settings before those mechanisms exist. It also MUST NOT persist clipboard text as configuration, cache, recent values, or diagnostics.

### Conceptual P1 snapshot

```toml
version = 1
enabled = true

[trigger]
hotkey = "development-default"
cancel_hotkey = "development-default"

[injection]
mode = "keyboard"
keyboard_interval_ms = 2
max_batch_elements = 32
max_payload_elements = 100000
focus_policy = "strict-to-available-evidence"
modifier_settle_timeout_ms = 500
clipboard_retry_budget_ms = 100

[diagnostics]
level = "info"
content_logging = false             # immutable false in P1
```

The values above illustrate the shape only. #13 measures native behavior and #3 freezes actual P1 defaults/ranges. Agents must not treat the example numbers as accepted constants.

## P1 stable semantics

### `enabled`

Controls whether new trigger commands are accepted. Disabling does not weaken cleanup or leave native registrations/session state inconsistent.

### `trigger.hotkey`

An explicit global trigger. Registration conflicts produce a typed visible state. P1 uses one reviewed development combination with no-repeat behavior where supported.

### `trigger.cancel_hotkey`

An explicit cancellation command available while a session is active. P1 does not permanently hijack bare `Esc` or capture unrelated keys.

### `injection.mode`

P1 accepts only `keyboard`. `clipboard` and `auto` are rejected as unsupported rather than silently mapped to keyboard.

### `keyboard_interval_ms`

Delay policy between semantic elements or native batches as defined by the plan. Values are bounded so they cannot create effectively infinite operations or zero-delay event floods without explicit review.

### `max_batch_elements`

Maximum native-neutral semantic elements submitted in one bounded dispatch. It defines a cancellation/target/modifier checkpoint boundary; the Windows adapter may produce multiple native events for one element.

The batch limit is not the total payload limit and is not expressed as raw `INPUT` event count in core policy.

### `max_payload_elements`

Maximum validated semantic text elements accepted for one operation. Exceeding it produces a typed `payload too large` outcome before injection.

A separate hard native acquisition cap also bounds clipboard allocation scanning/copying before complete semantic validation. The native cap is an implementation safety boundary, not necessarily a user-facing setting. #13/#3 must define exact units and conversions without unchecked allocation.

### `focus_policy`

P1 uses `strict-to-available-evidence`: capture destination evidence at trigger time, revalidate before and between batches, and stop on known change, disappearance, or evidence becoming unavailable/ambiguous after dispatch begins.

This does not promise exact logical-field/caret identity when an application exposes only one native render host.

### `modifier_settle_timeout_ms`

Maximum bounded wait for trigger/conflicting physical modifiers to become safe before first dispatch. Timeout fails explicitly. ClipType never releases the user's physical keys.

### `clipboard_retry_budget_ms`

Total bounded budget for transient current-clipboard acquisition contention. The live coordinator owns retry timing so cancellation and the originally captured target remain observable. The low-level adapter performs one bounded attempt.

### Diagnostics

`content_logging` is shown only to make the invariant visible; it is not a switch users or agents may turn on in P1. Clipboard/injected text, samples, fingerprints, focused contents, and window titles remain forbidden in ordinary diagnostics.

## Internal validation requirements

A validated P1 snapshot must guarantee:

- batch size is non-zero and below an implementation-safe maximum;
- total payload and native acquisition limits are finite;
- interval, modifier wait, clipboard retry, and shutdown/join budgets are finite;
- retry counts/durations cannot overflow when converted to platform timing units;
- explicit keyboard mode is supported by current capabilities;
- strict target policy is not silently downgraded;
- content logging remains disabled;
- invalid values never fall back to unsafe unbounded behavior.

## P2 conceptual product configuration

Windows productization may extend the schema after the clipboard transaction and UI are designed:

```toml
version = 1
enabled = true

[trigger]
hotkey = "platform-default"
cancel_hotkey = "platform-default"

[injection]
mode = "auto"                     # auto | keyboard | clipboard
keyboard_interval_ms = 2
max_batch_elements = 32
max_payload_elements = 100000
clipboard_threshold = 0            # use benchmark-derived default
restore_clipboard = true
focus_policy = "strict-to-available-evidence"
modifier_settle_timeout_ms = 500
clipboard_retry_budget_ms = 100

[ui]
show_status_notifications = true
start_at_login = false
```

This remains illustrative until P2 contracts/tests freeze exact field names and defaults.

## Future semantics

### `clipboard` mode

Requires temporary clipboard write, paste action, and safe conditional restoration. A newer external clipboard value is never overwritten merely to restore an old snapshot.

### `auto` mode and `clipboard_threshold`

The planner selects only eligible mechanisms. Threshold defaults are benchmark/compatibility driven. An explicit user mode does not silently fall back to another mode.

### Persistent storage and migration

When persistent configuration enters scope:

- schema versioning and migration tests are required;
- atomic write/recovery behavior is defined;
- security-sensitive invalid values fail visibly;
- configuration and backups contain no clipboard text;
- permissions follow platform conventions.

### App profiles

Target-specific profiles remain deferred until compatibility evidence justifies them. Profiles may later select mode, interval, or threshold by safe application identity, but they do not weaken mandatory target/cancellation/privacy rules.

## Platform namespaces

Platform-specific settings are added only when portable semantics cannot express the actual capability. They must not be used to force unsupported behavior or convert one compositor/application observation into a general compatibility claim.
