package net.nostalgica.modernica.common.mixin.perf.bad_optimizations.toast_culling;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.components.toasts.ToastManager;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.List;

/** Avoids extracting HUD render state when there are no visible toasts and music toasts are disabled. */
@ClientOnlyMixin
@Mixin(ToastManager.class)
abstract class ToastManagerMixin {
    @Shadow @Final private Minecraft minecraft;
    @Shadow @Final private List<?> visibleToasts;

    @Inject(method = "extractRenderState", at = @At("HEAD"), cancellable = true)
    private void modernica$skipEmptyToastExtraction(CallbackInfo ci) {
        if (visibleToasts.isEmpty() && !minecraft.options.musicToast().get().renderToast()) {
            ci.cancel();
        }
    }
}
