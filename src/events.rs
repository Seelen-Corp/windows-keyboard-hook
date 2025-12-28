use std::sync::LazyLock;

use crossbeam_channel::{Receiver, Sender};

use crate::{log_on_dev, state::KeyboardState, VKey};

static EVENT_LOOP_CHANNEL: LazyLock<(Sender<EventLoopEvent>, Receiver<EventLoopEvent>)> =
    LazyLock::new(crossbeam_channel::unbounded);

static ACTION_CHANNEL: LazyLock<(Sender<KeyAction>, Receiver<KeyAction>)> =
    LazyLock::new(crossbeam_channel::unbounded);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventLoopEvent {
    Stop,
    Keyboard(KeyboardInputEvent),
}

impl EventLoopEvent {
    pub(crate) fn send(self) {
        if EVENT_LOOP_CHANNEL.0.send(self).is_err() {
            log_on_dev!("Failed to send event");
        }
    }

    pub(crate) fn reciever() -> Receiver<EventLoopEvent> {
        EVENT_LOOP_CHANNEL.1.clone()
    }
}

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

impl KeyAction {
    pub(crate) fn send(self) {
        if ACTION_CHANNEL.0.send(self).is_err() {
            log_on_dev!("Failed to send key action");
        }
    }

    pub(crate) fn reciever() -> Receiver<KeyAction> {
        ACTION_CHANNEL.1.clone()
    }
}
