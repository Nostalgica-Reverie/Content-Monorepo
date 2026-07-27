/* pwkeys backend for Linux: NOT YET IMPLEMENTED, deliberately.
 *
 * The spec's answer for this platform is the Secret Service API spoken over a
 * hand-rolled D-Bus client (packwandc.md 5.2 and 8.4): no libsecret, because
 * linking a third-party library is exactly what this layer exists to remove,
 * while speaking a wire protocol over a Unix socket is fine.
 *
 * That work is real and is not done. What it requires, so the next person does
 * not have to rediscover it:
 *
 *   1. Connect to $DBUS_SESSION_BUS_ADDRESS -- both the `unix:path=` and the
 *      `unix:abstract=` forms.
 *   2. The SASL text handshake: a leading NUL byte, `AUTH EXTERNAL <uid in
 *      hex>`, read `OK <guid>`, then `BEGIN`.
 *   3. A D-Bus message marshaller. This is the bulk of it and the part that
 *      has to be exactly right: per-type alignment, arrays as a 4-byte length
 *      followed by padding to the element's alignment, structs aligned to 8,
 *      variants carrying an inline signature, and the header-field array.
 *   4. Four calls against org.freedesktop.secrets: OpenSession("plain"),
 *      CreateItem on the default collection, SearchItems + GetSecret, and
 *      Item.Delete -- the last two of which can return a prompt path that has
 *      to be driven to completion.
 *
 * WHY IT IS A STUB RATHER THAN A FIRST ATTEMPT
 *
 * A marshaller that is subtly wrong does not fail loudly. It writes a
 * malformed body, and the reply is either an error that looks like a
 * permissions problem or -- worse -- a successful store of a corrupted secret.
 * This file cannot be compiled, let alone run, on the machine it was written
 * on (packwandc.md 8.2: CI is Linux and cross-builds Windows; the reference
 * dev machine is Windows and has no Linux toolchain), so a first attempt here
 * would be unreviewable binary-protocol code guarded only by a code review.
 * An honest PWC_ENOSYS that names what is missing is worth more than that.
 *
 * The failure is loud by construction: PwcKeyStore surfaces the recorded
 * detail through packwandc::Error, so a Linux user gets this message rather
 * than a silent failure to persist their token.
 *
 * Until this lands, Linux hosts keep the refresh token in memory only, which
 * means re-authenticating once per process start. Nothing above the TokenStore
 * trait needs to change when it does land.
 */

#include "packwandc/kernel/pwc_arch_keys.h"
#include "packwandc/kernel/pwc_error.h"

static const char pwc_keys_unimplemented[] =
    "pwkeys has no Linux backend yet: Secret Service over D-Bus is unimplemented "
    "(see arch/linux/keys.c)";

pwc_status pwc_arch_keys_save(const uint8_t *secret, size_t secret_len) {
    (void) secret;
    (void) secret_len;
    return PWC_FAIL(PWC_ENOSYS, "arch/linux", pwc_keys_unimplemented);
}

pwc_status pwc_arch_keys_load(uint8_t *buffer, size_t capacity, size_t *out_len) {
    (void) buffer;
    (void) capacity;
    if (out_len != nullptr) {
        *out_len = 0u;
    }
    return PWC_FAIL(PWC_ENOSYS, "arch/linux", pwc_keys_unimplemented);
}

pwc_status pwc_arch_keys_clear(void) { return PWC_FAIL(PWC_ENOSYS, "arch/linux", pwc_keys_unimplemented); }
