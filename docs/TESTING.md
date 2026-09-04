# Testing and Evidence

## Evidence principle

ClipType separates policy proof, native mechanism proof, controlled end-to-end behavior, package lifecycle, compatibility, and release provenance. A green hosted runner does not by itself prove every physical user, Windows client build, application, render host, remote session, or integrity configuration.

Every report must state its evidence class and keep clipboard/target content out of output.

## Native-neutral quality gate

Run on Linux and Windows where applicable:

```text
cargo fmt --all -- --check
cargo check --locked --all-targets -p cliptype-core -p cliptype-platform -p cliptype-app
cargo test --locked -p cliptype-core -p cliptype-platform -p cliptype-app
cargo clippy --locked --all-targets -p cliptype-core -p cliptype-platform -p cliptype-app -- -D warnings
```

Coverage includes:

- strict configuration and limits;
- Unicode normalization and line-break semantics;
- explicit mode no-fallback and auto backend selection;
- Code mode's keyboard actions, leading-indentation stripping, pair skipping,
  and string/comment literal handling;
- Auto preference for short CJK, emoji, combining, and mixed-Unicode text;
- clipboard revision/snapshot behavior;
- one-session concurrency, cancellation, bounded retry/wait/shutdown;
- destination and integrity revalidation;
- modifier conflicts;
- partial/progress-unknown no-retry behavior;
- settings parsing, atomic persistence, backup recovery, and redacted errors;
- content-free debug/status/outcome types.

## Full Windows workspace gate

```text
cargo metadata --locked --format-version 1 --no-deps
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --release --locked -p cliptype
```

Windows unit/contract coverage includes:

- bounded `CF_UNICODETEXT` allocation/decoding;
- clipboard sequence-number evidence;
- Unicode and special-key `SendInput` encoding;
- balanced revision-guarded Paste chord;
- conservative native accepted-count classification;
- foreground/focus/integrity evidence and degradation fail-closed behavior;
- global hotkey registration/message-loop ownership;
- tray menu enum transitions and notification text;
- start-at-login registry value creation, matching, and cleanup.

## Controlled E2E

The opt-in controlled harnesses create a generated native Win32 edit target and generated clipboard fixtures. They never print fixture/target plaintext.

### P1 controlled path

Exercises current clipboard read through bounded keyboard injection into the native edit target. Repeated runs verify Unicode, line breaks, target evidence, native delivery, and privacy-output behavior.

### P2 controlled paths

Exercises:

- explicit keyboard;
- explicit clipboard;
- auto-short selecting keyboard;
- auto-long selecting clipboard;
- expected UTF-16 result;
- selected backend;
- completed batch count;
- unchanged clipboard text and sequence revision;
- privacy sentinel absence from ordinary output.

Separate coordinator tests cover clipboard revision changes, target changes, modifier conflicts, cancellation, and partial/unknown native results that are difficult or unsafe to force through an interactive target.

## Native and host smoke

- **P1 Windows Native Spike** validates actual clipboard, target, hotkey/message-loop, modifier, and input adapter assumptions.
- **P1 Windows Host Smoke** builds and starts the composition root, uses the private command queue for controlled shutdown, and verifies command teardown.
- Release builds retain the Windows GUI subsystem while debug/CI modes can expose deterministic console status.

## Backend benchmark

The benchmark workflow measures controlled keyboard and clipboard paths at supported payload points, emits content-free rows, and records one supported crossover recommendation. The default threshold is a product policy input, not a universal claim about every destination application's paste performance.

## Package lifecycle gate

The Windows package workflow:

1. builds the optimized executable;
2. stages licenses and install/uninstall documentation;
3. creates the development ZIP;
4. parses PowerShell scripts;
5. installs to an isolated per-user directory;
6. runs and stops the GUI-subsystem executable through redirected standard handles;
7. verifies initial and enabled startup settings;
8. verifies the current-user Run value;
9. uninstalls and removes product-owned files/settings/startup state;
10. scans staged distributable files for privacy sentinels.

The process exit code and controlled `status=stopped` record are both required. PowerShell's ambient `$LASTEXITCODE` is not used as a substitute for the actual GUI-subsystem process exit code.

## Windows compatibility matrix

`.github/workflows/windows-compatibility.yml` runs the complete workspace, controlled P1/P2 paths, Clippy, release build, privacy checks, and artifact hashing on:

- `windows-2022` x86_64;
- `windows-2025` x86_64.

These are Windows Server Desktop Experience CI environments. They establish a stable Win32 mechanism/build range but do not certify physical client hotkey timing or every named application.

Client support is defined in `docs/COMPATIBILITY.md`: Windows 11 x64 is recommended; Windows 10 22H2 x64 is best-effort API-compatible with an operating-system support/security caveat.

## macOS arm64 local candidate gate

`.github/workflows/p4-macos-arm64.yml` is a PR/manual quality gate for the P4
Flutter runner. It requires an Apple Silicon runner, installs Flutter 3.47.2
and Rust 1.98.0, runs the locked arm64 workspace checks/tests/Clippy, runs the
Flutter format/analyze/test gate, builds `ClipType.app`, scans every bundled
Mach-O file for the arm64 slice, and verifies bundle codesign integrity.

This workflow does not grant Accessibility, exercise a physical target
application, collect clipboard fixtures, sign with Developer ID, notarize, or
publish an artifact. Those cases remain local/physical evidence gates and are
reported separately in `local-evidence/`.

## Release pipeline validation

On pull requests, `.github/workflows/windows-release.yml` performs a dry run that:

- validates the beta semantic version and matching notes;
- reruns workspace check/test/Clippy;
- builds the exact release executable;
- creates ZIP and portable assets;
- generates dependency/license and build metadata;
- scans distributable files;
- records Authenticode status transparently;
- uploads the exact asset set for inspection.

On `main`, after `release/VERSION` changes, the publication job additionally:

- refuses an existing tag/release;
- generates and verifies `SHA256SUMS.txt`;
- creates Sigstore keyless bundles using GitHub OIDC;
- verifies every bundle against the exact release workflow identity;
- generates GitHub artifact attestations;
- creates the public GitHub prerelease bound to the exact source commit.

The initial beta does not claim Authenticode trusted-publisher signing.

## Privacy tests

Generated markers are used to detect accidental plaintext escape in:

- debug and display output;
- coordinator status/outcomes;
- settings errors/files;
- controlled E2E logs;
- package staging;
- release staging and metadata.

A privacy test must never use real user clipboard content. Evidence files contain only counts, categories, source SHA, run IDs, digests, environment labels, and explicit scope limitations.

## Physical and named-application evidence

Useful field evidence includes Windows build/architecture, application/version, elevation relation, mode, outcome category, and whether the issue involved target evidence, modifiers, revision, hotkey ownership, partial input, or destination semantics.

Do not attach clipboard text, credentials, private messages, focused-field contents, screenshots containing secrets, or raw memory dumps. Named reports expand the compatibility matrix; they do not permit weakening fail-closed behavior.

## Release-blocking failure categories

Publication is blocked by any unresolved failure involving:

- clipboard data loss or external-change overwrite;
- plaintext leakage or network transmission;
- target redirection or evidence degradation accepted as Same;
- modifier release not owned by ClipType;
- automatic elevation or security-boundary bypass;
- retry after partial/unknown native input;
- unbounded work, wait, retry, or shutdown;
- package install/startup/uninstall residue;
- failed checksum/signature/attestation verification;
- compatibility wording broader than evidence;
- existing tag/release collision.
