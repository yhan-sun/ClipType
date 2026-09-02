# ClipType Documentation

This index describes the current repository after completion of the P1 Windows vertical slice and P2 Windows productization work.

## Current status

- P1 controlled Windows clipboard-to-input evidence and the fail-closed target-evidence fix are merged into `main`.
- P2 implements the native Windows tray product, persistent settings, keyboard/clipboard/auto modes, startup integration, packaging, compatibility gates, and public beta release pipeline.
- `release/VERSION` declares the next public GitHub prerelease.
- Windows x86_64 is the only shipped architecture. Other platforms remain architectural research, not product claims.

## Product and support

- [Product](PRODUCT.md) — user intent, modes, product surface, and non-goals.
- [Compatibility](COMPATIBILITY.md) — supported Windows environments, application mechanism contract, and limitations.
- [Configuration](CONFIGURATION.md) — versioned per-user settings and bounds.
- [Release Process](RELEASE.md) — package contents, Sigstore signing, GitHub attestations, publication, and rollback.
- [Roadmap](ROADMAP.md) — completed P1/P2 milestones and later work.

## Architecture and safety

- [Architecture](ARCHITECTURE.md) — crate boundaries and runtime composition.
- [Injection Engine](INJECTION_ENGINE.md) — planning, target evidence, batching, cancellation, and outcome semantics.
- [Platform Backends](PLATFORMS.md) — native mechanisms and constraints.
- [Security and Privacy](SECURITY_PRIVACY.md) — clipboard confidentiality, diagnostics, privilege boundaries, and threat model.
- [Technology](TECHNOLOGY.md) — implementation choices and toolchain.
- [Architecture Decision Records](adr/README.md) — accepted cross-cutting decisions.

## Engineering process and evidence

- [Testing](TESTING.md) — deterministic, native, controlled E2E, compatibility, packaging, signing, and release evidence.
- [Development Workflow](DEVELOPMENT_WORKFLOW.md) — branch, review, and validation rules.
- [Dependency Policy](DEPENDENCY_POLICY.md) — dependency and license expectations.
- [References](REFERENCES.md) — primary platform/API references.
- [P1 Phase](phases/P1_WINDOWS_VERTICAL_SLICE.md) — Windows vertical-slice scope and gate.
- [P2 Phase](phases/P2_WINDOWS_PRODUCTIZATION.md) — Windows productization scope and gate.
- [P1 Automated Evidence](testing/P1_AUTOMATED_EVIDENCE.md) — historical P1 candidate evidence.

The final P2/public-release evidence report is recorded only after the exact candidate workflows complete; it must distinguish hosted controlled evidence from physical/named-application observations.

## Authority

For repository work, follow `AGENTS.md`. Current explicit maintainer/user instructions outrank older task text. Documentation must describe observed implementation and evidence, not planned or assumed behavior.
