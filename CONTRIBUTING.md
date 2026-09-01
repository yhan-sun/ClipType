# Contributing to ClipType

ClipType is documentation-first and architecture-driven. Contributions should be small, testable, privacy-preserving, and explicit about platform behavior.

## Before starting

Read `AGENTS.md`, `docs/README.md`, the roadmap phase you are working in, and relevant ADRs. For material architectural changes, open an issue/design discussion before implementation.

## Change classes

- **Core behavior:** platform-independent application policy and state machine.
- **Platform adapter:** Windows, macOS, X11, or Wayland native integration.
- **UI/shell:** tray, settings, onboarding, permission surfaces.
- **Infrastructure:** build, CI, packaging, release automation.
- **Documentation/ADR:** design, behavior, compatibility, or process changes.

Keep a PR primarily in one class when practical.

## Development principles

1. Preserve the privacy invariants in `docs/SECURITY_PRIVACY.md`.
2. Keep platform APIs behind adapter boundaries.
3. Prefer explicit capabilities and typed errors over silent fallback.
4. Do not promise untested platform/compositor compatibility.
5. Add tests for policy and regression behavior.
6. Update documentation with behavior changes.

## Commit style

Use conventional, concise prefixes where useful:

- `docs:` documentation only
- `feat:` user-visible capability
- `fix:` bug fix
- `refactor:` behavior-preserving structure change
- `test:` tests only
- `build:` build/packaging
- `ci:` CI automation

## Pull requests

A PR should explain:

- problem and scope;
- what is intentionally out of scope;
- architecture/ADR impact;
- security and privacy impact;
- platform compatibility impact;
- verification performed;
- remaining unverified paths;
- rollback strategy when behavior is risky.

Use `.github/PULL_REQUEST_TEMPLATE.md`.

## Architecture changes

Do not silently change an accepted architectural decision. Add a new ADR in `docs/adr/`, mark the old ADR superseded when appropriate, and update dependent documents.

## Contribution license

ClipType is dual-licensed under `MIT OR Apache-2.0`. Unless explicitly stated otherwise, any intentional contribution submitted for inclusion and accepted into the repository is licensed under the same terms (inbound equals outbound).

No Contributor License Agreement or Developer Certificate of Origin sign-off is currently required. This does not remove the contributor's responsibility to have the right to submit the contribution.

## Third-party and reference code

Do not copy source, tests, generated artifacts, documentation text, images, or other copyrightable material from reference projects without an explicit provenance, compatibility, and attribution review. Architecture and public behavior may be studied; implementation should prefer official platform documentation and original code.

Before adding a dependency or vendored component, follow [`docs/DEPENDENCY_POLICY.md`](docs/DEPENDENCY_POLICY.md). Missing, custom, copyleft, source-available, or distribution-sensitive terms require explicit maintainer review.
