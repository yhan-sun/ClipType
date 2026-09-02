//! Bounded Unicode-oriented keyboard dispatch for Windows.

use std::mem::size_of;

use cliptype_core::{CapabilityState, NativeEventCount, TextAtom, TextBatch};
use cliptype_platform::{
    DispatchResult, KeyboardCapabilities, KeyboardError, KeyboardPort, ModifierMask,
    ModifierObservation, ModifierPort, NativeDispatchCount, NativeError, NativeErrorKind,
};
use windows_sys::Win32::{
    Foundation::GetLastError,
    UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, SendInput, VK_BACK, VK_CONTROL, VK_LWIN, VK_MENU, VK_RETURN, VK_RWIN,
        VK_SHIFT, VK_TAB,
    },
};

/// Stateless Windows keyboard adapter.
///
/// Each `dispatch` call emits exactly one already-bounded semantic batch. The
/// application coordinator owns pacing, cancellation, target checks, and every
/// decision about whether another batch may be attempted.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsKeyboard;

impl WindowsKeyboard {
    pub const fn new() -> Self {
        Self
    }
}

impl KeyboardPort for WindowsKeyboard {
    fn capabilities(&self) -> KeyboardCapabilities {
        KeyboardCapabilities {
            unicode_text: CapabilityState::Available,
            line_break: CapabilityState::Available,
            tab: CapabilityState::Available,
            modifier_observation: CapabilityState::Available,
        }
    }

    fn dispatch(&self, batch: TextBatch<'_>) -> Result<DispatchResult, KeyboardError> {
        dispatch_encoded(encode_batch(batch)?)
    }

    fn dispatch_backspace(&self) -> Result<DispatchResult, KeyboardError> {
        dispatch_encoded(encode_backspace())
    }
}

impl ModifierPort for WindowsKeyboard {
    fn observe_modifiers(&self) -> ModifierObservation {
        let mut held = ModifierMask::NONE;
        if key_is_down(VK_SHIFT) {
            held = held | ModifierMask::SHIFT;
        }
        if key_is_down(VK_CONTROL) {
            held = held | ModifierMask::CONTROL;
        }
        if key_is_down(VK_MENU) {
            held = held | ModifierMask::ALT;
        }
        if key_is_down(VK_LWIN) || key_is_down(VK_RWIN) {
            held = held | ModifierMask::WINDOWS;
        }

        if held.is_empty() {
            ModifierObservation::Clear
        } else {
            ModifierObservation::Held(held)
        }
    }
}

fn key_is_down(key: u16) -> bool {
    // SAFETY: the virtual-key value is one of the fixed modifier constants.
    unsafe { GetAsyncKeyState(i32::from(key)) < 0 }
}

struct EncodedBatch {
    inputs: Vec<INPUT>,
    semantic_boundaries: Vec<u32>,
}

fn dispatch_encoded(encoded: EncodedBatch) -> Result<DispatchResult, KeyboardError> {
    let requested = NativeEventCount::try_from(encoded.inputs.len())
        .map_err(|_| KeyboardError::InvalidBatch)?;
    let input_size = i32::try_from(size_of::<INPUT>()).map_err(|_| KeyboardError::InvalidBatch)?;

    // SAFETY: `encoded.inputs` is initialized and remains alive for the whole
    // call, `requested` exactly matches its length, and `input_size` is the ABI
    // size of one `INPUT` value.
    let accepted = unsafe { SendInput(requested.get(), encoded.inputs.as_ptr(), input_size) };

    let zero_reason = if accepted == 0 {
        // `SendInput` does not identify UIPI as the cause of a zero result.
        let code = unsafe { GetLastError() };
        Some(NativeError::new(
            NativeErrorKind::BlockedCauseUnknown,
            (code != 0).then_some(code),
        ))
    } else {
        None
    };

    classify_accepted(
        requested,
        accepted,
        &encoded.semantic_boundaries,
        zero_reason,
    )
}

fn encode_backspace() -> EncodedBatch {
    EncodedBatch {
        inputs: vec![
            keyboard_input(VK_BACK, 0, 0),
            keyboard_input(VK_BACK, 0, KEYEVENTF_KEYUP),
        ],
        semantic_boundaries: vec![2],
    }
}

fn encode_batch(batch: TextBatch<'_>) -> Result<EncodedBatch, KeyboardError> {
    let capacity = batch
        .len()
        .checked_mul(4)
        .ok_or(KeyboardError::InvalidBatch)?;
    let mut inputs = Vec::with_capacity(capacity);
    let mut semantic_boundaries = Vec::with_capacity(batch.len());

    for atom in batch.atoms() {
        match atom {
            TextAtom::Scalar(value) => push_scalar(&mut inputs, *value),
            TextAtom::LineBreak => push_virtual_key(&mut inputs, VK_RETURN),
            TextAtom::Tab => push_virtual_key(&mut inputs, VK_TAB),
        }

        semantic_boundaries
            .push(u32::try_from(inputs.len()).map_err(|_| KeyboardError::InvalidBatch)?);
    }

    if inputs.is_empty() {
        return Err(KeyboardError::InvalidBatch);
    }

    Ok(EncodedBatch {
        inputs,
        semantic_boundaries,
    })
}

fn push_scalar(inputs: &mut Vec<INPUT>, value: char) {
    let mut storage = [0_u16; 2];
    for unit in value.encode_utf16(&mut storage).iter().copied() {
        inputs.push(keyboard_input(0, unit, KEYEVENTF_UNICODE));
        inputs.push(keyboard_input(0, unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
    }
}

fn push_virtual_key(inputs: &mut Vec<INPUT>, key: u16) {
    inputs.push(keyboard_input(key, 0, 0));
    inputs.push(keyboard_input(key, 0, KEYEVENTF_KEYUP));
}

fn keyboard_input(virtual_key: u16, scan: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: virtual_key,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn classify_accepted(
    requested: NativeEventCount,
    accepted: u32,
    semantic_boundaries: &[u32],
    zero_reason: Option<NativeError>,
) -> Result<DispatchResult, KeyboardError> {
    if accepted > requested.get() {
        return Err(KeyboardError::Native(NativeError::new(
            NativeErrorKind::InvalidData,
            None,
        )));
    }

    if accepted == requested.get() {
        return Ok(DispatchResult::Complete { events: requested });
    }
    if accepted == 0 {
        return Ok(DispatchResult::NoneAccepted {
            requested,
            native: zero_reason,
        });
    }

    let counts = NativeDispatchCount {
        requested,
        accepted: NativeEventCount::new(accepted),
    };
    if semantic_boundaries.binary_search(&accepted).is_ok() {
        Ok(DispatchResult::Partial { counts })
    } else {
        Ok(DispatchResult::ProgressUnknown { counts })
    }
}

#[cfg(test)]
mod tests {
    use cliptype_core::{DispatchBatchLimit, NativeEventCount, TextAtom, TextBatch};
    use cliptype_platform::{DispatchResult, KeyboardError};

    use super::{classify_accepted, encode_backspace, encode_batch};

    fn batch<'a>(atoms: &'a [TextAtom]) -> TextBatch<'a> {
        TextBatch::new(
            atoms,
            DispatchBatchLimit::new(atoms.len()).expect("non-empty fixture"),
        )
        .expect("bounded fixture")
    }

    #[test]
    fn scalar_encoding_tracks_supplementary_boundaries() {
        let atoms = [TextAtom::scalar('A'), TextAtom::scalar('😀')];
        let encoded = encode_batch(batch(&atoms)).expect("supported batch");

        assert_eq!(encoded.inputs.len(), 6);
        assert_eq!(encoded.semantic_boundaries, vec![2, 6]);
    }

    #[test]
    fn cjk_scalar_is_one_complete_unicode_action() {
        let atoms = [TextAtom::scalar('你')];
        let encoded = encode_batch(batch(&atoms)).expect("supported batch");

        assert_eq!(encoded.inputs.len(), 2);
        assert_eq!(encoded.semantic_boundaries, vec![2]);
    }

    #[test]
    fn corrective_backspace_is_one_complete_key_pair() {
        let encoded = encode_backspace();

        assert_eq!(encoded.inputs.len(), 2);
        assert_eq!(encoded.semantic_boundaries, vec![2]);
    }

    #[test]
    fn line_break_and_tab_are_complete_key_pairs() {
        let atoms = [TextAtom::LineBreak, TextAtom::Tab];
        let encoded = encode_batch(batch(&atoms)).expect("supported batch");

        assert_eq!(encoded.inputs.len(), 4);
        assert_eq!(encoded.semantic_boundaries, vec![2, 4]);
    }

    #[test]
    fn native_acceptance_is_conservative_at_partial_boundaries() {
        let requested = NativeEventCount::new(6);
        let boundaries = [2, 6];

        assert!(matches!(
            classify_accepted(requested, 6, &boundaries, None),
            Ok(DispatchResult::Complete { .. })
        ));
        assert!(matches!(
            classify_accepted(requested, 2, &boundaries, None),
            Ok(DispatchResult::Partial { .. })
        ));
        assert!(matches!(
            classify_accepted(requested, 3, &boundaries, None),
            Ok(DispatchResult::ProgressUnknown { .. })
        ));
        assert!(matches!(
            classify_accepted(requested, 0, &boundaries, None),
            Ok(DispatchResult::NoneAccepted { .. })
        ));
    }

    #[test]
    fn impossible_native_count_fails_closed() {
        let result = classify_accepted(NativeEventCount::new(2), 3, &[2], None);

        assert!(matches!(result, Err(KeyboardError::Native(_))));
    }
}
