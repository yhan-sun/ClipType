# P1 Automated Evidence

**Result:** `P1 AUTOMATED GATE READY`  
**Tested implementation commit:** `598309c37e07dfbc7656559e019ba4cbbcdd5cb5`  
**Evidence date:** 2026-09-01 UTC / 2026-09-02 Asia/Tokyo

This report records reproducible automated evidence for the Windows keyboard-only vertical slice. It deliberately separates hosted-runner results from physical interactive desktop and representative-application observations. The latter remain visible in follow-up issue #33 and block broad Windows support claims, public beta and release promotion.

## Environment

The Windows jobs ran on:

- Microsoft Windows Server 2025 Datacenter;
- build `10.0.26100`;
- runner image `windows-2025-vs2026`;
- image version `20260824.214.3`;
- Rust `1.98.0 (88d9e12ae 2026-08-18)`;
- target host `x86_64-pc-windows-msvc`;
- LLVM `22.1.8`.

The native-neutral job ran on Ubuntu 24.04.4 with the corresponding pinned Rust 1.98.0 toolchain.

## Final workflow evidence

All required workflows passed against implementation commit `598309c37e07dfbc7656559e019ba4cbbcdd5cb5`:

| Workflow | Run | Result | Evidence |
|---|---:|---|---|
| Rust CI | `33541490263` | success | formatting; Linux native-neutral check/test/Clippy; Windows workspace metadata/check/test/Clippy |
| P1 Windows Native Spike | `33541490240` | success | thread-owned registration, message queue/worker signal, teardown and content-free modifier observation |
| P1 Windows Host Smoke | `33541490253` | success | real host build, two-hotkey registration, private cross-thread shutdown, unregister and clean exit |
| P1 Controlled Windows E2E | `33541491123` | success | real clipboard-to-coordinator-to-bounded-`SendInput` path into a controlled Win32 edit target, repeated three times |

The Rust CI commands were:

```text
cargo fmt --all -- --check
cargo check --locked --all-targets -p cliptype-core -p cliptype-platform -p cliptype-app
cargo test --locked -p cliptype-core -p cliptype-platform -p cliptype-app
cargo clippy --locked --all-targets -p cliptype-core -p cliptype-platform -p cliptype-app -- -D warnings
cargo metadata --locked --format-version 1 --no-deps
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The Windows workspace executed 58 non-documentation tests:

- 24 core policy/contract tests;
- 3 platform-contract tests;
- 18 coordinator integration tests across two suites;
- 13 Windows adapter/event-source tests.

All 58 passed. The Linux native-neutral subset executed 45 tests and passed.

## Automated layers

### Deterministic native-neutral policy and coordinator tests

The workspace verifies:

- redacted sensitive-text and target diagnostics;
- strongly typed native/payload/batch/retry bounds;
- ASCII, CJK, supplementary Unicode and combining-mark preservation;
- CRLF, lone CR and LF normalization;
- explicit Tab/control policy;
- immutable bounded keyboard plans;
- one-session arbitration;
- target capture before clipboard acquisition;
- cancellation during modifier settling, clipboard retries and active work;
- bounded clipboard retry exhaustion;
- payload and capability rejection;
- target failure/change/unavailable evidence before and between batches;
- modifier settle timeout and later modifier conflict;
- complete, none, partial and progress-unknown dispatch decisions;
- no retry after partial/progress-unknown native input;
- known integrity restriction versus blocked-cause-unknown;
- bounded shutdown and return to idle;
- content-free status and privacy sentinel behavior.

### Windows adapter tests

Windows jobs compile, lint and test:

- bounded `CF_UNICODETEXT` allocation-size, terminator and UTF-16 validation;
- clipboard ownership/copy/cleanup wrappers;
- foreground/process/thread/native-focus evidence;
- degraded versus detailed target comparison;
- query-only process integrity evidence;
- Unicode `SendInput` construction for BMP and supplementary scalars;
- line-break and Tab key pairs;
- conservative accepted-event classification;
- impossible count failure;
- modifier observation without releasing user keys;
- owned hotkey message translation;
- thread message-queue signalling and registration cleanup.

### Windows host smoke

`P1 Windows Host Smoke` starts the real development executable on a Windows hosted runner. The executable registers both global development hotkeys, receives a private shutdown message posted from another thread, unregisters both hotkeys and exits cleanly.

This proves the concrete process/message-loop/teardown composition in that environment. It does not prove that a human physically activated the trigger or cancellation hotkey from another foreground application.

### Controlled Win32 edit E2E

`P1 Controlled Windows E2E` creates an isolated visible multiline Win32 edit target, installs a generated clipboard fixture, activates/focuses the controlled target, and runs the real:

```text
generated CF_UNICODETEXT
  -> Coordinator
  -> WindowsClipboard
  -> normalization and keyboard plan
  -> WindowsTarget revalidation
  -> WindowsKeyboard bounded SendInput batches
  -> controlled Win32 edit queue
  -> UTF-16 assertion
```

The final run executed three isolated repetitions. Each produced only the content-free record:

```text
result=ok expected_utf16_units=71 observed_utf16_units=71 batches=9
```

All three repetitions matched 71 expected UTF-16 units, observed 71 UTF-16 units and completed 9 bounded semantic batches.

The fixture covers ASCII, punctuation, CJK, supplementary Unicode, a combining mark and normalized line-break behavior. Tab/control-policy behavior remains covered by deterministic and native event-construction tests because physical Tab intentionally changes focus in many desktop controls.

## Ordering and lifecycle conclusions

The automated evidence establishes:

- one session is reserved before worker creation;
- initial target capture precedes every clipboard call;
- the message-loop owner performs no clipboard retry or synthetic text dispatch;
- cancellation is independently signalable while work is active;
- retries, modifier settling, inter-batch waits and shutdown are bounded;
- known target change, disappearance or unavailable evidence stops later dispatch;
- physical modifier keys are observed but never forcibly released;
- each `SendInput` invocation contains one bounded batch;
- partial/progress-unknown input is never retried;
- known higher-integrity restriction is not conflated with an unexplained zero native result;
- contained failures return the coordinator to idle;
- hotkey registration is cleaned up on controlled host shutdown.

## Privacy method and result

The controlled workflow uses the generated distinctive marker `CLIPTYPE_E2E_PRIVATE_SENTINEL_7841`. Its content is permitted only inside the controlled clipboard and target assertion buffers.

Ordinary runtime and test output is limited to result categories, UTF-16 unit counts, generation, batch counts and lifecycle/timing categories. The workflow searched its collected output and failed if the distinctive marker appeared. The final run passed this check.

The production host and coordinator do not emit clipboard text, injected text, samples, prefixes/suffixes, persistent content fingerprints, focused-field content or window titles.

## Evidence boundary and deferred work

A successful controlled hosted-runner E2E proves the complete implementation path in that constrained Windows environment. It does not prove:

- human physical global-hotkey activation from another foreground application;
- Chromium-family browser behavior;
- Electron/VS Code behavior;
- Windows Terminal or equivalent terminal behavior;
- logical-field movement hidden inside one native render host;
- elevated target behavior in an unlocked user desktop;
- measured physical trigger-modifier release and cancellation timing.

All of those obligations are preserved in issue #33. They are not silently marked as passed and must be completed before `WINDOWS BETA READY`, broad compatibility claims or release promotion.

## Final recommendation

The implementation and reproducible automated P1 gate are ready for final architecture/security/privacy review:

```text
P1 AUTOMATED GATE READY
```

This recommendation authorizes neither tag, publish, public beta nor release. Representative interactive compatibility remains gated by #33.
