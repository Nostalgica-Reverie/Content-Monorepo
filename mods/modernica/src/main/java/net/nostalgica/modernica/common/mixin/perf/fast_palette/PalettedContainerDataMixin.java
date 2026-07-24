package net.nostalgica.modernica.common.mixin.perf.fast_palette;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;

import net.minecraft.world.level.chunk.PalettedContainer;
import net.nostalgica.modernica.perf.fast_palette.FastPaletteData;

@Mixin(PalettedContainer.Data.class)
abstract class PalettedContainerDataMixin<T> implements FastPaletteData<T> {

    @Unique
    private T[] mfh$cachedPalette;

    @Override
    public final T[] mfh$getCachedPalette() {
        return this.mfh$cachedPalette;
    }

    @Override
    public final void mfh$setCachedPalette(T[] palette) {
        this.mfh$cachedPalette = palette;
    }
}
