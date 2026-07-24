package net.nostalgica.modernica.common.mixin.perf.dynamic_entity_renderers;

import net.minecraft.client.renderer.entity.EntityRenderDispatcher;
import net.minecraft.client.renderer.entity.EntityRenderer;
import net.minecraft.client.renderer.entity.state.EntityRenderState;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EntityType;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import net.nostalgica.modernica.entity.EntityRendererMap;
import org.objectweb.asm.Opcodes;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.Map;

@Mixin(EntityRenderDispatcher.class)
@ClientOnlyMixin
public class EntityRenderDispatcherMixin {
    @Shadow private Map<EntityType<?>, EntityRenderer<?, ?>> renderers;

    private EntityRendererMap mfix$dynamicRenderers;

    @Inject(method = "getRenderer(Lnet/minecraft/world/entity/Entity;)Lnet/minecraft/client/renderer/entity/EntityRenderer;", at = @At("RETURN"), cancellable = true, require = 0)
    private <T extends Entity> void checkNullnessFromEntity(T entity, CallbackInfoReturnable<EntityRenderer<? super T, ?>> cir) {
        // apparently some mods yeet the renderers map and cause issues
        if(cir.getReturnValue() == null && this.mfix$dynamicRenderers != null)
            cir.setReturnValue((EntityRenderer<? super T, ?>)mfix$dynamicRenderers.get(entity.getType()));
    }

    @Inject(method = "getRenderer(Lnet/minecraft/client/renderer/entity/state/EntityRenderState;)Lnet/minecraft/client/renderer/entity/EntityRenderer;", at = @At("RETURN"), cancellable = true, require = 0)
    private <S extends EntityRenderState> void checkNullnessFromState(S renderState, CallbackInfoReturnable<EntityRenderer<?, ? super S>> cir) {
        // 26.1+ uses state-based renderer lookup; pull the type from render state.
        if(cir.getReturnValue() == null && this.mfix$dynamicRenderers != null)
            cir.setReturnValue((EntityRenderer<?, ? super S>)mfix$dynamicRenderers.get(renderState.entityType));
    }

    @Redirect(method = "onResourceManagerReload", at = @At(value = "FIELD", opcode = Opcodes.PUTFIELD, target = "Lnet/minecraft/client/renderer/entity/EntityRenderDispatcher;renderers:Ljava/util/Map;"))
    private void setRendererField(EntityRenderDispatcher instance, Map<EntityType<?>, EntityRenderer<?, ?>> incomingMap) {
        this.renderers = incomingMap;
        this.mfix$dynamicRenderers = (EntityRendererMap)incomingMap;
    }
}
