## Task packet

- Issue / task ID:
- Roadmap phase / wave:
- Risk level: `R0 | R1 | R2 | R3 | R4`
- Base commit:
- Head commit:
- `Closes #...` only when every acceptance criterion is met:

## Summary

Describe the concrete outcome and the smallest change that provides it.

## Required context read

- [ ] `AGENTS.md`
- [ ] relevant normative documents
- [ ] relevant accepted ADRs
- [ ] current phase execution plan
- [ ] dependency task/PR handoffs

List the specific documents/ADRs/handoffs used:

## Scope

### In scope

-

### Explicitly out of scope

-

### Allowed-write boundaries

-

Explain any changed file outside the task packet's allowed-write area.

## Architecture and contracts

- [ ] No architecture decision changes
- [ ] Existing ADR(s) apply: <!-- list -->
- [ ] New/superseding ADR included
- [ ] Shared contracts unchanged
- [ ] Shared contract change is explicit and coordinated

Describe crate dependency impact, public contract changes, runtime/thread ownership changes, and intentionally deferred abstractions.

## Security and privacy

- [ ] Clipboard/injected plaintext is not logged or persisted
- [ ] Ordinary `Debug`/`Display`/error/status paths are content-free
- [ ] Permission and platform security boundaries are preserved
- [ ] Cancellation, target/focus, and modifier safety are considered
- [ ] Native input/retry operations are bounded
- [ ] Partial or unknown synthetic input is not blindly retried
- [ ] No unrelated keystrokes, focused text, or window titles are captured

Threat-model delta / notes:

## Native / unsafe / FFI review

Complete for native/unsafe work; otherwise write `Not applicable`.

- Native APIs/features:
- Ownership/lifetime rules:
- Pointer/buffer/count/encoding invariants:
- Thread or message-loop requirements:
- Cleanup/error translation:
- Safe-wrapper guarantee:
- Why the unsafe region is minimal:

## Platform evidence

| Platform/environment | Affected | Evidence type | Verified | Notes / limitations |
|---|---|---|---|---|
| Windows interactive desktop | | `CI / controlled E2E / manual` | | |
| Windows CI/build | | | | |
| Platform-neutral crates on non-Windows | | | | |
| macOS | | | | |
| Linux X11 | | | | |
| Linux Wayland | | | | |

Do not present headless CI as proof of interactive native input.

## Verification performed

Commands/checks actually run:

```text

```

Interactive/manual procedure and environment:

Measured bounds/results where applicable:

- batch/checkpoint settings:
- cancellation latency:
- focus/target behavior:
- modifier behavior:
- clipboard contention/payload limit:

## Privacy sentinel

- Fixture marker used:
- Locations searched:
- Result:

Do not paste real clipboard content into this PR.

## Not verified

List unavailable OS/session/application/race paths precisely. Do not write `all tests pass` when only a subset was executed.

## Documentation and compatibility

- [ ] No documentation change required, with reason
- [ ] Normative documentation updated
- [ ] Phase execution documentation updated
- [ ] Compatibility wording updated only from evidence
- [ ] Research report added/updated for a spike

## Dependency and license impact

- New/changed dependencies:
- Current-phase need:
- License compatibility:
- Transitive/unsafe/platform impact:
- Attribution/source-copy impact:

## Rollback, risks, and handoff

- Failure modes / rollback:
- Known limitations:
- Follow-ups intentionally out of scope:
- Exact next dependent task:

## Authority

- [ ] This PR does not merge, tag, publish, release, elevate privileges, or broaden support claims without explicit maintainer approval.
