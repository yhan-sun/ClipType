# Platform Backend Design

This document records intended native mechanisms and known platform constraints. Exact crate choices may change without an ADR; changing the mechanism or security model requires the repository decision process.

## Windows

Windows is the first implementation platform. Detailed P1 sequencing is in [`phases/P1_WINDOWS_VERTICAL_SLICE.md`](phases/P1_WINDOWS_VERTICAL_SLICE.md).

### P1 native discovery gate

Before shared contracts are frozen, P1 runs an interactive Windows spike covering message-loop ownership, Unicode and control semantics, target evidence, modifier state, dispatch results, clipboard contention, and UIPI reporting. Research output informs contracts but is not itself a production adapter.

### Clipboard

Current plain text is read through Win32 clipboard APIs using `CF_UNICODETEXT` semantics. Native clipboard handles are borrowed from the OS; the adapter copies UTF-16 data into owned memory before unlocking/closing.

For P1 manual-trigger behavior:

- acquire current text only after an explicit trigger;
- use bounded handling for transient clipboard contention;
- do not persist/cache plaintext;
- do not implement clipboard history;
- do not require `AddClipboardFormatListener` or `WM_CLIPBOARDUPDATE`.

A listener may be evaluated later for content-blind status/change metadata or future product behavior, but continuous plaintext capture is not the P1 architecture.

### Keyboard injection

Use Win32 `SendInput`. Prefer Unicode/text-oriented events where they preserve intended semantics and reduce keyboard-layout dependence.

P1 translates native-neutral semantic atoms into bounded batches. It explicitly tests:

- ASCII and punctuation;
- CJK;
- combining marks;
- supplementary Unicode represented through UTF-16 units;
- line-break alternatives;
- Tab;
- unsupported controls.

Native event counts may prove complete, none, or partial insertion without proving an exact semantic-text prefix. Partial/unknown results are not retried.

### Modifier state

`SendInput` does not reset physical keyboard state. P1 therefore uses a bounded pre-dispatch gate for trigger modifiers and may recheck conflicting modifiers between batches. ClipType never releases arbitrary physical user keys to make injection succeed.

### UIPI

`SendInput` is subject to Windows User Interface Privilege Isolation. A normal-integrity process cannot inject into a higher-integrity target.

Reporting must separate evidence from inference:

- if target-integrity comparison reliably proves a higher-integrity target, report a known security-boundary restriction;
- if integrity evidence is unavailable and native insertion accepts no events, report a blocked/native-unknown result with appropriate guidance;
- do not claim every zero-result is definitely UIPI;
- do not elevate automatically or attempt to circumvent the boundary.

### Target/focus evidence

Use the strongest practical non-content combination of:

- foreground top-level window;
- owning process and GUI thread;
- active/focused native window evidence exposed by GUI-thread inspection;
- optional integrity relationship for security reporting.

Do not read focused text or log window titles by default.

The adapter can detect many window/application changes, but some applications host multiple logical fields inside one native render surface. P1 therefore promises revalidation against available window/thread/focus evidence, not exact logical-caret identity in every application. Evidence becoming invalid/ambiguous during strict injection fails safely.

### Hotkey and message loop

Use native global hotkey registration and report conflicts. Prefer the `RegisterHotKey`/`WM_HOTKEY` model with no-repeat behavior rather than a low-level global keyboard hook.

The registering thread owns the relevant message queue/lifecycle. It must remain responsive while an injection worker runs, so native text dispatch does not execute as one blocking operation on the hotkey message-loop thread.

### P1 presentation

P1 requires only a minimal development host/status surface. It does not require a tray, settings window, installer, auto-start, or Windows App SDK adoption. Product-quality Windows presentation is P2.

### Expected support level

Windows is the first production-quality target over P1/P2. P1 itself produces evidence for a vertical slice, not a public universal-compatibility claim.

## macOS

P4 includes a local, Apple Silicon-only Flutter/AppKit composition root. The
candidate is one normal per-user process with one Flutter engine, one
`NSStatusItem`, one Settings window, and one global shortcut owner. It is an
unsigned local candidate and does not change the public Universal 2,
Developer ID, notarization, or named-application support gates.

### Clipboard

Use `NSPasteboard.general` for text access. `NSPasteboard.changeCount` can indicate ownership/content changes; the P4 bridge requests current clipboard data only inside a Rust-triggered session. No clipboard value crosses the Flutter/Swift status boundary, and no history or restore transaction is added.

### Keyboard injection

Use Core Graphics `CGEvent` facilities for synthetic keyboard/text events, with explicit Unicode behavior tests. The P4 Swift shell does not perform input itself; it delegates the bounded session to the Rust coordinator and `cliptype-macos` adapter.

### Permissions

Cross-application synthetic input commonly requires user-granted system permission. Use Accessibility trust APIs to detect/onboard, never to bypass consent. P4 requests permission only after an explicit UI/menu action and observes the result for a bounded interval.

### Focus/target

Use workspace/accessibility/window APIs only for target identity and
permission-safe focus evidence. Do not inspect focused text or window titles.
Native controls use exact focused-element identity. An initial `AXWebArea`
render-host classification selects stable process plus focused-window identity
for the session because Monaco may rebuild its focused Accessibility node
during normal typing. If a Chromium editor does not expose a traversable
`AXWebArea` parent chain, support for the web-only `AXDOMIdentifier` or
`AXDOMClassList` attribute name supplies the initial classification without
reading either value. A later same-window node may temporarily lose that
classification without looking like a target change. A process/window change,
missing stable window identity, disappearance, or capture failure still stops
safely; logical field changes within one web render surface may be
indistinguishable. The fixed bridge returns only content-free categories and
counters.

### Distribution

macOS release requires code signing/notarization planning before claiming
general availability. The unreleased `v0.2.0-beta.3` candidate prepares a
clearly labelled additive arm64 testing preview built on an Apple Silicon
runner; `v0.2.0-beta.2` remains immutable. The preview is ad-hoc signed and is
not a Universal 2, Developer ID, notarized, Gatekeeper-ready, or general public
macOS release. Publication remains blocked until the repository's physical
release evidence is complete.

### Shell and command ownership

The Flutter runner uses `io.cliptype/native` and `io.cliptype/events`. Swift/AppKit
owns one `NSStatusItem`, native menu commands, Carbon Trigger/Cancel
registration, `SMAppService`, and the Flutter window lifecycle. Candidate hotkey
pairs are probed and applied transactionally. The local recorder captures only
while its Flutter control has focus; no event tap, global monitor, or keylogger
is installed.

## Linux X11

### Clipboard

Use X11 selection semantics. Continuous clipboard-manager behavior is not needed for the initial flow; obtain current selection content when triggered and implement only ownership needed for later temporary paste transactions. XFixes notifications may be used for change metadata if required.

### Keyboard injection

Use the XTEST extension/native X11 facilities. Unicode behavior depends on keymap/input semantics and requires an explicit compatibility matrix.

### Focus

Use X11 focus/window identity where available and describe evidence limitations honestly.

### Security note

X11 permits broad client interaction; ClipType still applies its explicit-trigger and privacy model rather than using the broadest possible access.

## Linux Wayland

Wayland is not one uniform backend. ClipType MUST probe protocols, compositor capabilities, portal availability, and device permissions at runtime.

### Clipboard capability options

1. `ext-data-control-v1` where exposed.
2. Legacy `wlr-data-control` may exist, but it is not the architectural endpoint.
3. XDG Desktop Portal Clipboard may be available only through compatible portal sessions and is not a universal transparent clipboard-manager API.
4. Standard Wayland data-device access is focus/seat oriented and does not itself provide universal global clipboard access.

### Keyboard capability options

1. `zwp_virtual_keyboard_v1` where exposed and authorized.
2. Linux `uinput` through `/dev/uinput`, potentially requiring a small capability-scoped helper.
3. Desktop/compositor-specific mechanisms may be researched but cannot silently become global support claims.

### Capability tiers

A Wayland environment may independently provide:

- clipboard read;
- clipboard write/restore;
- global trigger;
- synthetic text/key input;
- focus evidence.

`Wayland supported` is not a boolean. `COMPATIBILITY.md` records actual combinations.

### Privileged helper

If uinput is required, the helper is Linux-only, minimal, local, and capability-scoped. It must not become a general root daemon or clipboard store.

## Platform fallback policy

Fallback is planner-visible and capability-safe. For example, if keyboard input is unavailable but clipboard paste is available, a future `auto` mode may choose paste. An explicit `keyboard` request fails clearly rather than silently changing the user's requested semantics.

Fallback does not cross a security boundary or automatically launch an external privileged command.

## Research/reference APIs

See `REFERENCES.md` for official API/protocol documentation and reference projects.
