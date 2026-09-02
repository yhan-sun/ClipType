# Windows icon resource identifiers

- `1`: ClipType executable/application icon.
- `2`: ClipType notification-area icon.

The identifiers are stable for the Windows beta channel. The tray adapter loads id `2` from the current executable module. Explorer and shortcut surfaces use id `1` as the primary icon group.
