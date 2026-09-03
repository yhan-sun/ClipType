//! Native-neutral custom shortcut values and validation.

use std::{fmt, str::FromStr};

/// Platform vocabulary used only for labels and known-reserved policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyPlatform {
    Windows,
    MacOS,
}

/// Modifier set stored independently of native virtual-key values.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct HotkeyModifiers(u8);

impl HotkeyModifiers {
    const CONTROL_BIT: u8 = 1 << 0;
    const ALT_BIT: u8 = 1 << 1;
    const SHIFT_BIT: u8 = 1 << 2;
    const META_BIT: u8 = 1 << 3;

    pub const CONTROL: Self = Self(Self::CONTROL_BIT);
    pub const ALT: Self = Self(Self::ALT_BIT);
    pub const SHIFT: Self = Self(Self::SHIFT_BIT);
    pub const META: Self = Self(Self::META_BIT);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn control(self) -> bool {
        self.contains(Self::CONTROL)
    }

    pub const fn alt(self) -> bool {
        self.contains(Self::ALT)
    }

    pub const fn shift(self) -> bool {
        self.contains(Self::SHIFT)
    }

    pub const fn meta(self) -> bool {
        self.contains(Self::META)
    }

    pub const fn has_primary(self) -> bool {
        self.control() || self.alt() || self.meta()
    }

    pub const fn bits(self) -> u8 {
        self.0
    }
}

impl fmt::Debug for HotkeyModifiers {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HotkeyModifiers")
            .field("control", &self.control())
            .field("alt", &self.alt())
            .field("shift", &self.shift())
            .field("meta", &self.meta())
            .finish()
    }
}

/// Bounded keyboard key vocabulary supported by settings and adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyKey {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Space,
    Tab,
    Enter,
    Escape,
    Backspace,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Backslash,
    Semicolon,
    Quote,
    Comma,
    Period,
    Slash,
    Backquote,
}

impl HotkeyKey {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
            Self::C => "c",
            Self::D => "d",
            Self::E => "e",
            Self::F => "f",
            Self::G => "g",
            Self::H => "h",
            Self::I => "i",
            Self::J => "j",
            Self::K => "k",
            Self::L => "l",
            Self::M => "m",
            Self::N => "n",
            Self::O => "o",
            Self::P => "p",
            Self::Q => "q",
            Self::R => "r",
            Self::S => "s",
            Self::T => "t",
            Self::U => "u",
            Self::V => "v",
            Self::W => "w",
            Self::X => "x",
            Self::Y => "y",
            Self::Z => "z",
            Self::Digit0 => "0",
            Self::Digit1 => "1",
            Self::Digit2 => "2",
            Self::Digit3 => "3",
            Self::Digit4 => "4",
            Self::Digit5 => "5",
            Self::Digit6 => "6",
            Self::Digit7 => "7",
            Self::Digit8 => "8",
            Self::Digit9 => "9",
            Self::F1 => "f1",
            Self::F2 => "f2",
            Self::F3 => "f3",
            Self::F4 => "f4",
            Self::F5 => "f5",
            Self::F6 => "f6",
            Self::F7 => "f7",
            Self::F8 => "f8",
            Self::F9 => "f9",
            Self::F10 => "f10",
            Self::F11 => "f11",
            Self::F12 => "f12",
            Self::F13 => "f13",
            Self::F14 => "f14",
            Self::F15 => "f15",
            Self::F16 => "f16",
            Self::F17 => "f17",
            Self::F18 => "f18",
            Self::F19 => "f19",
            Self::F20 => "f20",
            Self::F21 => "f21",
            Self::F22 => "f22",
            Self::F23 => "f23",
            Self::F24 => "f24",
            Self::Space => "space",
            Self::Tab => "tab",
            Self::Enter => "enter",
            Self::Escape => "escape",
            Self::Backspace => "backspace",
            Self::Insert => "insert",
            Self::Delete => "delete",
            Self::Home => "home",
            Self::End => "end",
            Self::PageUp => "pageup",
            Self::PageDown => "pagedown",
            Self::ArrowLeft => "left",
            Self::ArrowRight => "right",
            Self::ArrowUp => "up",
            Self::ArrowDown => "down",
            Self::Minus => "minus",
            Self::Equal => "equal",
            Self::BracketLeft => "bracket-left",
            Self::BracketRight => "bracket-right",
            Self::Backslash => "backslash",
            Self::Semicolon => "semicolon",
            Self::Quote => "quote",
            Self::Comma => "comma",
            Self::Period => "period",
            Self::Slash => "slash",
            Self::Backquote => "backquote",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Digit0 => "0",
            Self::Digit1 => "1",
            Self::Digit2 => "2",
            Self::Digit3 => "3",
            Self::Digit4 => "4",
            Self::Digit5 => "5",
            Self::Digit6 => "6",
            Self::Digit7 => "7",
            Self::Digit8 => "8",
            Self::Digit9 => "9",
            Self::ArrowLeft => "Left",
            Self::ArrowRight => "Right",
            Self::ArrowUp => "Up",
            Self::ArrowDown => "Down",
            Self::PageUp => "Page Up",
            Self::PageDown => "Page Down",
            Self::BracketLeft => "[",
            Self::BracketRight => "]",
            Self::Backslash => "\\",
            Self::Semicolon => ";",
            Self::Quote => "'",
            Self::Comma => ",",
            Self::Period => ".",
            Self::Slash => "/",
            Self::Minus => "-",
            Self::Equal => "=",
            Self::Backquote => "`",
            _ => match self {
                Self::A => "A",
                Self::B => "B",
                Self::C => "C",
                Self::D => "D",
                Self::E => "E",
                Self::F => "F",
                Self::G => "G",
                Self::H => "H",
                Self::I => "I",
                Self::J => "J",
                Self::K => "K",
                Self::L => "L",
                Self::M => "M",
                Self::N => "N",
                Self::O => "O",
                Self::P => "P",
                Self::Q => "Q",
                Self::R => "R",
                Self::S => "S",
                Self::T => "T",
                Self::U => "U",
                Self::V => "V",
                Self::W => "W",
                Self::X => "X",
                Self::Y => "Y",
                Self::Z => "Z",
                Self::F1 => "F1",
                Self::F2 => "F2",
                Self::F3 => "F3",
                Self::F4 => "F4",
                Self::F5 => "F5",
                Self::F6 => "F6",
                Self::F7 => "F7",
                Self::F8 => "F8",
                Self::F9 => "F9",
                Self::F10 => "F10",
                Self::F11 => "F11",
                Self::F12 => "F12",
                Self::F13 => "F13",
                Self::F14 => "F14",
                Self::F15 => "F15",
                Self::F16 => "F16",
                Self::F17 => "F17",
                Self::F18 => "F18",
                Self::F19 => "F19",
                Self::F20 => "F20",
                Self::F21 => "F21",
                Self::F22 => "F22",
                Self::F23 => "F23",
                Self::F24 => "F24",
                Self::Space => "Space",
                Self::Tab => "Tab",
                Self::Enter => "Enter",
                Self::Escape => "Esc",
                Self::Backspace => "Backspace",
                Self::Insert => "Insert",
                Self::Delete => "Delete",
                Self::Home => "Home",
                Self::End => "End",
                _ => unreachable!(),
            },
        }
    }
}

impl FromStr for HotkeyKey {
    type Err = HotkeyParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let lower = value.trim().to_ascii_lowercase();
        let key = match lower.as_str() {
            "a" => Self::A,
            "b" => Self::B,
            "c" => Self::C,
            "d" => Self::D,
            "e" => Self::E,
            "f" => Self::F,
            "g" => Self::G,
            "h" => Self::H,
            "i" => Self::I,
            "j" => Self::J,
            "k" => Self::K,
            "l" => Self::L,
            "m" => Self::M,
            "n" => Self::N,
            "o" => Self::O,
            "p" => Self::P,
            "q" => Self::Q,
            "r" => Self::R,
            "s" => Self::S,
            "t" => Self::T,
            "u" => Self::U,
            "v" => Self::V,
            "w" => Self::W,
            "x" => Self::X,
            "y" => Self::Y,
            "z" => Self::Z,
            "0" => Self::Digit0,
            "1" => Self::Digit1,
            "2" => Self::Digit2,
            "3" => Self::Digit3,
            "4" => Self::Digit4,
            "5" => Self::Digit5,
            "6" => Self::Digit6,
            "7" => Self::Digit7,
            "8" => Self::Digit8,
            "9" => Self::Digit9,
            "f1" => Self::F1,
            "f2" => Self::F2,
            "f3" => Self::F3,
            "f4" => Self::F4,
            "f5" => Self::F5,
            "f6" => Self::F6,
            "f7" => Self::F7,
            "f8" => Self::F8,
            "f9" => Self::F9,
            "f10" => Self::F10,
            "f11" => Self::F11,
            "f12" => Self::F12,
            "f13" => Self::F13,
            "f14" => Self::F14,
            "f15" => Self::F15,
            "f16" => Self::F16,
            "f17" => Self::F17,
            "f18" => Self::F18,
            "f19" => Self::F19,
            "f20" => Self::F20,
            "f21" => Self::F21,
            "f22" => Self::F22,
            "f23" => Self::F23,
            "f24" => Self::F24,
            "space" => Self::Space,
            "tab" => Self::Tab,
            "enter" | "return" => Self::Enter,
            "escape" | "esc" => Self::Escape,
            "backspace" => Self::Backspace,
            "insert" => Self::Insert,
            "delete" | "del" => Self::Delete,
            "home" => Self::Home,
            "end" => Self::End,
            "pageup" | "page-up" => Self::PageUp,
            "pagedown" | "page-down" => Self::PageDown,
            "left" | "arrow-left" => Self::ArrowLeft,
            "right" | "arrow-right" => Self::ArrowRight,
            "up" | "arrow-up" => Self::ArrowUp,
            "down" | "arrow-down" => Self::ArrowDown,
            "minus" | "-" => Self::Minus,
            "equal" | "=" => Self::Equal,
            "bracket-left" | "[" => Self::BracketLeft,
            "bracket-right" | "]" => Self::BracketRight,
            "backslash" | "\\" => Self::Backslash,
            "semicolon" | ";" => Self::Semicolon,
            "quote" | "'" => Self::Quote,
            "comma" | "," => Self::Comma,
            "period" | "." => Self::Period,
            "slash" | "/" => Self::Slash,
            "backquote" | "`" => Self::Backquote,
            _ => return Err(HotkeyParseError::UnknownToken),
        };
        Ok(key)
    }
}

/// Native-neutral shortcut specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HotkeySpec {
    pub modifiers: HotkeyModifiers,
    pub key: HotkeyKey,
}

impl HotkeySpec {
    pub const fn new(modifiers: HotkeyModifiers, key: HotkeyKey) -> Self {
        Self { modifiers, key }
    }

    pub fn validate(self) -> Result<Self, HotkeyValidationError> {
        if !self.modifiers.has_primary() {
            return Err(HotkeyValidationError::MissingPrimaryModifier);
        }
        Ok(self)
    }

    pub fn validate_for(self, platform: HotkeyPlatform) -> Result<Self, HotkeyValidationError> {
        self.validate()?;
        if self.is_reserved_on(platform) {
            return Err(HotkeyValidationError::Reserved);
        }
        if !self.is_supported_on(platform) {
            return Err(HotkeyValidationError::Unsupported);
        }
        Ok(self)
    }

    pub const fn is_supported_on(self, platform: HotkeyPlatform) -> bool {
        match platform {
            HotkeyPlatform::Windows => true,
            HotkeyPlatform::MacOS => !matches!(
                self.key,
                HotkeyKey::F21 | HotkeyKey::F22 | HotkeyKey::F23 | HotkeyKey::F24
            ),
        }
    }

    pub const fn is_reserved_on(self, platform: HotkeyPlatform) -> bool {
        match platform {
            HotkeyPlatform::Windows => {
                if matches!(self.key, HotkeyKey::F12) {
                    return true;
                }
                if self.modifiers.alt() && matches!(self.key, HotkeyKey::F4 | HotkeyKey::Tab) {
                    return true;
                }
                if self.modifiers.control()
                    && self.modifiers.alt()
                    && matches!(self.key, HotkeyKey::Delete)
                {
                    return true;
                }
                if self.modifiers.control()
                    && self.modifiers.shift()
                    && matches!(self.key, HotkeyKey::Escape)
                {
                    return true;
                }
                self.modifiers.meta()
                    && matches!(
                        self.key,
                        HotkeyKey::L | HotkeyKey::D | HotkeyKey::R | HotkeyKey::E | HotkeyKey::Tab
                    )
            }
            HotkeyPlatform::MacOS => {
                if self.modifiers.meta()
                    && matches!(self.key, HotkeyKey::Space | HotkeyKey::Tab | HotkeyKey::Q)
                {
                    return true;
                }
                if self.modifiers.meta()
                    && self.modifiers.alt()
                    && matches!(self.key, HotkeyKey::Escape)
                {
                    return true;
                }
                if self.modifiers.control()
                    && self.modifiers.meta()
                    && matches!(self.key, HotkeyKey::Q)
                {
                    return true;
                }
                self.modifiers.meta()
                    && self.modifiers.shift()
                    && matches!(
                        self.key,
                        HotkeyKey::Digit3 | HotkeyKey::Digit4 | HotkeyKey::Digit5
                    )
            }
        }
    }

    pub fn canonical(self) -> String {
        let mut parts = Vec::with_capacity(5);
        if self.modifiers.control() {
            parts.push("ctrl");
        }
        if self.modifiers.alt() {
            parts.push("alt");
        }
        if self.modifiers.shift() {
            parts.push("shift");
        }
        if self.modifiers.meta() {
            parts.push("meta");
        }
        parts.push(self.key.canonical_name());
        parts.join("+")
    }

    pub fn label(self, platform: HotkeyPlatform) -> String {
        match platform {
            HotkeyPlatform::Windows => {
                let mut parts = Vec::with_capacity(5);
                if self.modifiers.control() {
                    parts.push("Ctrl");
                }
                if self.modifiers.alt() {
                    parts.push("Alt");
                }
                if self.modifiers.shift() {
                    parts.push("Shift");
                }
                if self.modifiers.meta() {
                    parts.push("Win");
                }
                parts.push(self.key.display_name());
                parts.join("+")
            }
            HotkeyPlatform::MacOS => {
                let mut label = String::new();
                if self.modifiers.control() {
                    label.push('⌃');
                }
                if self.modifiers.alt() {
                    label.push('⌥');
                }
                if self.modifiers.shift() {
                    label.push('⇧');
                }
                if self.modifiers.meta() {
                    label.push('⌘');
                }
                label.push_str(self.key.display_name());
                label
            }
        }
    }
}

impl fmt::Display for HotkeySpec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.canonical())
    }
}

impl FromStr for HotkeySpec {
    type Err = HotkeyParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.trim().is_empty() {
            return Err(HotkeyParseError::Empty);
        }
        let mut modifiers = HotkeyModifiers::empty();
        let mut key = None;
        for token in value.split('+').map(str::trim) {
            if token.is_empty() {
                return Err(HotkeyParseError::UnknownToken);
            }
            let lower = token.to_ascii_lowercase();
            let modifier = match lower.as_str() {
                "ctrl" | "control" => Some(HotkeyModifiers::CONTROL),
                "alt" | "option" => Some(HotkeyModifiers::ALT),
                "shift" => Some(HotkeyModifiers::SHIFT),
                "meta" | "win" | "windows" | "cmd" | "command" => Some(HotkeyModifiers::META),
                _ => None,
            };
            if let Some(modifier) = modifier {
                if modifiers.contains(modifier) {
                    return Err(HotkeyParseError::DuplicateModifier);
                }
                modifiers = modifiers.with(modifier);
                continue;
            }
            if key.is_some() {
                return Err(HotkeyParseError::MultipleKeys);
            }
            key = Some(token.parse()?);
        }
        let spec = Self {
            modifiers,
            key: key.ok_or(HotkeyParseError::MissingKey)?,
        };
        spec.validate().map_err(HotkeyParseError::Validation)
    }
}

/// Trigger and cancel shortcuts are applied as one transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HotkeyPair {
    pub trigger: HotkeySpec,
    pub cancel: HotkeySpec,
}

impl HotkeyPair {
    pub const fn new(trigger: HotkeySpec, cancel: HotkeySpec) -> Self {
        Self { trigger, cancel }
    }

    pub fn validate(self) -> Result<Self, HotkeyValidationError> {
        self.trigger.validate()?;
        self.cancel.validate()?;
        if self.trigger == self.cancel {
            return Err(HotkeyValidationError::DuplicatePair);
        }
        Ok(self)
    }

    pub fn validate_for(self, platform: HotkeyPlatform) -> Result<Self, HotkeyValidationError> {
        self.validate()?;
        self.trigger.validate_for(platform)?;
        self.cancel.validate_for(platform)?;
        Ok(self)
    }
}

impl Default for HotkeyPair {
    fn default() -> Self {
        let modifiers = HotkeyModifiers::CONTROL
            .with(HotkeyModifiers::ALT)
            .with(HotkeyModifiers::SHIFT);
        Self {
            trigger: HotkeySpec::new(modifiers, HotkeyKey::V),
            cancel: HotkeySpec::new(modifiers, HotkeyKey::X),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAvailability {
    Available,
    Conflict,
    Reserved,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyApplyResult {
    Applied,
    Rejected(HotkeyAvailability),
    RolledBack(HotkeyAvailability),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyValidationError {
    MissingPrimaryModifier,
    DuplicatePair,
    Reserved,
    Unsupported,
}

impl fmt::Display for HotkeyValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingPrimaryModifier => {
                "shortcut requires Control, Alt/Option, or Meta/Command"
            }
            Self::DuplicatePair => "trigger and cancel shortcuts must differ",
            Self::Reserved => "shortcut is reserved or unsafe on this platform",
            Self::Unsupported => "shortcut is unsupported on this platform",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for HotkeyValidationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyParseError {
    Empty,
    UnknownToken,
    DuplicateModifier,
    MultipleKeys,
    MissingKey,
    Validation(HotkeyValidationError),
}

impl fmt::Display for HotkeyParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "shortcut is empty",
            Self::UnknownToken => "shortcut contains an unknown token",
            Self::DuplicateModifier => "shortcut repeats a modifier",
            Self::MultipleKeys => "shortcut contains multiple non-modifier keys",
            Self::MissingKey => "shortcut is missing a non-modifier key",
            Self::Validation(error) => return error.fmt(formatter),
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for HotkeyParseError {}

#[cfg(test)]
mod tests {
    use super::{
        HotkeyKey, HotkeyModifiers, HotkeyPair, HotkeyParseError, HotkeyPlatform, HotkeySpec,
        HotkeyValidationError,
    };

    #[test]
    fn canonical_round_trip_is_deterministic() {
        let spec: HotkeySpec = "shift+ctrl+alt+v".parse().expect("valid shortcut");
        assert_eq!(spec.canonical(), "ctrl+alt+shift+v");
        assert_eq!(spec.canonical().parse(), Ok(spec));
        assert_eq!(spec.label(HotkeyPlatform::Windows), "Ctrl+Alt+Shift+V");
        assert_eq!(spec.label(HotkeyPlatform::MacOS), "⌃⌥⇧V");
    }

    #[test]
    fn bare_and_shift_only_shortcuts_fail_closed() {
        assert!(matches!(
            "v".parse::<HotkeySpec>(),
            Err(HotkeyParseError::Validation(
                HotkeyValidationError::MissingPrimaryModifier
            ))
        ));
        assert!(matches!(
            "shift+v".parse::<HotkeySpec>(),
            Err(HotkeyParseError::Validation(
                HotkeyValidationError::MissingPrimaryModifier
            ))
        ));
    }

    #[test]
    fn trigger_and_cancel_must_differ() {
        let spec: HotkeySpec = "ctrl+alt+v".parse().expect("valid shortcut");
        assert_eq!(
            HotkeyPair::new(spec, spec).validate(),
            Err(HotkeyValidationError::DuplicatePair)
        );
    }

    #[test]
    fn platform_reservations_are_explicit() {
        let windows_f12: HotkeySpec = "ctrl+alt+f12".parse().expect("structurally valid");
        assert_eq!(
            windows_f12.validate_for(HotkeyPlatform::Windows),
            Err(HotkeyValidationError::Reserved)
        );
        let mac_spotlight: HotkeySpec = "cmd+space".parse().expect("structurally valid");
        assert_eq!(
            mac_spotlight.validate_for(HotkeyPlatform::MacOS),
            Err(HotkeyValidationError::Reserved)
        );
    }

    #[test]
    fn default_pair_is_non_f12_and_platform_valid() {
        let pair = HotkeyPair::default();
        assert_eq!(pair.trigger.key, HotkeyKey::V);
        assert_eq!(pair.cancel.key, HotkeyKey::X);
        assert!(pair.validate_for(HotkeyPlatform::Windows).is_ok());
        assert!(pair.validate_for(HotkeyPlatform::MacOS).is_ok());
    }

    #[test]
    fn parser_rejects_duplicates_and_multiple_keys() {
        assert_eq!(
            "ctrl+ctrl+v".parse::<HotkeySpec>(),
            Err(HotkeyParseError::DuplicateModifier)
        );
        assert_eq!(
            "ctrl+v+x".parse::<HotkeySpec>(),
            Err(HotkeyParseError::MultipleKeys)
        );
    }

    #[test]
    fn modifiers_are_native_neutral_bits() {
        let modifiers = HotkeyModifiers::CONTROL
            .with(HotkeyModifiers::ALT)
            .with(HotkeyModifiers::SHIFT);
        assert!(modifiers.control());
        assert!(modifiers.alt());
        assert!(modifiers.shift());
        assert!(!modifiers.meta());
    }
}
