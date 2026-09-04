# Configuration

## Location

The Windows product stores one per-user settings file:

```text
%LOCALAPPDATA%\ClipType\config.toml
```

The macOS product stores the same schema under the standard per-user Application Support directory:

```text
~/Library/Application Support/ClipType/config.toml
```

Clipboard text and target data are never stored beside the configuration.

## Current schema

P3 uses schema version `2` and a deliberately small, flat TOML vocabulary:

```toml
version = 2
enabled = true
mode = "auto"
auto_clipboard_threshold = 256
speed = "normal"
characters_per_second = 40
jitter_percent = 0
typo_probability_percent = 0
notifications = true
start_at_login = false
trigger_hotkey = "ctrl+alt+shift+v"
cancel_hotkey = "ctrl+alt+shift+x"
```

The parser is strict. Unknown keys, duplicate keys, inline/untrusted trailing text, malformed quoting, invalid booleans/numbers, unsupported enum values, zero threshold, unsupported shortcuts, duplicate Trigger/Cancel values, or unsupported schema versions fail visibly rather than being ignored.

Schema version `1` is migrated deterministically. The old preset-only `hotkey` value maps to a reviewed non-F12 Trigger/Cancel pair and preserves all other valid settings. The migrated schema is written on the next successful settings save.

## Fields

### `version`

Must be `2`. Version `1` is accepted only through the explicit migration path. Other versions fail closed.

### `enabled`

Controls acceptance of new trigger commands. `false` rejects new sessions before target or clipboard work. It does not weaken cleanup, cancellation, command registration, or shutdown behavior.

### `mode`

Accepted values:

- `"keyboard"` — bounded Unicode-oriented keyboard actions;
- `"clipboard"` — one revision-guarded ordinary paste command;
- `"code"` — one revision-guarded whole-block paste for source code and
  structured text;
- `"auto"` — freeze one eligible backend from payload size and capabilities.

Explicit modes do not silently fall back. Clipboard mode requires both Paste and a known content-blind revision witness. Auto uses clipboard only when both are fully available.

Code mode requires the same Paste and revision capabilities as Clipboard mode.
It preserves the clipboard's delimiters, line breaks, and indentation and does
not emit per-character keyboard, Return, Tab, or typo-correction actions.

### `auto_clipboard_threshold`

A non-zero semantic-element count. In auto mode, clipboard becomes preferred
at or above this point when its capabilities are fully available. Auto also
prefers the revision-guarded clipboard path for any non-ASCII text, including
CJK, emoji, combining marks, and mixed Unicode, even below this threshold.
When guarded paste is unavailable, Auto can choose the Unicode keyboard path
when that path is proven safe. The threshold remains the size crossover for
otherwise keyboard-friendly text.

The default is `256`. It is a policy crossover covered by the backend benchmark, not a universal statement about every destination's performance.

### `speed`

Accepted values are `"slow"`, `"normal"`, `"fast"`, and `"custom"`. The three presets map to 8, 40, and 120 characters per second. Editing the exact rate sets the value to `"custom"`.

### `characters_per_second`

The exact keyboard-mode target pacing rate, from 1 through 250 actions per second. Operating-system scheduling and destination processing can make the measured rate lower. One Unicode scalar, line break, Tab, wrong adjacent character, corrective Backspace, or corrected character consumes one timing slot. The configured value is therefore an action rate; enabling corrected typos intentionally reduces the throughput of original text.

### `jitter_percent`

Independent bounded timing jitter from 0 through 95 percent. Jitter is sampled for every emitted keyboard action, including corrective Backspace. Clipboard paste remains one atomic destination-owned action and is not stretched into artificial per-character timing.

### `typo_probability_percent`

An opt-in corrected-typo probability from 0 through 25 percent. The default is 0. Eligible ASCII characters may be replaced by a US-QWERTY adjacent character, followed by Backspace and the intended character. CJK, emoji, combining marks, whitespace, line breaks, and Tab are never mutated.

Do not enable typo simulation for passwords, source code, terminals, commands, identifiers, checksums, or exact-data entry. Clipboard and Code modes never apply typo simulation.

### `notifications`

Controls fixed content-free tray/menu-bar notifications. Disabling notifications does not disable safety checks or internal content-free status.

### `start_at_login`

Controls the platform-owned per-user login item:

- Windows uses one ClipType value under the current user's Run key;
- macOS uses the app-owned `SMAppService.mainApp` registration when supported.

No administrator/root privilege, service, scheduled task, machine-wide registry write, or consent bypass is used.

### `trigger_hotkey` and `cancel_hotkey`

Each value is a canonical, native-neutral shortcut string. Supported tokens include:

- modifiers: `ctrl`, `alt`, `shift`, `meta`;
- letters and digits;
- `f1` through `f24` where the platform supports them;
- navigation keys and selected punctuation names such as `left`, `pageup`, `minus`, `slash`, and `bracket-left`.

Examples:

```toml
trigger_hotkey = "ctrl+alt+shift+v"
cancel_hotkey = "ctrl+alt+shift+x"
```

Rules:

- at least one of Control, Alt/Option, or Meta/Command is required;
- Shift-only and bare-key shortcuts are rejected;
- Trigger and Cancel must differ;
- known system-reserved or unsafe combinations are rejected per platform;
- Windows F12 is rejected as a custom/recommended binding;
- the graphical recorder captures only local events while its control owns focus;
- the platform adapter probes actual OS registration before persistence;
- successful OS registration cannot prove that every foreground application or hook-based tool will not also react, so the UI may report `Cannot fully verify`.

The configuration never stores native virtual-key codes, Carbon event references, window handles, or captured key history.

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

These are mandatory safety invariants, not user switches.

## Persistence and recovery

Saving uses adjacent files:

```text
config.toml.tmp
config.toml.bak
```

The store:

1. validates the complete proposed settings and shortcut pair;
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
- arbitrary key capture or recorded key history;
- telemetry, account, or network configuration;
- elevation or security-boundary bypass.

Clipboard mode does not use a `restore_clipboard` option because the product never rewrites or restores the clipboard.

## Manual recovery

When configuration is invalid:

1. close ClipType through the tray or menu bar when possible;
2. preserve the invalid file only if needed for debugging and it contains no manually added secrets;
3. replace it with the complete version-2 example above or remove both primary and backup to regenerate defaults;
4. restart ClipType;
5. reapply desired settings through the graphical settings window.

Do not attach a real settings file to a public issue if it has been manually edited to contain private data. Reproduce with generated values and report only the content-free error category.
