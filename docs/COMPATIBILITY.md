# Compatibility Model

ClipType reports compatibility by **capability and evidence**, not merely by operating-system name.

## Support labels

- **Supported:** release-gated and continuously verified for the stated environment/capability.
- **Degraded:** usable with a weaker explicitly documented guarantee.
- **Experimental:** implementation exists but is not part of the stable contract.
- **Unavailable:** the environment lacks the capability or ClipType does not implement it.
- **Planned:** not implemented.

## Evidence labels

- **Contract:** behavior guaranteed by an official API or deterministic internal contract.
- **Automated runner:** exercised on an OS runner, without implying an unlocked representative desktop.
- **Controlled interactive:** exercised against the P1 test target in an unlocked desktop session.
- **Representative observation:** manually exercised against a named application/version/category.

Automated runner success does not by itself create a support label.

## Capability dimensions

Each environment is evaluated for:

1. current clipboard text read;
2. clipboard temporary write;
3. safe clipboard restoration;
4. global trigger;
5. Unicode keyboard/text injection;
6. multiline injection;
7. target/focus evidence strength;
8. cancellation;
9. privileged setup requirement;
10. packaging/signing maturity.

## Current matrix

No end-user environment is currently `Supported`. P1-S01 has only established official-contract and automated-runner evidence for a Windows thread-owned hotkey registration/message-queue probe.

| Environment | Keyboard mode | Clipboard mode | Focus guard | Current evidence | Target milestone |
|---|---|---|---|---|---|
| Windows 11 | Planned/P1 implementation | Planned/P2 | Planned/P1 | Contract + limited automated probe | P1/P2 |
| Windows 10 | Planned | Planned | Planned | None yet | P2 |
| macOS current releases | Planned | Planned | Planned | None yet | P3 |
| Linux X11 mainstream desktops | Planned | Planned | Planned | None yet | P4 |
| Wayland wlroots-family | Capability-dependent | Capability-dependent | Capability-dependent | None yet | P5 |
| Wayland GNOME/Mutter | Capability-dependent | Capability-dependent | Capability-dependent | None yet | P5 |
| Wayland KDE/KWin | Capability-dependent | Capability-dependent | Capability-dependent | None yet | P5 |

## Target application categories

Compatibility must be tested across:

- controlled native text target;
- Chromium-based browser;
- Firefox;
- VS Code/Electron editor;
- terminal emulator;
- office-style rich editor;
- remote desktop client;
- VM console where practical;
- elevated/admin target on Windows, where a security restriction is expected.

Specific applications become named support claims only after repeatable controlled and representative evidence.

## Focus evidence vocabulary

Record the strongest actual guarantee:

- top-level target identity;
- native focused-control identity;
- render-host-limited identity;
- degraded/ambiguous evidence;
- unavailable evidence.

Do not describe render-host-limited evidence as exact logical-field or caret tracking.

## Compatibility record

Each observation should include:

- ClipType commit/version;
- OS version/build/architecture;
- session and evidence class;
- backend/capability path;
- configured bounds;
- target application/version/category;
- Unicode/multiline/cancel/focus/modifier results;
- known limitations and skips.

## Wayland rule

Never collapse Wayland into one checkbox. A successful wlroots test does not imply GNOME or KDE compatibility, and clipboard availability does not imply synthetic-input availability.
