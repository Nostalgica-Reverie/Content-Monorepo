package net.nostalgica.modernica.common.mixin.perf.thread_priorities;

import com.llamalad7.mixinextras.sugar.Local;
import net.minecraft.client.server.IntegratedServer;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(IntegratedServer.class)
@ClientOnlyMixin
public class IntegratedServerMixin {
    @Inject(method = "<init>", at = @At("RETURN"))
    private void adjustServerPriority(CallbackInfo ci, @Local(ordinal = 0, argsOnly = true) Thread thread) {
        int pri = 4;
        thread.setPriority(pri);
    }
}
