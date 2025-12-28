use std::sync::{Arc, LazyLock, Mutex};

use win_hotkeys::{events::KeyboardInputEvent, Hotkey, HotkeyManager, VKey};

static LATEST_PRESSED: LazyLock<Arc<Mutex<Vec<VKey>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

fn main() {
    let hkm = HotkeyManager::current();

    hkm.register_hotkey(Hotkey::new(VKey::P, [VKey::Control, VKey::Alt], || {
        println!("Hotkey CTRL + ALT + P was triggered");
        let m = HotkeyManager::current();

        println!("Starting stealing mode");
        m.steal_keyboard(|| {
            println!("Stealing mode canceled");
            // remove listener as is un-neded
            HotkeyManager::current().remove_global_keyboard_listener();
            LATEST_PRESSED.lock().unwrap().clear(); // clean state
        });

        m.set_global_keyboard_listener(|event| {
            if let KeyboardInputEvent::KeyDown {
                key,
                state: keyboard_state,
            } = event
            {
                println!("Global key down: {key:?}");

                if key == VKey::Return {
                    println!("Enter pressed, stealing mode finished");
                    let last = LATEST_PRESSED.lock().unwrap();

                    println!("Registering new shortcut: {last:#?}");
                    let shortcut = Hotkey::from_keys(last.as_slice());
                    let manager = HotkeyManager::current();
                    if let Err(e) = manager.register_hotkey(shortcut) {
                        println!("Failed to register shortcut: {e}");
                    }

                    manager.free_keyboard(); // end stealing mode
                } else {
                    *LATEST_PRESSED.lock().unwrap() = keyboard_state.pressing;
                }
            }
        });
    }))
    .unwrap();

    let event_loop_thread = HotkeyManager::start_keyboard_capturing().unwrap();
    // Block until the event loop thread exits
    event_loop_thread.join().unwrap();
}
