//! This module defines the `Hotkey` struct, which represents a keyboard hotkey.
//! A hotkey is composed of a trigger key, one or more modifier keys, and a callback function
//! that is executed when the hotkey is triggered.

use crate::state::KeyboardState;
use crate::VKey;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Defines what should happen with the key event after hotkey triggers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerBehavior {
    /// Allow the key event to propagate to other applications
    PassThrough,
    /// Consume the key event and prevent further processing
    StopPropagation,
}

/// Defines when a hotkey should trigger
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TriggerTiming {
    /// Trigger when the key combination is pressed down
    OnKeyDown,
    /// Trigger when the trigger key is released
    OnKeyUp,
}

/// Represents a finalised keyboard shortcut ready for registration.
/// Construct via [`Hotkey::new`] / [`HotkeyBuilder::new`] and convert with `.into()`.
pub struct Hotkey {
    /// key that must be pressed to trigger this hotkey
    pub trigger_key: VKey,
    /// when the hotkey should trigger (on key down or key up)
    pub trigger_timing: TriggerTiming,
    /// keys that must be pressed before the trigger key ex: [CTRL] + [A]
    pub modifiers: Vec<VKey>,
    /// action to perform when this hotkey is triggered
    pub behaviour: TriggerBehavior,
    /// will ignore the `paused` global state
    pub bypass_pause: bool,
    /// if true, the hotkey will only trigger if keys was pressed in a strict sequence
    pub strict_sequence: bool,
    /// callback function to execute when this hotkey is triggered
    pub callback: Arc<dyn Fn() + Send + Sync + 'static>,
    /// precomputed expected keyboard state for matching — avoids per-keypress allocation
    pub(crate) expected_state: KeyboardState,
}

impl Hotkey {
    /// Convenience shorthand — returns a [`HotkeyBuilder`].
    /// Equivalent to [`HotkeyBuilder::new`].
    pub fn new<M, F>(trigger_key: VKey, modifiers: M, callback: F) -> HotkeyBuilder
    where
        M: AsRef<[VKey]>,
        F: Fn() + Send + Sync + 'static,
    {
        HotkeyBuilder::new(trigger_key, modifiers, callback)
    }

    /// Convenience shorthand — returns a [`HotkeyBuilder`].
    /// Equivalent to [`HotkeyBuilder::from_keys`].
    pub fn from_keys<T: AsRef<[VKey]>>(keys: T) -> HotkeyBuilder {
        HotkeyBuilder::from_keys(keys)
    }

    fn compute_expected_state(
        trigger_key: VKey,
        modifiers: &[VKey],
        timing: TriggerTiming,
    ) -> KeyboardState {
        let mut state = KeyboardState::new();
        for key in modifiers {
            state.keydown(*key);
        }
        state.keydown(trigger_key);
        if timing == TriggerTiming::OnKeyUp {
            state.keyup(trigger_key);
        }
        state
    }

    /// Executes the callback associated with the hotkey.
    pub fn execute(&self) {
        (self.callback)()
    }

    /// Checks if current keyboard state should trigger hotkey callback.
    /// This should only be called if the most recent keypress is the
    /// trigger key for the hotkey.
    pub fn is_trigger_state(&self, changed: &VKey, state: &KeyboardState) -> bool {
        // last changed key must be the trigger key
        if self.trigger_key != *changed {
            return false;
        }

        let expected = &self.expected_state;

        // Verify all required non-modifier keys are pressed
        for key in &expected.pressing {
            if !key.is_modifier_key() && !state.is_down(*key) {
                return false;
            }
        }

        if self.strict_sequence {
            if expected.sequence.len() != state.sequence.len() {
                return false;
            }

            for (i, key) in expected.sequence.iter().enumerate() {
                if !key.matches(&state.sequence[i]) {
                    return false;
                }
            }
        }

        // Verify modifier key states match exactly
        // example hotkey "Win + A" won't trigger if "Win + Alt + A" is pressed
        expected.is_win_pressed() == state.is_win_pressed()
            && expected.is_menu_pressed() == state.is_menu_pressed()
            && expected.is_shift_pressed() == state.is_shift_pressed()
            && expected.is_control_pressed() == state.is_control_pressed()
    }

    /// Returns a hash representing the hotkey combination
    pub fn as_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl From<HotkeyBuilder> for Hotkey {
    fn from(builder: HotkeyBuilder) -> Self {
        let expected_state = Hotkey::compute_expected_state(
            builder.trigger_key,
            &builder.modifiers,
            builder.trigger_timing,
        );

        Hotkey {
            trigger_key: builder.trigger_key,
            trigger_timing: builder.trigger_timing,
            modifiers: builder.modifiers,
            behaviour: builder.behaviour,
            bypass_pause: builder.bypass_pause,
            strict_sequence: builder.strict_sequence,
            callback: builder.callback,
            expected_state,
        }
    }
}

impl fmt::Debug for Hotkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hotkey")
            .field("trigger_key", &self.trigger_key)
            .field("trigger_action", &self.behaviour)
            .field("trigger_timing", &self.trigger_timing)
            .field("modifiers", &self.modifiers)
            .field("callback", &"<callback>")
            .finish()
    }
}

impl Eq for Hotkey {}
impl PartialEq for Hotkey {
    fn eq(&self, other: &Self) -> bool {
        self.trigger_key == other.trigger_key
            && self.modifiers == other.modifiers
            && self.trigger_timing == other.trigger_timing
    }
}

impl Hash for Hotkey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.trigger_key.hash(state);
        self.modifiers.hash(state);
        self.trigger_timing.hash(state);
    }
}

/// Builder for constructing a [`Hotkey`]. Obtain one via [`Hotkey::new`] or [`HotkeyBuilder::new`].
/// Call `.into()` or pass directly to [`crate::HotkeyManager::register_hotkey`] to finalise.
pub struct HotkeyBuilder {
    /// key that must be pressed to trigger this hotkey
    pub trigger_key: VKey,
    /// when the hotkey should trigger (on key down or key up)
    pub trigger_timing: TriggerTiming,
    /// keys that must be pressed before the trigger key ex: [CTRL] + [A]
    pub modifiers: Vec<VKey>,
    /// action to perform when this hotkey is triggered
    pub behaviour: TriggerBehavior,
    /// will ignore the `paused` global state
    pub bypass_pause: bool,
    /// if true, the hotkey will only trigger if keys was pressed in a strict sequence
    pub strict_sequence: bool,
    /// callback function to execute when this hotkey is triggered
    pub callback: Arc<dyn Fn() + Send + Sync + 'static>,
}

impl HotkeyBuilder {
    /// Creates a new [`HotkeyBuilder`].
    pub fn new<M, F>(trigger_key: VKey, modifiers: M, callback: F) -> HotkeyBuilder
    where
        M: AsRef<[VKey]>,
        F: Fn() + Send + Sync + 'static,
    {
        HotkeyBuilder {
            trigger_key,
            modifiers: modifiers.as_ref().to_vec(),
            behaviour: TriggerBehavior::StopPropagation,
            trigger_timing: TriggerTiming::OnKeyDown,
            bypass_pause: false,
            strict_sequence: false,
            callback: Arc::new(callback),
        }
    }

    /// Last key in `keys` is used as the trigger; the rest become modifiers.
    pub fn from_keys<T: AsRef<[VKey]>>(keys: T) -> HotkeyBuilder {
        let mut keys: Vec<VKey> = keys.as_ref().to_vec();
        let trigger_key = keys.pop().unwrap_or(VKey::None);
        HotkeyBuilder::new(trigger_key, keys, || {})
    }

    pub fn trigger(mut self, key: VKey) -> Self {
        self.trigger_key = key;
        self
    }

    pub fn modifiers<T: AsRef<[VKey]>>(mut self, keys: T) -> Self {
        self.modifiers = keys.as_ref().to_vec();
        self
    }

    /// Sets the behavior when hotkey triggers
    pub fn behavior(mut self, action: TriggerBehavior) -> Self {
        self.behaviour = action;
        self
    }

    /// Makes the hotkey work even when global hotkeys are paused
    pub fn bypass_pause(mut self) -> Self {
        self.bypass_pause = true;
        self
    }

    pub fn strict_sequence(mut self) -> Self {
        self.strict_sequence = true;
        self
    }

    /// Sets when the hotkey should trigger (on key down or key up)
    pub fn trigger_timing(mut self, timing: TriggerTiming) -> Self {
        self.trigger_timing = timing;
        self
    }

    pub fn action<F>(mut self, action: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callback = Arc::new(action);
        self
    }

    /// Finalises the builder and returns a [`Hotkey`] with the expected state computed.
    pub fn build(self) -> Hotkey {
        self.into()
    }
}
