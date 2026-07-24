package net.nostalgica.modernica.common.mixin.perf.thread_unsafe_random;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

import net.minecraft.util.RandomSource;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.levelgen.RandomSupport;
import net.nostalgica.modernica.util.ThreadUnsafeRandom;

/** {@code Level#random} is only ever driven from its own tick thread; see {@link ThreadUnsafeRandom}
 * for why the thread-safe default is wasted here. */
@Mixin(Level.class)
abstract class LevelMixin {

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/util/RandomSource;create()Lnet/minecraft/util/RandomSource;"), require = 0, expect = 0)
    private RandomSource mfh$threadUnsafeLevelRandom() {
        return new ThreadUnsafeRandom(RandomSupport.generateUniqueSeed());
    }
}
