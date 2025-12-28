use win_hotkeys::{Hotkey, HotkeyManager, TriggerBehavior, TriggerTiming, VKey};

fn main() {
    let hkm = HotkeyManager::current();

    println!("=== Trigger Timing Example ===\n");

    // OnKeyDown example (default behavior)
    hkm.register_hotkey(Hotkey::new(VKey::A, [VKey::Control], || {
        println!("CTRL + A pressed (OnKeyDown - default)");
    }))
    .unwrap();

    // OnKeyUp example
    hkm.register_hotkey(
        Hotkey::new(VKey::B, [VKey::Control], || {
            println!("CTRL + B released (OnKeyUp)");
        })
        .trigger_timing(TriggerTiming::OnKeyUp),
    )
    .unwrap();

    // Same key combo, different timings - demonstrates both can coexist
    hkm.register_hotkey(
        Hotkey::new(VKey::C, [VKey::Control], || {
            println!("CTRL + C pressed");
        })
        .trigger_timing(TriggerTiming::OnKeyDown),
    )
    .unwrap();

    hkm.register_hotkey(
        Hotkey::new(VKey::C, [VKey::Control], || {
            println!("CTRL + C released");
        })
        .trigger_timing(TriggerTiming::OnKeyUp),
    )
    .unwrap();

    // OnKeyUp with PassThrough - event propagates to system
    hkm.register_hotkey(
        Hotkey::new(VKey::D, [VKey::Control], || {
            println!("CTRL + D released (PassThrough - event reaches system)");
        })
        .trigger_timing(TriggerTiming::OnKeyUp)
        .behavior(TriggerBehavior::PassThrough),
    )
    .unwrap();

    // OnKeyUp with StopPropagation - event is blocked
    hkm.register_hotkey(
        Hotkey::new(VKey::E, [VKey::Control], || {
            println!("CTRL + E released (StopPropagation - event blocked)");
        })
        .trigger_timing(TriggerTiming::OnKeyUp)
        .behavior(TriggerBehavior::StopPropagation),
    )
    .unwrap();

    // OnKeyUp without modifiers
    hkm.register_hotkey(
        Hotkey::new(VKey::F5, [], || {
            println!("F5 released");
        })
        .trigger_timing(TriggerTiming::OnKeyUp),
    )
    .unwrap();

    println!("Hotkeys registered. Try:");
    println!("  CTRL + A (press) - triggers on key down");
    println!("  CTRL + B (release) - triggers on key up");
    println!("  CTRL + C (press & release) - triggers both events");
    println!("  CTRL + D (release) - triggers with PassThrough");
    println!("  CTRL + E (release) - triggers with StopPropagation");
    println!("  F5 (release) - triggers on release without modifiers\n");

    let event_loop_thread = HotkeyManager::start_keyboard_capturing().unwrap();
    event_loop_thread.join().unwrap();
}
