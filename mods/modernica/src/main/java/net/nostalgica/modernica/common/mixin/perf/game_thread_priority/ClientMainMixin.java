package net.nostalgica.modernica.common.mixin.perf.game_thread_priority;

import net.minecraft.client.main.Main;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Ported from Hydrogen's {@code mixin.thread.MixinClientMain} (no ModernFix equivalent;
 * ModernFix only tunes the integrated-server and worker-pool threads, not the client main thread).
 */
@Mixin(Main.class)
public class ClientMainMixin {
    @Inject(method = "main", at = @At("HEAD"))
    private static void mfh$setGameThreadPriority(String[] args, CallbackInfo ci) {
        Thread.currentThread().setPriority(Thread.NORM_PRIORITY);
    }
}
