# Release Process

## Public release channel

The first public channel is the Windows x86_64 beta declared by `release/VERSION`. A new public release is produced only when that file changes on `main` or a maintainer explicitly dispatches the release workflow.

The release workflow is `.github/workflows/windows-release.yml`. It rebuilds from the exact `main` commit, re-runs workspace checks/tests/Clippy, assembles the package, creates a dependency inventory and build metadata, signs the assets, generates GitHub artifact attestations, and creates an immutable GitHub prerelease.

## Required assets

Every Windows beta release contains:

- `ClipType-<version>-windows-x86_64.zip` — per-user package;
- `ClipType-<version>-windows-x86_64.exe` — portable executable;
- `DEPENDENCIES.json` — locked Cargo dependency and license metadata;
- `BUILD-INFO.txt` — release version, source commit, target, toolchain, and signing boundary;
- `SHA256SUMS.txt` — SHA-256 digests for the published primary assets;
- one `.sigstore.json` bundle for every primary asset and manifest;
- release notes with compatibility and known limitations;
- repository-hosted GitHub artifact attestations for the primary assets.

Licenses, configuration reference, release notes, install/uninstall scripts, dependency inventory, and build metadata are also embedded in the ZIP package.

## Signing model

### Sigstore release signing

The release archive, portable executable, dependency inventory, build metadata, and checksum manifest are signed using keyless Sigstore signing. The identity is the exact release workflow on `refs/heads/main`, authenticated by GitHub Actions OIDC. The workflow verifies every generated bundle before publication.

Users can verify a downloaded file with Cosign:

```text
cosign verify-blob \
  --bundle <asset>.sigstore.json \
  --certificate-identity "https://github.com/yhan-sun/ClipType/.github/workflows/windows-release.yml@refs/heads/main" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  <asset>
```

### GitHub artifact attestation

GitHub artifact attestations bind each primary asset to the repository, workflow, and source commit. They can be verified with:

```text
gh attestation verify <asset> --repo yhan-sun/ClipType
```

### Authenticode boundary

A Sigstore signature proves release-workflow identity and artifact integrity; it does not make Windows display a trusted software publisher. The first beta does not claim Authenticode publisher signing because no trusted Windows code-signing certificate or managed signing account is configured.

When a trusted certificate is configured, Authenticode signing must occur in an isolated release environment after build and before packaging, use timestamping, verify the certificate chain and signature, and preserve the Sigstore and GitHub provenance layers. Until then, release notes must state that Windows reputation or SmartScreen warnings may appear.

## Release gate

Publication is blocked unless all of the following are true:

1. the P1 safety and controlled-E2E dependencies are merged into `main`;
2. the P2 product PR is based on current `main` and is no longer a draft;
3. formatting, native-neutral checks/tests/Clippy, Windows workspace checks/tests/Clippy, native spike, host smoke, P1/P2 controlled E2E, backend benchmark, package smoke, and compatibility matrix succeed for the candidate;
4. compatibility wording matches the actual evidence and retains elevated-target, logical-field, terminal, remote-session, and architecture limitations;
5. no release-blocking privacy, target-safety, data-loss, privilege, packaging, or signing issue remains open;
6. the release workflow dry run successfully builds the exact asset set;
7. `release/VERSION` and its matching release notes exist and the tag does not already exist.

## Versioning

Release tags use semantic versions. Pre-1.0 public productization releases use prerelease identifiers such as `v0.1.0-beta.1`. The tag is created from the exact published `main` commit by the release workflow.

The Git tag itself is not claimed to be a maintainer GPG/SSH-signed tag. Integrity is provided by signed release assets, checksums, GitHub artifact attestations, immutable workflow/run records, and the release-to-source commit binding.

## Rollback and revocation

Published assets are never silently replaced. If an issue is found:

- mark the affected release clearly in release notes or a security advisory;
- preserve the original assets and provenance for auditability;
- publish a new patch/beta version rather than mutating the old release;
- remove a release only when publication itself exposed sensitive data, malware, or a legal/safety issue that requires removal;
- document the affected versions and remediation without exposing clipboard content.

## Maintainer checklist

Before changing `release/VERSION`:

- review the exact compatibility evidence and known limitations;
- verify release notes and dependency/license inventory;
- confirm no plaintext or privacy sentinel appears in distributable files;
- confirm installer/startup/uninstaller rollback behavior;
- verify release workflow action pins and permissions;
- confirm the release is correctly marked prerelease until the stable gate is separately met.
