# Glossary

**Clipboard snapshot** — Ephemeral representation of current clipboard text plus minimal metadata required for one injection attempt/restore decision.

**Injection** — Delivery of clipboard text to the currently focused target using synthetic keyboard/text events or a clipboard-paste transaction.

**Keyboard backend** — Backend that emits native synthetic keyboard/text input.

**Clipboard backend / clipboard-paste mode** — Backend that temporarily writes clipboard content, triggers paste, and attempts safe restoration.

**Code mode** — Explicit whole-block guarded-paste mode for source code and
structured text. It preserves existing delimiters and indentation without
emitting per-character keyboard or auto-indent-triggering events.

**Auto planner** — Policy that selects an eligible backend based on payload, configuration, target, and runtime capabilities.

**Target** — The application/window/control context intended to receive input. ClipType primarily identifies it through OS focus/foreground evidence, not screen coordinates.

**Focus guard** — Safety mechanism that detects meaningful target change and aborts an active injection.

**Capability** — Runtime evidence that a platform mechanism is available and authorized, e.g. clipboard read, virtual keyboard, focus identity.

**Adapter** — OS-specific implementation of a core/application port.

**Port** — Platform-independent semantic interface required by the application/core.

**UIPI** — Windows User Interface Privilege Isolation, which restricts lower-integrity processes from injecting input into higher-integrity targets.

**Accessibility permission** — macOS user consent/security mechanism commonly required for cross-application input/accessibility operations.

**X11/XTEST** — X Window System and extension facilities usable for synthetic input on X11 sessions.

**Wayland compositor** — Display/input server implementation such as Mutter, KWin, or wlroots-based compositors; protocol availability differs between them.

**Data control** — Wayland protocol family intended to let privileged clients manage selections/clipboard.

**Virtual keyboard protocol** — Wayland protocol that allows authorized clients to provide keyboard events as if from a physical keyboard.

**uinput** — Linux kernel userspace input module allowing a process with appropriate device permissions to create virtual input devices.

**Privileged helper** — Minimal optional process used only when a platform capability such as `/dev/uinput` access requires an isolated permission boundary.

**ADR** — Architecture Decision Record: an immutable record of a significant decision, context, alternatives, and consequences.
