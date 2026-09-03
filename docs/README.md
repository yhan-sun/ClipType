# ClipType Documentation

This index describes the repository after the P1 Windows vertical slice, P2 Windows productization, and the start of P3 cross-platform product work.

## Current status

- P1 controlled Windows clipboard-to-input evidence and the fail-closed target-evidence fix are merged into `main`.
- P2 implements the native Windows tray product, persistent settings, keyboard/clipboard/auto modes, human-paced typing controls, startup integration, packaging, compatibility gates, branding, and public prerelease pipeline.
- `v0.1.0-beta.1` is a Windows x86_64 GitHub prerelease.
- macOS is not part of `v0.1.0-beta.1`.
- P3 targets a graphical settings window, user-recorded Trigger/Cancel shortcuts with OS conflict probing and rollback, and a signed/notarized macOS Universal 2 product for `v0.2.0-beta.1`.

## Product and support

- [Product](PRODUCT.md) — current shipped intent, modes, product surface, and non-goals.
- [Compatibility](COMPATIBILITY.md) — current Windows support contract and limitations.
- [Configuration](CONFIGURATION.md) — current versioned per-user settings and bounds.
- [Release Process](RELEASE.md) — package contents, Sigstore signing, GitHub attestations, publication, and rollback.
- [Roadmap](ROADMAP.md) — delivered P1/P2 and active P3 milestones.

## Architecture and safety

- [Architecture](ARCHITECTURE.md) — current crate boundaries and runtime composition.
- [Injection Engine](INJECTION_ENGINE.md) — planning, target evidence, pacing, cancellation, and outcome semantics.
- [Platform Backends](PLATFORMS.md) — native mechanisms and constraints.
- [Security and Privacy](SECURITY_PRIVACY.md) — clipboard confidentiality, diagnostics, permission/privilege boundaries, and threat model.
- [Technology](TECHNOLOGY.md) — implementation choices, shared settings UI, native shells, and toolchain.
- [Architecture Decision Records](adr/README.md) — accepted cross-cutting decisions, including ADR-0009 for the P3 UI/process model.

## Engineering process and evidence

- [Testing](TESTING.md) — deterministic, native, controlled E2E, compatibility, packaging, signing, and release evidence.
- [Development Workflow](DEVELOPMENT_WORKFLOW.md) — branch, review, and validation rules.
- [Dependency Policy](DEPENDENCY_POLICY.md) — dependency and license expectations.
- [References](REFERENCES.md) — primary platform/API references.
- [P1 Phase](phases/P1_WINDOWS_VERTICAL_SLICE.md) — Windows vertical-slice scope and gate.
- [P2 Phase](phases/P2_WINDOWS_PRODUCTIZATION.md) — Windows productization scope and gate.
- [P3 Phase](phases/P3_CROSS_PLATFORM_UI_MACOS.md) — graphical settings, custom shortcuts, macOS adapters, product shell, signing/notarization, and cross-platform prerelease gate.
- [P1 Automated Evidence](testing/P1_AUTOMATED_EVIDENCE.md) — historical P1 candidate evidence.

Current release documentation describes observed Windows behavior only. P3 documents planned behavior until implementation and exact-candidate evidence exist. It must not be used to imply that macOS or arbitrary custom shortcuts are already shipped.

## Authority

For repository work, follow `AGENTS.md`. Current explicit maintainer/user instructions outrank older task text. Documentation must distinguish shipped behavior, accepted architecture, planned implementation, and actually observed evidence.
