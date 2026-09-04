# ClipType P4 local findings

This file records limitations of the local run. It contains no clipboard
fixture, focused content, user identity, absolute home path, or credential.

| ID | Status | Finding | Disposition |
|---|---|---|---|
| P4-01 | BLOCKED | macOS Accessibility was not granted or revoked during this run. | The UI truthfully showed a no-trust state; the final state machine distinguishes `Not requested` from `Not granted`. Changing macOS security settings was left to an explicit user action. End-to-end cross-application input remains unverified. |
| P4-02 | NOT RUN | Chinese/Japanese/Korean/emoji/combining/multiline matrix in TextEdit, browser, editor, and terminal. | Rust Auto routing and macOS adapters are implemented and covered by deterministic policy tests; physical target evidence is still required. |
| P4-03 | NOT RUN | Trigger/Cancel injection, long-session cancellation, focus switch, target close, and clipboard revision change. | Accessibility permission and a controlled, non-sensitive target fixture were not available for this run. |
| P4-04 | NOT RUN | Occupied-shortcut conflict, rollback, release, and restart persistence. | OS probe of the current pair passed; an independent conflict owner was not started. |
| P4-05 | BLOCKED | Physical complete-combination recorder delivery was not established through the desktop automation input path. | Escape cancellation, Delete clearing, static validation, and recorder behavior are covered by Flutter tests; no physical PASS is claimed. |
| P4-06 | NOT RUN | Five cold-start timings, ten Settings reopen timings, trigger-to-first-action latency, and cancel-to-stop latency. | One stable idle sample was collected; latency and repeated timing budget are still open. |
| P4-07 | NOT RUN | User-controlled Start at Login enable/disable persistence. | The `SMAppService` adapter is wired; no login-item state was changed during this run. |
| P4-08 | NOT RUN | Developer ID signing, notarization, stapling, and Gatekeeper assessment. | Deliberately outside the unsigned local candidate; no Apple credentials were used. |
| P4-09 | BLOCKED | The standard AppleEvent quit request returned macOS `-128` while the ClipType process then exited; the menu-item path was not isolated from that request. | No process leak was observed; keep the menu-level quit path as a follow-up smoke check. |
| P4-10 | BLOCKED | Final clean-release CUA re-inspection was attempted after relaunch, but macOS was locked and automatic unlock was unavailable. | The same build family and UI had already been inspected while unlocked; no additional visual claim is made for the locked attempt. |
| P4-11 | NOT RUN | `flutter doctor -v` retained an intentional exact-tag unknown-channel warning and an Xcode Simulator-runtime enumeration warning. | Doctor exited 0 and the macOS desktop Release build passed; no simulator path was required for this arm64 desktop run. |
| P4-12 | NOT RUN | Physical visual inspection after switching the Settings UI to Simplified Chinese. | The localization implementation and deterministic vocabulary test passed, but the desktop was locked before the switched UI could be visually re-inspected. |

None of these findings is converted into a release claim. The final report is
`LOCAL P4 RUN PARTIAL` because the real UI and build are working but required
physical permission and target matrices remain open.
