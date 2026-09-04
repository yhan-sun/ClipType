# ClipType Documentation

This index describes the repository after the P1 Windows vertical slice, P2 Windows productization, the P3 cross-platform product plan, and the local P4 macOS Apple Silicon runner.

## Current status

- P1 controlled Windows clipboard-to-input evidence and the fail-closed target-evidence fix are merged into `main`.
- P2 implements the native Windows tray product, persistent settings, keyboard/clipboard/auto modes, human-paced typing controls, startup integration, packaging, compatibility gates, branding, and public prerelease pipeline.
- `v0.1.0-beta.1` is a Windows x86_64 GitHub prerelease.
- macOS is not part of `v0.1.0-beta.1`.
- P3 targets a graphical settings window, user-recorded Trigger/Cancel shortcuts with OS conflict probing and rollback, and a signed/notarized macOS Universal 2 product for `v0.2.0-beta.1`.
- P4 adds a real Flutter macOS settings/menu-bar candidate backed by the Rust runtime and restricted to local Apple Silicon arm64 validation. It is not a public macOS beta.
- The legacy Rust/Slint macOS composition root has been removed; `apps/cliptype-flutter` is now the sole macOS settings/front-end entry point. The shared Slint crate remains only for the Windows product until a separate Windows UI decision.

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
- [Technology](TECHNOLOGY.md) — implementation choices, platform settings UI, native shells, and toolchain.
- [Architecture Decision Records](adr/README.md) — accepted cross-cutting decisions, including ADR-0010 and ADR-0012 for the local Flutter macOS arm64 runner and sole macOS front end.

## Engineering process and evidence

- [Testing](TESTING.md) — deterministic, native, controlled E2E, compatibility, packaging, signing, and release evidence.
- [Development Workflow](DEVELOPMENT_WORKFLOW.md) — branch, review, and validation rules.
- [Dependency Policy](DEPENDENCY_POLICY.md) — dependency and license expectations.
- [References](REFERENCES.md) — primary platform/API references.
- [P1 Phase](phases/P1_WINDOWS_VERTICAL_SLICE.md) — Windows vertical-slice scope and gate.
- [P2 Phase](phases/P2_WINDOWS_PRODUCTIZATION.md) — Windows productization scope and gate.
- [P3 Phase](phases/P3_CROSS_PLATFORM_UI_MACOS.md) — graphical settings, custom shortcuts, macOS adapters, product shell, signing/notarization, and cross-platform prerelease gate.
- [P4 Local macOS Phase](phases/P4_MACOS_ARM64_LOCAL.md) — the arm64-only Flutter/AppKit/Rust candidate and its local evidence boundary.
- [P1 Automated Evidence](testing/P1_AUTOMATED_EVIDENCE.md) — historical P1 candidate evidence.

Current release documentation describes observed Windows behavior only. P3 documents the broader cross-platform release plan, while P4 records a local arm64 candidate and exact local evidence. Neither document implies that a public macOS beta, Universal 2 artifact, or arbitrary named-application compatibility is shipped.

## Authority

For repository work, follow `AGENTS.md`. Current explicit maintainer/user instructions outrank older task text. Documentation must distinguish shipped behavior, accepted architecture, planned implementation, and actually observed evidence.
