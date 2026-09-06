# ClipType Documentation

This index describes the repository after the P1 Windows vertical slice, P2 Windows productization, the P3 cross-platform implementation work, and the P4 macOS Apple Silicon Flutter rebuild.

## Current status

- P1 controlled Windows clipboard-to-input evidence and the fail-closed target-evidence fix are merged into `main`.
- P2 implements the native Windows tray product, persistent settings, keyboard/clipboard/code/auto modes, human-paced typing controls, startup integration, packaging, compatibility gates, branding, and the public prerelease pipeline.
- `v0.2.0-beta.8` is the current public prerelease. Windows x86_64 remains the primary beta channel.
- The same `v0.2.0-beta.8` release carries a clearly labelled macOS Apple Silicon arm64 testing preview.
- The legacy Rust/Slint macOS composition root has been removed; `apps/cliptype-flutter` is the sole macOS settings/front-end entry point. The shared Slint crate remains only for the Windows product until a separate Windows UI decision.
- The earlier P3 Universal 2 / shared-macOS-UI plan is historical. P4 intentionally narrowed macOS scope to Apple Silicon arm64 and Flutter.
- Remaining release-acceptance work is physical/evidence based: representative Windows interactive validation is tracked in #33; physical Apple Silicon behavior and trusted Apple distribution promotion are tracked in #61.

## Product and support

- [Product](PRODUCT.md) — current shipped intent, modes, product surface, and non-goals.
- [Compatibility](COMPATIBILITY.md) — current Windows/macOS evidence contract and limitations.
- [Configuration](CONFIGURATION.md) — current versioned per-user settings and bounds.
- [Release Process](RELEASE.md) — package contents, Sigstore signing, GitHub attestations, publication, and rollback.
- [Roadmap](ROADMAP.md) — delivered phases and the remaining evidence/signing gates.

## Architecture and safety

- [Architecture](ARCHITECTURE.md) — current crate boundaries and runtime composition.
- [Injection Engine](INJECTION_ENGINE.md) — planning, target evidence, pacing, cancellation, and outcome semantics.
- [Platform Backends](PLATFORMS.md) — native mechanisms and constraints.
- [Security and Privacy](SECURITY_PRIVACY.md) — clipboard confidentiality, diagnostics, permission/privilege boundaries, and threat model.
- [Technology](TECHNOLOGY.md) — implementation choices, platform settings UI, native shells, and toolchain.
- [Architecture Decision Records](adr/README.md) — accepted cross-cutting decisions, including ADR-0010/0012 for the Flutter macOS composition root and ADR-0022 for the task-oriented macOS settings UX.

## Engineering process and evidence

- [Testing](TESTING.md) — deterministic, native, controlled E2E, compatibility, packaging, signing, and release evidence.
- [Development Workflow](DEVELOPMENT_WORKFLOW.md) — branch, review, and validation rules.
- [Dependency Policy](DEPENDENCY_POLICY.md) — dependency and license expectations.
- [References](REFERENCES.md) — primary platform/API references.
- [P1 Phase](phases/P1_WINDOWS_VERTICAL_SLICE.md) — historical Windows vertical-slice scope and gate.
- [P2 Phase](phases/P2_WINDOWS_PRODUCTIZATION.md) — historical Windows productization scope and gate.
- [P3 Phase](phases/P3_CROSS_PLATFORM_UI_MACOS.md) — historical shared-UI/Universal-2 plan and implementation record.
- [P4 Local macOS Phase](phases/P4_MACOS_ARM64_LOCAL.md) — the arm64-only Flutter/AppKit/Rust product line and its evidence boundary.
- [P1 Automated Evidence](testing/P1_AUTOMATED_EVIDENCE.md) — historical P1 candidate evidence.

Current release documentation describes the Windows beta and the clearly labelled
macOS arm64 testing preview. No Intel/Rosetta/Universal 2 artifact, trusted
Apple distribution, or arbitrary named-application compatibility is implied.

## Authority

For repository work, follow `AGENTS.md`. Current explicit maintainer/user instructions outrank older task text. Documentation must distinguish shipped behavior, accepted architecture, planned implementation, and actually observed evidence.
