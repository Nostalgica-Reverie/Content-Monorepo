package net.nostalgica.modernica.fabric.mixin.feature.force_close_loading_screen;

import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientCommonPacketListenerImpl;
import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.client.multiplayer.CommonListenerCookie;
import net.minecraft.network.Connection;
import net.nostalgica.modernica.feature.forcecloseloadingscreen.CapturedFrame;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Ported from kennytv's forcecloseloadingscreen (MIT). */
@Mixin(ClientPacketListener.class)
public abstract class ClientPacketListenerMixin extends ClientCommonPacketListenerImpl {

    protected ClientPacketListenerMixin(final Minecraft minecraft, final Connection connection, final CommonListenerCookie cookie) {
        super(minecraft, connection, cookie);
    }

    @Inject(method = "handleLogin", at = @At("HEAD"))
    public void handleLogin(final CallbackInfo ci) {
        if (this.minecraft.isSameThread() && this.minecraft.level != null) {
            CapturedFrame.captureLastFrame();
        }
    }
}
