# Release Strategy

## Versioning

Before 1.0, use semantic-version-like releases where minor versions may refine configuration/compatibility with clear release notes. At 1.0, follow Semantic Versioning for documented public interfaces and configuration compatibility.

## Channels

Planned channels:
- development/nightly artifacts: no stability promise;
- beta/pre-release: compatibility candidate for real users;
- stable: passes full release gate.

## Release gate

A stable release requires:

1. required CI/tests pass for claimed platforms;
2. compatibility matrix updated from evidence;
3. no known release-blocking security/privacy issue;
4. no clipboard plaintext in logs/artifacts;
5. platform permissions/onboarding documented;
6. packaging smoke-tested;
7. changelog/release notes list compatibility changes and known limitations;
8. dependency/license checks pass;
9. binaries are signed/notarized where the platform release policy requires it;
10. source commit/tag and produced artifacts are traceable.

## Platform artifacts

### Windows
Target signed installer/package plus checksums/signature metadata. Exact installer technology is selected during P2.

### macOS
Signed and notarized application distribution. Entitlements/Accessibility behavior must be documented and validated.

### Linux
Begin with a well-documented binary/archive for proven backends. Additional formats may include distro packages, AppImage, or Flatpak only when their sandbox/portal behavior matches advertised capabilities.

## Release notes must include

- user-visible changes;
- security/privacy changes;
- configuration migrations;
- new/removed compatibility claims;
- known limitations;
- permission/setup changes;
- contributor acknowledgements where appropriate.

## Rollback

Input/clipboard regressions can be safety-sensitive. Keep previous stable artifacts available until the new release is proven, and ensure configuration changes do not prevent reverting without documentation.

## Signing keys and secrets

Never store signing secrets in the repository. CI uses scoped secret storage and least privilege. Release workflows should produce provenance/attestations where practical once packaging exists.

## No autonomous release rule

AI agents and automation MUST NOT merge, tag, publish, or promote a release unless the maintainer explicitly requests that action.