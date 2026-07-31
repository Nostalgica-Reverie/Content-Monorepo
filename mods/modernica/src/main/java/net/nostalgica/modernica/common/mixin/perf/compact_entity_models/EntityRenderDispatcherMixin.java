package net.nostalgica.modernica.common.mixin.perf.compact_entity_models;

import net.minecraft.client.renderer.entity.EntityRenderDispatcher;
import net.minecraft.server.packs.resources.ResourceManager;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import net.nostalgica.modernica.perf.CompactEntityModelCache;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Releases cube models owned by the previous resource reload generation. */
@Mixin(EntityRenderDispatcher.class)
@ClientOnlyMixin
public class EntityRenderDispatcherMixin {
    @Inject(method = "onResourceManagerReload", at = @At("HEAD"))
    private void modernica$clearCubeCache(ResourceManager resourceManager, CallbackInfo ci) {
        CompactEntityModelCache.clear();
    }
}
