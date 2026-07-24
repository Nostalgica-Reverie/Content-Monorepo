package net.nostalgica.modernica.common.mixin.perf.release_protochunks;

import com.llamalad7.mixinextras.injector.ModifyExpressionValue;
import com.llamalad7.mixinextras.sugar.Local;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.chunk.ImposterProtoChunk;
import net.nostalgica.modernica.Modernica;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;

@Mixin(ImposterProtoChunk.class)
public class ImposterProtoChunkMixin {
    @Shadow
    @Final
    private boolean allowWrites;

    /**
     * @author embeddedt
     * @reason Hide live BlockEntity instances from worldgen through ImposterProtoChunk wrappers.
     */
    @ModifyExpressionValue(method = "getBlockEntity", at = @At(value = "INVOKE", target = "Lnet/minecraft/world/level/chunk/LevelChunk;getBlockEntity(Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/level/block/entity/BlockEntity;"))
    private BlockEntity avoidLeakingLiveBE(BlockEntity original, @Local(ordinal = 0, argsOnly = true) BlockPos pos) {
        if (!this.allowWrites && original != null && original.getLevel() != null) {
            Modernica.LOGGER.debug("Blocked accessing the main level BlockEntity at {} from the ImposterProtoChunk wrapper, as this is unsafe during worldgen.", pos, new Exception("Stacktrace"));
            return null;
        } else {
            return original;
        }
    }
}
