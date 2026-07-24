package net.nostalgica.modernica.common.mixin.core;

import net.minecraft.client.multiplayer.ClientPacketListener;
import net.nostalgica.modernica.ModernicaClient;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(value = ClientPacketListener.class, priority = 1500)
@ClientOnlyMixin
public class ClientPlayNetHandlerMixin {
    @Inject(method = "handleUpdateRecipes", at = @At("RETURN"))
    private void signalRecipes(CallbackInfo ci) {
        ModernicaClient.INSTANCE.onRecipesUpdated();
    }
}
