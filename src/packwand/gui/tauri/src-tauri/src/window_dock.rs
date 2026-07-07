//! "Docked" launch mode: once the game process's window appears, move it
//! (position only — never resize, so the player's configured resolution is
//! untouched) to sit immediately beside the Packwand window. This is
//! deliberately *not* true window embedding (no HWND reparenting) — the
//! game stays a real, independent top-level OS window; only its position
//! changes. Windows-only for now: finding/moving a window by owning PID
//! needs a different API on macOS (Accessibility permissions) and is
//! effectively unsupported under Wayland on Linux, both out of scope here.

use std::time::Duration;

use tauri::AppHandle;

/// Polls for a window owned by `pid` and, once found, positions it flush to
/// the right of the main Packwand window. Best-effort: logs and gives up
/// after `timeout` rather than blocking the launch or failing it — a modded
/// instance can take a long time to reach its main menu.
pub fn dock_game_window(app: &AppHandle, pid: u32, timeout: Duration) {
    #[cfg(target_os = "windows")]
    {
        windows_impl::run(app, pid, timeout);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, pid, timeout);
        eprintln!(
            "dock_game_window: window docking is only implemented on Windows for now (pid {pid})"
        );
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use std::time::{Duration, Instant};

    use tauri::{AppHandle, Manager};
    use windows::core::BOOL;
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible, SetWindowPos, SWP_NOACTIVATE,
        SWP_NOSIZE, SWP_NOZORDER,
    };

    struct FindContext {
        pid: u32,
        found: Option<isize>,
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let ctx = unsafe { &mut *(lparam.0 as *mut FindContext) };
        let mut window_pid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut window_pid)) };
        if window_pid == ctx.pid && unsafe { IsWindowVisible(hwnd) }.as_bool() {
            ctx.found = Some(hwnd.0 as isize);
            return BOOL(0); // stop enumeration
        }
        BOOL(1) // continue
    }

    /// Finds a visible top-level window owned by `pid`, if any exists yet.
    fn find_window_for_pid(pid: u32) -> Option<isize> {
        let mut ctx = FindContext { pid, found: None };
        let _ = unsafe {
            EnumWindows(
                Some(enum_proc),
                LPARAM(std::ptr::addr_of_mut!(ctx) as isize),
            )
        };
        ctx.found
    }

    fn move_window(hwnd_raw: isize, x: i32, y: i32) -> bool {
        let hwnd = HWND(hwnd_raw as *mut _);
        unsafe {
            SetWindowPos(
                hwnd,
                None,
                x,
                y,
                0,
                0,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOSIZE,
            )
            .is_ok()
        }
    }

    pub fn run(app: &AppHandle, pid: u32, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(hwnd) = find_window_for_pid(pid) {
                position_beside_main_window(app, hwnd, pid);
                return;
            }
            if Instant::now() >= deadline {
                eprintln!(
                    "dock_game_window: timed out waiting for the game window to appear (pid {pid})"
                );
                return;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    fn position_beside_main_window(app: &AppHandle, hwnd: isize, pid: u32) {
        let Some(window) = app.get_webview_window("main") else {
            return;
        };
        let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) else {
            return;
        };
        let x = pos.x + size.width as i32;
        let y = pos.y;
        if !move_window(hwnd, x, y) {
            eprintln!("dock_game_window: failed to reposition the game window (pid {pid})");
        }
    }
}
