use crate::{state::KeyboardState, VKey};

/// Enum representing keyboard input events.
///
/// **note**: This doesn't represent the real hardware event, as hooks on high priority
/// can override the pressed keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardInputEvent {
    KeyDown {
        /// The virtual key code of the key.
        key: VKey,
        /// The updated keyboard state due to this event.
        state: KeyboardState,
    },
    KeyUp {
        /// The virtual key code of the key.
        key: VKey,
        /// The updated keyboard state due to this event.
        state: KeyboardState,
    },
}

/// Enum representing how to handle keypress.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyAction {
    Allow,
    Block,
}
