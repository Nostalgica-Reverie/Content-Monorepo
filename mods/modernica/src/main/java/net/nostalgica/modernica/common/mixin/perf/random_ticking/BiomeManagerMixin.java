package net.nostalgica.modernica.common.mixin.perf.random_ticking;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import net.minecraft.world.level.biome.BiomeManager;

/** {@code 1024} is a power of two, so {@code Math.floorMod(seed >> 24, 1024)} is exactly
 * {@code (seed >> 24) & 1023} - same result, no division. */
@Mixin(BiomeManager.class)
abstract class BiomeManagerMixin {

    @Overwrite
    public static double getFiddle(long seed) {
        return (double) (((seed >> 24) & 1023) - 512) * (0.9 / 1024.0);
    }
}
