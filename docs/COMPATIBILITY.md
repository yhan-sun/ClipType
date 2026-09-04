# Compatibility

## Current release channel

ClipType `v0.1.0-beta.1` is the first public Windows x86_64 beta. The compatibility promise is evidence-based and narrower than “all Windows applications.”

## Supported Windows environments

| Environment | Support level | Evidence and conditions |
|---|---|---|
| Windows 11 x64, interactive desktop | **Supported beta / recommended** | Uses stable Win32 clipboard, foreground-target, global-hotkey, registry, tray, and `SendInput` APIs. Normal-integrity process and an unlocked interactive desktop are required. |
| Windows 10 22H2 x64, interactive desktop | **Best-effort compatibility** | Uses the same Win32 API surface, but Windows 10 is outside standard Microsoft support. Use only on a system receiving applicable security updates. |
| Windows Server 2022 x64 with Desktop Experience | **Mechanism-compatible** | Full workspace, controlled Unicode keyboard/clipboard/auto E2E, release build, and privacy checks run on the `windows-2022` hosted image. Hosted CI is not a physical-user hotkey claim. |
| Windows Server 2025 x64 with Desktop Experience | **Mechanism-compatible / CI reference** | Full workspace, controlled E2E, host lifecycle, install/startup/uninstall, packaging, and release build run on the `windows-2025` hosted image. |
| Windows on ARM64 | **Not supported in this release** | No ARM64 release artifact or compatibility gate. |
| 32-bit Windows | **Not supported** | No x86 artifact or test matrix. |
| Server Core, services, locked/non-interactive sessions | **Not supported** | ClipType is an interactive per-user tray application, not a service. |

“Supported beta” means the shipped x86_64 binary and documented native mechanisms are supported within the constraints below. It does not assert that every application exposes sufficient focus evidence or consumes synthetic input identically.

## macOS Apple Silicon local candidate

The P4 local candidate is restricted to an interactive Apple Silicon Mac:

| Environment | Support level | Evidence and conditions |
|---|---|---|
| macOS arm64 with an unlocked interactive desktop | **Local candidate / evidence required** | Flutter macOS settings shell, Swift/AppKit status item, Carbon hot-key registration, Rust bridge, and arm64 release packaging are locally exercised. Accessibility permission and target-application behavior remain separate physical evidence gates. |
| macOS x86_64 or Rosetta | **Not supported by P4** | No x86_64 build, Universal 2 artifact, or Rosetta claim is made. |
| Signed/notarized public macOS distribution | **Not provided by P4** | The local candidate is unsigned and has no Developer ID/notarization evidence. |

P4 does not claim Chinese/Unicode delivery, named-application compatibility,
permission grant/revocation, conflict rollback, cancellation latency, or
trigger latency until those physical cases are present in an exact-SHA report.

## Application compatibility

### Supported application category

ClipType supports ordinary editable desktop targets that accept at least one of:

- Unicode-oriented Win32 `SendInput` keyboard events; or
- the standard current-clipboard `Ctrl+V` command.

This category includes many native and framework-based desktop applications, browsers, editors, chat clients, office applications, and terminal front ends, but category membership alone is not a per-product certification. Destination applications own their paste, formatting, shortcut, and command-submission semantics.

### Backend behavior

| Backend | Compatibility contract |
|---|---|
| `keyboard` | Sends bounded Unicode/key batches. Stops on target change, evidence loss, conflicting modifiers, cancellation, partial input, or unknown native progress. |
| `clipboard` | Verifies a content-blind clipboard revision and sends one bounded `Ctrl+V`. It never writes, clears, owns, restores, or stores the clipboard. The destination may choose an existing rich-text clipboard format. |
| `auto` | Freezes one backend at session start from Unicode shape, payload size, and proven capabilities; non-ASCII text prefers revision-guarded paste. Explicit modes never silently fall back. |

## Known boundaries

### Elevated targets

A normal-integrity ClipType process does not inject into a higher-integrity application. ClipType does not auto-elevate or bypass Windows User Interface Privilege Isolation. Run both applications at the same integrity level instead of elevating ClipType merely to bypass this protection.

### Focus evidence

ClipType captures and revalidates non-content destination evidence. Native controls can usually be distinguished. Applications that host multiple logical fields inside one shared render surface may expose only a top-level/render-host identity. In that case ClipType does not claim an exact logical-field or caret guarantee.

### Terminals and operational input

A terminal may execute a pasted or injected line break. Multiline clipboard content must be reviewed before use in terminals, shells, database consoles, remote-management tools, or administrative interfaces.

### Global hotkeys

Global trigger and cancel commands use reviewed system registrations with no-repeat behavior. Registration can fail because another application owns the combination. The menu bar remains the control surface. P4 applies a complete pair transactionally and keeps the old pair when a candidate fails; a successful OS registration still cannot prove that application-local shortcuts or hook-based tools will remain silent.

### Rich clipboard formats

Clipboard and Code modes leave all formats unchanged and invoke ordinary paste. Rich targets may prefer HTML, RTF, image, or application-specific formats already present beside `CF_UNICODETEXT`. Use keyboard mode when Unicode text-event semantics are required; use Code mode when an editor's per-character pair completion or auto-indent must not duplicate the copied block.

### Remote and virtual desktops

RDP, VDI, automation agents, secure desktops, and virtualization layers can change foreground, clipboard, and input routing. They are not included in the public beta support promise unless a named environment has separate evidence.

## Evidence classes

Compatibility statements distinguish:

1. deterministic native-neutral policy tests;
2. Windows adapter contract tests;
3. hosted Windows controlled E2E using a purpose-built native edit target;
4. package install/startup/uninstall smoke;
5. user-reported or maintainer-observed named applications on physical interactive desktops.

Hosted Windows runners prove build, native API, controlled-target, and lifecycle behavior. They do not prove a human physically pressed a global hotkey in every client OS or application. Named reports can expand the application matrix without weakening the underlying safety rules.

## Reporting a compatibility result

Include only content-free information:

- ClipType release and source/release asset digest;
- Windows edition, version, build, and architecture;
- application name/version and whether it was elevated;
- selected backend and outcome category;
- whether the issue involved hotkey registration, focus evidence, modifiers, clipboard revision, partial input, or destination semantics.

Never attach real clipboard contents, credentials, private messages, focused-field contents, or raw crash dumps containing sensitive process memory.


## Pre-beta regression gate

The current public-beta candidate remains blocked by Issue #41 until Chinese input reliability, per-action pacing, tray controls, and a replacement exact-SHA interactive validation are complete. Earlier Issue #33 beta-ready wording is superseded and must not be used as a release authorization.
