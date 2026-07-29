#include "packwandc/uapi/pwc_raw_input.h"
#include "packwandc/kernel/pwc_error.h"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

static INIT_ONCE pwc_raw_once = INIT_ONCE_STATIC_INIT;
static CRITICAL_SECTION pwc_raw_lock;
static HWND pwc_raw_window = NULL;
static WNDPROC pwc_raw_previous = NULL;
static uint32_t pwc_raw_head = 0u;
static uint32_t pwc_raw_tail = 0u;
static uint64_t pwc_raw_lost = 0u;
static pwc_raw_input_event pwc_raw_queue[PWC_RAW_INPUT_QUEUE_CAPACITY];

static BOOL CALLBACK pwc_raw_init_lock(PINIT_ONCE once, PVOID parameter, PVOID *context) {
    (void) once;
    (void) parameter;
    (void) context;
    InitializeCriticalSection(&pwc_raw_lock);
    return TRUE;
}

static void pwc_raw_ensure_lock(void) {
    (void) InitOnceExecuteOnce(&pwc_raw_once, pwc_raw_init_lock, NULL, NULL);
}

static bool pwc_raw_is_focused(HWND window) { return GetForegroundWindow() == GetAncestor(window, GA_ROOT); }

static void pwc_raw_push(const pwc_raw_input_event *event) {
    EnterCriticalSection(&pwc_raw_lock);
    const uint32_t next = (pwc_raw_head + 1u) % PWC_RAW_INPUT_QUEUE_CAPACITY;
    if (next == pwc_raw_tail) {
        ++pwc_raw_lost;
    } else {
        pwc_raw_queue[pwc_raw_head] = *event;
        pwc_raw_head = next;
    }
    LeaveCriticalSection(&pwc_raw_lock);
}

static void pwc_raw_decode(LPARAM input_handle) {
    RAWINPUT input = {0};
    UINT size = (UINT) sizeof(input);
    if (GetRawInputData((HRAWINPUT) input_handle, RID_INPUT, &input, &size, (UINT) sizeof(RAWINPUTHEADER)) ==
        (UINT) -1) {
        return;
    }

    pwc_raw_input_event event = {
        .struct_size = (uint32_t) sizeof(event),
        .timestamp_ms = (uint32_t) GetMessageTime(),
    };
    if (input.header.dwType == RIM_TYPEKEYBOARD) {
        event.kind = PWC_RAW_INPUT_KEYBOARD;
        event.make_code = input.data.keyboard.MakeCode;
        event.flags = input.data.keyboard.Flags;
        event.virtual_key = input.data.keyboard.VKey;
    } else if (input.header.dwType == RIM_TYPEMOUSE) {
        event.kind = PWC_RAW_INPUT_MOUSE;
        event.button_flags = input.data.mouse.usButtonFlags;
        event.delta_x = input.data.mouse.lLastX;
        event.delta_y = input.data.mouse.lLastY;
        if ((event.button_flags & RI_MOUSE_WHEEL) != 0u) {
            event.wheel_delta = (int16_t) input.data.mouse.usButtonData;
        }
    } else {
        return;
    }
    pwc_raw_push(&event);
}

static LRESULT CALLBACK pwc_raw_window_proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    WNDPROC previous = NULL;
    bool active = false;
    EnterCriticalSection(&pwc_raw_lock);
    previous = pwc_raw_previous;
    active = pwc_raw_window == window;
    LeaveCriticalSection(&pwc_raw_lock);

    if (active && message == WM_INPUT && pwc_raw_is_focused(window)) {
        pwc_raw_decode(lparam);
    }
    return previous != NULL ? CallWindowProcW(previous, window, message, wparam, lparam)
                            : DefWindowProcW(window, message, wparam, lparam);
}

static void pwc_raw_unregister(void) {
    const RAWINPUTDEVICE devices[] = {
        {.usUsagePage = 0x01u, .usUsage = 0x02u, .dwFlags = RIDEV_REMOVE, .hwndTarget = NULL},
        {.usUsagePage = 0x01u, .usUsage = 0x06u, .dwFlags = RIDEV_REMOVE, .hwndTarget = NULL},
    };
    (void) RegisterRawInputDevices(devices, 2u, (UINT) sizeof(devices[0]));
}

pwc_status pwc_raw_input_start(uintptr_t native_window) {
    HWND const window = (HWND) native_window;
    if (window == NULL || !IsWindow(window)) {
        return PWC_FAIL(PWC_EINVAL, "raw-input", "invalid Packwand window handle");
    }

    pwc_raw_ensure_lock();
    EnterCriticalSection(&pwc_raw_lock);
    if (pwc_raw_window != NULL) {
        LeaveCriticalSection(&pwc_raw_lock);
        return PWC_FAIL(PWC_EPERM, "raw-input", "raw input is already active");
    }

    const RAWINPUTDEVICE devices[] = {
        {.usUsagePage = 0x01u, .usUsage = 0x02u, .dwFlags = 0u, .hwndTarget = window},
        {.usUsagePage = 0x01u, .usUsage = 0x06u, .dwFlags = 0u, .hwndTarget = window},
    };
    if (!RegisterRawInputDevices(devices, 2u, (UINT) sizeof(devices[0]))) {
        LeaveCriticalSection(&pwc_raw_lock);
        return PWC_FAIL(PWC_EIO, "raw-input", "RegisterRawInputDevices failed");
    }

    SetLastError(0);
    const LONG_PTR previous = SetWindowLongPtrW(window, GWLP_WNDPROC, (LONG_PTR) pwc_raw_window_proc);
    if (previous == 0 && GetLastError() != 0) {
        pwc_raw_unregister();
        LeaveCriticalSection(&pwc_raw_lock);
        return PWC_FAIL(PWC_EIO, "raw-input", "could not attach raw input window procedure");
    }

    pwc_raw_previous = (WNDPROC) previous;
    pwc_raw_window = window;
    pwc_raw_head = 0u;
    pwc_raw_tail = 0u;
    pwc_raw_lost = 0u;
    LeaveCriticalSection(&pwc_raw_lock);
    return PWC_OK;
}

void pwc_raw_input_stop(void) {
    pwc_raw_ensure_lock();
    EnterCriticalSection(&pwc_raw_lock);
    if (pwc_raw_window != NULL) {
        (void) SetWindowLongPtrW(pwc_raw_window, GWLP_WNDPROC, (LONG_PTR) pwc_raw_previous);
        pwc_raw_unregister();
        pwc_raw_window = NULL;
        pwc_raw_previous = NULL;
        pwc_raw_head = 0u;
        pwc_raw_tail = 0u;
    }
    LeaveCriticalSection(&pwc_raw_lock);
}

pwc_status pwc_raw_input_read(pwc_raw_input_event *out) {
    if (out == NULL) {
        return PWC_FAIL(PWC_EINVAL, "raw-input", "null output event");
    }

    pwc_raw_ensure_lock();
    EnterCriticalSection(&pwc_raw_lock);
    if (pwc_raw_window == NULL) {
        LeaveCriticalSection(&pwc_raw_lock);
        return PWC_FAIL(PWC_ECANCELED, "raw-input", "raw input is not active");
    }
    if (pwc_raw_tail == pwc_raw_head) {
        LeaveCriticalSection(&pwc_raw_lock);
        return PWC_EAGAIN;
    }
    *out = pwc_raw_queue[pwc_raw_tail];
    pwc_raw_tail = (pwc_raw_tail + 1u) % PWC_RAW_INPUT_QUEUE_CAPACITY;
    LeaveCriticalSection(&pwc_raw_lock);
    return PWC_OK;
}

pwc_status pwc_raw_input_dropped(uint64_t *out) {
    if (out == NULL) {
        return PWC_FAIL(PWC_EINVAL, "raw-input", "null dropped count");
    }

    pwc_raw_ensure_lock();
    EnterCriticalSection(&pwc_raw_lock);
    *out = pwc_raw_lost;
    LeaveCriticalSection(&pwc_raw_lock);
    return PWC_OK;
}
