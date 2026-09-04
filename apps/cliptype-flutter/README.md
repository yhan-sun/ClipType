# ClipType macOS Flutter runner

This directory contains the P4 local Apple Silicon macOS candidate. It is a
real ClipType settings/menu-bar application, not a Flutter counter demo.

The candidate is arm64-only (`aarch64-apple-darwin`) and is not a public beta,
Universal 2 artifact, or signed/notarized release. The broader release and
compatibility boundary is in [P4](../../docs/phases/P4_MACOS_ARM64_LOCAL.md)
and [ADR-0010](../../docs/adr/0010-flutter-macos-arm64-runner.md). Flutter is
the sole macOS front end; see [ADR-0012](../../docs/adr/0012-flutter-sole-macos-frontend.md).

## Build and run

Use the pinned local toolchain:

```bash
export PATH="$HOME/Developer/flutter-3.47.2/bin:$PATH"
cd apps/cliptype-flutter
flutter pub get
flutter analyze
flutter test
flutter build macos --release
open build/macos/Build/Products/Release/ClipType.app
```

The Xcode target builds the Rust static library for
`aarch64-apple-darwin` before compiling and linking the Swift shell. The
release output is `ClipType.app` with bundle identifier
`io.cliptype.ClipType`.

## Runtime boundary

Flutter presents General, Shortcuts, Typing, Permissions, and About pages. The
fixed channels are:

- MethodChannel: `io.cliptype/native`;
- EventChannel: `io.cliptype/events`.

The settings window supports English and Simplified Chinese. The selected
display language is a non-sensitive presentation preference and is mirrored to
the native status menu; it is separate from Rust's product settings.

Swift/AppKit owns the single status item/menu, window lifecycle, Carbon global
hot-key registration, Accessibility request/status, System Settings link,
`SMAppService`, and the Rust bridge. Rust owns settings validation and
persistence, the one-session coordinator, backend selection, clipboard
revision guard, target/focus/modifier protection, pacing, cancellation, and
content-free outcomes.

In Auto mode, any non-ASCII clipboard text—including Chinese, Japanese,
Korean, emoji, combining marks, and mixed Unicode—prefers one
revision-guarded Command+V when the paste capabilities are available. The
configured threshold remains the size crossover for otherwise ASCII-safe
text; Auto may use the bounded Unicode keyboard path when guarded paste is
unavailable.

The bridge carries bounded settings, commands, enum categories, and counters
only. It never carries clipboard text, injected text, focused values, window
titles, key history, or content fingerprints. The local shortcut recorder is
focused-control-only; there is no global event tap, global keyboard monitor, or
keylogger.

## Permissions and lifecycle

Accessibility is displayed as not granted until macOS reports trust. A request
is initiated only by an explicit user action. If Trigger is pressed without
trust, the operation fails closed and opens the macOS Privacy & Security /
Accessibility page; the app never changes the consent itself. The native shell
observes the result for a short bounded window after opening the page. Closing
Settings hides the window while the menu-bar process and registered commands
remain alive. Quit performs bounded native shutdown.

Start at Login uses `SMAppService.mainApp` where available. The local candidate
has no privileged helper and no extra entitlements; macOS consent is never
bypassed.

## Verification

Run the full command list and record results in
`../../local-evidence/P4_LOCAL_MACOS_ARM64_REPORT.md`. Do not put clipboard
fixtures, focused content, user identity, absolute home paths, tokens, or
credentials in that report or in command logs.
