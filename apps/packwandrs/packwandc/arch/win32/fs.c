#include "packwandc/kernel/pwc_arch_fs.h"
#include "packwandc/kernel/pwc_error.h"
#include <windows.h>
#include <wchar.h>

enum { PWC_FS_PATH_CAPACITY = 32768 };

static pwc_status pwc_fs_utf8_to_wide(const uint8_t *input, size_t length, WCHAR *out, size_t capacity) {
    if (length > INT_MAX || capacity > INT_MAX) {
        return PWC_EOVERFLOW;
    }
    const int written = MultiByteToWideChar(
        CP_UTF8, MB_ERR_INVALID_CHARS, (LPCCH) input, (int) length, out, (int) (capacity - 1u));
    if (written <= 0) {
        return PWC_EINVAL;
    }
    out[written] = L'\0';
    return PWC_OK;
}

static pwc_status
pwc_fs_build_path(const uint8_t *root, size_t root_len, const uint8_t *path, size_t path_len, WCHAR *out) {
    WCHAR root_wide[PWC_FS_PATH_CAPACITY] = {0};
    WCHAR path_wide[PWC_FS_PATH_CAPACITY] = {0};
    PWC_TRY(pwc_fs_utf8_to_wide(root, root_len, root_wide, PWC_FS_PATH_CAPACITY));
    PWC_TRY(pwc_fs_utf8_to_wide(path, path_len, path_wide, PWC_FS_PATH_CAPACITY));
    const DWORD root_written = GetFullPathNameW(root_wide, PWC_FS_PATH_CAPACITY, out, nullptr);
    if (root_written == 0u || root_written >= PWC_FS_PATH_CAPACITY) {
        return PWC_EOVERFLOW;
    }
    size_t used = wcslen(out);
    if (used + 1u + wcslen(path_wide) >= PWC_FS_PATH_CAPACITY) {
        return PWC_EOVERFLOW;
    }
    if (used > 0u && out[used - 1u] != L'\\') {
        out[used] = L'\\';
        ++used;
        out[used] = L'\0';
    }
    for (size_t index = 0u; path_wide[index] != L'\0'; ++index) {
        if (path_wide[index] == L'/') {
            path_wide[index] = L'\\';
        }
    }
    if (wcscat_s(out, PWC_FS_PATH_CAPACITY, path_wide) != 0) {
        return PWC_EOVERFLOW;
    }
    return PWC_OK;
}

static pwc_status pwc_fs_reject_reparse(WCHAR *path, bool include_final) {
    const size_t length = wcslen(path);
    for (size_t index = 4u; index <= length; ++index) {
        if (path[index] != L'\\' && !(include_final && index == length)) {
            continue;
        }
        const WCHAR saved = path[index];
        path[index] = L'\0';
        const DWORD attributes = GetFileAttributesW(path);
        path[index] = saved;
        if (attributes != INVALID_FILE_ATTRIBUTES && (attributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0u) {
            return PWC_EPERM;
        }
    }
    return PWC_OK;
}

pwc_status pwc_arch_fs_read(const uint8_t *root,
                            size_t root_len,
                            const uint8_t *path,
                            size_t path_len,
                            uint8_t *buffer,
                            size_t capacity,
                            size_t *out_len) {
    WCHAR target[PWC_FS_PATH_CAPACITY] = {0};
    PWC_TRY(pwc_fs_build_path(root, root_len, path, path_len, target));
    PWC_TRY(pwc_fs_reject_reparse(target, true));
    HANDLE file = CreateFileW(target,
                              GENERIC_READ,
                              FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                              nullptr,
                              OPEN_EXISTING,
                              FILE_ATTRIBUTE_NORMAL,
                              nullptr);
    if (file == INVALID_HANDLE_VALUE) {
        return GetLastError() == ERROR_FILE_NOT_FOUND ? PWC_ENOENT : PWC_EIO;
    }
    LARGE_INTEGER size = {0};
    if (GetFileSizeEx(file, &size) == 0 || size.QuadPart < 0) {
        (void) CloseHandle(file);
        return PWC_EIO;
    }
    *out_len = (size_t) size.QuadPart;
    if ((uint64_t) size.QuadPart > capacity) {
        (void) CloseHandle(file);
        return PWC_EOVERFLOW;
    }
    size_t total = 0u;
    while (total < *out_len) {
        const size_t remaining = *out_len - total;
        const DWORD chunk = remaining > UINT32_MAX ? UINT32_MAX : (DWORD) remaining;
        DWORD read = 0u;
        if (ReadFile(file, &buffer[total], chunk, &read, nullptr) == 0 || read == 0u) {
            (void) CloseHandle(file);
            return PWC_EIO;
        }
        total += read;
    }
    return CloseHandle(file) != 0 ? PWC_OK : PWC_EIO;
}

pwc_status pwc_arch_fs_atomic_write(const uint8_t *root,
                                    size_t root_len,
                                    const uint8_t *path,
                                    size_t path_len,
                                    const uint8_t *content,
                                    size_t content_len) {
    WCHAR target[PWC_FS_PATH_CAPACITY] = {0};
    WCHAR parent[PWC_FS_PATH_CAPACITY] = {0};
    WCHAR temporary[MAX_PATH] = {0};
    PWC_TRY(pwc_fs_build_path(root, root_len, path, path_len, target));
    if (wcscpy_s(parent, PWC_FS_PATH_CAPACITY, target) != 0) {
        return PWC_EOVERFLOW;
    }
    WCHAR *const separator = wcsrchr(parent, L'\\');
    if (separator == nullptr) {
        return PWC_EINVAL;
    }
    *separator = L'\0';
    PWC_TRY(pwc_fs_reject_reparse(parent, true));
    if (GetTempFileNameW(parent, L"pwc", 0u, temporary) == 0u) {
        return PWC_EIO;
    }
    HANDLE file = CreateFileW(
        temporary, GENERIC_WRITE, 0u, nullptr, TRUNCATE_EXISTING, FILE_ATTRIBUTE_TEMPORARY, nullptr);
    if (file == INVALID_HANDLE_VALUE) {
        (void) DeleteFileW(temporary);
        return PWC_EIO;
    }
    size_t total = 0u;
    while (total < content_len) {
        const size_t remaining = content_len - total;
        const DWORD chunk = remaining > UINT32_MAX ? UINT32_MAX : (DWORD) remaining;
        DWORD written = 0u;
        if (WriteFile(file, &content[total], chunk, &written, nullptr) == 0 || written == 0u) {
            (void) CloseHandle(file);
            (void) DeleteFileW(temporary);
            return PWC_EIO;
        }
        total += written;
    }
    const BOOL flushed = FlushFileBuffers(file);
    const BOOL closed = CloseHandle(file);
    if (flushed == 0 || closed == 0 ||
        MoveFileExW(temporary, target, MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) == 0) {
        (void) DeleteFileW(temporary);
        return PWC_EIO;
    }
    return PWC_OK;
}

pwc_status pwc_arch_fs_watch_open(const uint8_t *root, size_t root_len, uintptr_t *out_native) {
    WCHAR root_wide[PWC_FS_PATH_CAPACITY] = {0};
    if (out_native == nullptr) {
        return PWC_EINVAL;
    }
    PWC_TRY(pwc_fs_utf8_to_wide(root, root_len, root_wide, PWC_FS_PATH_CAPACITY));
    HANDLE directory = CreateFileW(root_wide,
                                   FILE_LIST_DIRECTORY,
                                   FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                                   nullptr,
                                   OPEN_EXISTING,
                                   FILE_FLAG_BACKUP_SEMANTICS,
                                   nullptr);
    if (directory == INVALID_HANDLE_VALUE) {
        return PWC_ENOENT;
    }
    *out_native = (uintptr_t) directory;
    return PWC_OK;
}

pwc_status pwc_arch_fs_watch_read(uintptr_t native, size_t *out_events) {
    uint8_t changes[4096] = {0};
    DWORD bytes = 0u;
    const DWORD filter = FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_DIR_NAME |
                         FILE_NOTIFY_CHANGE_SIZE | FILE_NOTIFY_CHANGE_LAST_WRITE |
                         FILE_NOTIFY_CHANGE_CREATION;
    if (native == 0u || out_events == nullptr) {
        return PWC_EINVAL;
    }
    if (ReadDirectoryChangesW(
            (HANDLE) native, changes, sizeof(changes), TRUE, filter, &bytes, nullptr, nullptr) == 0) {
        return PWC_EIO;
    }
    *out_events = bytes == 0u ? 0u : 1u;
    return bytes == 0u ? PWC_EAGAIN : PWC_OK;
}

pwc_status pwc_arch_fs_watch_close(uintptr_t native) {
    if (native == 0u) {
        return PWC_EINVAL;
    }

    /* CancelIoEx BEFORE CloseHandle, and this ordering is load-bearing.
     *
     * pwc_arch_fs_watch_read calls ReadDirectoryChangesW synchronously, so a
     * poller thread is parked inside the kernel on this handle. Closing a
     * handle with I/O outstanding does not reliably complete that I/O -- the
     * observed behaviour is that the poller stays blocked, and the join in
     * pwc_sched_shutdown then waits on it forever. That hangs process exit,
     * which is the worst possible way for this to fail.
     *
     * CancelIoEx with a NULL OVERLAPPED cancels every outstanding request on
     * the handle regardless of which thread issued it, which is exactly the
     * cross-thread cancellation needed here.
     *
     * ERROR_NOT_FOUND means there was nothing in flight -- the common case when
     * no stream was ever started -- and is not a failure. */
    if (CancelIoEx((HANDLE) native, nullptr) == 0) {
        const DWORD code = GetLastError();
        if (code != ERROR_NOT_FOUND) {
            (void) CloseHandle((HANDLE) native);
            return PWC_FAIL_PLATFORM(PWC_EIO, "arch/win32", "CancelIoEx on the watch handle failed", code);
        }
    }

    if (CloseHandle((HANDLE) native) == 0) {
        return PWC_FAIL_PLATFORM(
            PWC_EIO, "arch/win32", "CloseHandle on the watch handle failed", GetLastError());
    }
    return PWC_OK;
}
