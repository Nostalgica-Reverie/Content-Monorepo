package net.nostalgica.modernica.common.mixin.perf.block_counting;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.shorts.ShortArrayList;
import net.minecraft.util.BitStorage;
import net.minecraft.util.SimpleBitStorage;
import net.nostalgica.modernica.perf.block_counting.BlockCountingBitStorage;

/** Decodes each backing {@code long} into its packed entries once per word (matching how the storage is
 * actually laid out) instead of calling {@code get(int)} - with its own division per call - once per
 * slot, 4096 times, just to build a histogram. */
@Mixin(SimpleBitStorage.class)
abstract class SimpleBitStorageMixin implements BitStorage, BlockCountingBitStorage {

    @Shadow
    @Final
    private long[] data;

    @Shadow
    @Final
    private int valuesPerLong;

    @Shadow
    @Final
    private int bits;

    @Shadow
    @Final
    private int size;

    @Override
    public final Int2ObjectOpenHashMap<ShortArrayList> mfh$countEntries() {
        int valuesPerLong = this.valuesPerLong;
        int bits = this.bits;
        long mask = (1L << bits) - 1L;
        int size = this.size;

        Int2ObjectOpenHashMap<ShortArrayList> result = new Int2ObjectOpenHashMap<>(Math.min(1 << bits, 64));
        int index = 0;
        for (long word : this.data) {
            int packed = 0;
            do {
                int paletteIndex = (int) (word & mask);
                word >>>= bits;
                packed++;

                result.computeIfAbsent(paletteIndex, key -> new ShortArrayList(64)).add((short) index);
                index++;
            } while (packed < valuesPerLong && index < size);
        }
        return result;
    }
}
