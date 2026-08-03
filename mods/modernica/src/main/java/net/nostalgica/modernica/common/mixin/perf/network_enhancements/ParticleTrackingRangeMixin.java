package net.nostalgica.modernica.common.mixin.perf.network_enhancements;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import net.minecraft.network.protocol.Packet;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.nostalgica.modernica.core.config.MixinGate;

/**
 * See {@link VarLongMixin}'s class doc for the independent-reimplementation context.
 * <p>
 * Caps how far particle-effect packets broadcast to players, mirroring Paper's long-published
 * {@code particle-tracking-range} setting: vanilla's per-player particle broadcast (the private
 * {@code ServerLevel#sendParticles(ServerPlayer, boolean, double, double, double, Packet)} overload all
 * of the class's other {@code sendParticles} entry points funnel into) uses a fixed internal radius
 * that isn't tied to the server's configured view/simulation distance. Particles are pure visual
 * effects with no gameplay state, so suppressing distant ones is safe by construction - unlike
 * suppressing movement packets, it can't cause a desync, only (at worst, if the configured range is set
 * too low) a missing visual effect far from the player. {@code overrideLimiter} (used by explicit
 * {@code /particle force} broadcasts) is always respected and left untouched.
 */
@Mixin(ServerLevel.class)
public class ParticleTrackingRangeMixin {
    @Inject(
            method = "sendParticles(Lnet/minecraft/server/level/ServerPlayer;ZDDDLnet/minecraft/network/protocol/Packet;)Z",
            at = @At("HEAD"),
            cancellable = true
    )
    private void mfh$limitParticleTrackingRange(ServerPlayer player, boolean overrideLimiter, double x, double y, double z,
                                                 Packet<?> packet, CallbackInfoReturnable<Boolean> cir) {
        if (overrideLimiter) {
            return;
        }
        int rangeBlocks = MixinGate.particleTrackingRangeBlocks();
        double rangeSq = (double) rangeBlocks * rangeBlocks;
        if (player.distanceToSqr(x, y, z) > rangeSq) {
            cir.setReturnValue(false);
        }
    }
}
