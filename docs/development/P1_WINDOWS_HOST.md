# P1 Windows Development Host

The P1 host is the smallest executable composition of ClipType's current Windows vertical slice. It is a development console host, not the P2 tray/settings product shell and not a release artifact.

## Run

On an interactive Windows desktop:

```powershell
cargo run --locked -p cliptype
```

The host registers:

- trigger: `Ctrl+Alt+Shift+F12`;
- cancellation: `Ctrl+Alt+Shift+F11`;
- controlled host shutdown: press Enter in the console.

Before pressing the trigger, copy plain text and focus the intended destination. The coordinator captures destination evidence at trigger time, waits for physical modifiers to clear, reads the current Unicode clipboard once with bounded retry, plans text, revalidates the destination, and emits bounded native batches.

## Runtime ownership

```text
Windows message-queue owner
  -> RegisterHotKey / GetMessageW / UnregisterHotKey
  -> forwards typed Trigger / Cancel / Shutdown commands

Coordinator
  -> atomically reserves one session
  -> captures target before clipboard acquisition
  -> owns cancellation, retry, focus/modifier checks and status

Injection worker
  -> reads clipboard through the port
  -> builds the pure keyboard plan
  -> dispatches one bounded SendInput batch at a time
```

Clipboard retry and synthetic input never run on the message-queue owner.

## Diagnostics

The console may print only typed lifecycle categories, generation, batch counts and completion categories. It does not print clipboard text, injected text, samples, prefixes/suffixes, content hashes, focused text or window titles.

## Automated smoke

`P1 Windows Host Smoke` runs on a Windows hosted runner and proves that the executable builds, registers its development hotkeys, receives a private shutdown message posted from another thread, unregisters, and exits cleanly.

This hosted smoke does **not** prove physical hotkey activation or foreground text injection into Chromium, VS Code, terminals, elevated applications or other representative desktop targets. Those observations remain a separate interactive compatibility follow-up and must not be inferred from a green workflow.
