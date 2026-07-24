package net.nostalgica.modernica.common.mixin.perf.fast_palette;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

import net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap;
import net.minecraft.world.level.chunk.HashMapPalette;
import net.minecraft.world.level.chunk.Palette;
import net.nostalgica.modernica.perf.fast_palette.FastPalette;
import net.nostalgica.modernica.perf.fast_palette.FastPaletteData;

/** {@link HashMapPalette} doesn't store the raw array itself - it delegates to its backing
 * {@link CrudeIncrementalIntIdentityHashBiMap}, which is fast-pathed separately by
 * {@link CrudeIncrementalIntIdentityHashBiMapMixin}. */
@Mixin(HashMapPalette.class)
abstract class HashMapPaletteMixin<T> implements Palette<T>, FastPalette<T> {

    @Shadow
    @Final
    private CrudeIncrementalIntIdentityHashBiMap<T> values;

    @Override
    @SuppressWarnings("unchecked")
    public final T[] mfh$getRawPalette(FastPaletteData<T> owner) {
        return ((FastPalette<T>) this.values).mfh$getRawPalette(owner);
    }
}
