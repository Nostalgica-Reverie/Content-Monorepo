package net.nostalgica.modernica.common.mixin.perf.fast_palette;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

import net.minecraft.world.level.chunk.LinearPalette;
import net.minecraft.world.level.chunk.Palette;
import net.nostalgica.modernica.perf.fast_palette.FastPalette;
import net.nostalgica.modernica.perf.fast_palette.FastPaletteData;

/** {@link LinearPalette} already stores its id-to-value mapping as a flat array - nothing to build. */
@Mixin(LinearPalette.class)
abstract class LinearPaletteMixin<T> implements Palette<T>, FastPalette<T> {

    @Shadow
    @Final
    private T[] values;

    @Override
    public final T[] mfh$getRawPalette(FastPaletteData<T> owner) {
        return this.values;
    }
}
