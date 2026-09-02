# Configuration

## Location

The Windows product stores one per-user settings file:

```text
%LOCALAPPDATA%\ClipType\config.toml
```

If `LOCALAPPDATA` is unavailable, the host may use the current user's roaming application-data root. Clipboard text and target data are never stored beside the configuration.

## Current schema

The public beta uses schema version `1` and a deliberately small, flat TOML vocabulary:

```toml
version = 1
enabled = true
mode = "auto"
auto_clipboard_threshold = 256
speed = "normal"
notifications = true
start_at_login = false
hotkey = "ctrl-alt-shift-function"
```

The parser is strict. Every field is required in an existing file. Unknown keys, duplicate keys, inline/untrusted trailing text, malformed quoting, invalid booleans/numbers, unsupported enum values, zero threshold, or unsupported schema version fail visibly rather than being ignored.

## Fields

### `version`

Must be `1`. Unknown versions fail closed. A future migration requires an explicit schema/migration implementation and tests.

### `enabled`

Controls acceptance of new trigger commands. `false` rejects new sessions before target or clipboard work. It does not weaken cleanup, cancellation, command registration, or shutdown behavior.

### `mode`

Accepted values:

- `"keyboard"` — bounded Unicode-oriented keyboard batches;
- `"clipboard"` — one revision-guarded ordinary paste command;
- `"auto"` — freeze one eligible backend from payload size and capabilities.

Explicit modes do not silently fall back. Clipboard mode requires both Paste and a known content-blind revision witness. Auto uses clipboard only when both are fully available.

### `auto_clipboard_threshold`

A non-zero semantic-element count. In auto mode, clipboard becomes preferred at or above this point when its capabilities are fully available. Auto can also choose clipboard for text that the keyboard planner cannot safely represent.

The default is `256`. It is a policy crossover covered by the backend benchmark, not a universal statement about every destination's performance.

### `speed`

Accepted values and current keyboard pacing:

| Value | Inter-batch interval |
|---|---:|
| `"slow"` | 12 ms |
| `"normal"` | 5 ms |
| `"fast"` | 1 ms |

The interval is applied between bounded keyboard batches. Clipboard mode sends one bounded Paste chord and does not use keyboard pacing.

### `notifications`

Controls fixed content-free tray balloon notifications. Disabling notifications does not disable safety checks or internal content-free status.

### `start_at_login`

Controls the product-owned current-user Run value. The command contains the quoted executable path plus `--background`. No administrator privilege, service, scheduled task, or machine-wide registry write is used.

Installer and tray changes update both the setting and startup registration with rollback on failure. Uninstall removes only product-owned state.

### `hotkey`

Accepted reviewed presets:

| Value | Trigger | Cancel |
|---|---|---|
| `"ctrl-alt-shift-function"` | Ctrl+Alt+Shift+F12 | Ctrl+Alt+Shift+F11 |
| `"ctrl-alt-function"` | Ctrl+Alt+F12 | Ctrl+Alt+F11 |
| `"ctrl-shift-function"` | Ctrl+Shift+F12 | Ctrl+Shift+F11 |

The Windows adapter uses `RegisterHotKey` with no-repeat behavior; it does not install a low-level keyboard hook. Another application can own the combination. A changed preset is saved immediately and becomes active after a controlled restart in this beta.

## Compiled safety configuration

The file intentionally does not expose every safety bound. The validated runtime configuration retains compiled finite defaults for:

- native clipboard byte limit;
- total semantic payload limit;
- dispatch batch limit;
- clipboard retry attempt/window budget;
- modifier settle timeout and poll interval;
- worker shutdown grace;
- Tab/control normalization policy;
- target/integrity evidence policy;
- no-retry handling after partial or unknown native progress.

These are mandatory safety invariants, not user switches. A later version may expose a bounded subset only after adding validation, migration, UI, and regression tests.

## Persistence and recovery

Saving uses adjacent files:

```text
config.toml.tmp
config.toml.bak
```

The store:

1. validates the proposed settings;
2. creates the parent directory;
3. writes/truncates the temporary file;
4. flushes and syncs it;
5. preserves a valid prior primary as the backup;
6. renames the temporary file into the primary location;
7. attempts rollback if replacement fails.

Loading order:

1. valid primary;
2. valid backup when the primary is missing/corrupt;
3. safe compiled defaults when both files are missing.

A corrupt primary is not silently interpreted with partially applied values. Diagnostics expose only error category and line/key category; they do not echo untrusted values.

## Privacy boundary

The schema has no field for:

- clipboard text/history/cache;
- transformed or generated text;
- target title/content;
- content samples, prefixes, suffixes, hashes, or fingerprints;
- arbitrary key capture;
- telemetry, account, or network configuration;
- elevation or security-boundary bypass.

Clipboard mode does not use a `restore_clipboard` option because the product never rewrites or restores the clipboard.

## Manual recovery

When configuration is invalid:

1. close ClipType through the tray when possible;
2. preserve the invalid file only if needed for debugging and it contains no manually added secrets;
3. replace it with the complete version-1 example above or remove both primary and backup to regenerate defaults;
4. restart ClipType;
5. reapply the desired reviewed settings through the tray.

Do not attach a real settings file to a public issue if it has been manually edited to contain private data. Reproduce with generated values and report only the content-free error category.
