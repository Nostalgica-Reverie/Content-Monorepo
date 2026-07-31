package net.nostalgica.modernica.common.mixin.perf.bad_optimizations.skip_non_demo_tutorial;

import net.minecraft.client.Minecraft;
import net.minecraft.client.tutorial.Tutorial;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** The tutorial has no active behavior outside demo worlds. */
@ClientOnlyMixin
@Mixin(Tutorial.class)
abstract class TutorialMixin {
    @Shadow @Final private Minecraft minecraft;

    @Inject(method = "tick()V", at = @At("HEAD"), cancellable = true)
    private void modernica$skipNonDemoTutorial(CallbackInfo ci) {
        if (!minecraft.isDemo()) {
            ci.cancel();
        }
    }
}
