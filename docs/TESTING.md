# Testing and Evidence

## Evidence principle

ClipType separates policy proof, native mechanism proof, controlled end-to-end behavior, package lifecycle, compatibility, and release provenance. A green hosted runner does not by itself prove every physical user, client build, application, render host, remote session, permission state, or integrity configuration.

Every report must state its evidence class and keep clipboard/target plaintext out of ordinary output.

## Native-neutral quality gate

Run on Linux and Windows where applicable:

```text
cargo fmt --all -- --check
cargo check --locked --all-targets -p cliptype-core -p cliptype-platform -p cliptype-app
cargo test --locked -p cliptype-core -p cliptype-platform -p cliptype-app
cargo clippy --locked --all-targets -p cliptype-core -p cliptype-platform -p cliptype-app -- -D warnings
```

Coverage includes:

- strict configuration and bounds;
- Unicode normalization and line-break semantics;
- explicit mode no-fallback and Auto backend selection;
- true characters-per-second pacing, bounded jitter, and opt-in corrected typos;
- Code mode's FIFO keyboard-only actions, five bounded pair families, indentation stripping, same-line and line-leading closer skipping, comment-boundary closing-line navigation, literal string/comment brackets, explicit triple-quoted boundaries, and long-tail completion;
- Auto preference for CJK/emoji/combining/mixed Unicode when guarded paste is available;
- clipboard revision/snapshot behavior without write/restore;
- one-session concurrency, cancellation, bounded retry/wait/shutdown;
- destination/integrity revalidation and evidence degradation;
- macOS render-host focus-node replacement versus real process/window changes;
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
- unchanged current clipboard contract and revision-change fail-closed behavior;
- conservative native accepted-count classification;
- foreground/focus/integrity evidence and degradation fail-closed behavior;
- global hotkey registration/message-loop ownership and transactional pair replacement;
- tray menu transitions/content-free notifications;
- start-at-login registry value creation, matching, and cleanup.

## Code-mode continuous regression gate

Repository CI treats Code-mode correctness as a continuous product contract rather than a one-off bug fixture.

- production mock-Quartz native contracts run on the exact candidate source with ASan/UBSan;
- unsafe modifier-observation mutations and missing prerequisites must fail closed;
- heap-style and editor fixtures exercise generated closers, line-leading closers, final tails, LF/CRLF equivalence, and forbidden Paste fallback;
- Swift/Rust bridge completion mappings remain explicit and content-free;
- macOS P4 executes the native/Swift contract before the full Rust/Flutter release build.

Hosted contracts do not prove a human VS Code/Monaco session; named-editor behavior remains physical evidence.

## Controlled Windows E2E

Opt-in controlled harnesses create a generated visible native Win32 edit target and generated clipboard fixtures. They never print fixture/target plaintext.

### P1 controlled path

Exercises current clipboard read through the coordinator and Windows keyboard adapter into the native edit target. Repeated runs verify Unicode, line breaks, target evidence, native delivery, and privacy-output behavior.

### P2 controlled paths

Exercises independent product-path processes for:

- explicit Keyboard;
- explicit Clipboard;
- Auto short payload;
- Auto long/non-ASCII selection as configured by policy;
- expected UTF-16 result;
- selected backend;
- unchanged clipboard text and sequence revision;
- privacy sentinel absence from ordinary output.

Separate coordinator tests cover revision changes, target changes, modifier conflicts, cancellation, timing/typo behavior, and partial/unknown native results that are difficult or unsafe to force through an interactive target.

### Windows compatibility matrix

`.github/workflows/windows-compatibility.yml` runs the complete workspace and the controlled product paths on:

- `windows-2022` x86_64;
- `windows-2025` x86_64.

The matrix also runs the controlled-host wait-contract regression and multiple independent P2 processes. A required repetition is not a success-on-retry policy: any failed/missing/duplicate case fails the job. The matrix then builds the distribution executable and verifies the expected release artifact.

These hosted Server Desktop Experience environments establish mechanism/build evidence; they do not certify physical client hotkey timing or every named application.

## Native and host smoke

- **P1 Windows Native Spike** validates actual target/hotkey/message-loop/modifier/input assumptions without pretending a private posted message is a physical hotkey press.
- **P1 Windows Host Smoke** builds and starts the composition root, uses the private command queue for controlled shutdown, and verifies command teardown.
- Release builds retain the Windows GUI subsystem while controlled test processes expose only content-free status.

## Backend benchmark

The benchmark workflow measures controlled Keyboard and Clipboard paths at supported payload points, emits content-free rows, and records policy evidence. Performance data never overrides target, revision, modifier, cancellation, or partial-progress safety.

## Package lifecycle gate

The Windows package workflow:

1. builds the optimized executable;
2. stages licenses and install/uninstall documentation;
3. creates the package;
4. parses PowerShell scripts;
5. installs to an isolated per-user directory;
6. runs/stops the GUI-subsystem executable through controlled handles;
7. verifies product/startup settings;
8. verifies the current-user Run value;
9. uninstalls and removes product-owned files/settings/startup state;
10. scans staged distributable files for privacy sentinels.

The actual process exit code and controlled stopped result are required.

## macOS Apple Silicon P4 gate

`.github/workflows/p4-macos-arm64.yml` is the authoritative Apple Silicon Flutter gate.

On pull requests it:

- checks out the exact candidate source and records source/tree/workflow identity;
- requires an arm64 hosted runner;
- validates the declared beta version and matching release notes;
- runs production Code-mode native/Swift bridge contract gates;
- installs pinned Flutter 3.47.2 and Rust 1.98.0;
- runs full locked Rust fmt/check/test/Clippy;
- runs Dart format, `flutter analyze`, `flutter test`, and Flutter macOS release build;
- ad-hoc signs and verifies the bundle;
- rejects non-arm64 Mach-O slices;
- installs the exact bundle at `/Applications/ClipType.app` and launch-smokes it;
- packages arm64 ZIP/DMG assets, creates a macOS SHA-256 manifest, verifies ZIP/DMG contents/signature/architecture, and uploads a temporary Actions artifact.

The publication job is skipped on pull requests.

On a `main` push caused by a new `release/VERSION`, the same exact-main build runs first. The attachment job then waits boundedly until **both** the Windows-created prerelease and its tag ref are visible and verifies that target/tag equal the exact `GITHUB_SHA` and the release is a prerelease. It refuses existing macOS asset names, verifies the candidate manifest, uploads only additive assets, re-downloads the published files, compares bytes, and re-verifies the macOS manifest.

This workflow does not grant persistent Accessibility permission or perform a human named-application session. It does not provide Developer ID, notarization, stapling, Intel/Rosetta, Universal 2, or a general macOS support claim.

## Release pipeline validation

On pull requests, `.github/workflows/windows-release.yml` performs a dry run that:

- validates the beta semantic version and matching notes;
- reruns workspace check/test/Clippy;
- builds the exact optimized release executable;
- creates ZIP and portable assets;
- generates dependency/license and build metadata;
- scans the distributable privacy boundary;
- records Authenticode status transparently;
- uploads the exact asset set for inspection.

The sign/attest/publish job is intentionally skipped on a pull-request event.

On `main`, after `release/VERSION` changes, the publication job additionally:

- refuses an existing tag/release;
- downloads the exact validated assets;
- generates and verifies `SHA256SUMS.txt`;
- creates Sigstore keyless bundles using GitHub OIDC;
- verifies every bundle against the exact release workflow identity;
- generates GitHub artifact attestations;
- creates the public prerelease/tag bound to the exact source commit.

Windows Authenticode trusted-publisher signing remains unconfigured unless a trusted certificate/managed signing service is explicitly added.

## Published beta.8 evidence

`v0.2.0-beta.8` is bound to source commit `830e2a2fde13047d1b946fca1adae8edf0186bb0`.

The exact release line passed the applicable main-branch Windows/Rust/product workflows. The Windows release job completed build, privacy validation, Sigstore signing, signature self-verification, GitHub attestation, and prerelease creation. The P4 Apple Silicon workflow completed exact-main build/install/package verification; after a transient GitHub Release/tag visibility race failed before macOS upload, the failed attachment job was rerun once the tag was visible and then passed exact tag/SHA/prerelease validation, additive upload, public re-download, byte comparison, and checksum verification.

The visibility race is a workflow-orchestration defect, not artifact corruption; the post-beta.8 cleanup workflow waits for both Release and tag visibility before entering upload verification.

## Privacy tests

Generated markers detect accidental plaintext escape in:

- debug/display output;
- coordinator status/outcomes;
- settings errors/files;
- controlled E2E logs;
- package staging;
- release staging/metadata.

A privacy test never uses real user clipboard content. Evidence files contain only counts, categories, source SHA, run IDs, digests, environment labels, and explicit scope limitations.

## Physical and named-application evidence

Hosted CI must not be promoted into claims it cannot make.

### Windows

Issue #33 owns the exact-release interactive matrix: actual Trigger/Cancel, representative native/browser/VS Code/terminal targets, CJK/Unicode, focus changes, rate/jitter/cancellation measurements, typo/Backspace behavior, integrity boundaries, and privacy sentinel.

### macOS

Issue #61 owns physical Apple Silicon Accessibility/menu/shortcut/focus/input/login-item/privacy evidence and any future Developer ID/notarized promotion. Intel/Rosetta/Universal 2 are outside the current P4 support scope.

## Release-blocking failure categories

Publication/promotion is blocked by any unresolved failure involving:

- clipboard data loss or external-change overwrite;
- plaintext leakage or network transmission;
- target redirection or evidence degradation accepted as Same beyond the documented render-host policy;
- modifier release not owned by ClipType;
- automatic elevation or security-boundary bypass;
- retry after partial/unknown native input;
- unbounded work, wait, retry, or shutdown;
- package install/startup/uninstall residue;
- failed checksum/signature/attestation verification;
- compatibility wording broader than evidence;
- existing tag/release collision;
- publication bytes or tag/source identity that do not match the exact tested candidate.
