# Design References

References are for APIs, constraints, and architectural research. They do not grant permission to copy source code. License review is required before code reuse.

## Official platform documentation

### Windows
- Microsoft `SendInput`: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
  - Native input insertion; documentation notes UIPI restrictions.
- Microsoft `AddClipboardFormatListener`: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-addclipboardformatlistener
  - Registers for `WM_CLIPBOARDUPDATE` clipboard-change notifications.
- Microsoft clipboard usage guide: https://learn.microsoft.com/en-us/windows/win32/dataxchg/using-the-clipboard

### macOS
- Apple `NSPasteboard`: https://developer.apple.com/documentation/appkit/nspasteboard
- Apple `NSPasteboard.changeCount`: https://developer.apple.com/documentation/appkit/nspasteboard/changecount
- Apple `CGEvent`: https://developer.apple.com/documentation/coregraphics/cgevent
- Apple `AXIsProcessTrusted`: https://developer.apple.com/documentation/applicationservices/1460720-axisprocesstrusted

### Linux / Wayland
- Linux kernel `uinput`: https://docs.kernel.org/input/uinput.html
  - Kernel documentation recommends considering libevdev as a less error-prone wrapper for new software.
- Wayland virtual keyboard protocol: https://wayland.app/protocols/virtual-keyboard-unstable-v1
- Wayland `ext-data-control-v1`: https://wayland.app/protocols/ext-data-control-v1
- XDG Desktop Portal Clipboard: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Clipboard.html
- Core Wayland data transfer model: https://wayland.freedesktop.org/docs/book/Protocol.html

## Reference projects

### Espanso
https://github.com/espanso/espanso

Most relevant reference for the product's mechanism separation. Espanso exposes clipboard/inject/auto backend concepts and a threshold for choosing clipboard injection for long text. ClipType uses this as evidence that dual injection strategies are practical, but defines its own safety, state machine, and architecture.

### ydotool
https://github.com/ReimuNotMoe/ydotool

Useful reference for Linux virtual input and `/dev/uinput`/daemon permission considerations.

### wtype
https://github.com/atx/wtype

Useful reference for Wayland virtual-keyboard-based text input and compositor protocol realities.

### CopyQ
https://github.com/hluk/CopyQ

Useful reference for cross-platform clipboard behavior and desktop integration. ClipType intentionally does not adopt clipboard-history scope for V1.

### Input Leap
https://github.com/input-leap/input-leap

Useful reference for cross-platform keyboard/mouse/clipboard integration and platform-specific compatibility boundaries.

## Research rules

When a design relies on a platform fact:
1. prefer official OS/protocol documentation;
2. use open-source projects to learn integration patterns and edge cases;
3. verify current behavior on supported OS versions;
4. record compatibility evidence in tests/docs;
5. never treat one compositor/application test as universal compatibility.