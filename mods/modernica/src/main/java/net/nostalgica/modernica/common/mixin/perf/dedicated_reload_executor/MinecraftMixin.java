package net.nostalgica.modernica.common.mixin.perf.dedicated_reload_executor;

import net.minecraft.TracingExecutor;
import net.minecraft.client.Minecraft;
import net.nostalgica.modernica.Modernica;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(Minecraft.class)
@ClientOnlyMixin
public class MinecraftMixin {
    @Redirect(method = { "<init>", "reloadResourcePacks(ZLnet/minecraft/client/Minecraft$GameLoadCookie;)Ljava/util/concurrent/CompletableFuture;" }, at = @At(value = "INVOKE", target = "Lnet/minecraft/util/Util;backgroundExecutor()Lnet/minecraft/TracingExecutor;", ordinal = 0))
    private TracingExecutor getResourceReloadExecutor() {
        return Modernica.resourceReloadExecutor();
    }
}
