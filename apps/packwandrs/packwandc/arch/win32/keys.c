/* pwkeys backend: Windows Credential Manager.
 *
 * Every failure here records the Win32 error code. A bare PWC_EIO out of a
 * credential call is close to undiagnosable -- "the vault said no" covers a
 * locked profile, a group-policy denial, an oversized blob and a corrupt
 * credential set -- and GetLastError() is the only thing separating them. It is
 * also clobbered by the next Win32 call, so every site below captures it before
 * doing any cleanup.
 */

#include "packwandc/kernel/pwc_arch_keys.h"
#include "packwandc/kernel/pwc_error.h"
#include <windows.h>
#include <wincred.h>
#include <string.h>

enum { PWC_KEYS_MAX_SECRET = 2560 };
static const WCHAR pwc_keys_target[] = L"packwand/msa-refresh-token";

pwc_status pwc_arch_keys_save(const uint8_t *secret, size_t secret_len) {
    /* Validated again here even though the module already did: there is no
     * trusted caller: tests and future modules reach arch directly. */
    if (secret == nullptr || secret_len == 0u) {
        return PWC_FAIL(PWC_EINVAL, "arch/win32", "pwc_arch_keys_save: empty secret");
    }
    if (secret_len > PWC_KEYS_MAX_SECRET || secret_len > UINT32_MAX) {
        return PWC_FAIL(PWC_EOVERFLOW, "arch/win32", "secret exceeds the credential blob cap");
    }
    WCHAR target[] = L"packwand/msa-refresh-token";
    /* Staged through a local because CREDENTIALW takes a non-const blob
     * pointer. Zeroised below on every path. */
    uint8_t local[PWC_KEYS_MAX_SECRET] = {0};
    memcpy(local, secret, secret_len);
    CREDENTIALW credential = {0};
    credential.Type = CRED_TYPE_GENERIC;
    credential.TargetName = target;
    credential.CredentialBlobSize = (DWORD) secret_len;
    credential.CredentialBlob = local;
    credential.Persist = CRED_PERSIST_LOCAL_MACHINE;

    pwc_status status = PWC_OK;
    if (CredWriteW(&credential, 0u) == 0) {
        status =
            PWC_FAIL_PLATFORM(PWC_EIO, "arch/win32", "CredWriteW rejected the credential", GetLastError());
    }
    /* Whole buffer, not just secret_len: cheap, and it cannot then drift out of
     * step with the write above. */
    (void) SecureZeroMemory(local, sizeof(local));
    return status;
}

pwc_status pwc_arch_keys_load(uint8_t *buffer, size_t capacity, size_t *out_len) {
    if (buffer == nullptr || capacity == 0u || out_len == nullptr) {
        return PWC_FAIL(PWC_EINVAL, "arch/win32", "pwc_arch_keys_load: null buffer or out_len");
    }
    PCREDENTIALW credential = nullptr;
    if (CredReadW(pwc_keys_target, CRED_TYPE_GENERIC, 0u, &credential) == 0) {
        const DWORD code = GetLastError();
        return code == ERROR_NOT_FOUND
                   ? PWC_FAIL_PLATFORM(PWC_ENOENT, "arch/win32", "no stored packwand credential", code)
                   : PWC_FAIL_PLATFORM(PWC_EIO, "arch/win32", "CredReadW failed", code);
    }
    const size_t length = credential->CredentialBlobSize;
    if (length > capacity) {
        /* Report the size needed so the caller can retry with a large enough
         * buffer instead of guessing. */
        *out_len = length;
        CredFree(credential);
        return PWC_FAIL(PWC_EOVERFLOW, "arch/win32", "stored credential is larger than the buffer");
    }
    memcpy(buffer, credential->CredentialBlob, length);
    *out_len = length;
    (void) SecureZeroMemory(credential->CredentialBlob, credential->CredentialBlobSize);
    CredFree(credential);
    return PWC_OK;
}

pwc_status pwc_arch_keys_clear(void) {
    if (CredDeleteW(pwc_keys_target, CRED_TYPE_GENERIC, 0u) != 0) {
        return PWC_OK;
    }
    const DWORD code = GetLastError();
    /* Deleting an absent credential reaches the caller's desired end state, so
     * clear() is idempotent rather than failing on a second call. */
    if (code == ERROR_NOT_FOUND) {
        return PWC_OK;
    }
    return PWC_FAIL_PLATFORM(PWC_EIO, "arch/win32", "CredDeleteW failed", code);
}
