# Compatibility Model

ClipType reports compatibility by **capability**, not merely by operating-system name.

## Support labels

- **Supported:** release-gated and continuously verified for the stated environment/capability.
- **Degraded:** usable but one or more safety/semantic guarantees are weaker and explicitly documented.
- **Experimental:** implementation exists but is not part of the stable compatibility contract.
- **Unavailable:** environment does not expose the required capability or ClipType does not implement it.
- **Planned:** not implemented yet.

## Capability dimensions

Each environment is evaluated for:

1. current clipboard text read;
2. clipboard temporary write;
3. safe clipboard restoration;
4. global trigger;
5. Unicode keyboard/text injection;
6. multiline injection;
7. target/focus guard;
8. cancellation;
9. privileged setup requirement;
10. packaging/signing maturity.

## Pre-implementation matrix

Everything is currently `Planned`; this table records target order, not a support claim.

| Environment | Keyboard mode | Clipboard mode | Focus guard | Target milestone |
|---|---|---|---|---|
| Windows 11 | Planned | Planned | Planned | P1/P2 |
| Windows 10 (supported editions subject to CI availability) | Planned | Planned | Planned | P2 |
| macOS current supported releases | Planned | Planned | Planned | P3 |
| Linux X11 mainstream desktops | Planned | Planned | Planned | P4 |
| Wayland wlroots-family | Planned/Capability-dependent | Planned/Capability-dependent | Planned/Capability-dependent | P5 |
| Wayland GNOME/Mutter | Planned/Capability-dependent | Planned/Capability-dependent | Planned/Capability-dependent | P5 |
| Wayland KDE/KWin | Planned/Capability-dependent | Planned/Capability-dependent | Planned/Capability-dependent | P5 |

## Target application matrix

Compatibility must also be tested across target categories rather than only editors:

- native text editor;
- Chromium-based browser;
- Firefox;
- VS Code/Electron editor;
- terminal emulator;
- office-style rich editor;
- remote desktop client;
- VM console where practical;
- elevated/admin target on Windows (expected security restriction documented).

Specific applications become named support claims only after repeatable testing.

## Compatibility evidence

A claim should record:
- ClipType version/commit;
- OS version;
- desktop environment/compositor/session type;
- backend/capability path selected;
- target application/version;
- Unicode/multiline/cancel/focus results;
- known limitations.

## Wayland rule

Never collapse Wayland testing into one checkbox. A successful wlroots test does not imply GNOME or KDE compatibility, and availability of clipboard protocols does not imply availability of synthetic keyboard protocols.