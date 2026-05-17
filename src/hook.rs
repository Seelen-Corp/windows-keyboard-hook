//! Provides a low-level implementation of a keyboard hook
//! using the Windows API. It captures keyboard events such as key presses
//! and releases, tracks the state of modifier keys, and communicates events
//! via channels to the rest of the application.

use std::{
    cell::RefCell,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    thread,
};

use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::{
        Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
            KEYEVENTF_KEYUP, VIRTUAL_KEY,
        },
        WindowsAndMessaging::{
            CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
            TranslateMessage, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_QUIT,
            WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

use crate::{
    error::{Result, WHKError},
    events::{KeyAction, KeyboardInputEvent},
    log_on_dev,
    manager::HotkeyManager,
    state::KeyboardState,
    VKey,
};

/// Unassigned Virtual Key code used to suppress Windows Key events.
const SILENT_KEY: VIRTUAL_KEY = VIRTUAL_KEY(0xE8);

static STARTED: AtomicBool = AtomicBool::new(false);
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);

thread_local! {
    /// Per-thread keyboard state — only accessed from the hook thread.
    static KEYBOARD_STATE: RefCell<KeyboardState> = {
        let mut s = KeyboardState::new();
        s.request_syncronization();
        RefCell::new(s)
    };
}

/// Starts the keyboard hook thread and returns its join handle.
pub fn start() -> Result<thread::JoinHandle<()>> {
    if STARTED.load(Ordering::Relaxed) {
        return Err(WHKError::AlreadyStarted);
    }

    let (tx, rx) = crossbeam_channel::unbounded::<bool>();
    let handle = thread::spawn(move || unsafe {
        let Ok(_keyborad_handle) =
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0)
        else {
            tx.send(false).unwrap();
            return;
        };

        tx.send(true).unwrap();
        HOOK_THREAD_ID.store(GetCurrentThreadId(), Ordering::Relaxed);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });

    if rx.recv()? {
        STARTED.store(true, Ordering::Relaxed);
        Ok(handle)
    } else {
        Err(WHKError::StartupFailed)
    }
}

pub fn stop() {
    let thread_id = HOOK_THREAD_ID.load(Ordering::Relaxed);
    if !STARTED.load(Ordering::Relaxed) || thread_id == 0 {
        return;
    }
    unsafe {
        let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM::default(), LPARAM::default());
    }
}

/// Hook procedure for handling keyboard events.
/// https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc
unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let next = || CallNextHookEx(None, code, wparam, lparam);
    if code < 0 {
        return next();
    }

    let event_type = wparam.0 as u32;
    let Some(event_data) = (lparam.0 as *const KBDLLHOOKSTRUCT).as_ref() else {
        return next();
    };

    let vk_code = event_data.vkCode as u16;
    if vk_code == SILENT_KEY.0 {
        return next();
    }

    match event_type {
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            let state = KEYBOARD_STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                state.keydown(vk_code);
                state.clone()
            });
            log_on_dev!("{state:?}");

            let is_win_pressed = state.is_win_pressed();
            let action = HotkeyManager::process_keyboard_event(KeyboardInputEvent::KeyDown {
                key: vk_code.into(),
                state,
            });

            if action == KeyAction::Block {
                if is_win_pressed {
                    // to avoid windows alone key opening the start menu,
                    // we need to send a silent key.
                    send_silent_key();
                }
                return LRESULT(1);
            }
        }
        WM_KEYUP | WM_SYSKEYUP => {
            let state = KEYBOARD_STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                state.keyup(vk_code);
                state.clone()
            });
            log_on_dev!("{state:?}");

            let action = HotkeyManager::process_keyboard_event(KeyboardInputEvent::KeyUp {
                key: vk_code.into(),
                state,
            });

            // we can't block key up events as this can cause issues on applications with inifinite key down states
            if action == KeyAction::Block && VKey::from_vk_code(vk_code).is_windows_key() {
                // sending silent key will cause the windows keyup event to be ignored
                send_silent_key();
            }
        }
        _ => {}
    };

    next()
}

/// Sends a keydown and keyup event for Unassigned Virtual Key 0xE8.
unsafe fn send_silent_key() {
    let inputs = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: SILENT_KEY,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: SILENT_KEY,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];
    SendInput(&inputs, size_of::<INPUT>() as i32);
}
