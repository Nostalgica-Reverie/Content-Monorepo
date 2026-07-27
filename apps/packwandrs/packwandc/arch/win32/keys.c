#include "packwandc/kernel/pwc_arch_keys.h"
#include <windows.h>
#include <wincred.h>
#include <string.h>

enum { PWC_KEYS_MAX_SECRET = 2560 };
static const WCHAR pwc_keys_target[] = L"packwand/msa-refresh-token";

pwc_status pwc_arch_keys_save(const uint8_t *secret, size_t secret_len) {
    if (secret_len > PWC_KEYS_MAX_SECRET || secret_len > UINT32_MAX) {
        return PWC_EOVERFLOW;
    }
    WCHAR target[] = L"packwand/msa-refresh-token";
    uint8_t local[PWC_KEYS_MAX_SECRET] = {0};
    memcpy(local, secret, secret_len);
    CREDENTIALW credential = {0};
    credential.Type = CRED_TYPE_GENERIC;
    credential.TargetName = target;
    credential.CredentialBlobSize = (DWORD) secret_len;
    credential.CredentialBlob = local;
    credential.Persist = CRED_PERSIST_LOCAL_MACHINE;
    const pwc_status status = CredWriteW(&credential, 0u) != 0 ? PWC_OK : PWC_EIO;
    (void) SecureZeroMemory(local, secret_len);
    return status;
}

pwc_status pwc_arch_keys_load(uint8_t *buffer, size_t capacity, size_t *out_len) {
    PCREDENTIALW credential = nullptr;
    if (CredReadW(pwc_keys_target, CRED_TYPE_GENERIC, 0u, &credential) == 0) {
        return GetLastError() == ERROR_NOT_FOUND ? PWC_ENOENT : PWC_EIO;
    }
    const size_t length = credential->CredentialBlobSize;
    if (length > capacity) {
        *out_len = length;
        CredFree(credential);
        return PWC_EOVERFLOW;
    }
    memcpy(buffer, credential->CredentialBlob, length);
    *out_len = length;
    (void) SecureZeroMemory(credential->CredentialBlob, credential->CredentialBlobSize);
    CredFree(credential);
    return PWC_OK;
}

pwc_status pwc_arch_keys_clear(void) {
    if (CredDeleteW(pwc_keys_target, CRED_TYPE_GENERIC, 0u) != 0 || GetLastError() == ERROR_NOT_FOUND) {
        return PWC_OK;
    }
    return PWC_EIO;
}
