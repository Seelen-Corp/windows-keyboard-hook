//! Provides a low-level implementation of a keyboard hook
//! using the Windows API. It captures keyboard events such as key presses
//! and releases, tracks the state of modifier keys, and communicates events
//! via channels to the rest of the application.

use crate::error::{Result, WHKError};
use crate::events::{EventLoopEvent, KeyAction, KeyboardInputEvent};
use crate::state::KEYBOARD_STATE;
use crate::{log_on_dev, VKey};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::{HANDLE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Power::{
    RegisterSuspendResumeNotification, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS,
};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
    TranslateMessage, DEVICE_NOTIFY_CALLBACK, KBDLLHOOKSTRUCT, MSG, PBT_APMRESUMEAUTOMATIC,
    PBT_APMRESUMESUSPEND, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN,
    WM_SYSKEYUP,
};

/// Timeout for blocking key events, measured in milliseconds.
const TIMEOUT: Duration = Duration::from_millis(250);

/// Unassigned Virtual Key code used to suppress Windows Key events.
const SILENT_KEY: VIRTUAL_KEY = VIRTUAL_KEY(0xE8);

static STARTED: AtomicBool = AtomicBool::new(false);
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);

/// Starts the keyboard hook thread.
pub fn start() -> Result<()> {
    if STARTED.load(Ordering::Relaxed) {
        return Err(WHKError::AlreadyStarted);
    }

    // Create/clear keyboard state
    KEYBOARD_STATE.lock().unwrap().clear();

    let (tx, rx) = crossbeam_channel::unbounded::<bool>();
    thread::spawn(move || unsafe {
        let Ok(_keyborad_handle) =
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0)
        else {
            tx.send(false).unwrap();
            return;
        };

        let mut recipient = DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
            Callback: Some(power_sleep_resume_proc),
            ..Default::default()
        };
        let Ok(_suspend_handle) = RegisterSuspendResumeNotification(
            HANDLE(&mut recipient as *mut _ as _),
            DEVICE_NOTIFY_CALLBACK,
        ) else {
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
        Ok(())
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

/// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registersuspendresumenotification
/// https://learn.microsoft.com/en-us/windows/win32/api/powrprof/nc-powrprof-device_notify_callback_routine
unsafe extern "system" fn power_sleep_resume_proc(
    _context: *const core::ffi::c_void,
    event: u32,
    _setting: *const core::ffi::c_void,
) -> u32 {
    log_on_dev!("Received power event: {event}");
    match event {
        PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => {
            KEYBOARD_STATE.lock().unwrap().request_syncronization();
        }
        _ => {}
    }
    0
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
            let state = {
                let mut state = KEYBOARD_STATE.lock().unwrap();
                state.keydown(vk_code);
                state.clone()
            };
            log_on_dev!("{state:?}");

            // Clear the actions channel of any previous action
            let response_rx = KeyAction::reciever();
            while response_rx.try_recv().is_ok() {}

            let is_win_pressed = state.is_win_pressed();
            EventLoopEvent::Keyboard(KeyboardInputEvent::KeyDown {
                key: vk_code.into(),
                state,
            })
            .send();

            // Wait for response on how to handle event
            if let Ok(action) = response_rx.recv_timeout(TIMEOUT) {
                match action {
                    KeyAction::Block => {
                        if is_win_pressed {
                            // to avoid windows alone key opening the start menu,
                            // we need to send a silent key.
                            send_silent_key();
                        }
                        return LRESULT(1);
                    }
                    KeyAction::Allow => {}
                }
            }
        }
        WM_KEYUP | WM_SYSKEYUP => {
            let state = {
                let mut state = KEYBOARD_STATE.lock().unwrap();
                state.keyup(vk_code);
                state.clone()
            };
            log_on_dev!("{state:?}");

            // Clear the actions channel of any previous action
            let response_rx = KeyAction::reciever();
            while response_rx.try_recv().is_ok() {}

            EventLoopEvent::Keyboard(KeyboardInputEvent::KeyUp {
                key: vk_code.into(),
                state,
            })
            .send();

            // we can't block key up events as this can cause issues on applications with inifinite key down states
            if let Ok(action) = response_rx.recv_timeout(TIMEOUT) {
                if action == KeyAction::Block && VKey::from_vk_code(vk_code).is_windows_key() {
                    // sending silent key will cause the windows keyup event to be ignored
                    send_silent_key();
                }
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
