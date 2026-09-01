# Compatibility Model

ClipType reports compatibility by **capability, evidence strength, and tested target**, not merely by operating-system name.

## Support labels

- **Supported:** release-gated and continuously verified for the stated environment/capability.
- **Degraded:** usable but one or more safety/semantic guarantees are weaker and explicitly documented.
- **Experimental:** implementation exists but is not part of the stable compatibility contract.
- **Unavailable:** the environment does not expose the required capability or ClipType does not implement it.
- **Planned:** not implemented yet.
- **Observed in P1:** a development-phase result for one recorded environment; it is not a release support label.

## Capability dimensions

Each environment is evaluated independently for:

1. current clipboard text read;
2. clipboard temporary write;
3. safe clipboard restoration;
4. global trigger;
5. Unicode keyboard/text injection;
6. line-break/Tab semantics;
7. target/focus evidence strength;
8. cancellation and checkpoint latency;
9. modifier-state safety;
10. privilege/security-boundary behavior;
11. packaging/signing maturity.

## Focus-evidence vocabulary

A `focus guard` claim must say what evidence is actually compared:

- **Top-level target evidence:** foreground top-level window plus process/thread identity.
- **Native focused-control evidence:** platform exposes a distinct focused native child/control and it is included in comparison.
- **Render-host-limited evidence:** multiple logical fields may share one native focus/render host; a logical caret move inside that host may be undetectable.
- **Unavailable/ambiguous evidence:** the platform/native call cannot provide a reliable comparison at that moment.

ClipType MUST NOT translate top-level or render-host evidence into a claim that the exact logical text field/caret is always unchanged. Under strict P1 policy, known target change, disappearance, or evidence becoming unavailable/ambiguous after dispatch starts stops future batches.

## Pre-implementation matrix

No production implementation has been merged. This table records target order, not a support claim.

| Environment | Keyboard mode | Clipboard mode | Focus evidence | Target milestone |
|---|---|---|---|---|
| Windows 11 | Planned in P1 | Planned in P2 | Planned; evidence level recorded per target | P1/P2 |
| Windows 10 | Planned after evidence/CI decision | Planned | Planned | P2 |
| macOS current supported releases | Planned | Planned | Planned | P3 |
| Linux X11 mainstream desktops | Planned | Planned | Planned | P4 |
| Wayland wlroots-family | Planned/capability-dependent | Planned/capability-dependent | Planned/capability-dependent | P5 |
| Wayland GNOME/Mutter | Planned/capability-dependent | Planned/capability-dependent | Planned/capability-dependent | P5 |
| Wayland KDE/KWin | Planned/capability-dependent | Planned/capability-dependent | Planned/capability-dependent | P5 |

Windows 10 is not automatically part of the P1 exit gate. It becomes a named claim only after the maintainer selects supported editions and evidence exists.

## P1 Windows evidence categories

P1 results must be separated into:

### Automated build/unit evidence

Proves compilation and deterministic contract/policy behavior. It does not prove an interactive global hotkey or native input delivery.

### Controlled interactive E2E evidence

Uses the project test target on a recorded interactive Windows desktop and may assert generated fixture text and safety outcomes.

### Representative target observation

Records behavior in a named real application/version/category. An observation is not automatically a stable support claim or evidence for every application in the category.

## Target application matrix

Compatibility is tested across target categories rather than only one editor:

- controlled native Win32 edit/control;
- Chromium-family browser;
- Firefox when scheduled/evidenced;
- VS Code/Electron editor;
- terminal emulator;
- office-style rich editor in later compatibility work;
- remote desktop client/VM console in later compatibility work;
- elevated/admin target on Windows for expected restriction evidence.

P1 requires the controlled Win32 target, a Chromium-family field, VS Code/editor, terminal, and elevated-target observation. Other categories remain later work unless explicitly added.

## Text-semantics evidence

For each tested target, record behavior for the applicable fixture classes:

- ASCII/punctuation;
- CJK;
- supplementary Unicode/emoji;
- combining marks;
- normalized line breaks;
- Tab;
- unsupported controls;
- long multi-batch text;
- configured payload limit.

A target may process native character packets and physical Enter/Tab keys differently. Therefore a success for printable Unicode does not automatically prove multiline or Tab compatibility.

Terminal-like targets may interpret emitted line breaks as operational input. Compatibility reports use benign fixtures and explicitly record that target-side command semantics are outside ClipType's ability to neutralize.

## Security-boundary wording

Windows evidence distinguishes:

- **Known higher-integrity restriction:** available process-integrity evidence proves the target relationship is blocked by the platform security boundary.
- **Blocked/native cause unknown:** native insertion accepted no events but the cause cannot be proven.

Do not report every zero-dispatch result as definitively caused by UIPI. ClipType does not bypass the boundary or elevate automatically.

## Compatibility evidence record

Every claim/observation should include:

- exact ClipType commit;
- Windows/macOS/Linux version and architecture;
- interactive session/desktop/compositor where relevant;
- backend/capability path;
- target application and version;
- focus-evidence level;
- text fixture class;
- batch/checkpoint/modifier policy relevant to the result;
- Unicode/multiline/Tab/cancel/focus result;
- evidence source: CI, controlled E2E, or representative observation;
- known limitations and skipped paths.

## Claim promotion rule

A P1 observation is promoted to a release support label only through the later productization/release gate. Documentation must not change `Planned` to `Supported` merely because a feature worked once.

## Wayland rule

Never collapse Wayland testing into one checkbox. A successful wlroots test does not imply GNOME or KDE compatibility, and clipboard capability does not imply synthetic keyboard or focus capability.
