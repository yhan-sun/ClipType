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

Describe dependency direction, public contract changes, and deferred behavior.

## Runtime / ownership

Describe affected thread, event-loop, cancellation, resource, and shutdown ownership. State `not applicable` when genuinely irrelevant.

## Security and privacy

- [ ] Clipboard/injected plaintext is absent from ordinary logs and persistence
- [ ] Permission/security boundaries are preserved
- [ ] Cancellation/focus/modifier behavior was considered
- [ ] Native lengths, waits, retries, and batches are bounded where applicable
- [ ] Partial/unknown input is not retried

Notes:

## Unsafe / FFI

List every native API, pointer/buffer/count invariant, encoding rule, cleanup requirement, and safe-wrapper guarantee introduced. State `none` when applicable.

## Dependencies and licensing

List dependency changes, current-phase need, target scoping, license, transitive/unsafe impact, and attribution requirements.

## Verification

### Automated / CI

Commands and jobs actually run:

```text

```

### Controlled interactive evidence

Environment and behavior actually exercised on an unlocked interactive desktop:

### Manual representative-target observations

Application/version/category and observed result:

### Not verified

List unavailable OS/session/application/race paths.

### Expected by contract

List behavior supported by pure tests/docs but not executed end to end.

Do not treat hosted/headless CI as proof of interactive native input.

## Privacy sentinel / artifacts

Describe the synthetic marker and where logs/status/test/crash artifacts were checked. State `not applicable` only when no sensitive text path exists.

## Documentation / compatibility

- [ ] No documentation change required
- [ ] Documentation updated
- [ ] Compatibility updated from observed evidence only

## Rollback, risks, and next handoff

Describe failure modes, rollback, known limitations, and the exact next task/contract that may rely on this change.

## Agent / release authority

- [ ] No merge, tag, publish, release, automatic elevation, or broad support claim was performed without explicit authority
