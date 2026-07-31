package net.nostalgica.modernica.common.mixin.perf.bad_optimizations.particle_culling;

import net.minecraft.client.particle.Particle;
import net.minecraft.client.particle.ParticleEngine;
import net.minecraft.client.particle.ParticleRenderType;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Map;
import java.util.Queue;

/** Avoids preparing particle render state for an empty particle engine. */
@ClientOnlyMixin
@Mixin(ParticleEngine.class)
abstract class ParticleEngineMixin {
    @Shadow @Final private Map<ParticleRenderType, Queue<Particle>> particles;

    @Inject(method = "extract", at = @At("HEAD"), cancellable = true)
    private void modernica$skipEmptyParticleExtraction(CallbackInfo ci) {
        if (particles.isEmpty()) {
            ci.cancel();
        }
    }
}
