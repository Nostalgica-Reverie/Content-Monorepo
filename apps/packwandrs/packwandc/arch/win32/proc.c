#include "packwandc/kernel/pwc_arch_proc.h"
#include <windows.h>

pwc_status pwc_arch_proc_adopt(uint32_t pid, uintptr_t *out_native) {
    if (pid == 0u || out_native == nullptr) {
        return PWC_EINVAL;
    }
    HANDLE job = CreateJobObjectW(nullptr, nullptr);
    if (job == nullptr) {
        return PWC_EIO;
    }
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION limits = {0};
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    if (SetInformationJobObject(job, JobObjectExtendedLimitInformation, &limits, sizeof(limits)) == 0) {
        (void) CloseHandle(job);
        return PWC_EIO;
    }
    HANDLE process =
        OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
    if (process == nullptr) {
        (void) CloseHandle(job);
        return PWC_ENOENT;
    }
    const BOOL assigned = AssignProcessToJobObject(job, process);
    (void) CloseHandle(process);
    if (assigned == 0) {
        (void) CloseHandle(job);
        return PWC_EIO;
    }
    *out_native = (uintptr_t) job;
    return PWC_OK;
}

pwc_status pwc_arch_proc_kill(uintptr_t native) {
    if (native == 0u) {
        return PWC_EINVAL;
    }
    return CloseHandle((HANDLE) native) != 0 ? PWC_OK : PWC_EIO;
}
pwc_status pwc_arch_proc_exists(uint32_t pid, uint32_t *out_alive) {
    if (pid == 0u || out_alive == nullptr) {
        return PWC_EINVAL;
    }
    HANDLE process = OpenProcess(SYNCHRONIZE, FALSE, pid);
    if (process == nullptr) {
        *out_alive = 0u;
        return PWC_OK;
    }
    *out_alive = WaitForSingleObject(process, 0u) == WAIT_TIMEOUT ? 1u : 0u;
    return CloseHandle(process) != 0 ? PWC_OK : PWC_EIO;
}
