# Controlled Windows target: message-pumped waits

## Failure and evidence boundary

Revalidating PR #72 against beta.7 main found a real first-attempt failure:

- Candidate: `10b34368c92bdca394cdccb07d2635e694e8429f`.
- Tested integration: `381630b5f92669b609da8232c1eab345f0db5005`.
- Common source tree: `756806fd3b60f5bddfe347cd5e98e8562bed78bb`.
- [Windows Compatibility Matrix run 33975891191, job 101332389007](https://github.com/yhan-sun/ClipType/actions/runs/33975891191/job/101332389007), attempt 1, Windows Server 2022 build 20348, image 20260830.290.1.
- Check, workspace tests and Clippy passed. P1 controlled input passed. P2 failed with `category=text_mismatch expected_utf16_units=73 observed_utf16_units=0`; the fixed fixture lengths identify the clipboard case. The executable build and artifact verification after that failed step were skipped, not passed.

The eight other workflows passing does not clear this failure. Preserve the failed run; do not rerun it until green and call the first attempt successful.

## Harness repair, not a production-policy workaround

The test owns its STATIC/EDIT target on the main thread, but blocked that same thread in `Coordinator::wait_for_idle` while the injection worker posted input. The target's message queue was only serviced after the coordinator finished. A window-owning test host must remain responsive during input, not behave like a blocked target.

The harness now pumps messages before each nonblocking idle observation, with the same eight-second timeout. Four unit tests cover pump-before-observe order, immediate timeout, progress across multiple pumps and failure of a permanently busy worker. The coordinator is triggered exactly once. Waiting for idle still does not establish delivery: the original exact target-text, expected-backend, completed-outcome and unchanged-clipboard-text/revision assertions remain mandatory, including the existing three-second target-text deadline.

Failure diagnostics expose only a fixed case label, expected backend, error category and Boolean foreground/focus/revision matches. They do not log fixture contents, user clipboard data, arbitrary window text, titles or native handles. No focus repair, chord replay, input retry, direct WM_PASTE injection, production backend change or relaxed assertion is introduced.

The blocked message loop is a concrete harness defect and a plausible contributor to the observed failure, not proof that all possible zero-delivery causes have been eliminated. Inspect the new diagnostics if a mismatch recurs; do not automatically classify another failure as environmental.

## Continuous verification

The compatibility workflow explicitly runs:

```sh
cargo test --locked -p cliptype --example p2_controlled_e2e
```

It then requires three fresh P2 processes on **each** of windows-2022 and windows-2025. Every process must exit zero and report exactly one success for each of keyboard, clipboard, auto-short and auto-long. Any failed, absent or duplicate case fails the job immediately. These are required repetitions, not success-on-retry. Privacy scanning occurs before captured test output is printed. The actual checked-out commit and tree are recorded separately from the PR head.

Full Rust, native Code contracts, P1/P2 E2E, benchmark, packaging, P3/P4 and publication-disabled release validation must be inspected against the new commit. The authoring environment has no Rust/Flutter toolchain or Windows desktop; only actually inspected CI runs establish those results. This document records the repair rationale, not a claimed passing run.

## References and remaining boundaries

Microsoft's [GetKeyboardState contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getkeyboardstate) associates keyboard state with message-queue consumption. [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) reports input-stream acceptance, not application text delivery. Neither is a reason to weaken the target assertion.

See [Code-mode gate evidence classes](CODE_MODE_CI_GATE.md) and [Code-navigation physical validation](CODE_NAVIGATION_FIX.md). Hosted Windows evidence is not a claim about every desktop application; mock Quartz/Swift contracts and arm64 packaging do not substitute for physical macOS Accessibility, editor input, focus, modifier and cancellation checks. Repository branch-protection administration, merging, tagging and publishing are outside this repair. No published beta.7 asset may be overwritten.
