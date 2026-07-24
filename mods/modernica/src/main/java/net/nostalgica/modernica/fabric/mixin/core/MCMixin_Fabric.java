package net.nostalgica.modernica.fabric.mixin.core;

import net.minecraft.client.Minecraft;
import net.nostalgica.modernica.ModernicaClient;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Minecraft.class)
@ClientOnlyMixin
public class MCMixin_Fabric {
    @Inject(method = "tick", at = @At("RETURN"))
    private void onRenderTickEnd(CallbackInfo ci) {
        ModernicaClient.INSTANCE.onRenderTickEnd();
    }
}
