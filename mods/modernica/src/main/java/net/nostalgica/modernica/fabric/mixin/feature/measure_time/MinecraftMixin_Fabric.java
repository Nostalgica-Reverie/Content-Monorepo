package net.nostalgica.modernica.fabric.mixin.feature.measure_time;

import net.minecraft.client.Minecraft;
import net.nostalgica.modernica.ModernicaClient;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Minecraft.class)
@ClientOnlyMixin
public class MinecraftMixin_Fabric {
    @Inject(method = "doWorldLoad", at = @At("HEAD"))
    private void recordWorldLoadStart(CallbackInfo ci) {
        ModernicaClient.worldLoadStartTime = System.nanoTime();
    }
}
