package net.nostalgica.modernica.common.mixin.perf.dedicated_reload_executor;

import net.minecraft.client.gui.screens.worldselection.WorldOpenFlows;
import net.nostalgica.modernica.Modernica;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;

import java.util.concurrent.Executor;

@Mixin(WorldOpenFlows.class)
@ClientOnlyMixin
public class WorldOpenFlowsMixin {
    @ModifyArg(method = "loadWorldDataBlocking", at = @At(value = "INVOKE", target = "Lnet/minecraft/server/WorldLoader;load(Lnet/minecraft/server/WorldLoader$InitConfig;Lnet/minecraft/server/WorldLoader$WorldDataSupplier;Lnet/minecraft/server/WorldLoader$ResultFactory;Ljava/util/concurrent/Executor;Ljava/util/concurrent/Executor;)Ljava/util/concurrent/CompletableFuture;"), index = 3)
    private Executor getResourceReloadExecutor(Executor service) {
        return Modernica.resourceReloadExecutor();
    }
}
