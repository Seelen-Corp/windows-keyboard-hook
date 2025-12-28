# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`win-hotkeys` is a lightweight, thread-safe Rust library for managing system-wide hotkeys on Windows. It uses the `WH_KEYBOARD_LL` low-level keyboard hook to provide global hotkey functionality with full support for the WIN key as a modifier.

**Key capabilities:**
- Thread-safe hotkey registration and management
- Flexible key combinations (single or multiple modifiers)
- Rust callbacks and closures for hotkey actions
- Keyboard stealing mode for capturing all keyboard input
- Pause/resume functionality for hotkey processing
- Human-readable key names with aliases

## Development Commands

### Building
```bash
# Build the library
cargo build

# Build with all features
cargo build --all-features

# Build release version
cargo build --release

# Build with verbose logging (development feature)
cargo build --features verbose
```

### Testing
```bash
# Run all tests
cargo test

# Run a specific test
cargo test <test_name>

# Run tests for a specific module
cargo test keys::tests
```

### Benchmarking
```bash
# Run all benchmarks (uses criterion)
cargo bench

# Run specific benchmark
cargo bench <benchmark_name>
```

### Examples
```bash
# Run an example
cargo run --example simple
cargo run --example sniffer
cargo run --example pass_through
cargo run --example pause

# List all examples
cargo run --example
```

Available examples:
- `simple.rs` - Basic hotkey registration
- `sniffer.rs` - Keyboard stealing mode demonstration
- `app_command.rs` - Application control examples
- `handles.rs` - Hotkey ID management
- `pass_through.rs` - PassThrough vs StopPropagation behavior
- `pause.rs` - Pause/resume hotkey processing
- `pomodoro_timer.rs` - Practical timer implementation
- `vkeys.rs` - Key name conversion examples

### Documentation
```bash
# Generate and open documentation
cargo doc --open

# Generate docs with all features
cargo doc --all-features --open
```

## Architecture

### Threading Model

The library uses a multi-threaded event-driven architecture with four key threads:

1. **Hook Thread** (`hook.rs`): Windows message loop capturing raw keyboard events via `WH_KEYBOARD_LL`
2. **Event Loop Thread** (`manager.rs`): Processes keyboard events, matches hotkeys, determines actions
3. **Executor Thread** (`client_executor.rs`): Runs user callbacks asynchronously to avoid deadlocks
4. **Main Thread**: Application control and hotkey registration

Communication between threads uses crossbeam unbounded channels for thread-safe message passing.

### Core Components

**HotkeyManager** (`manager.rs`):
- Central manager for hotkey registration, unregistration, and execution
- Maintains static `HOTKEYS` HashMap grouped by trigger key
- Manages global state via atomic flags: `PAUSED`, `STEALING`, `STARTED`
- Provides keyboard stealing mode via `steal_keyboard()` and `free_keyboard()`
- Built-in system hotkeys: WIN+L (lock screen) and CTRL+ALT+DELETE (security screen)

**Hotkey** (`hotkey.rs`):
- Represents a keyboard shortcut with trigger key, modifiers, behavior, and callback
- Builder pattern: `.trigger()`, `.modifiers()`, `.behavior()`, `.bypass_pause()`, `.action()`
- Two trigger behaviors:
  - `PassThrough`: Allow key event propagation to system
  - `StopPropagation`: Block/consume key event

**KeyboardState** (`state.rs`):
- Singleton tracking currently pressed keys via `LazyLock<Arc<Mutex<>>>`
- Synchronizes with OS using `GetAsyncKeyState` (requires 10 consecutive cycles for stability)
- Provides helpers: `is_shift_pressed()`, `is_control_pressed()`, etc.
- Maintains ordered list of pressed keys

**VKey** (`keys.rs`):
- Enum of Windows virtual key codes based on Microsoft Virtual-Key Codes
- Supports 90+ key definitions (letters, numbers, function keys, multimedia keys)
- Key aliases:
  - `Ctrl` → `Control` (VK_CONTROL)
  - `LCtrl` → `LControl` (VK_LCONTROL), `RCtrl` → `RControl` (VK_RCONTROL)
  - `Alt` → `Menu` (VK_MENU)
  - `LAlt` → `LMenu` (VK_LMENU), `RAlt` → `RMenu` (VK_RMENU)
  - `Win` → `LWin` (VK_LWIN)
- Parsing: Supports hex strings (`0x29`), official names (`VK_MENU`), short names (`MENU`), and aliases
- **Important**: Left/right variants (e.g., `Shift` vs `LShift`/`RShift`) behave differently:
  - As modifiers: `Shift` matches either `LShift` or `RShift`
  - As trigger keys: `Shift` only matches the generic shift key (not present on most keyboards)

**Hook** (`hook.rs`):
- Low-level keyboard hook implementation using `WH_KEYBOARD_LL`
- 250ms timeout for key action response handling
- Silent key injection (0xE8 scan code) for replacing intercepted keys
- Power state awareness: Handles sleep/resume events via `power_sleep_resume_proc()`
- Runs on dedicated thread with Windows message loop

### State Management

- **Global State**: Static `LazyLock` for lazy initialization of mutexes, channels, and data structures
- **Atomic Flags**: `AtomicBool` for `PAUSED`, `STEALING`, `STARTED` states
- **Thread Safety**: All shared state protected by `Mutex` or `Arc`/`ArcSwapOption` for lock-free swaps
- **Event Channels**: crossbeam unbounded channels for inter-thread communication

### Error Handling

Custom `WHKError` enum (using `thiserror`) with variants:
- `AlreadyStarted`: Hook already running
- `StartupFailed`: Hook initialization failed
- `HotKeyAlreadyRegistered`: Duplicate hotkey
- `HotkeyInvalidTriggerKey`: Invalid trigger key (e.g., `VKey::None`)
- `InvalidKey`: Unknown key name during parsing
- `SendFailed`, `RecvFailed`: Channel communication errors
- `LockError`: Mutex poisoning

## Important Implementation Details

### Modifier Key Handling

When using generic modifier keys (`Shift`, `Control`, `Alt`) as modifiers, the library matches either left or right variants. However, when used as trigger keys, only the exact key matches. Always use specific variants (`LShift`, `RShift`, etc.) for trigger keys.

### Callback Execution

User callbacks run on the executor thread, not the hook thread. This prevents deadlocks and ensures the hook remains responsive. Never block in callbacks for extended periods.

### Power State Handling

The hook automatically handles system sleep/resume events. On resume, the keyboard state is synchronized to prevent stuck keys.

### Silent Key Injection

When replacing keys (`KeyAction::Replace`), the library injects a silent key (scan code 0xE8) that won't trigger other hooks or be visible to applications.

## Feature Flags

- `serde`: Enables serialization/deserialization support for `VKey` enum
- `verbose`: Enables debug logging via `log_on_dev!` macro (development only)

## Testing Notes

- Tests must run on Windows (library is Windows-only via `#![cfg(windows)]`)
- Unit tests are embedded in source files (e.g., `keys.rs:tests` module)
- Integration tests require administrator privileges for low-level keyboard hooks
- Benchmarks use criterion and generate HTML reports in `target/criterion/`

## Common Patterns

### Registering a Hotkey
```rust
let mut hkm = HotkeyManager::new();
hkm.register_hotkey(VKey::A, &[VKey::Control], || {
    println!("CTRL+A pressed");
}).unwrap();
hkm.event_loop(); // Blocks until stopped
```

### Using Builder Pattern
```rust
let hotkey = Hotkey::new()
    .trigger(VKey::B)
    .modifiers(&[VKey::LWin, VKey::Shift])
    .behavior(TriggerBehavior::StopPropagation)
    .action(|| println!("WIN+SHIFT+B pressed"));
```

### Keyboard Stealing Mode
```rust
hkm.steal_keyboard(|| {
    println!("Keyboard released");
});
// All keyboard input now routed to callback
hkm.free_keyboard(); // Manually release
```

### Pause/Resume
```rust
let pause_handler = hkm.pause();
pause_handler.pause(); // Pause hotkey processing
pause_handler.resume(); // Resume hotkey processing
```