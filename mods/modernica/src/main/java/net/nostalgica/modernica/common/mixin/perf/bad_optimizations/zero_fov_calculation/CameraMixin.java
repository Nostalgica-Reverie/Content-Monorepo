package net.nostalgica.modernica.common.mixin.perf.bad_optimizations.zero_fov_calculation;

import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import net.minecraft.client.Camera;
import net.minecraft.client.CameraType;
import net.minecraft.client.Minecraft;
import net.minecraft.client.player.AbstractClientPlayer;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;

/** Avoids calculating a player FOV modifier when the user's FOV-effect scale already makes it irrelevant. */
@ClientOnlyMixin
@Mixin(Camera.class)
abstract class CameraMixin {
    @Shadow @Final private Minecraft minecraft;

    @WrapOperation(method = "tickFov", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/player/AbstractClientPlayer;getFieldOfViewModifier(ZF)F"))
    private float modernica$skipZeroScaleFov(AbstractClientPlayer player, boolean firstPerson, float fovEffectScale, Operation<Float> original) {
        if (fovEffectScale != 0.0F) {
            return original.call(player, firstPerson, fovEffectScale);
        }
        return minecraft.options.getCameraType() == CameraType.FIRST_PERSON && player.isScoping() ? 0.1F : 1.0F;
    }
}
