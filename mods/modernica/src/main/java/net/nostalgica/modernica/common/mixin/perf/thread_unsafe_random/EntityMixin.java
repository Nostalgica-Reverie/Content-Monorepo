package net.nostalgica.modernica.common.mixin.perf.thread_unsafe_random;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

import net.minecraft.util.RandomSource;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.levelgen.RandomSupport;
import net.nostalgica.modernica.util.ThreadUnsafeRandom;

/** Every {@link Entity} owns a private {@code random} field that only that entity's own tick logic
 * ever touches; see {@link ThreadUnsafeRandom} for why the thread-safe default is wasted here. */
@Mixin(Entity.class)
abstract class EntityMixin {

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/util/RandomSource;create()Lnet/minecraft/util/RandomSource;"), require = 0, expect = 0)
    private RandomSource mfh$threadUnsafeEntityRandom() {
        return new ThreadUnsafeRandom(RandomSupport.generateUniqueSeed());
    }
}
