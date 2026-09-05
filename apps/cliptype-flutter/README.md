# ClipType macOS Flutter runner

This directory contains the P4 Apple Silicon macOS candidate. It is a real
ClipType settings/menu-bar application, not a Flutter counter demo.

The candidate is arm64-only (`aarch64-apple-darwin`). A clearly labelled
ad-hoc-signed arm64 testing preview is attached to `v0.2.0-beta.6`; it is not a
general macOS beta, Universal 2 artifact, or signed/notarized release. The
broader release and compatibility boundary is in [P4](../../docs/phases/P4_MACOS_ARM64_LOCAL.md)
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
`io.github.yhan-sun.ClipType`. The identifier is kept stable across the
frontend migration so signed macOS builds continue to refer to the ClipType
product rather than a new app identity. Local Flutter builds use an ad-hoc
signature, whose code hash changes when the app is rebuilt; after replacing a
local app, remove any stale ClipType entry and add the current
`/Applications/ClipType.app` again under Accessibility.

## Runtime boundary

Flutter presents General, Shortcuts, Typing, Permissions, and About pages. The
fixed channels are:

- MethodChannel: `io.cliptype/native`;
- EventChannel: `io.cliptype/events`.

The settings window supports English and Simplified Chinese. The selected
display language is a non-sensitive presentation preference and is mirrored to
the native status menu; it is separate from Rust's product settings.

Product settings use automatic persistence. Discrete controls such as switches,
mode selection, and shortcut recording save as soon as a valid value is chosen;
text fields and sliders coalesce rapid edits for a short bounded interval. The
settings window has no Apply step: it shows Pending, Saving, Saved, or a
recoverable failure state, and each page can restore its own defaults.

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

Code mode is the explicit choice for source code and structured text. It uses
keyboard actions, skips leading indentation so the editor can supply it, and
moves right over matching closing delimiters or quotes that the editor has
already auto-generated. For a closer at the start of a source line, it avoids
inserting a duplicate Return and uses right-arrow line navigation to pass the
editor-generated closing line regardless of indentation width. Python-style
triple-quoted boundaries (`"""` and
`'''`) are typed explicitly because editors do not reliably auto-complete them.
Markdown triple-backtick fences are typed literally, and pair handling remains
active inside them. Brackets and quotes inside strings/comments remain literal.
This mode assumes the destination editor's ordinary auto-pair behavior is on.
Actions are sent in source order with bounded general and navigation-only
settle intervals so asynchronous auto-pair and auto-indent updates can finish
before a dependent action. If the editor generates ordinary brackets inside a normal quoted
string, Code mode consumes those inner generated closers before the string
boundary as well.

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

When the visible Trigger button is used, the shell hides the settings window
and yields briefly before capturing the destination, so the previously active
application can regain focus. The global Trigger shortcut remains available
when the destination should stay visibly focused throughout the action.

Start at Login uses `SMAppService.mainApp` where available. The local candidate
has no privileged helper and no extra entitlements; macOS consent is never
bypassed.

## Verification

Run the full command list and record results in
`../../local-evidence/P4_LOCAL_MACOS_ARM64_REPORT.md`. Do not put clipboard
fixtures, focused content, user identity, absolute home paths, tokens, or
credentials in that report or in command logs.
