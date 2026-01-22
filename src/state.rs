//! This module provides the `KeyboardState` struct to track the state of keyboard keys.
//! It supports key press (`keydown`), key release (`keyup`), and querying key state (`is_down`).

use std::sync::{Arc, LazyLock, Mutex};

use crate::{log_on_dev, VKey};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

/// this is an arbitrary number, on local tests it don't need more than 3, but we use 10 just to be sure
const SYNC_COUNT_NEEDED_TO_BE_CONSIDERATED_SAFE: u8 = 10;

/// singleton Keyboard State
pub(crate) static KEYBOARD_STATE: LazyLock<Arc<Mutex<KeyboardState>>> = LazyLock::new(|| {
    let mutex = Mutex::new(KeyboardState::new());
    Arc::new(mutex)
});

/// Represents a state of pressed keys on a keyboard.
/// Can be used to track the current state of the keyboard
/// or to represent a keyboard state for hotkeys.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct KeyboardState {
    pub pressing: Vec<VKey>,
    /// History of pressed keys over a sequence.
    /// This is cleared when a new key is pressed after all keys were released.
    pub sequence: Vec<VKey>,
    needs_sync: bool,
    sync_count: u8,
}

impl KeyboardState {
    /// Creates a new `KeyboardState` with all keys released.
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks a key as pressed. If the key is already pressed, will send it to the end
    pub fn keydown<K: Into<VKey>>(&mut self, key: K) {
        if self.needs_sync {
            self.sync();
        }
        let key = key.into();

        // Clear sequence when starting a fresh key press after all keys were released
        if self.pressing.is_empty() {
            self.sequence.clear();
        }

        self.pressing.retain(|k| k != key);
        self.pressing.push(key);

        // Add to sequence if:
        // 1. It's different from the last key, OR
        // 2. It's the same key but was NOT already pressed (re-press after release)
        // This handles:
        // - Hold: A B B B B C -> A B C (consecutive B's without release)
        // - Re-press: A B (release B) B (release B) B -> A B
        if self.sequence.last() != Some(&key) {
            self.sequence.push(key);
        }
    }

    /// Marks a key as released.
    pub fn keyup<K: Into<VKey>>(&mut self, key: K) {
        let key = key.into();
        self.pressing.retain(|k| k != key);
        // Note: sequence is NOT cleared here when pressing becomes empty
        // It will be cleared on the next keydown() call (when starting a fresh sequence)
    }

    /// Checks if a key is currently pressed.
    pub fn is_down<K: Into<VKey>>(&self, key: K) -> bool {
        let key = key.into();
        self.pressing.iter().any(|k| key.matches(k))
    }

    /// Checks if all keys in a slice are currently pressed.
    pub fn are_down(&self, keys: &[VKey]) -> bool {
        keys.iter().all(|key| self.is_down(*key))
    }

    /// Checks if any key in a slice is currently pressed.
    pub fn some_is_down(&self, keys: &[VKey]) -> bool {
        keys.iter().any(|key| self.is_down(*key))
    }

    pub fn is_shift_pressed(&self) -> bool {
        self.some_is_down(&[VKey::LShift, VKey::RShift, VKey::Shift])
    }

    pub fn is_control_pressed(&self) -> bool {
        self.some_is_down(&[VKey::LControl, VKey::RControl, VKey::Control])
    }

    pub fn is_menu_pressed(&self) -> bool {
        self.some_is_down(&[VKey::LMenu, VKey::Menu, VKey::RMenu])
    }

    pub fn is_win_pressed(&self) -> bool {
        self.some_is_down(&[VKey::LWin, VKey::RWin])
    }

    /// Clears the state of all keys, marking them as released.
    pub fn clear(&mut self) {
        self.pressing.clear();
        self.sequence.clear();
        log_on_dev!("KeyboardState cleared");
    }

    pub fn request_syncronization(&mut self) {
        self.needs_sync = true;
        self.sync_count = 0;
    }

    /// Checks the state of each pressed key against
    /// the OS and removes them if they are not pressed.
    pub fn sync(&mut self) {
        for vk_code in 0..256 {
            if !Self::async_is_key_down(vk_code) {
                self.keyup(vk_code);
            }
        }
        self.sync_count += 1;
        if self.sync_count >= SYNC_COUNT_NEEDED_TO_BE_CONSIDERATED_SAFE {
            self.sync_count = 0;
            self.needs_sync = false;
        }
    }

    /// Returns whether a key is currently pressed according to the OS.
    pub fn async_is_key_down(key: u16) -> bool {
        let state: i16 = unsafe { GetAsyncKeyState(key.into()) };
        // Check if the high-order bit is set (on intergers this bit is set if the value is negative)
        state < 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keydown() {
        let mut keyboard = KeyboardState::new();
        keyboard.keydown(65);
        assert_eq!(
            keyboard.pressing[0],
            VKey::from_vk_code(65),
            "Key 65 should be set"
        );

        keyboard.keydown(129);
        assert_eq!(
            keyboard.pressing[1],
            VKey::from_vk_code(129),
            "Key 129 should be set"
        );
    }

    #[test]
    fn test_keyup() {
        let mut keyboard = KeyboardState::new();
        keyboard.keydown(65); // Press key 65
        keyboard.keyup(65); // Release key 65
        assert_eq!(keyboard.pressing.first(), None, "Key 65 should be cleared");

        keyboard.keydown(129); // Press key 129
        keyboard.keyup(129); // Release key 129
        assert_eq!(keyboard.pressing.get(1), None, "Key 129 should be cleared");
    }

    #[test]
    fn test_clear() {
        let mut keyboard = KeyboardState::new();
        keyboard.keydown(65); // Press key 65
        keyboard.keydown(129); // Press key 129
        keyboard.clear(); // Clear all keys
        assert_eq!(
            keyboard.pressing,
            Vec::new(),
            "KeyboardState should be cleared after clear()"
        );
    }

    #[test]
    fn test_clone() {
        let mut keyboard = KeyboardState::new();
        keyboard.keydown(65); // Press key 65
        let cloned_keyboard = keyboard.clone();
        assert_eq!(
            keyboard, cloned_keyboard,
            "Cloned KeyboardState should be equal to the original"
        );

        // Modify the original and ensure the clone is unaffected
        keyboard.keydown(129);
        assert_ne!(
            keyboard, cloned_keyboard,
            "Cloned KeyboardState should not reflect changes to the original"
        );
    }

    #[test]
    fn test_equality() {
        let mut keyboard1 = KeyboardState::new();
        let mut keyboard2 = KeyboardState::new();

        // Both are empty and should be equal
        assert_eq!(
            keyboard1, keyboard2,
            "Two empty KeyboardState instances should be equal"
        );

        // Modify one and ensure inequality
        keyboard1.keydown(65);
        assert_ne!(
            keyboard1, keyboard2,
            "KeyboardState instances with different flags should not be equal"
        );

        // Make them equal again
        keyboard2.keydown(65);
        assert_eq!(
            keyboard1, keyboard2,
            "KeyboardState instances with the same flags should be equal"
        );
    }

    #[test]
    fn test_multiple_keys() {
        let mut keyboard = KeyboardState::new();

        // Press multiple keys
        keyboard.keydown(65);
        keyboard.keydown(70);
        keyboard.keydown(129);

        assert!(keyboard.is_down(65), "Key 65 should be set");
        assert!(keyboard.is_down(70), "Key 70 should be set");
        assert!(keyboard.is_down(129), "Key 129 should be set");

        // Release some keys
        keyboard.keyup(65);
        keyboard.keyup(70);

        assert!(!keyboard.is_down(65), "Key 65 should be cleared");
        assert!(!keyboard.is_down(70), "Key 70 should be cleared");
        assert_eq!(
            keyboard.pressing[0],
            VKey::from_vk_code(129),
            "Key 129 should remain set"
        );
    }
}
