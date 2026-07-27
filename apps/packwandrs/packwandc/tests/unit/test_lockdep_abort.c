#include "packwandc/kernel/pwc_lockdep.h"
int main(void) {
    pwc_lockdep state;
    pwc_lockdep_init(&state);
    if (pwc_lockdep_acquire(&state, 2u) != PWC_OK) {
        return 2;
    }
    (void) pwc_lockdep_acquire(&state, 1u);
    return 3;
}
