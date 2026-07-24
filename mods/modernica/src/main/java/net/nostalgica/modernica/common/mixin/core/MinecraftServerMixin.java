package net.nostalgica.modernica.common.mixin.core;

import net.minecraft.util.Util;
import net.minecraft.server.MinecraftServer;
import net.nostalgica.modernica.duck.ITimeTrackingServer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MinecraftServer.class)
public class MinecraftServerMixin implements ITimeTrackingServer {
    private long mfix$lastTickStartTime = -1L;

    @Override
    public long mfix$getLastTickStartTime() {
        return mfix$lastTickStartTime;
    }

    @Inject(method = "runServer", at = @At(value = "INVOKE", target = "Lnet/minecraft/server/MinecraftServer;processPacketsAndTick(Z)V"))
    private void trackTickTime(CallbackInfo ci) {
        mfix$lastTickStartTime = Util.getMillis();
    }
}
