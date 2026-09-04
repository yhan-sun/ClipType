# Compatibility

## Current release channel

ClipType `v0.2.0-beta.3` is the current prerelease candidate. Windows x86_64 remains the primary beta channel, with an additive macOS arm64 testing preview. Under the current explicit maintainer authorization, this narrowly labelled prerelease may be published after its exact-head automated gates pass. Physical evidence remains required before expanding named-application, Accessibility, or general macOS compatibility claims. The compatibility promise is evidence-based and narrower than “all applications.”

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

## macOS Apple Silicon testing preview

The P4 candidate is restricted to an interactive Apple Silicon Mac:

| Environment | Support level | Evidence and conditions |
|---|---|---|
| macOS arm64 with an unlocked interactive desktop | **Testing preview / evidence required** | `v0.2.0-beta.3` carries a clearly labelled arm64 Flutter preview. The exact-main workflow builds, ad-hoc signs, installs `/Applications/ClipType.app`, launch-smokes it, verifies arm64 bundle/ZIP/DMG integrity, and attaches the assets additively. Accessibility permission and real target-application behavior remain separate physical evidence gates. |
| macOS x86_64 or Rosetta | **Not supported by P4** | No x86_64 build, Universal 2 artifact, or Rosetta claim is made. |
| Signed/notarized public macOS distribution | **Not provided by P4** | The attached preview is ad-hoc signed and has no Developer ID/notarization evidence. |

P4 does not claim named-application compatibility, persistent Accessibility
grant/revocation behavior, conflict rollback, cancellation latency, trigger
latency, or a successful human VS Code/Monaco session until those physical
cases are present in an exact-release report.

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
| `code` | Separate keyboard-only backend; Paste is unavailable by design. Pair-aware behavior is limited to `()`, `{}`, `[]`, `""`, and `''`. Matching source closers skip editor-generated closers; line-leading closers, including after `//` comments, cross the editor-generated closing line without a duplicate Return. Brackets in strings/comments, triple quotes, Markdown fences, and single backticks are literal. It assumes ordinary auto-pair and auto-indent are enabled. |
| `auto` | Freezes one backend at session start from Unicode shape, payload size, and proven capabilities; non-ASCII text prefers revision-guarded paste. Explicit modes never silently fall back. |

## Known boundaries

### Elevated targets

A normal-integrity ClipType process does not inject into a higher-integrity application. ClipType does not auto-elevate or bypass Windows User Interface Privilege Isolation. Run both applications at the same integrity level instead of elevating ClipType merely to bypass this protection.

### Focus evidence

ClipType captures and revalidates non-content destination evidence. Native
controls can usually be distinguished. On macOS, an initial `AXWebArea`
classification selects a sticky frontmost-process plus focused-window policy,
so repeated Monaco focus-node replacement in the same window, including a
temporarily unclassified replacement node, does not look like a target switch.
Switching process/window or losing stable window evidence still stops before
the next action. Multiple logical fields inside one shared render surface may
remain indistinguishable, so ClipType does not claim an exact logical-field or
caret guarantee there.

### Terminals and operational input

A terminal may execute a pasted or injected line break. Multiline clipboard content must be reviewed before use in terminals, shells, database consoles, remote-management tools, or administrative interfaces.

### Global hotkeys

Global trigger and cancel commands use reviewed system registrations with no-repeat behavior. Registration can fail because another application owns the combination. The menu bar remains the control surface. P4 applies a complete pair transactionally and keeps the old pair when a candidate fails; a successful OS registration still cannot prove that application-local shortcuts or hook-based tools will remain silent.

### Rich clipboard formats

Clipboard mode leaves all formats unchanged and invokes ordinary paste. Code
mode uses only Unicode keyboard/key-navigation events and is intended for text
editors with ordinary auto-pair and auto-indent enabled. Its pair grammar is
limited to `()`, `{}`, `[]`, `""`, and `''`; string/comment brackets and
single backticks are literal, while triple-quoted boundaries are sent
explicitly. Markdown fences are literal boundaries and pair handling continues
inside them. Rich targets may prefer HTML, RTF, image, or application-specific
formats already present beside `CF_UNICODETEXT`; use Clipboard mode for those
formats.

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
- whether the issue involved target evidence, modifiers, revision, hotkey ownership, partial input, or destination semantics.

Never attach real clipboard contents, credentials, private messages, focused-field contents, or raw crash dumps containing sensitive process memory.

## Physical-evidence follow-up

Publishing the narrowly labelled `v0.2.0-beta.3` prerelease is not itself a
broad compatibility claim. Issues #41 and #33 remain physical-evidence
trackers; until an exact-release interactive report reconciles them, the
project must not claim universal CJK keyboard behavior, completed persistent
Accessibility onboarding, or verified real VS Code/Monaco Code-mode support.
