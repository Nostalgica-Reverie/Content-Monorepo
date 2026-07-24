package net.nostalgica.modernica.common.mixin.bugfix.singleplayer_keepalive_kick;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

import net.minecraft.network.chat.Component;
import net.minecraft.server.network.ServerCommonPacketListenerImpl;

/**
 * Vanilla's keepalive timeout kick applies to the singleplayer host too, even though there's no real
 * network between an integrated server and its own client - a debugger pause, a slow chunk-save, or
 * anything else that briefly stalls the game loop can be enough to trip it and kick the host from their
 * own world. Only the "you timed out" disconnect is suppressed, and only for the singleplayer owner;
 * every other disconnect reason (and every other player) is unaffected.
 */
@Mixin(ServerCommonPacketListenerImpl.class)
abstract class ServerCommonPacketListenerImplMixin {

    @Shadow
    protected abstract boolean isSingleplayerOwner();

    @Redirect(
            method = "keepConnectionAlive",
            at = @At(value = "INVOKE", target = "Lnet/minecraft/server/network/ServerCommonPacketListenerImpl;disconnect(Lnet/minecraft/network/chat/Component;)V")
    )
    private void mfh$ignoreSingleplayerTimeout(ServerCommonPacketListenerImpl self, Component reason) {
        if (this.isSingleplayerOwner() && Component.translatable("disconnect.timeout").equals(reason)) {
            return;
        }
        self.disconnect(reason);
    }
}
