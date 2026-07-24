package net.nostalgica.modernica.common.mixin.perf.faster_item_rendering;

import net.minecraft.client.renderer.item.ItemStackRenderState;
import net.minecraft.world.item.ItemDisplayContext;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(ItemStackRenderState.class)
@ClientOnlyMixin
public interface ItemStackRenderStateAccessor {
    @Accessor
    ItemDisplayContext getDisplayContext();
}
