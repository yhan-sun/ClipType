# ClipType

ClipType is a privacy-first Windows tray utility that reads the current clipboard only after an explicit trigger and delivers it to the current destination through bounded native input.

The repository also contains the local macOS Apple Silicon candidate. On macOS,
`apps/cliptype-flutter` is the only settings/front-end composition root; the
legacy Rust/Slint macOS application has been removed. The Rust core and macOS
adapters remain shared runtime components.

## Public beta

The first public release channel is `v0.1.0-beta.1` for Windows x86_64.

- **Recommended client:** Windows 11 x64 interactive desktop.
- **Best-effort client:** Windows 10 22H2 x64, with an explicit operating-system support/security caveat.
- **CI reference environments:** Windows Server 2022 and Windows Server 2025 Desktop Experience.
- **Not shipped:** Windows ARM64, 32-bit Windows, Server Core, services, and non-interactive sessions.

See [Compatibility](docs/COMPATIBILITY.md) for the exact support contract and limitations.

## Product modes

- `keyboard` — bounded Unicode-oriented `SendInput` batches with target, modifier, cancellation, and partial-progress guards.
- `clipboard` — verifies the current clipboard revision and sends one ordinary `Ctrl+V`; ClipType never rewrites or restores the clipboard.
- `auto` — freezes one proven backend per session from Unicode shape, payload size, and available capabilities; non-ASCII text prefers guarded paste.

Explicit modes never silently fall back.

## Safety and privacy

ClipType:

- has no clipboard history and no continuous plaintext watcher;
- does not persist or transmit clipboard/injected text;
- does not read focused-field contents or log window titles;
- captures destination evidence before clipboard acquisition and revalidates it before dispatch;
- runs as one normal, unprivileged user process;
- does not auto-elevate or bypass Windows integrity boundaries;
- never blindly retries partial or progress-unknown synthetic input;
- keeps clipboard reads, modifier waits, native batches, cancellation, and shutdown bounded.

Clipboard text can contain secrets or operational commands. Review it before triggering input, especially in terminals or administrative tools.

## Windows user experience

The Windows product provides:

- native notification-area tray icon and context menu;
- global trigger and independent cancel hotkey presets;
- enabled, notification, mode, speed, and start-at-login settings;
- strict versioned per-user configuration with backup recovery;
- current-user installation and uninstallation without elevation;
- content-free status notifications and controlled shutdown.

Configuration is stored under the current user's local application data directory. See [Configuration](docs/CONFIGURATION.md).

## Install and verification

Public release assets include a ZIP package, portable executable, SHA-256 manifest, dependency inventory, build metadata, Sigstore bundles, and GitHub artifact attestations.

The first beta uses Sigstore keyless signing and GitHub provenance. It is not represented as Authenticode publisher-signed, so Windows reputation or SmartScreen warnings may appear. See [Release Process](docs/RELEASE.md) and the matching release notes for verification commands.

## Development

Prerequisites:

- Rust `1.98.0` as pinned by `rust-toolchain.toml`;
- Windows for native adapter/product tests;
- PowerShell for Windows packaging scripts.

Native-neutral checks:

```text
cargo fmt --all -- --check
cargo check --locked --all-targets -p cliptype-core -p cliptype-platform -p cliptype-app
cargo test --locked -p cliptype-core -p cliptype-platform -p cliptype-app
cargo clippy --locked --all-targets -p cliptype-core -p cliptype-platform -p cliptype-app -- -D warnings
```

Full Windows workspace gate:

```text
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --release --locked -p cliptype
```

Controlled E2E harnesses are opt-in because they temporarily place generated fixtures on the current clipboard. The repository workflows set the required opt-in variables in isolated CI jobs.

## Repository map

```text
apps/cliptype/          Windows product composition root and controlled harnesses
apps/cliptype-flutter/  macOS Apple Silicon Flutter settings/menu-bar app
crates/cliptype-core/   platform-independent policy, limits, state, and settings vocabulary
crates/cliptype-app/    coordinator and persistent settings application boundary
crates/cliptype-platform/ native-neutral ports and result contracts
crates/cliptype-windows/ Win32 clipboard, target, input, hotkey, tray, paste, and startup adapters
crates/cliptype-macos/  macOS clipboard, target, input, hotkey, permission, and startup adapters
crates/cliptype-flutter-bridge/ fixed content-free Rust ABI used by the Flutter shell
packaging/windows/      per-user install/uninstall package scripts
docs/                   architecture, security, compatibility, testing, ADRs, and release process
```

## Documentation

Start with [Documentation Index](docs/README.md), then read:

- [Product](docs/PRODUCT.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Injection Engine](docs/INJECTION_ENGINE.md)
- [Security and Privacy](docs/SECURITY_PRIVACY.md)
- [Compatibility](docs/COMPATIBILITY.md)
- [Testing](docs/TESTING.md)
- [Release Process](docs/RELEASE.md)
- [Architecture Decision Records](docs/adr/README.md)

## License

Licensed under either Apache License 2.0 or MIT, at your option. See `LICENSE-APACHE` and `LICENSE-MIT`.
