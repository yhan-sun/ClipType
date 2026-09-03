# P3 macOS Native Mechanism Spike

## Status

- Issue: #48 (`P3-S01`)
- Product candidate: Draft PR #55
- Candidate SHA: `ba0b724cfd96bba2161518d50197358b7df35b57`
- Validation branch: `validation/p3-s01-macos-native-evidence-ba0b724`
- Current evidence class: source review and hosted build automation only

**P3 macOS production adapters may proceed: NO**

No real interactive Mac evidence is committed yet. Production adapters already
exist in the draft candidate, but this document does not retroactively treat
implementation or hosted compilation as the required native-mechanism proof.
Issue #49 remains logically dependent on an explicit P3-S01 YES.

## Observed result

### Source and automated boundary

The candidate contains narrow Objective-C/Core Graphics/AppKit/Carbon/AX and
`SMAppService` adapters, a Rust composition root, Universal 2 CI, ad-hoc candidate
packaging, and a protected Developer ID/notarization workflow. Existing automated
checks can prove compilation, unit policy, architecture slices, bundle assembly,
and content-free static behavior. They cannot prove user-consent, physical input,
focus identity, menu-bar lifecycle, or login-item behavior in an unlocked client
session.

This validation branch adds:

- a content-free native runtime probe;
- a generated-fixture two-field AppKit target;
- a complete P3-S01 case matrix;
- exact-SHA preparation, result recording, privacy sentinel, and finalization
  scripts;
- a pinned Intel/Apple Silicon hosted validation workflow;
- the physical-Mac runbook in `docs/testing/P3_MACOS_INTERACTIVE_RUNBOOK.md`.

### Code-review observations requiring physical proof

1. `AXIsProcessTrusted` exposes current trust, not a durable distinction between
   “never requested” and “previously denied.” The current candidate presents a
   false trust result as `NotGranted`; the product requirement for a distinct
   initial `Not requested` presentation still requires a reviewed state model.
2. Focus identity currently uses frontmost PID plus `CFHash` of the focused
   `AXUIElement`. Stability across repeated element snapshots and representative
   applications is not yet observed.
3. Detailed focus evidence degrades to top-level process evidence when AX access
   is unavailable. The physical gate must confirm strict coordinator behavior
   stops rather than silently adopting a weaker destination after dispatch starts.
4. Carbon registration stages new shortcuts before releasing old shortcuts and
   preserves old registrations on ordinary conflict. Trigger/Cancel cross-swap is
   explicitly classified unsupported; its user-visible behavior remains to be
   recorded.
5. `NSStatusItem`, Slint settings, permission prompts, and Carbon hotkeys share the
   application/main run loop, while input runs on the coordinator worker. Actual
   responsiveness and shutdown ordering are unobserved.
6. `SMAppService.mainApp` must be tested from a stable installed app identity.
   Running a loose command-line probe is not product login-item evidence.
7. Ad-hoc signing is development-only. No Developer ID identity, notarization,
   stapled ticket, or clean-Mac Gatekeeper result is currently claimed.

## Inference

- The selected APIs are plausible production mechanisms and are consistent with
  the native-neutral ports.
- The coordinator's one-semantic-action dispatch boundary should reduce the old
  Windows batch spill window, but macOS focus behavior must be measured rather
  than inferred.
- Carbon temporary registration can provide a useful conflict signal, but it
  cannot guarantee absence of application-local or hook-based conflicts.
- A stable app path/signature is likely important to Accessibility identity;
  physical observations must determine the supported install/update boundary.

These are inferences, not support claims.

## Current limitations

- no Apple Silicon unlocked-desktop observation;
- no second macOS-version observation;
- no Intel runtime observation;
- no permission denied/granted/revoked sequence;
- no real Unicode, Command-V, focus-switch, or Secure Event Input observation;
- no physical global-hotkey conflict/replacement/Cancel observation;
- no menu-bar/settings/VoiceOver/Retina lifecycle evidence;
- no `SMAppService` login-cycle evidence;
- no privacy-sentinel scan from a real run;
- no Developer ID, notarization, stapling, or Gatekeeper acceptance evidence.

## Recommended contract

### Clipboard and paste

- read only the current `NSPasteboard.general` plain text;
- enforce the native hard bound before application processing;
- use `changeCount` as content-blind revision evidence;
- reject a known revision change before one balanced Command-V chord;
- never write, clear, restore, cache, or monitor pasteboard content in the product;
- never retry paste after partial/unknown progress.

### Keyboard and modifiers

- emit one validated semantic action at a time through Core Graphics;
- preserve Unicode scalar semantics for composed clipboard text;
- treat line break, Tab, and Backspace as explicit actions;
- apply pacing/jitter/cancel/focus/modifier checks to every action;
- never synthesize release of a physical user modifier;
- stop on Secure Event Input or unknown native progress.

### Destination evidence

- capture frontmost process immediately at trigger;
- add focused Accessibility-element identity only when authorized;
- compare against the original evidence before every action;
- fail closed on change, disappearance, ambiguity, or post-start degradation;
- never read values, selections, document text, or window titles;
- state same-render-host limits explicitly.

### Permission

- query trust without prompting;
- prompt or open settings only after an explicit user action;
- distinguish the product's local onboarding state from the OS's current trust
  value without claiming unobservable denial history;
- detect granted-to-false as revocation while the process remains alive;
- no repeated prompt loop, TCC manipulation, or consent bypass.

### Hotkeys and shell

- keep one application-owned Carbon registration controller on the main run loop;
- validate and probe Trigger/Cancel as one pair;
- stage free candidates before releasing old registrations;
- preserve the complete previous pair on conflict/failure;
- capture no unrelated keys and keep no key history;
- retain one `NSStatusItem`, one menu, and one hidden-on-close settings window;
- cancel and wait boundedly before native teardown.

### Login item

- use only `SMAppService.mainApp` from the installed application bundle;
- expose not registered, enabled, approval required, not found, unsupported, and
  unknown states honestly;
- do not install a separate privileged helper.

## Thread ownership

| Surface | Required owner |
|---|---|
| AppKit application, status item, menu, settings presentation | main thread |
| Accessibility prompt and System Settings action | main thread, explicit user action |
| Carbon registration/probe/replacement | application/main run loop |
| Coordinator injection session | one bounded worker |
| Clipboard, target, modifier snapshots | bounded adapter calls owned by coordinator |
| Core Graphics input dispatch | coordinator worker, one semantic action per call |
| Shutdown | main shell requests cancel, waits boundedly, then drops native controllers |

## Permission matrix to observe

| Phase | Trust query | Expected product state | Expected input behavior |
|---|---:|---|---|
| clean start before explicit action | false | initial/not requested boundary | unavailable |
| prompt dismissed or explicit remediation not granted | false | not granted | unavailable |
| consent granted | true | granted | conditionally available |
| permission revoked while running after true was seen | false | revoked | stop/fail closed |
| native query/evidence cannot be interpreted | unknown | unknown | fail closed |

## Proposed production bounds

- current clipboard UTF-8 hard limit: 8 MiB;
- one semantic action per native dispatch;
- one fixed backend selected before dispatch;
- no target adoption/refocus;
- no automatic permission prompt;
- no public compatibility wording beyond named environments and applications;
- unsigned/ad-hoc artifacts remain development candidates only.

## Unresolved questions

1. Is `CFHash` stable enough for repeated focused-element identity in native,
   Chromium, editor, and terminal targets?
2. What exact identity change causes Accessibility consent loss across ad-hoc
   rebuild, Developer ID signing, app move, upgrade, and bundle replacement?
3. Does Core Graphics Unicode dispatch preserve all required fixture classes in
   each representative target, including decomposed combining marks?
4. What happens when Secure Event Input is enabled between action checks?
5. What is the measured focus-change and Cancel stop bound on each architecture?
6. Does Carbon pair replacement remain transactional under occupied candidates,
   cross-swap, rapid Apply, and active injection?
7. Does `SMAppService` require approval on each supported macOS version, and does
   exactly one process start after a real login cycle?
8. Can Intel runtime evidence still be obtained on a supported physical Mac?

## Exit condition

Run the complete case matrix on the exact candidate using the physical-Mac
runbook, attach sanitized evidence, and obtain independent review. Until every
required item is resolved and the final report explicitly changes to YES, this
spike remains `NO` and does not authorize merge, signing, notarization, tagging,
or release.
