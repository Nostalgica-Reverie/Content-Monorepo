/* pwproc backend: Win32 Job Objects (packwandc.md 5.1).
 *
 * This file is the reason pwproc exists. The Rust supervisor used to tear down
 * process trees with `taskkill /T /F /PID <pid>`: a string-formatted subprocess
 * that races PID reuse, can fail silently, and cannot guarantee the tree died.
 *
 * A job object with JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE replaces all of that.
 * The child is assigned to the job at adopt time; closing the job handle kills
 * every process in it -- including grandchildren the child spawned after we
 * stopped looking -- atomically, in-process, with no PID lookup to race.
 */

#include "packwandc/kernel/pwc_arch_proc.h"
#include "packwandc/kernel/pwc_error.h"
#include <windows.h>

pwc_status pwc_arch_proc_adopt(uint32_t pid, uintptr_t *out_native) {
    if (pid == 0u || out_native == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/win32", "pwc_arch_proc_adopt: zero pid or null out");
    }
    HANDLE job = CreateJobObjectW(nullptr, nullptr);
    if (job == nullptr) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/win32", "CreateJobObjectW failed", GetLastError());
    }
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION limits = {0};
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    if (SetInformationJobObject(job, JobObjectExtendedLimitInformation, &limits, sizeof(limits)) == 0) {
        /* Captured before CloseHandle, which overwrites the thread's last
         * error and would leave the record describing the cleanup rather than
         * the failure. Same pattern at every site below. */
        const DWORD code = GetLastError();
        (void) CloseHandle(job);
        return PWC_FAIL_PLATFORM(
            PWC_EIO, "arch/win32", "SetInformationJobObject(KILL_ON_JOB_CLOSE) failed", code);
    }
    HANDLE process =
        OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
    if (process == nullptr) {
        const DWORD code = GetLastError();
        (void) CloseHandle(job);
        return PWC_FAIL_PLATFORM(PWC_ENOENT, "arch/win32", "OpenProcess failed for the adopted pid", code);
    }
    const BOOL assigned = AssignProcessToJobObject(job, process);
    const DWORD assign_code = assigned == 0 ? GetLastError() : 0u;
    (void) CloseHandle(process);
    if (assigned == 0) {
        (void) CloseHandle(job);
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/win32", "AssignProcessToJobObject failed", assign_code);
    }
    *out_native = (uintptr_t) job;
    return PWC_OK;
}

pwc_status pwc_arch_proc_kill(uintptr_t native) {
    if (native == 0u) {
        return PWC_FAIL(PWC_EINVAL, "arch/win32", "pwc_arch_proc_kill: null job handle");
    }
    /* Closing the last handle to the job is the kill. No TerminateProcess call
     * and no enumeration: the kernel walks the job's membership itself, which
     * is what makes the tree guarantee hold. */
    if (CloseHandle((HANDLE) native) == 0) {
        return PWC_FAIL_PLATFORM(
            PWC_EIO, "arch/win32", "CloseHandle on the job object failed", GetLastError());
    }
    return PWC_OK;
}

pwc_status pwc_arch_proc_exists(uint32_t pid, uint32_t *out_alive) {
    if (pid == 0u || out_alive == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/win32", "pwc_arch_proc_exists: zero pid or null out");
    }
    HANDLE process = OpenProcess(SYNCHRONIZE, FALSE, pid);
    if (process == nullptr) {
        const DWORD code = GetLastError();
        /* Only "no such process" means dead. ERROR_ACCESS_DENIED means the
         * process is very much alive and merely not openable by this token --
         * reporting that as dead would let the tree-kill test pass against a
         * surviving tree, which is precisely the failure pwproc exists to make
         * impossible. Anything else is an unknown and must not be guessed at
         * either. */
        if (code == ERROR_INVALID_PARAMETER) {
            *out_alive = 0u;
            return PWC_OK;
        }
        if (code == ERROR_ACCESS_DENIED) {
            *out_alive = 1u;
            return PWC_OK;
        }
        return PWC_FAIL_PLATFORM(
            PWC_EIO, "arch/win32", "OpenProcess failed for an indeterminate reason", code);
    }
    /* A signalled process object is an exited one; still-running processes
     * leave the wait timing out. */
    const DWORD waited = WaitForSingleObject(process, 0u);
    if (waited == WAIT_FAILED) {
        const DWORD code = GetLastError();
        (void) CloseHandle(process);
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/win32", "WaitForSingleObject on the process failed", code);
    }
    *out_alive = waited == WAIT_TIMEOUT ? 1u : 0u;
    if (CloseHandle(process) == 0) {
        return PWC_FAIL_PLATFORM(PWC_EIO, "arch/win32", "CloseHandle on the process failed", GetLastError());
    }
    return PWC_OK;
}
