use crate::error::WHKError;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::{collections::HashMap, hash::Hash, sync::LazyLock};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

macro_rules! vkeys_definition {
    ($($name:ident = $value:ident $(aliases [$($alias:literal),*])? $(const $cName:ident)? ,)*) => {
        /// Represents a virtual key (VK) code.
        ///
        /// # Abreviations
        /// - IME => Input Method Editor
        /// - OEM => Original Equipment Manufacturer
        ///
        /// # See Also
        /// - [Microsoft Virtual-Key Codes](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes)
        #[derive(Debug, Clone, Copy, FromPrimitive, IntoPrimitive)]
        #[repr(u16)]
        pub enum VKey {
            $(
                $name = $value.0,
            )*
            /// the scan code of the key has no mapping
            None = VK__none_.0,
            #[num_enum(catch_all)]
            UnknownOrReserved(u16),
        }

        static VKEY_ALIASES_MAP: LazyLock<HashMap<String, VKey>> = LazyLock::new(|| {
            let mut m = HashMap::new();
            $(
                m.insert(stringify!($name).to_ascii_lowercase(), VKey::$name);
                $($(
                    m.insert($alias.to_ascii_lowercase(), VKey::$name);
                )*)?
            )*
            m
        });

        impl VKey {
            $($(
                #[allow(non_upper_case_globals)]
                pub const $cName: VKey = VKey::$name;
            )?)*

            pub fn to_string(&self) -> String {
                match self {
                    $(
                        VKey::$name => stringify!($name).to_owned(),
                    )*
                    VKey::None => "0xFF".to_owned(),
                    VKey::UnknownOrReserved(key) => format!("0x{:X}", key),
                }
            }

            fn try_from_aliases(alias: &str) -> Option<Self> {
                VKEY_ALIASES_MAP.get(&alias.to_ascii_lowercase()).copied()
            }

            /// Creates a `VKey` from a string representation of the key.
            ///
            /// NOTE: Certain common aliases for keys are accepted in addition to the Microsoft Virtual-Key Codes names
            ///
            /// WIN maps to `VKey::LWin`
            /// CTRL maps to `VKey::Control`
            /// ALT maps to `VKey::Menu`
            ///
            /// # See Also
            /// - [Microsoft Virtual-Key Codes](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes)
            ///
            pub fn from_keyname(key_name: &str) -> Result<VKey, WHKError> {
                if let Some(key) = Self::from_maybe_hex_string(key_name) {
                    return Ok(key);
                }

                if let Some(key) = Self::try_from_aliases(&key_name) {
                    return Ok(key);
                }

                let mut vk_name = key_name.to_ascii_uppercase();
                if !vk_name.starts_with("VK_") {
                    vk_name = format!("VK_{}", vk_name);
                }

                let key = match vk_name.as_str() {
                    $(
                      stringify!($value) => VKey::$name,
                    )*
                    _ => return Err(WHKError::InvalidKey(key_name.to_string())),
                };

                Ok(key)
            }
        }
    };
}

vkeys_definition! {
    // VK_LBUTTON
    // VK_RBUTTON
    // VK_CANCEL
    // VK_MBUTTON
    // VK_XBUTTON1
    // VK_XBUTTON2
    // 0x07 Reserved
    Back = VK_BACK aliases ["Backspace"],
    Tab = VK_TAB,
    // 0x0A-0x0B Reserved
    Clear = VK_CLEAR,
    Return = VK_RETURN aliases ["Enter"],
    // 0x0E-0x0F Reserved
    Shift = VK_SHIFT,
    Control = VK_CONTROL aliases ["Ctrl"] const Ctrl,
    Menu = VK_MENU aliases ["Alt"] const Alt,
    Pause = VK_PAUSE,

    Capital = VK_CAPITAL aliases ["CapsLock"],
    ImeKana = VK_KANA aliases ["Hangul", "VK_HANGUL"] const Hangul,
    ImeOn = VK_IME_ON,
    ImeJunja = VK_JUNJA,
    ImeFinal = VK_FINAL,
    ImeHanja = VK_HANJA aliases ["Kanji", "VK_KANJI"] const Kanji,
    ImeOff = VK_IME_OFF,
    Escape = VK_ESCAPE aliases ["Esc"],
    ImeConvert = VK_CONVERT,
    ImeNonConver = VK_NONCONVERT,
    ImeAccept = VK_ACCEPT,
    ImeModeChange = VK_MODECHANGE,

    Space = VK_SPACE,
    Prior = VK_PRIOR aliases ["PageUp"],
    Next = VK_NEXT aliases ["PageDown"],
    End = VK_END,
    Home = VK_HOME,
    Left = VK_LEFT aliases ["ArrowLeft"],
    Up = VK_UP aliases ["ArrowUp"],
    Right = VK_RIGHT aliases ["ArrowRight"],
    Down = VK_DOWN aliases ["ArrowDown"],
    Select = VK_SELECT,
    Print = VK_PRINT,
    Execute = VK_EXECUTE,
    Snapshot = VK_SNAPSHOT aliases ["Screenshot"],
    Insert = VK_INSERT,
    Delete = VK_DELETE,
    Help = VK_HELP,

    Digit0 = VK_0 aliases ["0"],
    Digit1 = VK_1 aliases ["1"],
    Digit2 = VK_2 aliases ["2"],
    Digit3 = VK_3 aliases ["3"],
    Digit4 = VK_4 aliases ["4"],
    Digit5 = VK_5 aliases ["5"],
    Digit6 = VK_6 aliases ["6"],
    Digit7 = VK_7 aliases ["7"],
    Digit8 = VK_8 aliases ["8"],
    Digit9 = VK_9 aliases ["9"],
    // 0x3A-0x40 Undefined
    A = VK_A,
    B = VK_B,
    C = VK_C,
    D = VK_D,
    E = VK_E,
    F = VK_F,
    G = VK_G,
    H = VK_H,
    I = VK_I,
    J = VK_J,
    K = VK_K,
    L = VK_L,
    M = VK_M,
    N = VK_N,
    O = VK_O,
    P = VK_P,
    Q = VK_Q,
    R = VK_R,
    S = VK_S,
    T = VK_T,
    U = VK_U,
    V = VK_V,
    W = VK_W,
    X = VK_X,
    Y = VK_Y,
    Z = VK_Z,

    LWin = VK_LWIN aliases ["Win"],
    RWin = VK_RWIN,
    Apps = VK_APPS,
    Sleep = VK_SLEEP,

    Numpad0 = VK_NUMPAD0,
    Numpad1 = VK_NUMPAD1,
    Numpad2 = VK_NUMPAD2,
    Numpad3 = VK_NUMPAD3,
    Numpad4 = VK_NUMPAD4,
    Numpad5 = VK_NUMPAD5,
    Numpad6 = VK_NUMPAD6,
    Numpad7 = VK_NUMPAD7,
    Numpad8 = VK_NUMPAD8,
    Numpad9 = VK_NUMPAD9,
    Multiply = VK_MULTIPLY,
    Add = VK_ADD,
    Separator = VK_SEPARATOR,
    Subtract = VK_SUBTRACT,
    Decimal = VK_DECIMAL,
    Divide = VK_DIVIDE,

    F1 = VK_F1,
    F2 = VK_F2,
    F3 = VK_F3,
    F4 = VK_F4,
    F5 = VK_F5,
    F6 = VK_F6,
    F7 = VK_F7,
    F8 = VK_F8,
    F9 = VK_F9,
    F10 = VK_F10,
    F11 = VK_F11,
    F12 = VK_F12,
    F13 = VK_F13,
    F14 = VK_F14,
    F15 = VK_F15,
    F16 = VK_F16,
    F17 = VK_F17,
    F18 = VK_F18,
    F19 = VK_F19,
    F20 = VK_F20,
    F21 = VK_F21,
    F22 = VK_F22,
    F23 = VK_F23,
    F24 = VK_F24,
    // 0x88-8F Reserved
    Numlock = VK_NUMLOCK,
    Scroll = VK_SCROLL aliases ["ScrollLock"],
    // 0x92-96 OEM specific
    // 0x97-9F Unassigned
    LShift = VK_LSHIFT,
    RShift = VK_RSHIFT,
    LControl = VK_LCONTROL aliases ["LCtrl"] const LCtrl,
    RControl = VK_RCONTROL aliases ["RCtrl"] const RCtrl,
    LMenu = VK_LMENU aliases ["LAlt"] const LAlt,
    RMenu = VK_RMENU aliases ["RAlt"] const RAlt,

    BrowserBack = VK_BROWSER_BACK,
    BrowserForward = VK_BROWSER_FORWARD,
    BrowserRefresh = VK_BROWSER_REFRESH,
    BrowserStop = VK_BROWSER_STOP,
    BrowserSearch = VK_BROWSER_SEARCH,
    BrowserFavorites = VK_BROWSER_FAVORITES,
    BrowserHome = VK_BROWSER_HOME,
    VolumeMute = VK_VOLUME_MUTE,
    VolumeDown = VK_VOLUME_DOWN,
    VolumeUp = VK_VOLUME_UP,
    MediaNextTrack = VK_MEDIA_NEXT_TRACK,
    MediaPrevTrack = VK_MEDIA_PREV_TRACK,
    MediaStop = VK_MEDIA_STOP,
    MediaPlayPause = VK_MEDIA_PLAY_PAUSE,
    LaunchMail = VK_LAUNCH_MAIL,
    LaunchMediaSelect = VK_LAUNCH_MEDIA_SELECT,
    LaunchApp1 = VK_LAUNCH_APP1,
    LaunchApp2 = VK_LAUNCH_APP2,
    // 0xB8-B9 Reserved
    Oem1 = VK_OEM_1,
    // For any country/region, the Equals and Plus key
    OemPlus = VK_OEM_PLUS aliases ["+", "="],
    // For any country/region, the Comma and Less Than key
    OemComma = VK_OEM_COMMA aliases [",", "<"],
    // For any country/region, the Dash and Underscore key
    OemMinus = VK_OEM_MINUS aliases ["-", "_"],
    // For any country/region, the Period and Greater Than key
    OemPeriod = VK_OEM_PERIOD aliases [".", ">"],
    Oem2 = VK_OEM_2,
    Oem3 = VK_OEM_3,
    // 0xC1-DA Reserved
    Oem4 = VK_OEM_4,
    Oem5 = VK_OEM_5,
    Oem6 = VK_OEM_6,
    Oem7 = VK_OEM_7,
    Oem8 = VK_OEM_8,
    // 0xE0 Reserved
    // 0xE1 OEM specific
    Oem102 = VK_OEM_102,
    // 0xE3-E4 OEM specific
    ImeProcessKey = VK_PROCESSKEY,
    // 0xE6 OEM specific
    Packet = VK_PACKET,
    // 0xE8 Unassigned (we use it internally as silent key)
    // 0xE9-F5 OEM specific
    Attention = VK_ATTN,
    CursorSelect = VK_CRSEL,
    ExtendSelect = VK_EXSEL,
    EraseEof = VK_EREOF,
    Play = VK_PLAY,
    Zoom = VK_ZOOM,
    NoName = VK_NONAME,
    Pa1 = VK_PA1,
    OemClear = VK_OEM_CLEAR,
}

#[allow(non_upper_case_globals)]
impl VKey {
    /// Similar to equals, but accounts for aliases and groups
    pub fn matches(&self, other: &VKey) -> bool {
        match self {
            VKey::Shift => matches!(other, VKey::LShift | VKey::RShift | VKey::Shift),
            VKey::Control => matches!(other, VKey::LControl | VKey::RControl | VKey::Control),
            VKey::Menu => matches!(other, VKey::LMenu | VKey::RMenu | VKey::Menu),
            // we don't gonna support differenciation between left and right windows keys
            VKey::LWin => matches!(other, VKey::LWin | VKey::RWin),
            VKey::RWin => matches!(other, VKey::LWin | VKey::RWin),
            _ => self == other,
        }
    }

    pub fn is_windows_key(&self) -> bool {
        matches!(self, VKey::LWin | VKey::RWin)
    }

    pub fn is_shift_key(&self) -> bool {
        matches!(self, VKey::LShift | VKey::RShift | VKey::Shift)
    }

    pub fn is_menu_key(&self) -> bool {
        matches!(self, VKey::LMenu | VKey::RMenu | VKey::Menu)
    }

    pub fn is_control_key(&self) -> bool {
        matches!(self, VKey::LControl | VKey::RControl | VKey::Control)
    }

    pub fn is_modifier_key(&self) -> bool {
        self.is_windows_key() || self.is_shift_key() || self.is_menu_key() || self.is_control_key()
    }

    /// Converts a `VKey` to its corresponding Windows Virtual-Key (VK) code.
    ///
    /// # See Also
    /// - [Microsoft Virtual-Key Codes](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes)
    ///
    pub fn to_vk_code(&self) -> u16 {
        u16::from(*self)
    }

    /// Returns a `VKey` based a Windows Virtual-Key (VK) code.
    ///
    /// # See Also
    /// - [Microsoft Virtual-Key Codes](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes)
    ///
    pub fn from_vk_code(vk_code: u16) -> VKey {
        VKey::from(vk_code)
    }

    fn from_maybe_hex_string(name: &str) -> Option<VKey> {
        // 1 byte hex code => Use the raw keycode value
        if name.len() >= 3 && name.len() <= 6 && name.starts_with("0x") || name.starts_with("0X") {
            return if let Ok(val) = u16::from_str_radix(&name[2..], 16) {
                Some(Self::from_vk_code(val))
            } else {
                None
            };
        }
        None
    }
}

impl std::str::FromStr for VKey {
    type Err = WHKError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        VKey::from_keyname(s)
    }
}

impl Eq for VKey {}
impl PartialEq<VKey> for VKey {
    fn eq(&self, other: &Self) -> bool {
        self.to_vk_code() == other.to_vk_code()
    }
}

impl PartialEq<VKey> for &VKey {
    fn eq(&self, other: &VKey) -> bool {
        self.to_vk_code() == other.to_vk_code()
    }
}

impl Hash for VKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_vk_code().hash(state);
    }
}

impl PartialOrd for VKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_vk_code().cmp(&other.to_vk_code())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for VKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for VKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum StringOrNumber {
            String(String),
            Number(u16),
        }

        match StringOrNumber::deserialize(deserializer)? {
            StringOrNumber::String(s) => VKey::from_keyname(&s).map_err(serde::de::Error::custom),
            StringOrNumber::Number(n) => Ok(VKey::from_vk_code(n)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_vk_code() {
        assert_eq!(VKey::Back.to_vk_code(), VK_BACK.0);
        assert_eq!(VKey::Return.to_vk_code(), VK_RETURN.0);
        assert_eq!(VKey::Space.to_vk_code(), VK_SPACE.0);
        assert_eq!(VKey::F12.to_vk_code(), VK_F12.0);
        assert_eq!(VKey::UnknownOrReserved(1234).to_vk_code(), 1234); // Unknown key
    }

    #[test]
    fn test_from_keyname() {
        assert_eq!(VKey::from_keyname("a").unwrap(), VKey::A);
        assert_eq!(VKey::from_keyname("A").unwrap(), VKey::A);
        assert_eq!(VKey::from_keyname("5").unwrap(), VKey::Digit5);
        assert_eq!(VKey::from_keyname("BACK").unwrap(), VKey::Back);
        assert_eq!(VKey::from_keyname("BaCk").unwrap(), VKey::Back); // Case-insensitive
        assert_eq!(VKey::from_keyname("VK_BACK").unwrap(), VKey::Back);
        assert_eq!(VKey::from_keyname("RETURN").unwrap(), VKey::Return);
        assert_eq!(VKey::from_keyname("0x29").unwrap(), VKey::Select);
        assert_eq!(
            VKey::from_keyname("0x29").unwrap(),
            VKey::UnknownOrReserved(0x29)
        );
        assert_eq!(
            VKey::from_keyname("0xff1").unwrap(),
            VKey::UnknownOrReserved(0xff1)
        );
        assert!(VKey::from_keyname("INVALID_KEY").is_err());
    }

    #[test]
    fn test_to_string() {
        assert_eq!(VKey::Back.to_string(), "Back");
        assert_eq!(VKey::Return.to_string(), "Return");
        assert_eq!(VKey::UnknownOrReserved(1234).to_string(), "0x4D2");
    }

    #[test]
    fn test_from_str() {
        use std::str::FromStr;
        assert_eq!(VKey::from_str("BACK").unwrap(), VKey::Back);
        assert_eq!(VKey::from_str("VK_BACK").unwrap(), VKey::Back);
        assert!(VKey::from_str("INVALID_KEY").is_err());
    }

    #[test]
    fn test_partial_eq() {
        assert_eq!(VKey::Back, VKey::Back); // Identical keys
        assert_eq!(VKey::UnknownOrReserved(1234), VKey::UnknownOrReserved(1234)); // Same key
        assert_ne!(VKey::UnknownOrReserved(1234), VKey::UnknownOrReserved(5678));
        // Different keys
    }

    #[test]
    fn test_custom_keycode_range() {
        assert_eq!(VKey::UnknownOrReserved(0).to_vk_code(), 0);
        assert_eq!(VKey::UnknownOrReserved(65535).to_vk_code(), 65535); // Maximum value for u16
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn test_serialize_deserialize_known_keys() {
        // Test with standard keys
        assert_tokens(&VKey::A, &[Token::Str("A")]);
        assert_tokens(&VKey::Back, &[Token::Str("Back")]);
        assert_tokens(&VKey::Return, &[Token::Str("Return")]);
        assert_tokens(&VKey::Space, &[Token::Str("Space")]);
        assert_tokens(&VKey::F12, &[Token::Str("F12")]);

        // Test with aliases
        assert_tokens(&VKey::Control, &[Token::Str("Control")]);
        assert_tokens(&VKey::LWin, &[Token::Str("LWin")]);
        assert_tokens(&VKey::Menu, &[Token::Str("Menu")]);
    }

    #[test]
    fn test_serialize_deserialize_unknown_keys() {
        // Test with unknown key codes
        let unknown_key = VKey::UnknownOrReserved(0x1234);
        assert_eq!(serde_json::to_string(&unknown_key).unwrap(), "\"0x1234\"");
        assert_eq!(
            serde_json::from_str::<VKey>("\"0x1234\"").unwrap(),
            unknown_key
        );
    }

    #[test]
    fn test_deserialize_from_aliases() {
        // Test deserialization from common aliases
        assert_eq!(
            serde_json::from_str::<VKey>("\"Ctrl\"").unwrap(),
            VKey::Control
        );
        assert_eq!(serde_json::from_str::<VKey>("\"Win\"").unwrap(), VKey::LWin);
        assert_eq!(serde_json::from_str::<VKey>("\"Alt\"").unwrap(), VKey::Menu);
        assert_eq!(
            serde_json::from_str::<VKey>("\"Enter\"").unwrap(),
            VKey::Return
        );
        assert_eq!(
            serde_json::from_str::<VKey>("\"CapsLock\"").unwrap(),
            VKey::Capital
        );
    }

    #[test]
    fn test_deserialize_from_vk_names() {
        // Test deserialization from VK_* names
        assert_eq!(
            serde_json::from_str::<VKey>("\"VK_BACK\"").unwrap(),
            VKey::Back
        );
        assert_eq!(
            serde_json::from_str::<VKey>("\"VK_RETURN\"").unwrap(),
            VKey::Return
        );
        assert_eq!(
            serde_json::from_str::<VKey>("\"VK_SPACE\"").unwrap(),
            VKey::Space
        );
    }

    #[test]
    fn test_deserialize_from_hex_strings() {
        // Test deserialization from hex strings
        assert_tokens(&VKey::UnknownOrReserved(0x29), &[Token::Str("0x29")]);
        assert_tokens(&VKey::UnknownOrReserved(0xFF), &[Token::Str("0xFF")]);
    }

    #[test]
    fn test_roundtrip_serialization() {
        // Test roundtrip for various key types
        let keys = [
            VKey::A,
            VKey::Back,
            VKey::Return,
            VKey::Space,
            VKey::F12,
            VKey::Control,
            VKey::LWin,
            VKey::Menu,
            VKey::UnknownOrReserved(0x1234),
        ];

        for key in keys {
            let serialized = serde_json::to_string(&key).unwrap();
            let deserialized: VKey = serde_json::from_str(&serialized).unwrap();
            assert_eq!(key, deserialized);
        }
    }

    #[test]
    fn test_deserialize_invalid_keys() {
        // Test invalid key strings
        assert!(serde_json::from_str::<VKey>("\"INVALID_KEY\"").is_err());
        assert!(serde_json::from_str::<VKey>("\"\"").is_err());
        assert!(serde_json::from_str::<VKey>("true").is_err());
    }

    #[test]
    fn test_deserialize_numeric_values() {
        // Test deserialization from numeric values
        assert_eq!(serde_json::from_str::<VKey>("65").unwrap(), VKey::A);
        assert_eq!(serde_json::from_str::<VKey>("8").unwrap(), VKey::Back);
        assert_eq!(
            serde_json::from_str::<VKey>("4660").unwrap(),
            VKey::UnknownOrReserved(0x1234)
        );
    }
}
