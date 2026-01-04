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
    // Sequence should be [Control, A] on first press
    // But [Control, A, A] on second press (should NOT trigger)
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

    // Now sequence is [Control, A, A] which should NOT match
    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state),
        "Second press of A should NOT trigger because sequence is now [Control, A, A]"
    );
}

#[test]
fn test_strict_sequence_modifier_key_pressed_multiple_times() {
    // Hotkey: Control + Shift + A (strict sequence)
    let hotkey = Hotkey::new(VKey::A, [VKey::Control, VKey::Shift], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::Control);
    state.keydown(VKey::Shift);
    state.keyup(VKey::Shift); // Release Shift
    state.keydown(VKey::Shift); // Press Shift again
    state.keydown(VKey::A);

    // Sequence is [Control, Shift, Shift, A] which should NOT match [Control, Shift, A]
    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state),
        "Should NOT trigger when modifier is released and pressed again"
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

    // Sequence should be [LWin, V, V] - NOT cleared
    assert_eq!(
        state.sequence,
        vec![VKey::LWin, VKey::V, VKey::V],
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

#[test]
fn test_strict_sequence_double_tap_exact() {
    // Hotkey: A + A (double-tap A, strict sequence)
    let hotkey = Hotkey::new(VKey::A, [VKey::A], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::A);
    state.keydown(VKey::A); // A appears twice in sequence

    assert!(
        hotkey.is_trigger_state(&VKey::A, &state),
        "Should trigger when A is pressed twice (double-tap)"
    );
}

#[test]
fn test_strict_sequence_triple_tap_should_not_match_double() {
    // Hotkey: A + A (double-tap A, strict sequence)
    let hotkey = Hotkey::new(VKey::A, [VKey::A], || {})
        .strict_sequence()
        .trigger_timing(TriggerTiming::OnKeyDown);

    let mut state = KeyboardState::new();
    state.keydown(VKey::A);
    state.keydown(VKey::A);
    state.keydown(VKey::A); // Triple tap

    // Sequence is [A, A, A] which should NOT match [A, A]
    assert!(
        !hotkey.is_trigger_state(&VKey::A, &state),
        "Triple-tap [A, A, A] should NOT match double-tap [A, A]"
    );
}

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
