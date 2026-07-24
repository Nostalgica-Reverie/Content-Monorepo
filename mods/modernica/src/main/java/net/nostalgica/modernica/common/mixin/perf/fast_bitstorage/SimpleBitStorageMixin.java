package net.nostalgica.modernica.common.mixin.perf.fast_bitstorage;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.util.BitStorage;
import net.minecraft.util.SimpleBitStorage;
import net.nostalgica.modernica.util.IntegerUtil;

/**
 * Replaces {@link SimpleBitStorage}'s per-call division/modulo indexing with the constant-divisor
 * reciprocal-multiplication technique (see {@link IntegerUtil}), and drops the bounds validation on the
 * hot {@code get}/{@code set}/{@code getAndSet} path (all call sites already guarantee valid indices).
 * <p>
 * {@code entriesPerLong = 64 / bits} is a per-instance constant, so its division can be precomputed once
 * (in the constructor injection below) as a 20-bit reciprocal: {@code index * reciprocal} then splits
 * into a quotient (which long holds the entry, via a shift) and a remainder. That remainder is scaled by
 * {@code entriesPerLong * bits} in the same multiply-shift step, which yields the bit offset within that
 * long directly - the same trick doubles as "which long" and "where in that long", in exchange for
 * needing 20 bits of precision on top of the 12-bit index range, which is why entry counts above 4096
 * (guarded below) aren't supported: the intermediate product would overflow a 32-bit int.
 */
@Mixin(SimpleBitStorage.class)
abstract class SimpleBitStorageMixin implements BitStorage {

    @Shadow
    @Final
    private int bits;

    @Shadow
    @Final
    private long[] data;

    @Shadow
    @Final
    private long mask;

    @Shadow
    @Final
    private int size;

    private static final int PRECISION_BITS = 20;

    @Unique
    private int mfh$reciprocal;

    @Unique
    private int mfh$remainderScale;

    @Inject(method = "<init>(II[J)V", at = @At("RETURN"))
    private void mfh$precomputeReciprocal(CallbackInfo ci) {
        if (this.size > 4096) {
            throw new IllegalStateException("SimpleBitStorage size " + this.size + " exceeds the 4096-entry limit the fast_bitstorage reciprocal math assumes");
        }
        int entriesPerLong = 64 / this.bits;
        this.mfh$reciprocal = (int) IntegerUtil.getUnsignedDivisorMagic(entriesPerLong, PRECISION_BITS);
        this.mfh$remainderScale = entriesPerLong * this.bits;
    }

    @Unique
    private int mfh$bitOffset(int index) {
        int scaled = this.mfh$reciprocal * index;
        return ((scaled & 0xFFFFF) * this.mfh$remainderScale) >>> PRECISION_BITS;
    }

    @Unique
    private int mfh$longIndex(int index) {
        return (this.mfh$reciprocal * index) >>> PRECISION_BITS;
    }

    @Overwrite
    @Override
    public int get(int index) {
        int longIndex = this.mfh$longIndex(index);
        int bitOffset = this.mfh$bitOffset(index);
        return (int) (this.data[longIndex] >>> bitOffset & this.mask);
    }

    @Overwrite
    @Override
    public void set(int index, int value) {
        int longIndex = this.mfh$longIndex(index);
        int bitOffset = this.mfh$bitOffset(index);
        long[] dataArray = this.data;
        long mask = this.mask;
        long entry = dataArray[longIndex];
        dataArray[longIndex] = entry & ~(mask << bitOffset) | ((long) value & mask) << bitOffset;
    }

    @Overwrite
    @Override
    public int getAndSet(int index, int value) {
        int longIndex = this.mfh$longIndex(index);
        int bitOffset = this.mfh$bitOffset(index);
        long[] dataArray = this.data;
        long mask = this.mask;
        long entry = dataArray[longIndex];
        dataArray[longIndex] = entry & ~(mask << bitOffset) | ((long) value & mask) << bitOffset;
        return (int) (entry >>> bitOffset & mask);
    }
}
