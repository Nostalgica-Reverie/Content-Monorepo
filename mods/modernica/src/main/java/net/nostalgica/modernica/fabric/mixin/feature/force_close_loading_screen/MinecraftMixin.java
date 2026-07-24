package net.nostalgica.modernica.fabric.mixin.feature.force_close_loading_screen;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.Screen;
import net.nostalgica.modernica.feature.forcecloseloadingscreen.CapturedFrame;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Ported from kennytv's forcecloseloadingscreen (MIT). */
@Mixin(Minecraft.class)
public abstract class MinecraftMixin {

    @Inject(at = @At("HEAD"), method = "clearClientLevel")
    public void clearClientLevel(final Screen screen, final CallbackInfo ci) {
        CapturedFrame.captureLastFrame();
    }

    @Inject(at = @At("HEAD"), method = "disconnect(Lnet/minecraft/client/gui/screens/Screen;ZZ)V")
    public void disconnect(final Screen screen, final boolean bl, final boolean bl2, final CallbackInfo ci) {
        CapturedFrame.initialJoin = true;
        CapturedFrame.clearCapturedTexture();
    }
}
