package net.nostalgica.modernica.common.mixin.bugfix.end_island_overflow;

import com.llamalad7.mixinextras.sugar.Local;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

/**
 * Fixes <a href="https://bugs.mojang.com/browse/MC-159283">MC-159283</a>: the End's island falloff
 * function computes {@code x*x + z*z} in 32-bit float, which loses precision (and can wrap sign) once
 * a player travels far enough from the origin, distorting island generation at extreme coordinates.
 * Redirects the {@code Mth.sqrt} call to compute the sum of squares in {@code long} first.
 * <p>
 * {@code EndIslandDensityFunction} is a package-protected nested record, so it has to be targeted by
 * name rather than by class literal.
 */
@Mixin(targets = "net/minecraft/world/level/levelgen/DensityFunctions$EndIslandDensityFunction")
abstract class EndIslandDensityFunctionMixin {

    @Redirect(
            method = "getHeightValue",
            at = @At(value = "INVOKE", target = "Lnet/minecraft/util/Mth;sqrt(F)F")
    )
    private static float mfh$fixOverflow(float unused, @Local(ordinal = 0, argsOnly = true) int x, @Local(ordinal = 1, argsOnly = true) int z) {
        return (float) Math.sqrt((double) ((long) x * (long) x + (long) z * (long) z));
    }
}
