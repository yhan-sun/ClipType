# Security and Privacy

## Security posture

ClipType handles clipboard text that may contain credentials, private communications, source code, personal data, or operational commands. The product therefore treats plaintext as ephemeral user-owned data and native input as a safety-sensitive side effect.

ClipType is a local, single-user, normal-integrity desktop process. It is not a security boundary and does not attempt to defeat Windows protections.

## Data lifecycle

### Clipboard acquisition

- Clipboard text is read only after an explicit trigger has reserved a session and captured destination evidence.
- Reads are bounded by a native byte limit and an application semantic-element limit.
- The Windows adapter copies `CF_UNICODETEXT` into owned memory while the clipboard is open, then releases native ownership promptly.
- Busy/temporary failures are retried only within a configured attempt/time budget.
- A content-blind clipboard sequence number is used as revision evidence. Its numeric value is redacted from diagnostics.

### Clipboard mode

Clipboard mode uses the clipboard value that is already current. It never writes, clears, owns, replaces, snapshots for restoration, or restores clipboard contents.

Code mode reads the current clipboard text and uses the keyboard adapter. It
skips source indentation and emits cursor-right actions for matching ordinary
pairs the editor is expected to have generated. Python-style triple-quoted
boundaries are emitted explicitly rather than assumed to be editor-generated.
Markdown triple-backtick fences are likewise emitted literally without reading
the fenced content. It does not use Clipboard paste or a revision-guarded paste
transaction. It does not add a second clipboard storage path or read destination
content.

The flow is:

1. capture destination evidence;
2. read bounded current text and a stable revision;
3. freeze the clipboard backend only if revision guarding is available;
4. revalidate destination, integrity, modifiers, cancellation, and revision;
5. send one balanced `Ctrl+V` chord;
6. classify native progress conservatively and stop.

A changed revision aborts before paste. Because ClipType never restores an earlier value, it cannot overwrite a newer external clipboard change.

### Keyboard mode

Keyboard mode normalizes text into semantic atoms and sends bounded Unicode/key batches. Plaintext lives only in the active plan and worker lifetime. Batches are not retried after partial or unknown progress.

### End of session

Owned plaintext is dropped when the plan/session ends. Rust and the operating system do not guarantee immediate physical memory zeroization; the product therefore does not claim forensic memory erasure. The stronger guarantee is architectural: no history, cache, persistence, network transport, crash upload, or diagnostic echo of plaintext is created.

## Data that is not collected

ClipType does not intentionally collect or persist:

- clipboard history;
- clipboard/injected text, substrings, samples, prefixes, suffixes, hashes, or fingerprints;
- focused-field contents;
- window titles or document names;
- arbitrary keyboard activity;
- application usage history;
- analytics, telemetry, identifiers, or network requests;
- screenshots or accessibility-tree content;
- memory dumps or automatic crash uploads.

## Destination safety

### Capture before clipboard work

The intended destination is captured before plaintext acquisition. Content-free evidence can include opaque process/window/focus identity and integrity relation.

### Revalidation

Before native dispatch, and between keyboard batches, ClipType re-captures evidence and compares it with the original. It stops when the target changed, disappeared, became ambiguous, or lost detail required by the original guarantee.

ClipType never refocuses the old target and never redirects remaining input to a new target.

### Shared render hosts

Some browsers, editors, terminals, and framework applications expose several logical fields through one render host. When Windows does not expose a stable child/focus identity, ClipType cannot promise exact logical-field or caret identity. The compatibility documentation states this limitation instead of inferring field contents.

## Native input safety

- Trigger modifiers must become clear within a bounded settle window.
- Physical Ctrl/Alt/Shift/Win state is observed; ClipType never emits releases for keys it did not press.
- Each keyboard batch and paste chord is bounded.
- Complete, partial, none-accepted, and progress-unknown native results remain distinct.
- Partial and progress-unknown results are terminal and never automatically retried.
- Cancellation is checked before dispatch and at safe boundaries; it cannot retract events Windows already accepted.
- Terminal newlines can execute commands. Users must review multiline clipboard content before triggering in shells, consoles, or administrative tools.

## Privilege boundary

ClipType runs as one unprivileged per-user process by default.

- no service;
- no driver;
- no privileged helper;
- no automatic elevation;
- no UIPI bypass;
- no cross-session injection;
- no hidden fallback that weakens integrity checks.

A normal-integrity ClipType process does not inject into a higher-integrity target. Users should normally run both applications at the same integrity level rather than elevating ClipType.

## Global command privacy

Global commands use reviewed `RegisterHotKey` combinations with no-repeat behavior. ClipType does not install a low-level keyboard hook and does not observe arbitrary keys. Trigger and cancel commands are typed messages owned by the product's message queue.

## Settings and startup

Settings contain only product configuration: mode, enabled state, threshold, speed, notifications, startup selection, and reviewed hotkey preset. The strict parser rejects unknown, duplicate, missing, or invalid values without echoing untrusted content.

Writes use a temporary file, flush/sync, validated backup, and replacement flow. Start-at-login uses only the current user's ClipType value under the standard Run key. Installation and startup require no elevation.

## Logging and user feedback

Normal output and notifications contain only content-free fields such as:

- generation and phase;
- selected backend;
- completed batch count;
- outcome/remediation category;
- settings source;
- capability or native-error category;
- test counts and artifact digests.

They must not include clipboard text, revision values, window handles, titles, focused-field contents, or test sentinels. Privacy tests deliberately place recognizable generated markers in fixtures and assert that markers do not escape into diagnostics or distributable files.

## Packaging and release integrity

Public beta assets are built from an exact `main` commit in GitHub Actions. The release provides:

- SHA-256 manifest;
- Sigstore keyless signatures authenticated by the release workflow's GitHub OIDC identity;
- verification of every signature before publication;
- GitHub artifact attestations binding assets to repository/workflow/source commit;
- dependency/license metadata and build information;
- immutable versioned release assets.

The first beta is not represented as Authenticode publisher-signed because no trusted Windows certificate is configured. Sigstore and GitHub provenance prove workflow identity and integrity, but Windows may still display reputation or SmartScreen warnings.

## Threats and mitigations

| Threat | Mitigation | Residual risk |
|---|---|---|
| Clipboard changes between read and paste | stable revision read and immediate pre-dispatch verification | changes after verification but before destination consumption remain an OS/application timing boundary |
| Focus changes during a session | capture before read and revalidate before/between dispatch | shared render hosts may not expose logical-field identity |
| Elevated destination | fail/stop at known higher integrity; no elevation bypass | user may manually run both processes elevated, increasing risk |
| Held trigger modifiers | bounded settle and per-dispatch modifier checks | hardware/driver state can be unavailable or change after observation |
| Partial native input | conservative terminal outcome; no retry | destination may have consumed a prefix |
| Clipboard plaintext in logs/artifacts | redacted types, content-free outputs, sentinels, package scans | process memory can still contain active plaintext temporarily |
| Malicious/corrupt settings | strict fixed schema, validation, backup recovery | local user can intentionally alter their own configuration |
| Compromised release asset | checksums, keyless signatures, attestations, immutable versioning | endpoint compromise can still replace local files after download |
| Untrusted destination behavior | explicit trigger and documented application semantics | destination may interpret paste/newlines as commands |

## Vulnerability reporting

Report security issues privately according to `SECURITY.md`. Do not include real clipboard contents, credentials, private documents, or memory dumps. Reproduce with generated placeholders and content-free environment/outcome information.
