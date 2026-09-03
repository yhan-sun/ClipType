# P3 macOS Interactive Validation Runbook

## Purpose

This runbook executes Issue #48 (`P3-S01`) against one exact ClipType product
candidate. It separates:

- source/build evidence;
- hosted macOS observations;
- unlocked physical-Mac observations;
- inference and known limitations;
- Developer ID/notarization evidence, which belongs to the later release gate.

A green hosted runner is not proof that Accessibility consent, Core Graphics
input, focused-element identity, global shortcuts, `NSStatusItem`, or
`SMAppService` behave correctly in a real user session.

## Fixed candidate for this validation branch

- Repository: `yhan-sun/ClipType`
- Product PR: #55
- Product candidate SHA: `16cf71367506f05c5d40fdcf946dee5c29db596f`
- Validation branch: `validation/p3-s01-macos-native-evidence-16cf713`

The validation branch may add only evidence tooling and documentation. Before
preparation, the script verifies that no product path changed after the declared
candidate SHA.

## Safety and privacy rules

1. Use only generated fixtures from `tools/macos/p3_s01_target.swift`.
2. Do not copy passwords, tokens, source code, personal messages, or other real
   user content into ClipType during validation.
3. Do not record clipboard text, injected text, focused-field contents, window
   titles, usernames, native handles, Accessibility objects, or the privacy
   sentinel in evidence.
4. Keep the tested `.app` at one stable path during the permission flow. Moving,
   rebuilding, or replacing an app can change the identity macOS associates with
   Accessibility consent.
5. Never disable SIP, bypass TCC, edit the TCC database, auto-approve consent, or
   run ClipType as root.
6. Terminal fixtures must not contain newline or other command-execution
   controls.
7. Stop and record `FAIL` or `BLOCKED`; do not weaken product behavior merely to
   make a test pass.

## Required environments

At minimum record:

- one Apple Silicon Mac in an unlocked local Console session on the current
  supported macOS release;
- one additional supported macOS version where available, or an explicit
  limitation explaining why it was unavailable;
- Intel runtime evidence, or an explicit Intel-runtime-unverified boundary;
- exact macOS version/build, architecture, hardware model, Xcode, SDK, Rust,
  bundle version, binary SHA-256, and code-sign state.

Hosted Intel and Apple Silicon builds satisfy source/build coverage only. They do
not satisfy `ENV-01`, permission, input, focus, shell, or login-item cases.

## Prepare the physical-Mac evidence workspace

Use a clean checkout of the validation branch and a Universal 2 candidate built
from the exact product SHA. Place the app at its final test path before requesting
Accessibility permission.

```bash
git fetch origin \
  validation/p3-s01-macos-native-evidence-16cf713 \
  feat/p3-cross-platform-ui-macos

git switch --detach origin/validation/p3-s01-macos-native-evidence-16cf713

git rev-parse 16cf71367506f05c5d40fdcf946dee5c29db596f

# Example stable test install. Do not replace it after permission is granted.
ditto /path/to/ClipType.app /Applications/ClipType-P3-S01.app

tools/macos/p3_s01_prepare.sh \
  16cf71367506f05c5d40fdcf946dee5c29db596f \
  /Applications/ClipType-P3-S01.app \
  p3-s01-evidence-8c52a74b5ab8
```

Preparation creates:

- `environment.tsv` — content-free environment and bundle metadata;
- `case-matrix.tsv` / `results.tsv` — the complete gate matrix;
- `probes/runtime-snapshot.txt` — content-free adapter capability snapshot;
- `probes/hotkey-cycle.txt` — automated Carbon conflict/apply/restore probe;
- `probes/status-item-smoke.txt` — create/update/drop smoke;
- `bin/p3_s01_probe` — explicit native-mechanism probe;
- `bin/p3_s01_target` — controlled two-field AppKit target;
- `screenshots/` — synthetic/empty screenshots only.

## Record a result

Use only short content-free detail codes and bounded measurements:

```bash
tools/macos/p3_s01_record.sh \
  p3-s01-evidence-8c52a74b5ab8 \
  CASE-ID PASS detail_code measurement
```

Allowed statuses are `PASS`, `FAIL`, `LIMITATION`, `BLOCKED`, and `NOT_RUN`.
A limitation is not automatically acceptable; the final independent reviewer
must explicitly accept it through `FINAL-01`.

## Launch the controlled native target

```bash
P3_S01_RESULTS_JSONL="$PWD/p3-s01-evidence-8c52a74b5ab8/target-results.jsonl" \
  "$PWD/p3-s01-evidence-8c52a74b5ab8/bin/p3_s01_target"
```

The target contains two native `NSTextView` controls and generated fixture
classes: ASCII, CJK, emoji, decomposed combining marks, multiline, Tab, and long.
Its evidence file contains fixture class, field, PASS/FAIL, and byte/scalar counts
only; it never writes target text.

## Test sequence

### 1. Baseline and bundle

1. Confirm the app is not running.
2. Confirm `lipo -archs` reports both `arm64` and `x86_64`.
3. Inspect `Info.plist`, resources, `LSUIElement`, deployment target, and bundle
   identifier.
4. Run `codesign --verify --deep --strict`.
5. Record whether `spctl` accepts or rejects the candidate. An ad-hoc candidate
   is expected to remain distinct from a public Developer ID build.
6. Run source quality commands on the exact checkout:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --release --locked -p cliptype-macos-app
```

Record `BUILD-01`, `BUNDLE-01`, and `BUNDLE-02`.

### 2. Permission state before consent

1. Remove only the prior ClipType test entry through normal System Settings when
   beginning a clean permission test. Do not edit TCC databases.
2. Launch the exact installed app.
3. Observe the settings permission label before pressing any permission button.
4. Attempt one generated ASCII trigger into the controlled target.
5. Confirm no text is injected and the UI reports a content-free permission/input
   limitation.

Record `PERM-01`. If the product cannot distinguish the initial state from a
previously denied state, record that as a limitation rather than inventing OS
history. The current native API exposes current trust, not a durable denial log.

### 3. Explicit request, grant, and remediation

1. Press **Request permission** once.
2. Confirm the macOS-owned prompt/settings flow appears only after this action.
3. Dismiss without granting; confirm there is no repeated prompt loop.
4. Press **Open System Settings** and verify the correct Accessibility pane.
5. Grant the installed ClipType app.
6. Return without restarting and observe whether the product moves to `Granted`.
7. Trigger the generated ASCII fixture into field A and verify it.

Record `PERM-02`, `PERM-03`, and `PERM-06`.

The probe command below observes the same system APIs for the probe executable,
not the ClipType app identity. Use it only as mechanism evidence:

```bash
p3-s01-evidence-8c52a74b5ab8/bin/p3_s01_probe permission-watch 300
```

Do not substitute probe permission for product-app permission evidence.

### 4. App identity stability

With permission granted to the stable installed candidate:

1. quit and relaunch the same app path;
2. verify trust persists;
3. record the exact signature class (ad-hoc or Developer ID);
4. if testing a rebuild/replacement, record whether consent persisted or required
   remediation;
5. never silently replace the binary and call it the same test.

Record `PERM-04`.

### 5. Keyboard-mode fixture matrix

Select **Keyboard** mode, zero typo probability, zero or a recorded jitter, and a
known characters-per-second value. For each fixture:

1. choose the fixture in the controlled target;
2. press **Copy fixture**;
3. press **Focus A**;
4. invoke the physical Trigger shortcut;
5. wait for completion;
6. press **Verify A**;
7. record the corresponding `KEY-*` case.

Required classes:

- `KEY-01` ASCII;
- `KEY-02` Chinese/Japanese/Korean;
- `KEY-03` emoji;
- `KEY-04` decomposed combining marks;
- `KEY-05` multiline in the controlled non-terminal target;
- `KEY-06` Tab, including whether the target inserts Tab or interprets focus
  navigation.

Do not claim universal Unicode support from one target. Representative app
results remain separate.

### 6. Pacing, jitter, typo, and Cancel

1. Use a long ASCII fixture and a low recorded characters-per-second setting.
2. Measure release-to-first-action and several inter-action intervals with a
   screen recording or another content-free timing method.
3. Enable a bounded nonzero jitter and record observed min/median/max intervals.
4. Enable a bounded typo probability and verify visible
   `wrong -> Backspace -> intended` order.
5. Confirm each wrong key, Backspace, and corrected key consumes its own pacing
   slot.
6. Trigger the long fixture and use the physical Cancel shortcut repeatedly.
7. Record cancel release-to-stop bounds without storing target text.

Record `KEY-07`, `HOTKEY-05`, and any timing measurement in a short token such as
`median_ms=25`.

### 7. Modifier contamination

1. Hold Shift, Control, Option, or Command while triggering.
2. Confirm ClipType waits for release or stops with a content-free modifier
   result.
3. Confirm it never synthesizes modifier-up events for the physical key.
4. Repeat while modifiers change during active typing.

Record `KEY-08`.

### 8. Clipboard and Auto modes

For **Clipboard** mode:

1. use each safe fixture class in the controlled target;
2. verify one ordinary Command-V result;
3. verify the clipboard remains the same generated fixture afterward;
4. verify no duplicate paste after an uncertain result;
5. attempt a controlled pasteboard revision change using a second test utility;
   record whether the race can actually be observed or remains too narrow.

For **Auto** mode:

1. set a known threshold;
2. verify short generated text selects Keyboard;
3. verify long generated text selects Clipboard;
4. confirm the backend does not switch during a session.

Record `CLIP-01` through `CLIP-04`, `MODE-01`, and `MODE-02`.

### 9. Focus and destination evidence

Use the long fixture at a slow speed:

- `TARGET-01`: keep field A focused and verify stable detailed evidence;
- `TARGET-02`: switch to another top-level app during typing; no destination
  adoption is allowed;
- `TARGET-03`: close the target during typing; ClipType must stop without crash;
- `TARGET-04`: switch A to B during typing and measure the maximum characters
  delivered after the focus change;
- `TARGET-05`: repeat in a same-render-host application such as Chromium and
  state the exact limitation;
- `TARGET-06`: remove Accessibility permission during active/idle operation and
  verify evidence degradation fails closed.

The current coordinator dispatches one semantic action per check boundary, so a
new validation must measure the post-fix bound rather than reusing Issue #33's
older batch-boundary result.

### 10. Global-hotkey conflict, replacement, and rollback

The preparation probe runs a self-contained OS registration cycle. The product
UI still requires an interactive transaction test.

Hold a known candidate pair with the probe:

```bash
p3-s01-evidence-8c52a74b5ab8/bin/p3_s01_probe \
  hold-hotkeys cmd+alt+shift+f17 cmd+alt+shift+f18 300
```

While the holder runs:

1. record that pair in the ClipType settings UI;
2. check availability and Apply;
3. confirm the candidate is classified as occupied;
4. confirm the complete old Trigger/Cancel pair still works;
5. stop the holder;
6. apply the now-free pair without restarting;
7. verify Trigger and Cancel delivery while a session is active;
8. restore the original pair.

Record `HOTKEY-01` through `HOTKEY-05`. Swapping Trigger and Cancel should be
recorded separately if the current Carbon transaction treats cross-swap as
unsupported.

### 11. Menu bar, settings, and lifecycle

1. Confirm one monochrome/template menu-bar icon and one native menu.
2. Verify Trigger, Cancel, Open Settings, Enabled, mode status, permission,
   Start at Login, About, and Quit.
3. Open/close/reopen Settings repeatedly; there must be one window and one status
   item.
4. Confirm closing hides rather than quits.
5. Confirm opening/dismissing Settings does not adopt the settings window as the
   later external injection destination.
6. Quit while idle and during a long injection; confirm bounded shutdown.
7. Relaunch and verify no duplicate processes, status items, or registrations.

Record `SHELL-01` through `SHELL-04`.

### 12. Start at Login

From the installed app bundle:

1. inspect the initial `SMAppService.mainApp` status;
2. enable Start at Login;
3. handle a system approval-required state honestly;
4. disable and verify unregister;
5. perform a real logout/login cycle where feasible;
6. confirm exactly one app instance starts and no helper outside the app-owned
   service is installed.

Record `STARTUP-01` and `STARTUP-02`. A registry/plist-only inspection without a
login cycle is a limitation, not full runtime proof.

### 13. Permission revocation while running

1. begin with `Granted` visible;
2. remove ClipType from Accessibility through System Settings while it runs;
3. observe the transition without restarting;
4. attempt a trigger and confirm no input;
5. confirm no automatic reprompt or bypass;
6. use the explicit remediation action to recover.

Record `PERM-05` and `TARGET-06`.

### 14. Representative applications

Use generated, benign fixtures only:

- one Chromium editor/textarea in Keyboard, Clipboard, and Auto modes;
- one code editor in Keyboard mode;
- one terminal with a short fixture containing no newline, Tab, shell metacharacter
  sequence, or command text.

Record exact application versions and behavior, but do not overgeneralize to all
applications. Record `APP-02` through `APP-04`.

### 15. Accessibility and visual smoke

1. Navigate all settings pages and controls with keyboard only.
2. Run a VoiceOver smoke for page names, shortcut controls, permission actions,
   warnings, Apply, and Reset.
3. Capture light, dark, and Retina screenshots using empty/synthetic state only.
4. Inspect screenshots before publication.

Record `ACCESS-01` and `VISUAL-01`.

### 16. Privacy sentinel

Run the dedicated script. It generates a synthetic marker without printing or
writing it into evidence, places it on the clipboard, asks for one physical
trigger into the controlled target, checks clipboard invariance, scans bounded
product/evidence roots, clears the test clipboard, and records only match count.

```bash
tools/macos/p3_s01_privacy_sentinel.sh \
  p3-s01-evidence-8c52a74b5ab8 \
  /Applications/ClipType-P3-S01.app
```

Then inspect public evidence manually and record `PRIV-02`.

## Finalize

An independent reviewer must inspect the exact environment, results, screenshots,
probe output, target result counts, limitations, and unresolved questions. Only
after that review may `FINAL-01` be recorded as PASS.

```bash
tools/macos/p3_s01_finalize.sh p3-s01-evidence-8c52a74b5ab8
```

The finalizer returns success only when every required case is `PASS` or an
explicitly reviewed `LIMITATION`, no case is failed/blocked/not-run, and
`FINAL-01` is PASS. Otherwise it produces:

```text
P3 macOS production adapters may proceed: NO
```

A P3-S01 YES authorizes production adapter work only. It does not authorize a
merge, tag, Developer ID signature, notarization, Gatekeeper claim, or public
release. Those remain separate exact-candidate gates.
