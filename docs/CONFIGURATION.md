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
characters_per_second = 40
jitter_percent = 0
typo_probability_percent = 0
notifications = true
start_at_login = false
hotkey = "ctrl-alt-shift-function"
```

The parser is strict. Existing files may omit the three human-typing fields for backward compatibility; they are migrated in memory from the legacy speed preset and written on the next settings save. All other documented fields are required. Unknown keys, duplicate keys, inline/untrusted trailing text, malformed quoting, invalid booleans/numbers, unsupported enum values, zero threshold, or unsupported schema version fail visibly rather than being ignored.

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

Accepted values are `"slow"`, `"normal"`, `"fast"`, and `"custom"`. The three presets map to 8, 40, and 120 characters per second. Selecting a tray adjustment sets the value to `"custom"`.

### `characters_per_second`

The exact keyboard-mode target pacing rate, from 1 through 250 actions per second. Operating-system scheduling and destination processing can make the measured rate lower. One Unicode scalar, line break, Tab, wrong adjacent character, corrective Backspace, or corrected character consumes one timing slot. The configured value is therefore an action rate; enabling corrected typos intentionally reduces the throughput of original text.

### `jitter_percent`

Independent bounded timing jitter from 0 through 95 percent. Jitter is sampled for every emitted keyboard action, including corrective Backspace. Clipboard paste remains one atomic destination-owned action and is not stretched into artificial per-character timing.

### `typo_probability_percent`

An opt-in corrected-typo probability from 0 through 25 percent. The default is 0. Eligible ASCII characters may be replaced by a US-QWERTY adjacent character, followed by Backspace and the intended character. CJK, emoji, combining marks, whitespace, line breaks, and Tab are never mutated.

Do not enable typo simulation for passwords, source code, terminals, commands, identifiers, checksums, or exact-data entry. Clipboard mode never applies typo simulation.

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
