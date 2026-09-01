# ClipType

> Type your clipboard anywhere.

ClipType is a privacy-first, cross-platform desktop utility that reads text from the system clipboard and injects it into the currently focused application using native OS input facilities.

**Project status:** documentation and architecture design. No production implementation has started yet.

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

The repository is intentionally documentation-first. Start with [`docs/README.md`](docs/README.md) once the documentation foundation commit lands.

Key documents will cover product scope, architecture, platform backends, injection semantics, security/privacy, compatibility, testing, releases, roadmap, ADRs, and AI-agent contribution rules.

## Architecture direction

The planned architecture is a Rust core with native platform adapters:

```text
Clipboard / Hotkey / Focus Events
             |
             v
       Application Engine
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

## Development policy

No implementation should begin until the documentation foundation and initial architecture decisions are reviewed. Changes that alter a documented architectural decision must add or supersede an ADR rather than silently changing behavior.

## License

License selection is intentionally deferred until the first implementation milestone. Do not copy source code from reference projects unless the selected license and attribution requirements have been reviewed.
