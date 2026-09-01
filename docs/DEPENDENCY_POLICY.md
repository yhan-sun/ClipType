# Dependency and Third-Party Code Policy

## Project license

ClipType is licensed under either of the following licenses, at the recipient's option:

- Apache License, Version 2.0 (`Apache-2.0`); or
- MIT License (`MIT`).

The repository SPDX expression is `MIT OR Apache-2.0`.

## Contribution licensing

Unless explicitly stated otherwise, an intentional contribution submitted for inclusion in ClipType is licensed under the same `MIT OR Apache-2.0` terms (inbound equals outbound).

ClipType does not currently require a Contributor License Agreement or Developer Certificate of Origin sign-off. The maintainer may introduce a reviewed governance change later; it must not be applied retroactively without a clear transition policy.

A submission conspicuously marked `Not a Contribution` is not accepted as project code merely because it appears in an issue or discussion.

## Dependency review

Every dependency requires a current roadmap-phase need, compatible licensing, maintained provenance, bounded transitive and unsafe impact, and appropriate target scoping.

### Normally acceptable after routine review

Permissive dependencies under well-known licenses such as `MIT`, `Apache-2.0`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, or `Zlib` may be accepted when their terms and notices are compatible with the distribution being built.

### Explicit maintainer review required

The following require explicit license and distribution review before addition:

- copyleft licenses, including GPL, LGPL, AGPL, MPL, EPL, and CDDL families;
- source-available, non-commercial, field-of-use, or otherwise non-open-source terms;
- custom, ambiguous, missing, or unrecognized licenses;
- dependencies with license exceptions or multiple-choice expressions whose selected path is unclear;
- vendored native binaries, SDK redistributables, fonts, datasets, media, generated bindings with special terms, or copied source;
- dependencies that require a `NOTICE`, attribution UI, relinking mechanism, source offer, or other distribution obligation.

P1 uses a default-deny posture for dependencies in this category until the review is recorded. This policy is an engineering gate, not legal advice.

## Build, development, and runtime dependencies

Build-only and development dependencies remain part of the supply-chain inventory even when they are not shipped. Runtime and bundled dependencies receive additional release-artifact review because their notices, source, or license files may need to accompany distributions.

`Cargo.lock` is committed for the application workspace. Dependency changes must update the lockfile deliberately and CI must use `--locked`.

## Reference projects and copied code

Reference projects may be studied for architecture, public behavior, interoperability, and native API usage. Source, tests, generated artifacts, documentation text, images, or other copyrightable material must not be copied unless all of the following are recorded:

1. exact provenance and version/commit;
2. applicable license and compatibility analysis;
3. required copyright and attribution notices;
4. whether the material was modified;
5. the destination release-artifact obligations.

A clean-room implementation from official platform documentation is preferred. Similarity to a reference implementation must not be hidden.

## Attribution inventory

When a dependency or copied component requires attribution, record it in a future `THIRD_PARTY_NOTICES` inventory before a distributable artifact is produced. Do not add an empty `NOTICE` or imply that no notices are required without inspecting the actual dependency graph.

## Review ownership

The task author records the initial license assessment. Reviewers verify it for native, security-sensitive, vendored, or distributed dependencies. The maintainer owns final exceptions and release decisions.
