package net.nostalgica.modernica.network.util;

/**
 * Independently written (not derived from any third-party mod's source - see
 * {@link net.nostalgica.modernica.common.mixin.perf.network_enhancements.VarLongMixin}'s
 * class doc for context) companion to Krypton's own {@code VarIntUtil}, extending the same
 * lookup-table technique from 32-bit VarInts to 64-bit VarLongs.
 * <p>
 * Maps VarLong byte sizes to a lookup table corresponding to the number of leading zero bits in the
 * long, from zero to 64.
 */
public class VarLongUtil {
    private static final int[] VARLONG_EXACT_BYTE_LENGTHS = new int[65];

    static {
        for (int i = 0; i <= 64; ++i) {
            VARLONG_EXACT_BYTE_LENGTHS[i] = (int) Math.ceil((63d - (i - 1)) / 7d);
        }
        VARLONG_EXACT_BYTE_LENGTHS[64] = 1; // Special case for 0.
    }

    public static int getVarLongLength(long value) {
        return VARLONG_EXACT_BYTE_LENGTHS[Long.numberOfLeadingZeros(value)];
    }
}
