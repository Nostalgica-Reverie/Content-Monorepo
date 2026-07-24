package net.nostalgica.modernica.util;

/**
 * Constant-divisor reciprocal helper, per the public technique described at
 * https://lemire.me/blog/2019/02/08/faster-remainders-when-the-divisor-is-a-constant-beating-compilers-and-libdivide/ :
 * for a fixed {@code divisor}, {@code (index * magic) >>> precisionBits} recovers {@code index / divisor}
 * without an actual division, as long as {@code divisor * maxIndex} stays within {@code 2^precisionBits}.
 */
public final class IntegerUtil {
    private IntegerUtil() {}

    /** {@code magic = ceil(2^precisionBits / divisor)}. */
    public static long getUnsignedDivisorMagic(long divisor, int precisionBits) {
        return (((1L << precisionBits) - 1) / divisor) + 1;
    }

    /**
     * Full-width (64-bit) variant: {@code magic = ceil(2^64 / divisor)}, computed via unsigned division
     * since {@code 2^64} itself doesn't fit in a signed long. Pair with {@link #unsignedFloorDiv}.
     * <p>
     * Unlike the reduced-precision form above (which needs {@code divisor * maxDividend < 2^precisionBits}
     * to stay accurate), this is correct for any unsigned {@code dividend} short of the extreme high end
     * of the 64-bit range (within a few ULPs of {@code 2^64} itself) - the rounding error introduced by
     * the ceiling is a fraction smaller than {@code 1/divisor} of one part in {@code 2^64}, so for any
     * dividend many orders of magnitude below {@code 2^64} (which every realistic use here is) that error
     * can never accumulate enough to flip the floor result.
     */
    public static long getUnsignedDivisorMagic64(long divisor) {
        return Long.divideUnsigned(-1L, divisor) + 1;
    }

    /** {@code floor(dividend / divisor)} for unsigned {@code dividend}, given {@code magic} from
     * {@link #getUnsignedDivisorMagic64}, via the high 64 bits of the full 128-bit product. */
    public static long unsignedFloorDiv(long dividend, long magic) {
        return Math.unsignedMultiplyHigh(dividend, magic);
    }
}
