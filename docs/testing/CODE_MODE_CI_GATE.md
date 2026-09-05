# Code-mode continuous regression gate

## Scope and baseline

This follow-up starts from `main@183cddd3b53037ffbb383f7a2b666bf955f13409`
(`v0.2.0-beta.6`). The native modifier-isolation fix originally proposed in
PR #68 is already present through PR #69. Do not merge the stale #68 branch
again or overwrite the separate macOS UI/UX work.

This change modifies tests, CI and evidence documentation only. Flutter remains
the sole macOS frontend. Code remains keyboard-only, with no Paste fallback;
physical modifiers, target changes, cancellation and partial/unknown outcomes
retain their existing fail-closed contracts. No release version is changed.

## Continuous checks

`Rust CI` retains its existing formatting, native-neutral and Windows workspace
jobs as merge-result integration checks. Its additional `Code-mode native
contracts (exact head)` job explicitly checks out the PR head, verifies
`git rev-parse HEAD` against the expected SHA, and runs:

```sh
python3 scripts/verify_code_navigation.py
python3 -m unittest discover -s scripts/tests -p 'test_code_navigation_gate.py' -v
```

The native contract compiles the production input functions against mock Quartz
with ASan and UBSan. Its 15 checks cover event ordering, explicit flags, private
sources, physical modifier observation, Unicode, allocation cleanup, permission
and Secure Input denial, clipboard revision guards and repeated navigation.

The six gate self-tests require the unmodified source to pass and verify that:

- combined-session synthetic event sources are rejected;
- combined-session physical-modifier observation is rejected;
- lingering Command flags on navigation key-up are rejected;
- missing compilers and missing source files fail rather than silently skip.

Mutation tests must reach the compiled native harness and produce a contract
failure. An unrelated compile error does not count as a caught regression.
Mutations exist only in temporary files; no actual keyboard events are posted.

The heap fixture tests additionally verify that missing keyboard navigation
cannot fall back to an available Paste backend, and that four repetitions
retain every function and the final tail with equivalent LF/CRLF plans.

## macOS quality and artifact identity

The P4 workflow is triggered by changes to regression scripts and the pinned
Rust toolchain as well as its existing application, crate and document paths.
It explicitly checks out and verifies the PR head, then executes the native
contract, its six self-tests, and `scripts/verify_bridge_completion.py` before
running the pinned Rust and Flutter gates.

The Swift check compiles the production snapshot mapping with the real bridge
header: 19 mapping/control cases and seven C ABI assertions. It is not a Rust
runtime link test. The macOS workspace tests separately exercise the Rust
completion mapping under `target_os = "macos"`.

The pinned Rust gate includes `fmt`, `check --workspace --all-targets --locked`,
`test --workspace --locked`, and Clippy with warnings denied. Flutter formatting,
analysis, tests, release build, arm64 architecture scans, ad-hoc signature checks,
install/launch smoke, ZIP/DMG verification and checksums remain required.

`BUILD-INFO-macos.txt` records the actual checked-out `source_commit` and
`source_tree`. `workflow_commit` separately records GitHub's event SHA, which
may be the synthetic merge commit on a PR. Do not present one as the other.
On a release-version push to main, the tested source and event commit coincide;
the existing additive publication and immutable-tag guards remain unchanged.

PR and manual runs do not publish. This task does not authorize merging,
tagging, changing `release/VERSION`, or publishing a release.

## Evidence and readiness checklist

Record the final PR head, base SHA, workflow run IDs, attempt numbers, actual
checkout identity and relevant job results. A pass on an earlier head is not a
pass on the final head. A missing, skipped, pending or cancelled validation job
is not a success. The intentional PR-only publication skip is not a missing
validation job.

Before an authorized merge/release:

- Verify the final head's native contracts, self-tests, Rust/Linux/Windows,
  controlled Windows E2E, package, benchmark, compatibility, product,
  release dry-run and P4 arm64 gates. Preserve both failed attempts and any
  justified rerun evidence; never substitute another commit's green run.
- Review main/head drift and the integration result. Revalidate after changing
  code or resolving conflicts. Do not force-push a shared branch.
- Match preview metadata and package checksums to the tested source. Rebuild
  release artifacts from the authorized final main commit; do not promote an
  earlier PR artifact as though it were the merge build.
- Obtain explicit merge/publication approval. A future release needs a fresh
  unused version/tag and matching notes; existing beta assets remain immutable.
- Keep physical Accessibility permission and real VS Code/Monaco input marked
  `NOT RUN` until exact-source/package evidence exists. Retain the arm64-only,
  ad-hoc-signed, unnotarized testing-preview boundary and all open physical gates.

The portable checks prove event construction, cleanup and mapping, not real
Quartz timing or destination text delivery. They never read the system
clipboard or a real editor document. Hosted CI does not establish physical
client compatibility, notarization or general macOS release readiness.
