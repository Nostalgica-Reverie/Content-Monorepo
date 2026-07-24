package net.nostalgica.modernica.common.mixin.perf.fast_palette;

import org.spongepowered.asm.mixin.Mixin;

import net.minecraft.world.level.chunk.Palette;
import net.nostalgica.modernica.perf.fast_palette.FastPalette;

/** Applies the {@link FastPalette} default (opt-out, i.e. "not fast-pathed") to every palette type. */
@Mixin(Palette.class)
interface PaletteMixin<T> extends FastPalette<T> {
}
