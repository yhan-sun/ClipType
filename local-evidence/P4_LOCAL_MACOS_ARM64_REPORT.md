# ClipType P4 Local Apple Silicon Report

## 1. Final verdict

`LOCAL P4 RUN PARTIAL`

The Flutter counter scaffold was replaced by a real ClipType UI and the
arm64-only release candidate built, launched, inspected, and passed the
automated quality/package checks listed below. The physical Accessibility,
target-application, Unicode, conflict, cancellation, and latency cases were
not all run, so this is not a macOS beta-ready result.

## 2. Source

- Repository: ClipType
- PR: #64
- Remote main HEAD: `609bc44a830d6f80405ce645227c7bd85b333559`
- Remote PR HEAD: `2902773cbc76fb6b31e7214ae5a5f401a40b105a`
- Local final HEAD: `2902773cbc76fb6b31e7214ae5a5f401a40b105a` (working-tree implementation; no push or merge)
- Local branch: `local/p4-arm64-run`
- Working tree: implementation and evidence remain uncommitted locally for review; no remote mutation was performed.
- Run date: `2026-09-04`
- Product state at start: `SCAFFOLD_ONLY`
- Product state at finish: `INTEGRATED_CANDIDATE`

## 3. Environment

- macOS: `26.6.2`, build `25G83`
- Architecture: `arm64`
- Xcode: `26.3`
- Swift: `6.2.4`
- Flutter: `3.47.2`
- Dart: `3.13.2`
- Rust: `1.98.0-aarch64-apple-darwin`
- Session: unlocked physical desktop
- Final post-build CUA re-inspection: BLOCKED because macOS was locked and could not be automatically unlocked; earlier unlocked UI inspection is recorded below.

### Toolchain notes

`flutter doctor -v` exited 0 but reported two non-desktop-blocking warnings:
the exact pinned Flutter tag checkout is shown as an unknown channel, and
Xcode could not enumerate installed Simulator runtimes. The first PATH warning
from an initial invocation was corrected before the quality gates; all actual
Flutter commands used the pinned SDK. The macOS desktop target and Release
build passed.

## 4. Implementation completed

- Flutter UI: General, Shortcuts, Typing, Permissions, and About pages with dark/light-aware Material controls, focused local shortcut recording, and English/Simplified Chinese display mode.
- MethodChannel/EventChannel: fixed `io.cliptype/native` and `io.cliptype/events`; commands and events are content-free.
- Swift/AppKit shell: one Settings window, one `NSStatusItem`/menu, native lifecycle, Carbon hotkeys, Accessibility onboarding/state, `SMAppService`, and fixed bridge ownership.
- Rust C ABI bridge: `cliptype-flutter-bridge` static library with bounded settings/status/command functions; panics do not cross FFI.
- Global hotkeys: transactional Trigger/Cancel registration and rollback path; current-pair OS availability was observed as available.
- Accessibility: explicit request/status path; the run observed `Not granted` without changing system security settings.
- CJK/Unicode: Rust Auto policy and macOS bounded adapters are implemented; Auto now prefers revision-guarded Command+V for any non-ASCII text, including short CJK, emoji, combining, and mixed-Unicode payloads. The physical CJK/Unicode target matrix is not run.
- Performance changes: no permanent idle 40 ms poll; active-session observation is bounded.
- ARM64 packaging: `ClipType.app`, arm64 main executable and bundled Mach-O scan passed.

## 5. Automated checks

| Check | Status | Command/evidence |
|---|---|---|
| Flutter format | PASS | `dart format --output=none --set-exit-if-changed lib test` |
| Flutter analyze | PASS | `flutter analyze` — no issues |
| Flutter tests | PASS | `flutter test` — 5 tests, including Simplified Chinese localization vocabulary |
| Rust fmt | PASS | `cargo +1.98.0 fmt --all -- --check` |
| Rust check | PASS | workspace, locked, all targets, `aarch64-apple-darwin` |
| Rust test | PASS | workspace, locked, `aarch64-apple-darwin`; all executed tests passed |
| Rust Clippy | PASS | workspace, all targets, locked, arm64, `-D warnings` |
| CJK/Unicode Auto policy regression | PASS | Rust core test covers short CJK, Japanese, Korean, emoji, combining, and mixed-Unicode payloads |
| Flutter release build | PASS | `flutter build macos --release` |
| Xcode/Swift link | PASS | release app linked the Rust static library and launched |
| ARM64-only scan | PASS | main executable and all 3 bundled Mach-O files reported `arm64` only |
| codesign integrity | PASS | `codesign --verify --deep --strict --verbose=2` |
| Hosted P4 workflow | NOT RUN | `.github/workflows/p4-macos-arm64.yml` was added for PR/manual execution; it was not dispatched in this local run |

The release candidate is unsigned locally; codesign PASS here means bundle
integrity verification, not Developer ID trust or notarization.

## 6. Interactive matrix

| Case | Target | Mode | Status | Content-free observation |
|---|---|---|---|---|
| Real product UI | ClipType Settings | n/a | PASS | General page displayed ClipType controls, not the Flutter counter demo. |
| Page navigation | Settings | n/a | PASS | General, Shortcuts, Typing, Permissions, and About pages were inspected. |
| Shortcut OS probe | Settings | n/a | PASS | Trigger and Cancel were reported `Available`; app-local conflicts remain unverifiable. |
| Recorder Escape | Settings | n/a | PASS | Recording prompt appeared; Escape restored the prior value. |
| Recorder Delete clear | Settings | n/a | PASS | Delete cleared Trigger; validation reported both shortcuts required; Reset restored the default. |
| Physical full recorder combo | Settings | n/a | BLOCKED | Desktop automation did not establish delivery of the full combination; widget tests pass, no physical PASS claimed. |
| Permissions display | Settings | n/a | PASS | Unlocked inspection accurately displayed the no-trust state with fixed remediation text; final code now distinguishes `Not requested` from `Not granted`. |
| Simplified Chinese UI mode | Settings | n/a | NOT RUN | Localization code and vocabulary test pass; physical visual inspection of the switched UI was not run after the desktop became locked. |
| Chinese | TextEdit/browser/editor | Auto | NOT RUN | Requires permission and a non-sensitive controlled fixture. |
| Keyboard Unicode | TextEdit | Keyboard | NOT RUN | Requires permission and target assertions. |
| Clipboard | TextEdit | Clipboard | NOT RUN | Requires permission, revision checks, and target assertions. |
| Trigger visible/hidden | Settings/external target | n/a | NOT RUN | OS registration was probed; end-to-end input was not run without Accessibility. |
| Conflict rollback | Controlled owner | n/a | NOT RUN | No independent conflict owner was started. |
| Cancel/focus/revision | Long session/target | n/a | NOT RUN | No physical session fixture was run. |
| Hide/reopen lifecycle | ClipType process | n/a | PASS | Hidden/closed Settings left the ClipType process alive; reopening showed one Settings window/status item. |
| Quit process | ClipType process | n/a | PASS | Quit request was followed by no running ClipType process; the standard AppleEvent command also returned macOS `-128`, so the menu-item path is not independently isolated. |
| Final clean-release visual re-inspection | ClipType Settings | n/a | BLOCKED | The relaunch succeeded at the process level, but CUA could not inspect the locked desktop. |

## 7. Performance

| Metric | Trials | Min | Median | Max | Budget | Result |
|---|---:|---:|---:|---:|---:|---|
| Cold start | 5 | — | — | — | 2500 ms | NOT RUN |
| Settings reopen | 10 | — | — | — | 150 ms | NOT RUN |
| Idle CPU | 5 samples | 0.0% | 0.0% | 0.0% | 2% | PASS for observed sample |
| Idle RSS | 5 samples | 107 MiB | 107 MiB | 107 MiB | 350 MiB | PASS for observed sample |
| Trigger latency | 5 | — | — | — | recorded | NOT RUN |
| Cancel latency | 5 | — | — | — | recorded | NOT RUN |

The idle sample was collected after the app had been stable for approximately
30 seconds. It is a smoke sample, not a repeated-start performance study.

## 8. Privacy

- Sentinel absent: NOT RUN with a physical clipboard marker; no marker was put in this report or command log.
- Clipboard unchanged: NOT RUN; no physical paste transaction was executed.
- No plaintext in logs/config/crash evidence: PASS by code review and content-free bridge/status design; no real clipboard data was used.
- No global event tap/keylogger: PASS by implementation inspection; the recorder is local-focus-only and no broad monitor is installed.

## 9. Blocking findings

| ID | Severity | Reproduction | Root cause | Fix/status |
|---|---|---|---|---|
| P4-01 | High for end-to-end claim | Open Permissions without granting trust | Accessibility was intentionally not changed during this run | Honest `Not granted` state; user-controlled grant/revoke evidence remains open. |
| P4-02 | High for beta claim | Attempt complete physical matrix | No controlled target/permission fixture was executed | Keep final verdict partial; do not claim beta readiness. |
| P4-03 | Medium | Deliver full recorder combo through desktop automation | Input path did not establish the combo | Widget regression coverage passes; physical recorder evidence remains open. |

## 10. Not verified

See [P4_LOCAL_FAILURES.md](P4_LOCAL_FAILURES.md). In particular: physical
Accessibility grant/revoke, Chinese and other Unicode targets, explicit
Keyboard/Clipboard/Auto delivery, conflict rollback, focus/target changes,
long-session Cancel, Start at Login mutation, repeated timing trials, physical
full-combination recording, menu-level Quit isolation, hosted CI execution, and
final clean-release visual re-inspection, and signing/notarization.

## 11. Recommendation

Do not write `MACOS ARM64 BETA READY`. Keep this branch as a local
`INTEGRATED_CANDIDATE` until the missing physical evidence is run on an
approved non-sensitive fixture and attached to an exact source revision.
