package net.nostalgica.modernica.fabric.mixin.feature.force_close_loading_screen;

import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.client.MouseHandler;
import net.minecraft.client.input.InputQuirks;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Ported from kennytv's forcecloseloadingscreen (MIT). */
@Mixin(MouseHandler.class)
public class MouseHandlerMixin {

    @Shadow
    @Final
    private Minecraft minecraft;

    @Shadow
    private boolean mouseGrabbed;

    @Inject(method = "grabMouse", at = @At(value = "HEAD", target = "Lnet/minecraft/client/Minecraft;setScreen(Lnet/minecraft/client/gui/screens/Screen;)V"))
    public void fixSprint(CallbackInfo ci) {
        if (this.minecraft.isWindowActive() && this.mouseGrabbed && !InputQuirks.REPLACE_CTRL_KEY_WITH_CMD_KEY) {
            KeyMapping.setAll();
        }
    }
}
