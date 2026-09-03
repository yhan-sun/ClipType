# ClipType v0.2.0-beta.1

This prerelease adds the P3 cross-platform product slice:

- native-compiled graphical Settings UI;
- user-recorded Trigger and Cancel shortcuts;
- operating-system conflict probing, atomic replacement, and rollback;
- live Windows shortcut updates;
- macOS menu-bar app, Accessibility onboarding, clipboard/revision guard, Unicode keyboard input, Command+V, focus protection, cancellation, and Start at Login;
- Apple Silicon and Intel builds assembled as a Universal 2 application;
- ZIP and DMG packaging.

## macOS distribution status

The first published macOS assets are an **ad-hoc-signed, unnotarized testing preview**. They are not Developer ID signed and Apple notarization has not been completed. Gatekeeper may block or warn when opening them. The asset names include `UNSIGNED` to prevent confusion with a normally signed public macOS build.

The maintainer requested publish-first sequencing. Physical Apple Silicon/Intel compatibility, Accessibility grant/revoke, global-hotkey conflicts and rollback, representative application behavior, Login Item behavior, privacy sentinel, and signed/notarized promotion are tracked in Issue #61 after publication.

Do not interpret this prerelease as a broad macOS compatibility claim or as a completed Developer ID/notarized distribution.
