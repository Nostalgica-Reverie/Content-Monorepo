package net.nostalgica.modernica.common.mixin.perf.bad_optimizations.debug_renderer_culling;

import net.minecraft.client.renderer.debug.DebugRenderer;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.List;

/** Vanilla's debug renderer has no work when the server has not supplied any debug renderers. */
@ClientOnlyMixin
@Mixin(DebugRenderer.class)
abstract class DebugRendererMixin {
    @Shadow @Final private List<DebugRenderer.SimpleDebugRenderer> renderers;

    @Inject(method = "emitGizmos", at = @At("HEAD"), cancellable = true)
    private void modernica$skipEmptyDebugRenderer(CallbackInfo ci) {
        if (renderers.isEmpty()) {
            ci.cancel();
        }
    }
}
