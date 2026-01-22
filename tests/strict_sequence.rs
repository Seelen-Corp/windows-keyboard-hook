//! Comprehensive tests for strict_sequence feature
//! These tests validate that strict sequences work correctly for both OnKeyDown and OnKeyUp
//!
//! Edge cases covered:
//! 1. Exact sequence matching (should trigger)
//! 2. Extra keys before/in middle/after sequence (should NOT trigger)
//! 3. Wrong order of keys (should NOT trigger)
//! 4. Repeated key presses while holding modifiers (should NOT trigger on second press)
//! 5. Sequence reset after all keys released
//! 6. OnKeyUp timing with strict sequences
//! 7. Complex multi-modifier sequences

use win_hotkeys::state::KeyboardState;
use win_hotkeys::VKey;
use win_hotkeys::{Hotkey, TriggerTiming};

// ============================================================================
// BASIC STRICT SEQUENCE TESTS - OnKeyDown
// ============================================================================

#[test]
fn test_strict_sequence_exact_match() {
    // Hotkey: Control + Shift + A (strict sequence)
    let hotkey = Hotkey::new(VKey::A, [VKey::Control, VKey::Shift], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Control);
    state.keydown(VKey::Shift);
    state.keydown(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Should trigger when sequence is exactly [Control, Shift, A]"
    );
}

#[test]
fn test_strict_sequence_extra_key_before() {
    // Hotkey: Control + Shift + A (strict sequence)
    let hotkey = Hotkey::new(VKey::A, [VKey::Control, VKey::Shift], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::B); // Extra key before
    state.keydown(VKey::Control);
    state.keydown(VKey::Shift);
    state.keydown(VKey::A);

    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state),
        "Should NOT trigger when extra key B is pressed before sequence"
    );
}

#[test]
fn test_strict_sequence_extra_key_in_middle() {
    // Hotkey: Control + Shift + A (strict sequence)
    let hotkey = Hotkey::new(VKey::A, [VKey::Control, VKey::Shift], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Control);
    state.keydown(VKey::B); // Extra key in the middle
    state.keydown(VKey::Shift);
    state.keydown(VKey::A);

    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state),
        "Should NOT trigger when extra key B is pressed in the middle of sequence"
    );
}

#[test]
fn test_strict_sequence_extra_key_after_trigger() {
    // Hotkey: Control + A (strict sequence)
    let hotkey = Hotkey::new(VKey::A, [VKey::Control], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Control);
    state.keydown(VKey::A);
    state.keydown(VKey::B); // Extra key after trigger

    // The hotkey should have triggered on line above when A was pressed
    // but NOT trigger now that B is pressed
    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state),
        "Sequence [Control, A, B] should not match strict [Control, A]"
    );
}

#[test]
fn test_strict_sequence_wrong_order() {
    // Hotkey: Control + Shift + A (strict sequence)
    let hotkey = Hotkey::new(VKey::A, [VKey::Control, VKey::Shift], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Shift); // Wrong order (Shift before Control)
    state.keydown(VKey::Control);
    state.keydown(VKey::A);

    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state),
        "Should NOT trigger when modifiers are pressed in wrong order"
    );
}

// ============================================================================
// REPEATED KEY PRESSES (Edge case: holding modifier, pressing trigger multiple times)
// ============================================================================

#[test]
fn test_strict_sequence_holding_modifier_pressing_trigger_twice() {
    // Hotkey: Control + A (strict sequence)
    // Key behavior with sequences:
    // - Hold (A → A without release): doesn't add duplicate, sequence stays [Control, A]
    // - Re-press (A → release A → A): does NOT add to sequence while Control is held
    let hotkey = Hotkey::new(VKey::A, [VKey::Control], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Control);
    state.keydown(VKey::A);

    // First press should trigger
    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "First press of A should trigger with sequence [Control, A]"
    );

    // Release and press A again while holding Control
    state.keyup(VKey::A);
    state.keydown(VKey::A);

    // When A is re-pressed after release (while Control is still held),
    // the sequence stays [Control, A] and should still trigger
    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Second press of A after release should trigger (sequence continues: [Control, A])"
    );

    // However, holding A (without release) should NOT add duplicates
    let mut state_hold = KeyboardState::new();
    state_hold.keydown(VKey::Control);
    state_hold.keydown(VKey::A);
    state_hold.keydown(VKey::A); // Hold (no release)
    state_hold.keydown(VKey::A); // Hold (no release)

    // Sequence is still [Control, A] because A was never released
    assert!(
        hotkey.is_trigger_state(&VKey::A, &state_hold),
        "Holding A (without release) should keep sequence as [Control, A]"
    );
}

#[test]
fn test_strict_sequence_modifier_key_pressed_multiple_times() {
    // Hotkey: Control + Shift + A (strict sequence)
    // Key behavior with sequences:
    // - Hold (Shift → Shift without release): doesn't add duplicate, sequence stays [Control, Shift]
    // - Re-press (Shift → release Shift → Shift): does NOT add to sequence while other keys are held
    let hotkey = Hotkey::new(VKey::A, [VKey::Control, VKey::Shift], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Control);
    state.keydown(VKey::Shift);
    state.keyup(VKey::Shift); // Release Shift
    state.keydown(VKey::Shift); // Press Shift again (does NOT add duplicate while Control is held)
    state.keydown(VKey::A);

    // When Shift is re-pressed after release (while Control is still held),
    // the sequence stays [Control, Shift, A] and should trigger
    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Should trigger when modifier is re-pressed after release (sequence continues: [Control, Shift, A])"
    );

    // However, HOLDING Shift (without release) should NOT add duplicates
    let mut state_hold = KeyboardState::new();
    state_hold.keydown(VKey::Control);
    state_hold.keydown(VKey::Shift);
    state_hold.keydown(VKey::Shift); // Hold (no release)
    state_hold.keydown(VKey::Shift); // Hold (no release)
    state_hold.keydown(VKey::A);

    // Sequence is [Control, Shift, A] because Shift was never released
    assert!(
        hotkey.is_trigger_state(&VKey::A, &state_hold),
        "Should trigger when Shift is held (without release), sequence: [Control, Shift, A]"
    );
}

// ============================================================================
// ONKEYUP TIMING WITH STRICT SEQUENCES
// ============================================================================

#[test]
fn test_strict_sequence_on_key_up_exact_match() {
    // Hotkey: LWin (strict sequence, on key up)
    // Expected sequence: [LWin] when released
    let hotkey = Hotkey::new(VKey::LWin, [], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();
    state.keydown(VKey::LWin);
    state.keyup(VKey::LWin);

    assert!(
        hotkey.is_trigger_state(&VKey::LWin, &state),
        "Should trigger when only LWin was pressed and released"
    );
}

#[test]
fn test_strict_sequence_on_key_up_with_extra_key() {
    // Hotkey: LWin (strict sequence, on key up)
    // This is the MAIN BUG CASE from the user's example
    // Sequence: [LWin, V] should NOT trigger
    let hotkey = Hotkey::new(VKey::LWin, [], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();
    state.keydown(VKey::LWin);
    state.keydown(VKey::V); // Extra key pressed
    state.keyup(VKey::LWin);

    assert!(
        !hotkey.is_trigger_state(&VKey::LWin, &state),
        "Should NOT trigger when another key (V) was pressed before releasing LWin"
    );
}

#[test]
fn test_strict_sequence_on_key_up_multiple_extra_keys() {
    // Hotkey: LWin (strict sequence, on key up)
    // Simulating user's log: LWin, V, V, V, V, V sequence
    let hotkey = Hotkey::new(VKey::LWin, [], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();
    state.keydown(VKey::LWin);
    state.keydown(VKey::V);
    state.keyup(VKey::V);
    state.keydown(VKey::V); // V pressed again
    state.keyup(VKey::V);
    state.keydown(VKey::V); // V pressed again
    state.keyup(VKey::LWin); // Finally release LWin

    // Sequence is [LWin, V, V, V] which should NOT match [LWin]
    assert!(
        !hotkey.is_trigger_state(&VKey::LWin, &state),
        "Should NOT trigger when V was pressed multiple times before releasing LWin"
    );
}

#[test]
fn test_strict_sequence_on_key_up_with_modifiers_exact() {
    // Hotkey: Control + A (strict sequence, on key up of A)
    let hotkey = Hotkey::new(VKey::A, [VKey::Control], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Control);
    state.keydown(VKey::A);
    state.keyup(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Should trigger when exact sequence [Control, A] is pressed and A is released"
    );
}

#[test]
fn test_strict_sequence_on_key_up_with_modifiers_extra_key() {
    // Hotkey: Control + A (strict sequence, on key up of A)
    let hotkey = Hotkey::new(VKey::A, [VKey::Control], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Control);
    state.keydown(VKey::B); // Extra key
    state.keydown(VKey::A);
    state.keyup(VKey::A);

    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state),
        "Should NOT trigger when extra key B was pressed in the sequence"
    );
}

// ============================================================================
// SEQUENCE RESET AFTER ALL KEYS RELEASED
// ============================================================================

#[test]
fn test_strict_sequence_resets_after_all_keys_released() {
    // Hotkey: LWin (strict sequence, on key up)
    let hotkey = Hotkey::new(VKey::LWin, [], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();

    // First sequence with extra key (should NOT trigger)
    state.keydown(VKey::LWin);
    state.keydown(VKey::V);
    state.keyup(VKey::V);
    state.keyup(VKey::LWin);

    assert!(
        !hotkey.is_trigger_state(&VKey::LWin, &state),
        "First sequence [LWin, V] should NOT trigger"
    );

    // CRITICAL: After all keys released, pressing a NEW key should start fresh sequence
    // The sequence should NOT be cleared immediately when pressing is empty,
    // but when a new key is pressed AFTER pressing becomes empty
    state.keydown(VKey::LWin);
    state.keyup(VKey::LWin);

    assert!(
        hotkey.is_trigger_state(&VKey::LWin, &state),
        "After sequence resets, clean [LWin] sequence should trigger"
    );
}

#[test]
fn test_sequence_clears_on_new_key_after_empty_not_on_release() {
    // This tests the core fix: sequence should clear when a new key is pressed
    // AFTER pressing becomes empty, not immediately when pressing becomes empty

    let mut state = KeyboardState::new();

    // Build up a sequence
    state.keydown(VKey::LWin);
    state.keydown(VKey::V);
    state.keydown(VKey::B);

    assert_eq!(state.sequence, vec![VKey::LWin, VKey::V, VKey::B]);

    // Release all keys
    state.keyup(VKey::B);
    state.keyup(VKey::V);
    state.keyup(VKey::LWin);

    // IMPORTANT: Sequence should still exist after releasing all keys
    // It should only clear when a NEW key is pressed
    assert_eq!(
        state.sequence,
        vec![VKey::LWin, VKey::V, VKey::B],
        "Sequence should NOT clear immediately when all keys released"
    );

    // Now press a new key - this should clear the old sequence
    state.keydown(VKey::A);

    assert_eq!(
        state.sequence,
        vec![VKey::A],
        "Sequence should clear when new key is pressed after all keys were released"
    );
}

#[test]
fn test_sequence_does_not_clear_if_keys_still_pressed() {
    let mut state = KeyboardState::new();

    state.keydown(VKey::LWin);
    state.keydown(VKey::V);

    // Release V but keep LWin pressed
    state.keyup(VKey::V);

    // Press V again while LWin is still down
    state.keydown(VKey::V);

    // When V is re-pressed after release (while LWin is still held),
    // it should NOT add a duplicate to the sequence
    assert_eq!(
        state.sequence,
        vec![VKey::LWin, VKey::V],
        "Sequence should continue when keys are re-pressed while others are still held"
    );
}

// ============================================================================
// NON-STRICT SEQUENCE (should allow extra keys)
// ============================================================================

#[test]
fn test_non_strict_sequence_allows_extra_keys() {
    // Hotkey: Control + A (NOT strict sequence)
    let hotkey =
        Hotkey::new(VKey::A, [VKey::Control], || {}).trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Control);
    state.keydown(VKey::B); // Extra key
    state.keydown(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Non-strict sequence should trigger even with extra keys"
    );
}

#[test]
fn test_non_strict_sequence_allows_repeated_presses() {
    // Hotkey: Control + A (NOT strict sequence)
    let hotkey =
        Hotkey::new(VKey::A, [VKey::Control], || {}).trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Control);
    state.keydown(VKey::A);
    state.keyup(VKey::A);
    state.keydown(VKey::A); // A pressed again

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Non-strict sequence should trigger on repeated key presses"
    );
}

// ============================================================================
// COMPLEX MULTI-MODIFIER SEQUENCES
// ============================================================================

#[test]
fn test_strict_sequence_complex_four_modifiers() {
    // Hotkey: Control + Shift + Alt + Win + A (strict sequence)
    let hotkey = Hotkey::new(
        VKey::A,
        [VKey::Control, VKey::Shift, VKey::Menu, VKey::LWin],
        || {},
    )
    .strict_sequence()
    .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Control);
    state.keydown(VKey::Shift);
    state.keydown(VKey::Menu);
    state.keydown(VKey::LWin);
    state.keydown(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Should trigger with exact complex sequence"
    );
}

#[test]
fn test_strict_sequence_complex_wrong_order() {
    // Hotkey: Control + Shift + Alt + A (strict sequence)
    let hotkey = Hotkey::new(VKey::A, [VKey::Control, VKey::Shift, VKey::Menu], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Menu); // Wrong order
    state.keydown(VKey::Control);
    state.keydown(VKey::Shift);
    state.keydown(VKey::A);

    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state),
        "Should NOT trigger when complex modifiers are in wrong order"
    );
}

// ============================================================================
// DOUBLE-TAP SEQUENCES (same key pressed twice)
// ============================================================================
// NOTE: With the hold key fix, double-tap sequences (e.g., [A, A]) are no longer
// supported in strict sequence mode because:
// 1. Consecutive keydowns without release are treated as holds: A A A → [A]
// 2. The expected sequence generation also produces [A] instead of [A, A]
// 3. Both create matching sequences [A], making them indistinguishable
//
// To support true double-taps, a different approach would be needed (e.g., tracking
// releases or using time-based detection). For now, double-tap tests are removed.

// ============================================================================
// EDGE CASE: Empty modifiers with strict sequence
// ============================================================================

#[test]
fn test_strict_sequence_no_modifiers_single_key() {
    // Hotkey: Just A (no modifiers, strict sequence)
    let hotkey = Hotkey::new(VKey::A, [], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Should trigger for single key with no modifiers"
    );
}

#[test]
fn test_strict_sequence_no_modifiers_but_other_key_pressed_first() {
    // Hotkey: Just A (no modifiers, strict sequence)
    let hotkey = Hotkey::new(VKey::A, [], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::B); // Another key pressed first
    state.keydown(VKey::A);

    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state),
        "Should NOT trigger if another key was pressed before in strict mode"
    );
}

// ============================================================================
// HOLD KEY WITH STRICT SEQUENCE AND KEY UP TRIGGER
// ============================================================================
// When a user holds a key down, Windows sends multiple WM_KEYDOWN events.
// These repeated keydown events for the SAME key should NOT cancel the strict
// sequence, as they represent a single continuous key press (hold).

#[test]
fn test_strict_sequence_hold_single_key_on_key_up() {
    // Hotkey: A (no modifiers, strict sequence, trigger on key up)
    // Simulating: Press A and hold (multiple keydown), then release
    // Expected: Should trigger because sequence is still just [A]
    let hotkey = Hotkey::new(VKey::A, [], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();

    // First keydown
    state.keydown(VKey::A);

    // Simulate holding the key (Windows sends multiple WM_KEYDOWN)
    state.keydown(VKey::A); // Hold event 1
    state.keydown(VKey::A); // Hold event 2
    state.keydown(VKey::A); // Hold event 3

    // Release the key
    state.keyup(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Should trigger when key is held (multiple keydown of same key) then released"
    );
}

#[test]
fn test_strict_sequence_hold_single_key_on_key_down() {
    // Hotkey: A (no modifiers, strict sequence, trigger on key down)
    // With the hold key fix, consecutive keydowns of the same key don't add duplicates
    // to the sequence, so the sequence remains [A] even after multiple keydowns
    let hotkey = Hotkey::new(VKey::A, [], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();

    // First keydown - should trigger
    state.keydown(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "First keydown should trigger"
    );

    // Simulate holding the key (subsequent keydowns of SAME key don't add to sequence)
    state.keydown(VKey::A); // Hold event 1 (sequence still [A])
    state.keydown(VKey::A); // Hold event 2 (sequence still [A])

    // The sequence is still [A], so it SHOULD trigger
    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Hold (consecutive keydowns of same key) should still trigger because sequence is still [A]"
    );

    // However, pressing a DIFFERENT key should break the sequence
    state.keydown(VKey::B); // Now sequence is [A, B]

    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state),
        "Pressing a different key (B) should break the sequence"
    );
}

#[test]
fn test_strict_sequence_hold_with_modifier_on_key_up() {
    // Hotkey: Control + A (strict sequence, trigger on key up of A)
    // Simulating: Press Control, press A and hold, then release A
    let hotkey = Hotkey::new(VKey::A, [VKey::Control], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();

    // Press modifier first
    state.keydown(VKey::Control);

    // Press trigger key
    state.keydown(VKey::A);

    // Simulate holding A (multiple keydown events)
    state.keydown(VKey::A); // Hold event 1
    state.keydown(VKey::A); // Hold event 2
    state.keydown(VKey::A); // Hold event 3
    state.keydown(VKey::A); // Hold event 4

    // Release A
    state.keyup(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Should trigger when A is held (multiple keydown) while Control is pressed, then A is released"
    );
}

#[test]
fn test_strict_sequence_hold_with_multiple_modifiers_on_key_up() {
    // Hotkey: Control + Shift + A (strict sequence, trigger on key up of A)
    // Simulating: Press Control, Shift, then A (held), then release A
    let hotkey = Hotkey::new(VKey::A, [VKey::Control, VKey::Shift], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();

    // Press modifiers in order
    state.keydown(VKey::Control);
    state.keydown(VKey::Shift);

    // Press trigger key
    state.keydown(VKey::A);

    // Simulate holding A (many keydown events)
    for _ in 0..10 {
        state.keydown(VKey::A);
    }

    // Release A
    state.keyup(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Should trigger even after many repeated keydown events (hold) of trigger key"
    );
}

#[test]
fn test_strict_sequence_hold_modifier_key_on_key_up() {
    // Hotkey: Control + A (strict sequence, trigger on key up of A)
    // Simulating: Press Control and HOLD it (multiple keydown), then press A, then release A
    let hotkey = Hotkey::new(VKey::A, [VKey::Control], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();

    // Press Control
    state.keydown(VKey::Control);

    // Simulate holding Control (multiple keydown events)
    state.keydown(VKey::Control); // Hold event 1
    state.keydown(VKey::Control); // Hold event 2
    state.keydown(VKey::Control); // Hold event 3

    // Press A
    state.keydown(VKey::A);

    // Release A
    state.keyup(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Should trigger when modifier key is held (multiple keydown) before trigger key is pressed and released"
    );
}

#[test]
fn test_strict_sequence_hold_both_modifier_and_trigger_on_key_up() {
    // Hotkey: Control + A (strict sequence, trigger on key up of A)
    // Simulating: Press and hold Control, then press and hold A, then release A
    let hotkey = Hotkey::new(VKey::A, [VKey::Control], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();

    // Press Control
    state.keydown(VKey::Control);

    // Hold Control
    state.keydown(VKey::Control);
    state.keydown(VKey::Control);

    // Press A
    state.keydown(VKey::A);

    // Hold A
    state.keydown(VKey::A);
    state.keydown(VKey::A);
    state.keydown(VKey::A);

    // Release A
    state.keyup(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Should trigger when both modifier and trigger keys are held (multiple keydown events each)"
    );
}

#[test]
fn test_strict_sequence_hold_vs_extra_key() {
    // This test verifies that holding a key (same key repeated) is different
    // from pressing a different key (which should cancel the sequence)

    // Hotkey: A (no modifiers, strict sequence, trigger on key up)
    let hotkey = Hotkey::new(VKey::A, [], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state_hold = KeyboardState::new();
    let mut state_extra = KeyboardState::new();

    // SCENARIO 1: Hold the same key (should trigger)
    state_hold.keydown(VKey::A);
    state_hold.keydown(VKey::A); // Hold
    state_hold.keydown(VKey::A); // Hold
    state_hold.keyup(VKey::A);

    // SCENARIO 2: Press a different key (should NOT trigger)
    state_extra.keydown(VKey::A);
    state_extra.keydown(VKey::B); // Different key!
    state_extra.keyup(VKey::A);

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state_hold),
        "Holding the same key (repeated keydown) should NOT cancel strict sequence"
    );

    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state_extra),
        "Pressing a different key SHOULD cancel strict sequence"
    );
}

#[test]
fn test_strict_sequence_long_hold_on_key_up() {
    // Hotkey: LWin (strict sequence, trigger on key up)
    // This simulates a realistic scenario where user holds Win key for a long time
    let hotkey = Hotkey::new(VKey::LWin, [], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyUp);

    let mut state = KeyboardState::new();

    // Press LWin
    state.keydown(VKey::LWin);

    // Simulate a very long hold (50 repeated keydown events)
    for _ in 0..50 {
        state.keydown(VKey::LWin);
    }

    // Release LWin
    state.keyup(VKey::LWin);

    assert!(
        hotkey.is_trigger_state(&VKey::LWin, &state),
        "Should trigger even after very long hold (many repeated keydown events)"
    );
}
