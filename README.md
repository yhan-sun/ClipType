# ClipType

> Type your clipboard anywhere.

ClipType is a privacy-first, cross-platform desktop utility that reads text from the system clipboard and injects it into the currently focused application using native OS input facilities.

**Project status:** P0 documentation foundation is complete. P1 Windows Vertical Slice is the current implementation phase; no production implementation has been merged yet.

## Goals

- Cross-platform: Windows, macOS, Linux/X11, and Linux/Wayland.
- Native system integration rather than browser automation.
- Reliable text injection into the currently focused target.
- Multiple injection strategies: simulated keyboard input, clipboard paste, and automatic selection.
- Local-first and privacy-first: clipboard contents are not persisted by default and must never appear in logs.
- Small, auditable core with explicit platform adapters.
- Safe cancellation and focus-change handling so the user remains in control.

## Non-goals for the first releases

- Clipboard history or cloud sync.
- OCR, image injection, or rich-text transformation.
- AI rewriting or remote services.
- Account systems.
- Macro/scripting language compatibility with general-purpose automation suites.

## Documentation

The repository is documentation-first. Start with [`docs/README.md`](docs/README.md).

Current implementation sequencing is defined in [`docs/phases/P1_WINDOWS_VERTICAL_SLICE.md`](docs/phases/P1_WINDOWS_VERTICAL_SLICE.md). The phase tracking issue is [#1](https://github.com/yhan-sun/ClipType/issues/1).

Key documents cover product scope, architecture, platform backends, injection semantics, security/privacy, compatibility, testing, releases, roadmap, ADRs, and AI-agent contribution rules.

## Architecture direction

The planned architecture is a Rust core with native platform adapters:

```text
Clipboard / Hotkey / Focus Events
             |
             v
       Application Service
             |
       Injection Planner
             |
       Injection Engine
             |
   +---------+----------+-----------+
   |                    |           |
Windows Adapter    macOS Adapter   Linux Adapters
```

UI and packaging are deliberately kept outside the core domain. Platform-specific APIs are hidden behind explicit ports so core behavior can be tested independently.

## Rust workspace baseline

P1 creates exactly five packages: `cliptype-core`, `cliptype-platform`,
`cliptype-app`, `cliptype-windows`, and the `apps/cliptype` composition root.
The authoritative toolchain is pinned in `rust-toolchain.toml`.

`Cargo.lock` is committed because this workspace builds an application. Update it
only as part of a deliberate dependency change, and use `--locked` in CI.
P1-S01 disposable Windows mechanism experiments belong under
`crates/cliptype-windows/examples/`; their evidence belongs in `docs/research/`.

## Development policy

Implementation follows roadmap phase gates and the repository [`AGENTS.md`](AGENTS.md) contract. Changes that alter a documented architectural decision must add or supersede an ADR rather than silently changing behavior.

P1 begins with workspace bootstrap, a bounded interactive Windows native-mechanism spike, then contract freeze and parallel adapter work. Do not skip directly to a production `SendInput` implementation before the runtime/thread/focus assumptions are evidenced.

## License

ClipType is licensed under either the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT License](LICENSE-MIT), at your option. The SPDX expression is `MIT OR Apache-2.0`.

Intentional contributions accepted into this repository are licensed under the same terms. ClipType currently requires neither a CLA nor DCO sign-off. See [`docs/DEPENDENCY_POLICY.md`](docs/DEPENDENCY_POLICY.md) for dependency, reference-code, and attribution rules.
