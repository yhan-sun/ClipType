# Configuration Model

This document freezes semantics, not exact Rust types or every default value.

## Principles

- Safe defaults require no configuration.
- Manual trigger remains the default activation model.
- Configuration cannot disable mandatory security/privacy invariants silently.
- Invalid security-sensitive values fail clearly rather than being ignored.
- Platform-specific options live under explicit platform namespaces.
- Config migrations are versioned once a stable release exists.

## Planned conceptual TOML

```toml
version = 1

enabled = true

[trigger]
hotkey = "platform-default"

[injection]
mode = "auto"                 # auto | keyboard | clipboard
keyboard_interval_ms = 2
chunk_size = 32
clipboard_threshold = 0        # 0 means use compiled/default policy value
restore_clipboard = true
focus_guard = "strict"

[ui]
show_status_notifications = true
start_at_login = false

[platform.windows]
# future platform-specific settings only when necessary

[platform.macos]
# permission behavior remains OS controlled

[platform.linux]
wayland_backend = "auto"       # capability probe; not a forced compatibility claim
```

This is illustrative. Exact field names/defaults are finalized with implementation and schema tests.

## Stable semantics

### `enabled`
Disables injection triggers without terminating the app.

### `trigger.hotkey`
A user-configurable global trigger. Registration conflicts produce visible error state.

### `injection.mode`
- `keyboard`: require keyboard capability; fail if unavailable.
- `clipboard`: require clipboard-paste capability; fail if unavailable.
- `auto`: planner selects among eligible mechanisms.

Explicit user mode MUST NOT silently degrade to a different mode.

### `keyboard_interval_ms`
Delay policy for keyboard event emission. Values are bounded to prevent accidental unusable settings/resource abuse.

### `chunk_size`
Maximum bounded work between cancellation/focus checks. Tuning may be backend-specific internally.

### `clipboard_threshold`
Auto-mode hint for considering clipboard mode for large payloads. The default is benchmark-driven and may evolve between pre-1.0 releases.

### `restore_clipboard`
Requests safe restoration after clipboard-paste transactions. Restoration is skipped rather than overwriting a newer external clipboard.

### `focus_guard`
V1 default is strict where supported. A platform with weaker evidence reports degraded capability rather than pretending strict guarantees.

## Configuration storage

Configuration itself may persist. Clipboard content may not be stored inside configuration, recent-values files, caches, or backups created by ClipType.

## Future app profiles

Target-specific profiles are deferred until compatibility evidence justifies them. A future structure may select mode/interval/threshold by application identity, but profiles are not part of P1.