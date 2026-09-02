# ADR-0008: Windows Beta Compatibility and Release Provenance

- Status: Accepted
- Date: 2026-09-02

## Context

P1 and P2 establish a complete Windows product path: bounded clipboard acquisition, target/integrity evidence, keyboard and current-clipboard paste backends, one-session coordination, cancellation, native tray/settings/startup integration, controlled E2E, benchmarking, packaging, and privacy checks.

A public beta needs two additional contracts:

1. a compatibility declaration that is broad enough for normal desktop use but does not convert hosted-runner or API evidence into a universal per-application guarantee; and
2. a cryptographic release-provenance model that can be implemented with the repository's current credentials.

The repository does not currently have a trusted Windows Authenticode certificate or managed signing account. Claiming a trusted Windows publisher would therefore be false. Publishing unsigned, unaudited archives would also be below the release requirements.

## Decision

### Compatibility scope

The first public release is a Windows x86_64 beta.

- Windows 11 x64 interactive desktop is the recommended supported client environment.
- Windows 10 22H2 x64 is best-effort API-compatible, with an explicit operating-system support/security caveat.
- Windows Server 2022 and 2025 Desktop Experience are mechanism-compatible CI environments, not substitutes for every physical client/application observation.
- ARM64, 32-bit Windows, Server Core, services, locked/non-interactive sessions, and cross-integrity injection are outside the release promise.

Application compatibility is stated by mechanism: ordinary editable targets that accept Unicode-oriented `SendInput` or standard `Ctrl+V`. Named frameworks and products are not automatically certified merely because they fall into that category.

The release retains explicit limitations for elevated targets, shared render hosts with multiple logical fields, terminal command semantics, rich clipboard formats, remote/virtual desktops, hotkey conflicts, and partial or unknown native progress.

### Release channel

The first public version is `v0.1.0-beta.1` and remains a GitHub prerelease. A stable release requires a separate gate and decision.

### Signing and provenance

The public release workflow:

1. checks out the exact `main` commit;
2. validates the declared version and release notes;
3. reruns workspace checks, tests, and Clippy;
4. builds the Windows x86_64 release executable;
5. produces a ZIP package, portable executable, dependency inventory, build metadata, and SHA-256 manifest;
6. signs each asset and manifest using Sigstore keyless signing with GitHub Actions OIDC;
7. verifies each Sigstore bundle before publication;
8. creates GitHub artifact attestations for primary assets;
9. creates the public GitHub prerelease and binds its tag to the exact source commit.

All external workflow actions are pinned to immutable commit SHAs. The release job receives only the minimum write/OIDC/attestation permissions needed for publication.

### Authenticode boundary

The first beta is not represented as Authenticode publisher-signed. Release notes and build metadata state this explicitly. Windows reputation or SmartScreen warnings may occur.

Adding Authenticode later requires a trusted certificate or managed signing service, isolated secret/key handling, timestamping, verification before packaging, and a new review of workflow permissions. Sigstore signatures and GitHub attestations remain in place even after Authenticode is added.

## Consequences

### Positive

- users can cryptographically verify assets without trusting a mutable checksum page;
- release provenance binds files to the repository workflow and exact commit;
- the compatibility promise is usable but does not exceed available evidence;
- no private signing key is stored in the repository or workflow secrets for the initial beta;
- publication is reproducible from `main` and cannot silently replace an existing tag/release;
- the absence of Authenticode is visible rather than hidden.

### Negative / trade-offs

- Windows does not display a trusted publisher for the first beta;
- hosted Windows Server evidence does not certify every Windows client build or named application;
- users need Cosign or GitHub CLI for full provenance verification;
- a later Authenticode integration still requires certificate procurement and operational controls;
- the release remains beta until broader field evidence and stable-version criteria are met.

## Rejected alternatives

### Claim universal Windows/application support

Rejected because target evidence, integrity boundaries, destination paste semantics, shared render hosts, and remote sessions can differ materially.

### Self-signed Authenticode certificate

Rejected as the primary public-signing claim because it does not establish a generally trusted Windows publisher and can mislead users about SmartScreen or certificate trust.

### Store a long-lived private signing key in repository secrets

Rejected for the initial beta because keyless OIDC signing provides a narrower credential surface and public transparency record.

### Publish only checksums

Rejected because checksums without an authenticated signer do not prove who produced the files.

### Replace assets under an existing release

Rejected because it breaks digest/provenance expectations. Fixes require a new version.
