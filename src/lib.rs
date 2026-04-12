//! # Win-Hotkeys
//!
//! Win-hotkeys is a Rust library for creating and managing global hotkeys on Windows.
//! It provides an ergonomic API for setting up keyboard hooks, registering hotkeys,
//! and handling keyboard events in a safe and efficient manner.
#![cfg(windows)]

mod client_executor;
pub mod error;
pub mod events;
pub mod hook;
mod hotkey;
mod keys;
mod manager;
pub mod state;
mod utils;

pub use hotkey::*;
pub use keys::*;
pub use manager::*;

use std::sync::atomic::AtomicBool;

/// indicates whether the hotkey manager is paused
static PAUSED: AtomicBool = AtomicBool::new(false);
/// indicates whether the hotkey manager is in stealing mode
static STEALING: AtomicBool = AtomicBool::new(false);

pub fn is_stealing_mode() -> bool {
    STEALING.load(std::sync::atomic::Ordering::Acquire)
}

pub fn is_paused() -> bool {
    PAUSED.load(std::sync::atomic::Ordering::Acquire)
}
