package net.nostalgica.modernica.fabric.mixin.feature.force_close_loading_screen;

import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.LevelLoadingScreen;
import net.nostalgica.modernica.feature.forcecloseloadingscreen.CapturedFrame;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Ported from kennytv's forcecloseloadingscreen (MIT). */
@Mixin(LevelLoadingScreen.class)
public abstract class LevelLoadingScreenMixin {

    @Inject(at = @At("HEAD"), method = "extractRenderState", cancellable = true)
    public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a, final CallbackInfo ci) {
        if (!CapturedFrame.initialJoin) {
            ci.cancel();
        }
    }

    @Inject(at = @At("HEAD"), method = "extractBackground", cancellable = true)
    public void extractBackground(final CallbackInfo ci) {
        if (!CapturedFrame.initialJoin) {
            ci.cancel();
        }
    }

    @Inject(at = @At("HEAD"), method = "onClose")
    public void onClose(final CallbackInfo ci) {
        CapturedFrame.initialJoin = false;
    }
}
