package net.nostalgica.modernica.common.mixin.feature.measure_time;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.screens.Screen;
import net.nostalgica.modernica.ModernicaClient;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;


@Mixin(Minecraft.class)
@ClientOnlyMixin
public class MinecraftMixin {
    // TODO re-add datapack reload time measurement
    @Shadow public Screen screen;

    @Inject(method = "tick", at = @At("HEAD"))
    private void onClientTick(CallbackInfo ci) {
        if(this.screen == null && ModernicaClient.INSTANCE != null) {
            ModernicaClient.INSTANCE.onGameLaunchFinish();
        }
    }

    @Inject(method = "doWorldLoad", at = @At("HEAD"))
    private void recordWorldLoadStart(CallbackInfo ci) {
        ModernicaClient.worldLoadStartTime = System.nanoTime();
    }
}
