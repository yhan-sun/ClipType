# Platform Backend Design

This document records intended native mechanisms and known platform constraints. Exact crate choices may change without an ADR; changing the mechanism/security model requires one.

## Windows

### Clipboard
Use the Win32 clipboard listener model rather than polling. `AddClipboardFormatListener` registers a window to receive `WM_CLIPBOARDUPDATE` when clipboard content changes. Current plain text is read through Win32 clipboard APIs using Unicode text formats.

For the V1 manual-trigger flow, continuous content capture is unnecessary: a listener can maintain metadata/status, while actual plaintext is acquired only when needed.

### Keyboard injection
Use Win32 `SendInput`. Prefer Unicode/text-oriented events where they preserve intended text semantics and reduce keyboard-layout dependence.

Important restriction: `SendInput` is subject to **UIPI**. A normal-integrity ClipType process cannot inject into a higher-integrity target. ClipType MUST report this as a platform security-boundary limitation rather than attempting to bypass it.

### Focus/target
Use native foreground-window/process identity sufficient for focus guard. Avoid reading target field contents.

### Hotkey
Use a native global hotkey mechanism and report registration conflicts.

### Expected support level
Windows is the first production-quality platform and defines the P1/P2 vertical slice.

## macOS

### Clipboard
Use `NSPasteboard.general` for text access. `NSPasteboard.changeCount` can indicate ownership/content changes; macOS does not provide the same Win32 listener model, so a lightweight change-count observation strategy may be used only when continuous observation is actually needed.

### Keyboard injection
Use Core Graphics `CGEvent` facilities for synthetic keyboard/text events, with explicit Unicode behavior tests.

### Permissions
Cross-application synthetic input requires user-granted system permissions in common configurations. Use Accessibility trust APIs such as `AXIsProcessTrusted`/the appropriate prompting variant to detect/onboard, never to bypass consent.

### Focus/target
Use workspace/accessibility/window APIs only for target identity and permission-safe focus evidence. Do not inspect focused text.

### Distribution
macOS release requires code signing/notarization planning before claiming general availability.

## Linux X11

### Clipboard
Use X11 selection semantics. Continuous clipboard-manager behavior is not needed for V1; obtain current selection content when triggered and implement only the ownership needed for temporary paste transactions. XFixes notifications may be used for change metadata if required.

### Keyboard injection
Use the XTEST extension/native X11 facilities. Unicode behavior depends on keymap/input semantics and requires an explicit compatibility test matrix.

### Focus
Use X11 focus/window identity where available for focus guard.

### Security note
X11 permits broad client interaction; ClipType still applies its own explicit-trigger and privacy model rather than exploiting the broadest possible access.

## Linux Wayland

Wayland is not one uniform backend. ClipType MUST probe protocols, compositor capabilities, portal availability, and device permissions at runtime.

### Clipboard capability options

1. **`ext-data-control-v1`** where the compositor exposes it. It is designed for privileged clipboard-management clients and is still a staging protocol.
2. Legacy `wlr-data-control` may exist on wlroots-based compositors, but it is deprecated in favor of `ext-data-control-v1` and MUST NOT be the architectural endpoint.
3. **XDG Desktop Portal Clipboard** can provide clipboard access only through compatible portal sessions (currently tied to Remote Desktop/Input Capture-style sessions) and therefore is not a universal transparent clipboard-manager API.
4. Standard Wayland data-device access is focus/seat oriented and does not by itself provide a universal global clipboard-manager capability.

The adapter reports what is actually available instead of assuming global clipboard read/write.

### Keyboard capability options

1. `zwp_virtual_keyboard_v1` where exposed and authorized by the compositor.
2. Linux `uinput`, which can create a virtual input device from userspace through `/dev/uinput`; this often requires device permissions and may require a small privileged helper.
3. Desktop/compositor-specific mechanisms may be investigated but cannot silently become global compatibility claims.

### Capability tiers

A Wayland session may independently have:
- clipboard read;
- clipboard write/restore;
- global trigger;
- synthetic text/key injection;
- focus evidence.

`Wayland supported` is therefore not a boolean. `docs/COMPATIBILITY.md` tracks combinations by environment.

### Privileged helper

If uinput is required, the helper is Linux-only, minimal, local, and capability-scoped. It must not become a general root daemon or clipboard store.

## Platform fallback policy

Fallback is planner-visible and capability-safe. Example: if keyboard injection is unavailable but clipboard paste is available, `auto` may use paste. An explicit `keyboard` request fails clearly rather than silently violating the user's mode choice.

Fallback MUST NOT cross a security boundary or launch an external privileged command automatically.

## Research/reference APIs

See `REFERENCES.md` for official API/protocol documentation and reference projects.