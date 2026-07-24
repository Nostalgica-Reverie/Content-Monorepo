package net.nostalgica.modernica.fabric.mixin.feature.force_close_loading_screen;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.LoadingOverlay;
import org.objectweb.asm.Opcodes;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** Ported from kennytv's forcecloseloadingscreen (MIT) */
@Mixin(LoadingOverlay.class)
public abstract class LoadingOverlayMixin {

    @Shadow
    @Final
    private Minecraft minecraft;

    @Shadow
    private long fadeOutStart;

    @Inject(at = @At("TAIL"), method = "extractRenderState")
    public void extractRenderState(final GuiGraphicsExtractor graphics, final int mouseX, final int mouseY, final float a, final CallbackInfo ci) {
        if (this.fadeOutStart != -1) {
            //STONECUTTER_FCLS_SET_OVERLAY
            this.minecraft.setOverlay(null);
        }
    }

    @Redirect(method = "extractRenderState", at = @At(value = "FIELD", target = "Lnet/minecraft/client/gui/screens/LoadingOverlay;fadeIn:Z", opcode = Opcodes.GETFIELD))
    private boolean fadeIn(final LoadingOverlay instance) {
        return false;
    }

    @Inject(at = @At("RETURN"), method = "isReadyToFadeOut", cancellable = true)
    public void isReadyToFadeOut(final CallbackInfoReturnable<Boolean> cir) {
        cir.setReturnValue(true);
    }
}
