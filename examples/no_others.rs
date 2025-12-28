use win_hotkeys::{Hotkey, HotkeyManager, TriggerTiming, VKey};

fn main() {
    let hkm = HotkeyManager::current();

    // If another key outside the needed sequence is pressed, the hotkey will not be triggered
    // al well won't be triggered if the order of pressing the modifiers is incorrect
    hkm.register_hotkey(
        Hotkey::new(VKey::A, [VKey::Control, VKey::Shift], || {
            println!("CTRL + A pressed on a strict sequence");
        })
        .strict_sequence(),
    )
    .unwrap();

    // If another key is pressed before the hotkey is triggered, the hotkey will not be triggered
    hkm.register_hotkey(
        Hotkey::new(VKey::LWin, [], || {
            println!("WIN pressed on a strict sequence");
        })
        .trigger_timing(TriggerTiming::OnKeyUp)
        .strict_sequence(),
    )
    .unwrap();

    let event_loop_thread = HotkeyManager::start_keyboard_capturing().unwrap();
    event_loop_thread.join().unwrap();
}
