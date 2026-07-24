package net.nostalgica.modernica.common.mixin.perf.cache_strongholds;

import net.minecraft.world.level.chunk.ChunkGeneratorStructureState;
import net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement;
import net.nostalgica.modernica.annotation.FeatureLevel;
import net.nostalgica.modernica.annotation.RequiresFeatureLevel;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(ConcentricRingsStructurePlacement.class)
@RequiresFeatureLevel(FeatureLevel.BETA)
public class ConcentricRingsStructurePlacementMixin {

    @Shadow @Final private int distance;
    @Shadow @Final private int spread;
    @Shadow @Final private int count;

    @Unique private static final int MFIX_MAX_BIOME_SNAP_SECTIONS_PER_AXIS = 7;
    @Unique private static final double MFIX_MAX_ROUNDING_ERROR = Math.sqrt(2.0) * 0.5;
    @Unique private static final double MFIX_MAX_BIOME_SNAP_ERROR = MFIX_MAX_BIOME_SNAP_SECTIONS_PER_AXIS * Math.sqrt(2.0);
    @Unique private static final double MFIX_MAX_POSITION_ERROR = MFIX_MAX_ROUNDING_ERROR + MFIX_MAX_BIOME_SNAP_ERROR;

    @Unique private long mfix$innerRadiusSq;
    @Unique private long mfix$outerRadiusSq;

    @Inject(
        method = "<init>(Lnet/minecraft/core/Vec3i;Lnet/minecraft/world/level/levelgen/structure/placement/StructurePlacement$FrequencyReductionMethod;FILjava/util/Optional;IIILnet/minecraft/core/HolderSet;)V",
        at = @At("RETURN")
    )
    private void mfix$computeRadiusBounds(CallbackInfo ci) {
        double maxNoise = this.distance * 1.25;

        double minDist = 4.0 * this.distance - maxNoise;
        double safeInnerRadius = minDist - MFIX_MAX_POSITION_ERROR;
        this.mfix$innerRadiusSq = (long)Math.max(0.0, Math.floor(safeInnerRadius * safeInnerRadius));

        if (this.spread == 0) {
            this.mfix$outerRadiusSq = Long.MAX_VALUE;
            return;
        }

        int maxCircle = this.mfix$computeMaxCircleIndex();
        double maxDist = 4.0 * this.distance + (double)this.distance * maxCircle * 6.0 + maxNoise;
        double safeOuterRadius = maxDist + MFIX_MAX_POSITION_ERROR;
        this.mfix$outerRadiusSq = (long)Math.ceil(safeOuterRadius * safeOuterRadius);
    }

    @Unique
    private int mfix$computeMaxCircleIndex() {
        int ringSpread = this.spread;
        int total = 0;
        int circle = 0;

        while (total + ringSpread < this.count) {
            total += ringSpread;
            circle++;
            ringSpread += 2 * ringSpread / (circle + 1);
            ringSpread = Math.min(ringSpread, this.count - total);
        }

        return circle;
    }

    /**
     * @author embeddedt, GPT-5.3-Codex
     * @reason Avoid calling getRingPositionsFor() when we know the current chunk lies outside the region where
     * concentric placement can even happen.
     */
    @Inject(method = "isPlacementChunk", at = @At("HEAD"), cancellable = true)
    private void mfix$earlyRejectByRadius(ChunkGeneratorStructureState structureState, int x, int z,
                                          CallbackInfoReturnable<Boolean> cir) {
        long distSq = (long)x * x + (long)z * z;
        if (distSq < this.mfix$innerRadiusSq || distSq > this.mfix$outerRadiusSq) {
            cir.setReturnValue(false);
        }
    }
}
