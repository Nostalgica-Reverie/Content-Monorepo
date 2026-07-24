package net.nostalgica.modernica.fabric.mixin.core;

import net.minecraft.server.MinecraftServer;
import net.nostalgica.modernica.Modernica;
import net.nostalgica.modernica.ModernicaFabric;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.lang.ref.WeakReference;

@Mixin(MinecraftServer.class)
public class MinecraftServerMixin {
    @Inject(method = "runServer", at = @At("HEAD"))
    private void changeServerReference(CallbackInfo ci) {
        ModernicaFabric.theServer = new WeakReference<>((MinecraftServer)(Object)this);
    }

    @Inject(method = "runServer", at = @At(value = "INVOKE", target = "Lnet/minecraft/util/Util;getNanos()J", ordinal = 0))
    private void hookServerStarted(CallbackInfo ci) {
        Modernica.INSTANCE.onServerStarted();
    }

    @Inject(method = "stopServer", at = @At("RETURN"))
    private void hookServerShutdown(CallbackInfo ci) {
        Modernica.INSTANCE.onServerDead((MinecraftServer)(Object)this);
    }
}
