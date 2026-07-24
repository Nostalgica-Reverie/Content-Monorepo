package net.nostalgica.modernica.common.mixin.perf.getblock;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.core.Holder;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.LevelHeightAccessor;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeManager;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.LightChunk;
import net.minecraft.world.level.chunk.PalettedContainerFactory;
import net.minecraft.world.level.chunk.StructureAccess;
import net.minecraft.world.level.chunk.UpgradeData;
import net.minecraft.world.level.levelgen.blending.BlendingData;

/**
 * Caches the chunk's section-index origin once at construction instead of deriving it from the height
 * accessor on every {@code getNoiseBiome} call, and drops that method's per-call bounds branching for a
 * simpler clamp.
 */
@Mixin(ChunkAccess.class)
abstract class ChunkAccessMixin implements BlockGetter, BiomeManager.NoiseBiomeSource, LightChunk, StructureAccess {

    @Shadow
    @Final
    protected LevelChunkSection[] sections;

    @Unique
    private int mfh$minSection;

    @Inject(method = "<init>", at = @At("TAIL"))
    private void mfh$cacheMinSection(ChunkPos chunkPos, UpgradeData upgradeData, LevelHeightAccessor levelHeightAccessor,
                                      PalettedContainerFactory palettedContainerFactory, long inhabitedTime,
                                      LevelChunkSection[] levelChunkSections, BlendingData blendingData, CallbackInfo ci) {
        this.mfh$minSection = levelHeightAccessor.getMinY() >> 4;
    }

    @Overwrite
    @Override
    public Holder<Biome> getNoiseBiome(int biomeX, int biomeY, int biomeZ) {
        int sectionY = (biomeY >> 2) - this.mfh$minSection;
        int rel = biomeY & 3;

        LevelChunkSection[] sections = this.sections;
        if (sectionY < 0) {
            sectionY = 0;
            rel = 0;
        } else if (sectionY >= sections.length) {
            sectionY = sections.length - 1;
            rel = 3;
        }

        return sections[sectionY].getNoiseBiome(biomeX & 3, rel, biomeZ & 3);
    }
}
