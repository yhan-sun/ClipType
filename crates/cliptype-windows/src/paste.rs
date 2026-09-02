//! Revision-guarded current-clipboard Paste dispatch for Windows.

use std::mem::size_of;

use cliptype_core::{CapabilityState, NativeEventCount};
use cliptype_platform::{
    ClipboardRevision, DispatchResult, NativeDispatchCount, NativeError, NativeErrorKind,
    PasteCapabilities, PasteError, PastePort,
};
use windows_sys::Win32::{
    Foundation::GetLastError,
    UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_CONTROL,
    },
};

#[link(name = "user32")]
unsafe extern "system" {
    #[link_name = "GetClipboardSequenceNumber"]
    fn get_clipboard_sequence_number() -> u32;
}

const VK_V: u16 = 0x56;
const PASTE_EVENT_COUNT: u32 = 4;

/// Stateless current-clipboard Paste adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsPaste;

impl WindowsPaste {
    pub const fn new() -> Self {
        Self
    }
}

impl PastePort for WindowsPaste {
    fn capabilities(&self) -> PasteCapabilities {
        PasteCapabilities {
            paste_chord: CapabilityState::Available,
            clipboard_revision_guard: CapabilityState::Available,
        }
    }

    fn dispatch_paste(
        &self,
        expected_revision: ClipboardRevision,
    ) -> Result<DispatchResult, PasteError> {
        verify_revision(expected_revision)?;
        let inputs = paste_inputs();
        let input_size =
            i32::try_from(size_of::<INPUT>()).map_err(|_| PasteError::InvalidRequest)?;

        // SAFETY: `inputs` contains exactly four initialized keyboard INPUT
        // records and remains alive throughout the call. One call preserves the
        // intended chord ordering without interleaving ClipType events.
        let accepted = unsafe { SendInput(PASTE_EVENT_COUNT, inputs.as_ptr(), input_size) };
        let native = if accepted == 0 {
            // `SendInput` does not identify UIPI as the cause. Preserve only a
            // numeric code and classify the cause as unknown.
            let code = unsafe { GetLastError() };
            Some(NativeError::new(
                NativeErrorKind::BlockedCauseUnknown,
                (code != 0).then_some(code),
            ))
        } else {
            None
        };

        classify_accepted(accepted, native)
    }
}

fn verify_revision(expected: ClipboardRevision) -> Result<(), PasteError> {
    let ClipboardRevision::Known(expected) = expected else {
        return Err(PasteError::InvalidRequest);
    };
    // SAFETY: the sequence query is content-blind and has no pointer or
    // ownership preconditions.
    let observed = unsafe { get_clipboard_sequence_number() };
    if observed == 0 {
        return Err(PasteError::Native(NativeError::new(
            NativeErrorKind::TemporarilyUnavailable,
            None,
        )));
    }
    if expected != u64::from(observed) {
        return Err(PasteError::ClipboardChanged);
    }
    Ok(())
}

fn paste_inputs() -> [INPUT; PASTE_EVENT_COUNT as usize] {
    [
        keyboard_input(VK_CONTROL, 0),
        keyboard_input(VK_V, 0),
        keyboard_input(VK_V, KEYEVENTF_KEYUP),
        keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP),
    ]
}

fn keyboard_input(key: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn classify_accepted(
    accepted: u32,
    zero_reason: Option<NativeError>,
) -> Result<DispatchResult, PasteError> {
    let requested = NativeEventCount::new(PASTE_EVENT_COUNT);
    match accepted {
        PASTE_EVENT_COUNT => Ok(DispatchResult::Complete { events: requested }),
        0 => Ok(DispatchResult::NoneAccepted {
            requested,
            native: zero_reason,
        }),
        1..=3 => Ok(DispatchResult::ProgressUnknown {
            counts: NativeDispatchCount {
                requested,
                accepted: NativeEventCount::new(accepted),
            },
        }),
        _ => Err(PasteError::Native(NativeError::new(
            NativeErrorKind::InvalidData,
            None,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use cliptype_platform::DispatchResult;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{KEYEVENTF_KEYUP, VK_CONTROL};

    use super::{PASTE_EVENT_COUNT, VK_V, classify_accepted, paste_inputs};

    #[test]
    fn paste_chord_is_one_balanced_ctrl_v_sequence() {
        let inputs = paste_inputs();
        assert_eq!(inputs.len(), PASTE_EVENT_COUNT as usize);

        // SAFETY: these union fields were initialized as keyboard inputs by the
        // helper under test.
        let keys: Vec<(u16, u32)> = inputs
            .iter()
            .map(|input| unsafe { (input.Anonymous.ki.wVk, input.Anonymous.ki.dwFlags) })
            .collect();
        assert_eq!(
            keys,
            vec![
                (VK_CONTROL, 0),
                (VK_V, 0),
                (VK_V, KEYEVENTF_KEYUP),
                (VK_CONTROL, KEYEVENTF_KEYUP),
            ]
        );
    }

    #[test]
    fn native_acceptance_is_conservative() {
        assert!(matches!(
            classify_accepted(4, None),
            Ok(DispatchResult::Complete { .. })
        ));
        assert!(matches!(
            classify_accepted(0, None),
            Ok(DispatchResult::NoneAccepted { .. })
        ));
        for accepted in 1..4 {
            assert!(matches!(
                classify_accepted(accepted, None),
                Ok(DispatchResult::ProgressUnknown { .. })
            ));
        }
        assert!(classify_accepted(5, None).is_err());
    }
}
