//! Defines the `HotkeyManager`, which manages the registration,
//! unregistration, and execution of hotkeys. It also handles the main event
//! loop that listens for keyboard events and invokes associated callbacks.

use arc_swap::ArcSwapOption;

use crate::client_executor::{self, run_on_executor_thread};
use crate::error::WHKError::HotKeyAlreadyRegistered;
use crate::error::{Result, WHKError};
use crate::events::{KeyAction, KeyboardInputEvent};
use crate::hotkey::{Hotkey, TriggerBehavior, TriggerTiming};
use crate::{hook, log_on_dev};
use crate::{is_stealing_mode, VKey, STEALING};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, RwLock};

type HotkeysMap = Arc<RwLock<HashMap<VKey, HashSet<Hotkey>>>>;
type KeyboardCallback = dyn Fn(KeyboardInputEvent) + Send + Sync + 'static;
type FreeKeyboardCallback = dyn Fn() + Send + Sync + 'static;

static HOTKEYS: LazyLock<HotkeysMap> = LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

static CLIENT_KEYBOARD_CALLBACK: ArcSwapOption<Box<KeyboardCallback>> =
    ArcSwapOption::const_empty();
static CLIENT_ON_FREE_KEYBOARD_CB: ArcSwapOption<Box<FreeKeyboardCallback>> =
    ArcSwapOption::const_empty();

/// Manages the hotkeys, including their registration, unregistration, and execution.
///
/// The `HotkeyManager` listens for keyboard events and triggers the corresponding
/// hotkey callbacks when events match registered hotkeys.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HotkeyManager {
    /// stores the registered hotkeys
    hotkeys: HotkeysMap,
}

impl HotkeyManager {
    pub fn current() -> HotkeyManager {
        HotkeyManager {
            hotkeys: HOTKEYS.clone(),
        }
    }

    /// Sets the stealing mode for the hotkey manager until the `ESC` key is pressed,
    /// or client manually frees the keyboard.
    pub fn steal_keyboard<F>(&self, on_free: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        log_on_dev!("Keyboard stealing mode enabled");
        STEALING.store(true, Ordering::SeqCst);
        CLIENT_ON_FREE_KEYBOARD_CB.store(Some(Arc::new(Box::new(on_free))));
    }

    /// Disables the stealing mode for the hotkey manager.
    pub fn free_keyboard(&self) {
        log_on_dev!("Keyboard stealing mode disabled");
        STEALING.store(false, Ordering::SeqCst);
        if let Some(on_free_cb) = CLIENT_ON_FREE_KEYBOARD_CB.swap(None) {
            run_on_executor_thread(on_free_cb);
        }
    }

    /// Registers a new hotkey.
    pub fn register_hotkey(&self, hotkey: impl Into<Hotkey>) -> Result<u64> {
        let hotkey = hotkey.into();
        if hotkey.trigger_key == VKey::None {
            return Err(WHKError::HotkeyInvalidTriggerKey(hotkey.trigger_key));
        }

        let id = hotkey.as_hash();
        let was_already_inserted = !self
            .hotkeys
            .write()?
            .entry(hotkey.trigger_key)
            .or_default()
            .insert(hotkey);

        if was_already_inserted {
            return Err(HotKeyAlreadyRegistered);
        }
        Ok(id)
    }

    /// Unregisters a hotkey by its unique id.
    pub fn unregister_hotkey(&self, hotkey_id: u64) -> Result<()> {
        for hotkeys in self.hotkeys.write()?.values_mut() {
            hotkeys.retain(|hotkey| hotkey.as_hash() != hotkey_id);
        }
        Ok(())
    }

    /// Unregisters all hotkeys.
    pub fn unregister_all(&self) -> Result<()> {
        *self.hotkeys.write()? = HashMap::new();
        Ok(())
    }

    /// Starts the keyboard hook and returns the hook thread's join handle.
    ///
    /// The handle can be joined to block the calling thread until
    /// `stop_keyboard_capturing()` is called.
    pub fn start_keyboard_capturing() -> Result<std::thread::JoinHandle<()>> {
        client_executor::start_executor_thread();
        hook::start()
    }

    pub(crate) fn process_keyboard_event(event: KeyboardInputEvent) -> KeyAction {
        if let Some(cb) = CLIENT_KEYBOARD_CALLBACK.load().as_ref() {
            let cb = cb.clone();
            let event = event.clone();
            run_on_executor_thread(Arc::new(move || {
                cb(event.clone());
            }));
        }

        let manager = HotkeyManager::current();

        if is_stealing_mode() {
            // Stealing mode only affects KeyDown events
            if let KeyboardInputEvent::KeyDown { key, state: _ } = &event {
                if key == VKey::Escape {
                    manager.free_keyboard();
                }
                // note: on ESC press we exit stealing mode, but still will block the ESC key
                return KeyAction::Block;
            }
        }

        // Extract vk_code, state, and event_type from both KeyDown and KeyUp events
        let (key, state, event_type) = match event {
            KeyboardInputEvent::KeyDown { key, state } => (key, state, TriggerTiming::OnKeyDown),
            KeyboardInputEvent::KeyUp { key, state } => (key, state, TriggerTiming::OnKeyUp),
        };

        let paused_state = HotkeysPauseHandler::current();

        if let Some(hotkeys) = HOTKEYS.read().unwrap().get(&key) {
            for hotkey in hotkeys {
                // Skip if timing doesn't match
                if hotkey.trigger_timing != event_type {
                    continue;
                }

                // Skip if paused (unless bypass_pause)
                if paused_state.is_paused() && !hotkey.bypass_pause {
                    continue;
                }

                // Check if keyboard state matches hotkey
                if !hotkey.is_trigger_state(&key, &state) {
                    continue;
                }

                // Execute hotkey callback
                run_on_executor_thread(hotkey.callback.clone());

                // Return appropriate action based on behavior
                return match hotkey.behaviour {
                    TriggerBehavior::PassThrough => KeyAction::Allow,
                    TriggerBehavior::StopPropagation => KeyAction::Block,
                };
            }
        }

        KeyAction::Allow
    }

    /// Stops the keyboard hook and cleans up resources.
    pub fn stop_keyboard_capturing() {
        hook::stop();
        client_executor::stop_executor_thread();
    }

    pub fn set_global_keyboard_listener<F>(&self, cb: F)
    where
        F: Fn(KeyboardInputEvent) + Send + Sync + 'static,
    {
        CLIENT_KEYBOARD_CALLBACK.store(Some(Arc::new(Box::new(cb))));
    }

    pub fn remove_global_keyboard_listener(&self) {
        CLIENT_KEYBOARD_CALLBACK.store(None);
    }

    /// Signals the `HotkeyManager` to pause processing of hotkeys.
    pub fn pause_handler(&self) -> HotkeysPauseHandler {
        HotkeysPauseHandler::current()
    }
}

/// A handle for signaling the `HotkeyManager` to stop processing hotkeys without
/// exiting the event loop or unregistering hotkeys. When paused, the `HotkeyManager`
/// will only process registered pause hotkeys.
///
/// The `PauseHandle` is used to manage the pause state of the `HotkeyManager`.
pub struct HotkeysPauseHandler {
    state: &'static AtomicBool,
}

impl HotkeysPauseHandler {
    /// Creates a new `PauseHandler` that controls the pause state of the `HotkeyManager`.
    pub fn current() -> Self {
        Self {
            state: &crate::PAUSED,
        }
    }

    /// Toggles the pause state of the `HotkeyManager`.
    ///
    /// If the `HotkeyManager` is currently paused, calling this method will resume
    /// normal hotkey processing. If it is active, calling this method will pause it.
    pub fn toggle(&self) {
        self.state
            .store(!self.state.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    /// Explicitly sets the pause state.
    pub fn set(&self, state: bool) {
        self.state.store(state, Ordering::Relaxed);
    }

    /// Returns whether the `HotkeyManager` is currently paused.
    ///
    /// When paused, only pause hotkeys will be processed while all others will
    /// be ignored.
    pub fn is_paused(&self) -> bool {
        self.state.load(Ordering::Relaxed)
    }
}
