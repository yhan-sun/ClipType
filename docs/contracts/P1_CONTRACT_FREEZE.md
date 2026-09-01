# P1 Native-Neutral Contract Freeze

**Issue:** [#3](../../issues/3)  
**Inputs:** accepted ADRs and [`P1_WINDOWS_NATIVE_SPIKE.md`](../research/P1_WINDOWS_NATIVE_SPIKE.md)  
**Scope:** types and ports required by the Windows keyboard-only vertical slice

## Package boundary

```text
cliptype-core
  sensitive values, count/bound units, immutable P1 configuration,
  semantic text atoms, capability/evidence vocabulary, outcomes

cliptype-platform -> cliptype-core
  clipboard, target, keyboard/modifier, native-error, and command-source ports

cliptype-app -> core + platform
  future live coordinator in P1-08

cliptype-windows -> core + platform
  future Win32 adapters in P1-04..P1-07
```

Neither `cliptype-core` nor `cliptype-platform` names a Win32 handle, callback, message, input structure, clipboard format constant, or Windows error type.

## Sensitive text

`SensitiveText` owns clipboard plaintext but does not implement `Clone`, `Display`, or serialization. Its `Debug` output contains only a UTF-8 byte count and `[REDACTED]`. Plaintext access is explicit through `expose` or consuming `into_inner`.

This reduces accidental disclosure and copies. It is not a promise that all operating-system or process-memory copies can be erased.

## Independent bounds

P1 represents independent units and limits for:

1. native clipboard bytes;
2. UTF-16 units;
3. normalized semantic elements;
4. per-dispatch semantic elements;
5. retry attempts;
6. native event counts.

A native allocation hard limit, total semantic payload limit, and dispatch batch limit are not interchangeable. Zero limits and overflowing `usize -> u32` native-event conversions fail closed.

The initial reviewed configuration is:

- native clipboard allocation: 8 MiB;
- total semantic payload: 65,536 elements;
- dispatch batch: 8 elements;
- inter-batch interval: 1 ms;
- modifier settle: 750 ms, sampled every 5 ms;
- clipboard retry: 8 attempts within 80 ms;
- worker shutdown grace: 2 seconds.

These are bounded P1 defaults, not public performance or compatibility claims.

## Semantic text

The native-neutral keyboard boundary accepts only:

- printable Unicode scalar;
- normalized line break;
- Tab when policy permits it.

`TextAtom::Scalar` and `TextBatch` redact their content in `Debug`. `TextBatch` is non-empty and checked against a dispatch limit before reaching a native adapter. P1-03 owns validation and normalization; P1-06 owns translation to native events.

## Clipboard port

`ClipboardPort::read_current_text` performs one bounded current-text acquisition attempt. It returns owned `SensitiveText` or content-free busy, empty, non-text, malformed, too-large, or native failure categories.

The adapter does not implement history, caching, a change listener, write/restore, or hidden retries. P1-08 owns cancellable retry timing.

## Target evidence

`TargetEvidence` contains an opaque `Any + Send + Sync` token plus safe process/thread metadata and an evidence-strength label. Normal formatting never exposes the token.

`TargetPort` captures evidence, compares expected and observed evidence, and reports integrity relation as:

- known restricted;
- known not restricted;
- unknown.

Comparison supports same, changed, disappeared, and unavailable/ambiguous. The contract does not promise exact logical-field or caret identity inside one native render host.

## Keyboard and modifier ports

`ModifierPort` reports clear, a mask of relevant held modifiers, or unknown. It never authorizes releasing physical user keys.

`KeyboardPort` receives one bounded semantic `TextBatch` and returns one of:

- complete;
- none accepted;
- partial native event acceptance;
- semantic progress unknown.

Native event counts contain no text. Every P1 dispatch result has retry disposition `Never`; the live coordinator must not duplicate a prefix after partial or unknown insertion.

## Command event source

`CommandEventSource` emits trigger, cancel, or shutdown. The trait intentionally has no `Send`/`Sync` supertrait because registration, message retrieval, and unregister belong to the platform owner thread. Native callbacks translate events only; they do not read clipboard text or dispatch input.

## Error and status posture

Errors/status expose only typed categories, numeric native codes where safe, counts, capability/evidence strength, and lifecycle state. They exclude clipboard text, injected text, samples, persistent fingerprints, focused-field contents, and window titles.

## Downstream branch rule

P1-03 through P1-07 must branch from the same merged P1-02 commit. Adapter pressure that cannot be represented by these contracts returns as a focused contract correction; an adapter must not create a private fork or move Windows policy into core.
