# Code-mode corrected-typo acceptance matrix

This note records the beta.6 behavior boundary for corrected-typo simulation in Code mode.

- The configured typo probability applies to source `CodeAction::Atom` actions.
- A selected typo is dispatched as `wrong atom -> Backspace -> correct source atom`.
- `CursorRight` and `CursorRightToLineEnd` are never typo-simulated.
- Temporary wrong atoms are restricted to a Code-safe ASCII subset and cannot be brackets, quotes, or `/`, so the simulation does not itself trigger pair/comment state.
- Non-ASCII source atoms do not receive fabricated QWERTY typos.
- Code mode remains keyboard-only and never falls back to paste.

Pre-PR native-neutral validation covers formatting, `cliptype-core`, `cliptype-platform`, and `cliptype-app` tests and Clippy with warnings denied. The authoritative release decision remains the exact-head PR matrix across Rust CI, Windows controlled/product/package/release/compatibility gates, and the macOS arm64 build/package gate.
