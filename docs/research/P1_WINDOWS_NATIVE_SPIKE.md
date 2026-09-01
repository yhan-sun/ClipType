# P1-S01 Windows Native Mechanism Spike

**Issue:** [#13](../../issues/13)  
**Purpose:** establish conservative contracts and runtime ownership before production adapters  
**Evidence classes:** official API contract, automated Windows runner probe, interactive desktop evidence

## 1. Result

**P1-02 contract freeze may proceed: YES, with explicit unknown/degraded states.**

The official Windows contracts and the automated thread/message-queue probe are sufficient to choose a conservative native-neutral API. They are **not** sufficient to claim interactive target compatibility. P1-10 remains responsible for unlocked interactive-desktop text delivery, representative applications, focus switching, timing measurements, and privacy-sentinel evidence.

No result in this report is a public support claim.

## 2. Evidence discipline

### Verified from official API contracts

The Microsoft API documentation establishes:

- `RegisterHotKey` with a null window associates the registration with the calling thread and posts `WM_HOTKEY` to that thread's message queue. `MOD_NOREPEAT` suppresses repeat notifications while the key remains held.
- `SendInput` returns the count of native input events inserted. It is constrained by UIPI and can inject only into equal- or lower-integrity targets. Neither its return value nor `GetLastError` identifies UIPI as the cause of a zero result.
- `SendInput` does not reset the current keyboard state; physical modifier state can affect injected events.
- `KEYEVENTF_UNICODE` uses Unicode packet semantics. A `KEYBDINPUT` carries a UTF-16 unit in `wScan`, so a supplementary scalar spans a surrogate pair and multiple native events.
- the high-order bit returned by `GetAsyncKeyState` reports whether a key is currently down.
- `GetForegroundWindow` can return null, and `GetGUIThreadInfo` exposes active/focus/caret native handles only when Windows can provide them.
- a handle returned by `GetClipboardData` remains owned by the system. ClipType must copy bounded data while the clipboard/handle is valid and must not free or leave that handle locked.

Primary references:

- <https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-registerhotkey>
- <https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-sendinput>
- <https://learn.microsoft.com/windows/win32/api/winuser/ns-winuser-keybdinput>
- <https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-getasynckeystate>
- <https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-getforegroundwindow>
- <https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-getguithreadinfo>
- <https://learn.microsoft.com/windows/win32/dataxchg/using-the-clipboard>

### Automated Windows runner probe

`crates/cliptype-windows/examples/p1_s01_windows_native.rs` performs a content-free automated probe on `windows-2025`:

1. creates/uses the current thread message queue;
2. registers a thread-owned `Ctrl+Alt+Shift+F24` development hotkey with no-repeat semantics;
3. starts a worker that posts a private application message to the owner thread;
4. blocks the owner in `GetMessageW` for that private message;
5. proves the owner receives the signal while work is on another thread;
6. unregisters the hotkey;
7. samples only the aggregate count of held modifier keys.

Verified on head `828a426b7e7d91978c3d0bb77b6edcf83bcaa425`:

- P1 Windows Native Spike run `33518031519`: `success`;
- Rust CI run `33518031529`: formatting, Linux native-neutral checks, and Windows workspace checks all `success`.

This proves registration/message-queue/worker signalling and teardown on the runner. The private posted message is not proof that Windows generated a real `WM_HOTKEY`, and the runner is not treated as representative interactive-desktop input evidence.

### Not yet verified interactively

The following remain P1-10 gate evidence:

- real global hotkey activation from another foreground application;
- exact modifier-release timing after a human trigger;
- `SendInput` delivery into controlled Win32, Chromium, VS Code, terminal, and elevated targets;
- target-specific CJK, supplementary Unicode, combining mark, line-break, and Tab behavior;
- observed partial native insertion;
- focus changes between native controls and logical fields sharing one render host;
- clipboard contention timing on a user's interactive desktop.

## 3. Runtime ownership recommendation

```text
Windows message-loop owner
  owns RegisterHotKey / UnregisterHotKey / GetMessageW
  translates WM_HOTKEY and shutdown into typed commands
  never performs clipboard retry or SendInput work
                 |
                 v
application coordinator
  atomically reserves one active-session slot
  captures destination evidence immediately
  owns cancellation and content-free status
  starts exactly one bounded worker
                 |
                 v
injection worker
  waits for physical modifiers to settle
  acquires clipboard with bounded retries
  normalizes and plans semantic text
  dispatches bounded native batches
  checks cancel/focus/modifiers between batches
```

A standard thread plus bounded channels/atomics is sufficient for P1. A full async runtime is not justified by the current evidence.

## 4. Contract recommendations for P1-02

### Sensitive text

Use a wrapper whose ordinary `Debug` output is redacted. Do not implement persistence/serialization by default and do not claim guaranteed memory erasure.

### Bounds

Represent three independent limits:

1. native clipboard acquisition hard bound;
2. total normalized semantic-element bound;
3. per-`SendInput` dispatch batch bound.

All byte, UTF-16-unit, Unicode-scalar, semantic-element, `u32`, and `i32` conversions must be checked. Invalid or overflowing values fail closed.

### Target evidence

Expose opaque comparable evidence plus safe application/process metadata. Comparisons must support `same`, `changed`, `disappeared`, and `unavailable/ambiguous`. Do not expose raw Win32 handles in core policy or promise exact logical-field/caret identity.

### Integrity evidence

Use a tri-state relation: `known restricted`, `known not restricted`, or `unknown`. A zero `SendInput` result with unknown integrity must remain `blocked/native cause unknown`, not be labelled definitely UIPI.

### Dispatch result

Represent:

- complete native batch;
- no events accepted;
- partial native event acceptance;
- semantic progress unknown.

Because a returned event count can end inside a key-down/up pair or UTF-16 surrogate sequence, the adapter must not invent an exact text prefix. Partial/unknown outcomes are never retried automatically.

### Modifier evidence

Expose bounded observation of Ctrl, Alt, Shift, and Windows keys. The coordinator may wait or abort; ClipType never releases physical user keys.

### Thread/lifecycle

The hotkey event source is thread-affine. Its registration, message pump, and teardown stay on the owner thread. Core/app contracts exchange typed commands/status rather than exposing Win32 callbacks.

## 5. Conservative P1 starting bounds

These are engineering starting points, not compatibility claims. P1-10 must measure and may tighten them without weakening safety.

| Setting | Initial value | Reason |
|---|---:|---|
| hard clipboard allocation bound | 8 MiB | prevents untrusted clipboard allocation/scan growth |
| total semantic payload | 65,536 elements | bounded development slice while still exercising long sessions |
| native dispatch batch | 8 semantic elements | frequent cancel/focus/modifier checkpoints |
| inter-batch interval | 1 ms | conservative target pacing; configurable in tests |
| modifier settle timeout | 750 ms | permits normal key release but remains bounded |
| modifier poll interval | 5 ms | responsive without a busy loop |
| clipboard retry budget | 8 attempts / 80 ms total | handles transient ownership without target drift becoming unbounded |
| worker shutdown grace | 2 seconds | bounded host teardown before reporting incomplete shutdown |

P1-02 should encode units and validation rather than hard-coding assumptions into native adapters.

## 6. Text mapping recommendation

Core policy emits semantic atoms:

- printable Unicode scalar;
- normalized line break;
- Tab when enabled;
- unsupported control error.

The Windows adapter converts printable scalars to UTF-16 Unicode key-down/key-up events. Line break and Tab mappings remain platform mechanisms and must be verified against target categories. Combining marks retain source order; ClipType does not normalize user text into a different Unicode normalization form.

## 7. Focus guarantee

P1 can promise bounded revalidation against available Windows evidence such as foreground top-level window, owning process/thread, and native focused window. It cannot yet promise that two logical fields implemented inside the same render host are distinguishable. Strict mode stops before a later batch when evidence is known changed, disappeared, or becomes unavailable after injection starts.

## 8. Clipboard ownership recommendation

The adapter performs one bounded acquisition attempt and copies UTF-16 data into owned sensitive memory before unlock/close. It validates the native allocation size, searches for the terminator only within that bound, and returns typed busy/non-text/empty/malformed/too-large outcomes. The application coordinator owns cancellable retry timing.

P1 does not need a clipboard listener, history, write, or restoration transaction.

## 9. Security and privacy

The probe does not read clipboard contents or call `SendInput`. It records no window title, focused text, raw user keys, content sample, or content fingerprint. The production path must retain the same content-free diagnostic posture.

## 10. Gate handoff

P1-02 may freeze conservative contracts from this report provided that:

- unknown/degraded evidence remains representable;
- partial/unknown dispatch remains non-retryable;
- all native operations and waits are bounded;
- interactive behavior is not claimed from hosted-runner evidence;
- P1-10 remains a mandatory phase gate before any compatibility/support claim.
