use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{CommandResult, SerializableError};

const IDLE_DELAY: Duration = Duration::from_millis(50);
const ACTIVE_DELAY: Duration = Duration::from_millis(2);
const MAX_BATCH: usize = 256;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawInputPayload {
    pub kind: &'static str,
    pub timestamp_ms: u32,
    pub make_code: u16,
    pub flags: u16,
    pub virtual_key: u16,
    pub button_flags: u16,
    pub delta_x: i32,
    pub delta_y: i32,
    pub wheel_delta: i16,
}

pub struct RawInputState {
    enabled: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
}

impl Drop for RawInputState {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.enabled.store(false, Ordering::Release);
        native::stop();
    }
}

pub fn start(app: &AppHandle) -> CommandResult<()> {
    let enabled = Arc::new(AtomicBool::new(false));
    let shutdown = Arc::new(AtomicBool::new(false));
    app.manage(RawInputState {
        enabled: Arc::clone(&enabled),
        shutdown: Arc::clone(&shutdown),
    });
    let pump_app = app.clone();
    std::thread::Builder::new()
        .name("packwand-raw-input".into())
        .spawn(move || pump(pump_app, enabled, shutdown))
        .map_err(|error| SerializableError::new("raw_input_thread", error.to_string()))?;
    Ok(())
}

fn pump(app: AppHandle, enabled: Arc<AtomicBool>, shutdown: Arc<AtomicBool>) {
    let mut last_dropped = 0;
    while !shutdown.load(Ordering::Acquire) {
        if !enabled.load(Ordering::Acquire) {
            std::thread::sleep(IDLE_DELAY);
            continue;
        }
        let batch = native::read_batch(MAX_BATCH);
        if !batch.is_empty() {
            let _ = app.emit("raw-input:batch", &batch);
        }
        let dropped = native::dropped();
        if dropped > last_dropped {
            let _ = app.emit("raw-input:dropped", dropped - last_dropped);
            last_dropped = dropped;
        }
        std::thread::sleep(ACTIVE_DELAY);
    }
}

pub fn set_enabled(app: &AppHandle, enabled: bool) -> CommandResult<bool> {
    let state = app.state::<RawInputState>();
    if state.enabled.load(Ordering::Acquire) == enabled {
        return Ok(enabled);
    }
    if enabled {
        #[cfg(windows)]
        {
            let window = app
                .get_webview_window("main")
                .ok_or_else(|| SerializableError::new("raw_input", "main window is unavailable"))?;
            let hwnd = window
                .hwnd()
                .map_err(|error| SerializableError::new("raw_input", error.to_string()))?;
            native::start(hwnd.0 as usize)
                .map_err(|error| SerializableError::new("raw_input", error))?;
        }
        #[cfg(not(windows))]
        return Err(SerializableError::new(
            "raw_input_unavailable",
            "Raw Input is currently available on Windows only",
        ));
    } else {
        state.enabled.store(false, Ordering::Release);
        native::stop();
    }
    state.enabled.store(enabled, Ordering::Release);
    Ok(enabled)
}

#[tauri::command]
pub fn raw_input_set_enabled(app: AppHandle, enabled: bool) -> CommandResult<bool> {
    set_enabled(&app, enabled)
}

#[cfg(not(windows))]
mod native {
    use super::RawInputPayload;
    pub fn stop() {}
    pub fn read_batch(_: usize) -> Vec<RawInputPayload> {
        Vec::new()
    }
    pub fn dropped() -> u64 {
        0
    }
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod native {
    use std::collections::VecDeque;
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::sync::{LazyLock, Mutex};

    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::Input::{
        GetRawInputData, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTDEVICE_FLAGS, RID_INPUT,
        RIDEV_REMOVE, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE, RegisterRawInputDevices,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, GA_ROOT, GWLP_WNDPROC, GetAncestor, GetForegroundWindow,
        GetMessageTime, IsWindow, SetWindowLongPtrW, WM_INPUT, WNDPROC,
    };

    use super::RawInputPayload;

    const CAPACITY: usize = 2048;
    const RI_MOUSE_WHEEL: u16 = 0x0400;

    #[derive(Default)]
    struct State {
        window: isize,
        previous: isize,
        queue: VecDeque<RawInputPayload>,
        dropped: u64,
    }
    static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::default()));

    pub fn start(native_window: usize) -> Result<(), String> {
        let window = HWND(native_window as *mut c_void);
        if window.0.is_null() || unsafe { !IsWindow(Some(window)).as_bool() } {
            return Err("invalid Packwand window handle".into());
        }
        let mut state = STATE.lock().map_err(|_| "raw input lock was poisoned")?;
        if state.window != 0 {
            return Err("raw input is already active".into());
        }
        let devices = devices(window, RAWINPUTDEVICE_FLAGS(0));
        unsafe { RegisterRawInputDevices(&devices, size_of::<RAWINPUTDEVICE>() as u32) }
            .map_err(|error| format!("RegisterRawInputDevices failed: {error}"))?;
        let previous =
            unsafe { SetWindowLongPtrW(window, GWLP_WNDPROC, window_proc as *const () as isize) };
        if previous == 0 {
            unregister();
            return Err("could not attach raw input window procedure".into());
        }
        state.window = window.0 as isize;
        state.previous = previous;
        state.queue.clear();
        state.dropped = 0;
        Ok(())
    }

    pub fn stop() {
        if let Ok(mut state) = STATE.lock()
            && state.window != 0
        {
            let window = HWND(state.window as *mut c_void);
            unsafe {
                SetWindowLongPtrW(window, GWLP_WNDPROC, state.previous);
            }
            unregister();
            state.window = 0;
            state.previous = 0;
            state.queue.clear();
        }
    }

    pub fn read_batch(max: usize) -> Vec<RawInputPayload> {
        STATE
            .lock()
            .map(|mut state| {
                let count = max.min(state.queue.len());
                state.queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    pub fn dropped() -> u64 {
        STATE.lock().map(|state| state.dropped).unwrap_or(0)
    }

    fn devices(window: HWND, flags: RAWINPUTDEVICE_FLAGS) -> [RAWINPUTDEVICE; 2] {
        [
            RAWINPUTDEVICE {
                usUsagePage: 1,
                usUsage: 2,
                dwFlags: flags,
                hwndTarget: window,
            },
            RAWINPUTDEVICE {
                usUsagePage: 1,
                usUsage: 6,
                dwFlags: flags,
                hwndTarget: window,
            },
        ]
    }

    fn unregister() {
        let _ = unsafe {
            RegisterRawInputDevices(
                &devices(HWND::default(), RIDEV_REMOVE),
                size_of::<RAWINPUTDEVICE>() as u32,
            )
        };
    }

    unsafe extern "system" fn window_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let (active, previous) = STATE
            .lock()
            .map(|state| (state.window == window.0 as isize, state.previous))
            .unwrap_or_default();
        if active
            && message == WM_INPUT
            && unsafe { GetForegroundWindow() == GetAncestor(window, GA_ROOT) }
        {
            unsafe {
                decode(lparam);
            }
        }
        if previous == 0 {
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        } else {
            let procedure: WNDPROC = unsafe { std::mem::transmute(previous) };
            unsafe { CallWindowProcW(procedure, window, message, wparam, lparam) }
        }
    }

    unsafe fn decode(lparam: LPARAM) {
        let mut input = RAWINPUT::default();
        let mut size = size_of::<RAWINPUT>() as u32;
        let read = unsafe {
            GetRawInputData(
                HRAWINPUT(lparam.0 as *mut c_void),
                RID_INPUT,
                Some((&mut input as *mut RAWINPUT).cast()),
                &mut size,
                size_of::<windows::Win32::UI::Input::RAWINPUTHEADER>() as u32,
            )
        };
        if read == u32::MAX {
            return;
        }
        let timestamp_ms = unsafe { GetMessageTime() } as u32;
        let payload = if input.header.dwType == RIM_TYPEKEYBOARD.0 {
            let keyboard = unsafe { input.data.keyboard };
            RawInputPayload {
                kind: "keyboard",
                timestamp_ms,
                make_code: keyboard.MakeCode,
                flags: keyboard.Flags,
                virtual_key: keyboard.VKey,
                button_flags: 0,
                delta_x: 0,
                delta_y: 0,
                wheel_delta: 0,
            }
        } else if input.header.dwType == RIM_TYPEMOUSE.0 {
            let mouse = unsafe { input.data.mouse };
            let buttons = unsafe { mouse.Anonymous.Anonymous };
            RawInputPayload {
                kind: "mouse",
                timestamp_ms,
                make_code: 0,
                flags: 0,
                virtual_key: 0,
                button_flags: buttons.usButtonFlags,
                delta_x: mouse.lLastX,
                delta_y: mouse.lLastY,
                wheel_delta: if buttons.usButtonFlags & RI_MOUSE_WHEEL != 0 {
                    buttons.usButtonData as i16
                } else {
                    0
                },
            }
        } else {
            return;
        };
        if let Ok(mut state) = STATE.lock() {
            if state.queue.len() == CAPACITY {
                state.dropped += 1;
            } else {
                state.queue.push_back(payload);
            }
        }
    }
}
