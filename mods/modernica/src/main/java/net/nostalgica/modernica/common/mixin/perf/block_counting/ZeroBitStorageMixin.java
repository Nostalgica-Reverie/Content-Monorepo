package net.nostalgica.modernica.common.mixin.perf.block_counting;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.shorts.ShortArrayList;
import net.minecraft.util.BitStorage;
import net.minecraft.util.ZeroBitStorage;
import net.nostalgica.modernica.perf.block_counting.BlockCountingBitStorage;

/** Every slot holds palette index 0 by construction. */
@Mixin(ZeroBitStorage.class)
abstract class ZeroBitStorageMixin implements BitStorage, BlockCountingBitStorage {

    @Shadow
    @Final
    private int size;

    @Override
    public final Int2ObjectOpenHashMap<ShortArrayList> mfh$countEntries() {
        int size = this.size;
        short[] indices = new short[size];
        for (int i = 0; i < size; i++) {
            indices[i] = (short) i;
        }
        Int2ObjectOpenHashMap<ShortArrayList> result = new Int2ObjectOpenHashMap<>(1);
        result.put(0, ShortArrayList.wrap(indices, size));
        return result;
    }
}
