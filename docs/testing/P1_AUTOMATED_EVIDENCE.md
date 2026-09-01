# P1 Automated Evidence

This report records automated evidence for the Windows keyboard-only vertical slice. It deliberately separates hosted-runner results from physical interactive desktop and representative-application observations.

## Implemented automated layers

### Deterministic native-neutral policy and coordinator tests

The workspace verifies text normalization, Unicode semantic elements, immutable bounded plans, one-session arbitration, target-before-clipboard ordering, cancellation, modifier settling, clipboard retry budgets, target loss/change, bounded multi-batch dispatch, partial/progress-unknown no-retry, integrity classifications, shutdown and privacy-safe status.

### Windows adapter tests

Windows jobs compile, lint and test bounded clipboard acquisition, foreground/native-focus evidence, integrity queries, Unicode `SendInput` event construction, conservative accepted-event classification, modifier observation, hotkey registration/message-queue ownership and cleanup.

### Windows host smoke

`P1 Windows Host Smoke` starts the real development executable on a Windows hosted runner, registers both global development hotkeys, receives a private cross-thread shutdown command, unregisters and exits.

### Controlled Win32 edit E2E

`P1 Controlled Windows E2E` creates an isolated visible Win32 multiline edit target, installs a generated clipboard fixture, activates/focuses the controlled target, runs the real `Coordinator` with the real Windows clipboard/target/keyboard adapters, pumps the target queue, and compares the final UTF-16 text without printing content. The job runs three isolated repetitions and checks that its distinctive privacy sentinel never enters ordinary output.

The fixture covers ASCII, punctuation, CJK, supplementary Unicode, a combining mark and normalized line break behavior. Tab/control-policy behavior remains covered by deterministic and native event-construction tests because physical Tab intentionally changes focus in many desktop controls.

## Evidence strength

A successful controlled hosted-runner E2E proves the complete implementation path in that constrained Windows environment. It does not prove:

- a human physical global-hotkey activation;
- Chromium, Electron/VS Code or terminal behavior;
- logical-field changes hidden inside one render host;
- administrator/elevated target behavior in an unlocked user desktop;
- measured physical modifier-release timing on a user's keyboard.

Those items remain an explicit compatibility follow-up before broad Windows support claims or a public beta. They are not silently marked as passed by this report.

## Privacy method

All ordinary runtime and test output is limited to result categories, UTF-16 unit counts, generation, batch counts and timing/status categories. The generated fixture is allowed only inside the controlled clipboard and target assertion buffers. The workflow fails if its distinctive marker appears in ordinary output.
