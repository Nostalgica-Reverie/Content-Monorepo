package net.nostalgica.modernica.common.mixin.perf.fast_bitstorage;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import net.minecraft.util.BitStorage;
import net.minecraft.util.ZeroBitStorage;

/** A zero-bit storage has exactly one possible value (0) for every entry by construction; drops the
 * bounds validation vanilla still runs on every {@code get}/{@code set} call despite that. */
@Mixin(ZeroBitStorage.class)
abstract class ZeroBitStorageMixin implements BitStorage {

    @Overwrite
    @Override
    public int get(int index) {
        return 0;
    }

    @Overwrite
    @Override
    public void set(int index, int value) {
    }

    @Overwrite
    @Override
    public int getAndSet(int index, int value) {
        return 0;
    }
}
