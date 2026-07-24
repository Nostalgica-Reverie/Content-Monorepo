package net.nostalgica.modernica.common.mixin.perf.game_thread_priority;

import net.minecraft.server.Main;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Ported from Hydrogen's {@code mixin.thread.MixinServerMain} (no ModernFix equivalent).
 */
@Mixin(Main.class)
public class ServerMainMixin {
    @Inject(method = "main", at = @At("HEAD"))
    private static void mfh$setGameThreadPriority(String[] args, CallbackInfo ci) {
        Thread.currentThread().setPriority(Thread.NORM_PRIORITY);
    }
}
