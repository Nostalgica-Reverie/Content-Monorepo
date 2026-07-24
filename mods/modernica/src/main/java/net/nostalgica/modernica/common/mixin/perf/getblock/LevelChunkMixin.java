package net.nostalgica.modernica.common.mixin.perf.getblock;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.core.BlockPos;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.LevelHeightAccessor;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.EmptyLevelChunk;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.PalettedContainerFactory;
import net.minecraft.world.level.chunk.UpgradeData;
import net.minecraft.world.level.levelgen.DebugLevelSource;
import net.minecraft.world.level.levelgen.blending.BlendingData;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.material.Fluids;
import net.minecraft.world.ticks.LevelChunkTicks;
import net.nostalgica.modernica.perf.getblock.GetBlockChunk;

/**
 * {@code getBlockState}/{@code getFluidState} are the single hottest read path in the game (worldgen,
 * lighting, rendering, and pathfinding all funnel through them constantly). Vanilla resolves both via
 * {@code ChunkAccess#getSection(int)} - a bounds-checked call - then a {@code PalettedContainer#get}
 * that recomputes the section-relative index from scratch every time. This caches the chunk's own
 * section-index origin once (like {@link ChunkAccessMixin}) and indexes the target section's palette
 * container directly with the same {@code x | z<<4 | y<<8} linear layout {@code PalettedContainer}
 * itself uses internally, skipping the redundant bounds check and index recomputation.
 */
@Mixin(LevelChunk.class)
abstract class LevelChunkMixin extends ChunkAccess implements GetBlockChunk {

    @Shadow
    @Final
    Level level;

    @Unique
    private static final BlockState AIR_BLOCKSTATE = Blocks.AIR.defaultBlockState();
    @Unique
    private static final BlockState VOID_AIR_BLOCKSTATE = Blocks.VOID_AIR.defaultBlockState();
    @Unique
    private static final FluidState AIR_FLUIDSTATE = Fluids.EMPTY.defaultFluidState();

    @Unique
    private int mfh$minSection;
    @Unique
    private boolean mfh$debug;
    @Unique
    private BlockState mfh$outOfBoundsState;

    public LevelChunkMixin(ChunkPos chunkPos, UpgradeData upgradeData, LevelHeightAccessor levelHeightAccessor,
                            PalettedContainerFactory palettedContainerFactory, long inhabitedTime,
                            LevelChunkSection[] levelChunkSections, BlendingData blendingData) {
        super(chunkPos, upgradeData, levelHeightAccessor, palettedContainerFactory, inhabitedTime, levelChunkSections, blendingData);
    }

    @Inject(
            method = "<init>(Lnet/minecraft/world/level/Level;Lnet/minecraft/world/level/ChunkPos;Lnet/minecraft/world/level/chunk/UpgradeData;Lnet/minecraft/world/level/chunk/LevelChunkTicks;Lnet/minecraft/world/level/chunk/LevelChunkTicks;J[Lnet/minecraft/world/level/chunk/LevelChunkSection;Lnet/minecraft/world/level/chunk/LevelChunk$PostLoadProcessor;Lnet/minecraft/world/level/levelgen/blending/BlendingData;)V",
            at = @At("TAIL")
    )
    private void mfh$cacheGetBlockState(Level level, ChunkPos chunkPos, UpgradeData upgradeData, LevelChunkTicks<?> ticks1,
                                         LevelChunkTicks<?> ticks2, long inhabitedTime, LevelChunkSection[] sections,
                                         LevelChunk.PostLoadProcessor postLoadProcessor, BlendingData blendingData, CallbackInfo ci) {
        this.mfh$minSection = level.getMinY() >> 4;
        boolean empty = (Object) this instanceof EmptyLevelChunk;
        this.mfh$debug = !empty && this.level.isDebug();
        this.mfh$outOfBoundsState = empty ? VOID_AIR_BLOCKSTATE : AIR_BLOCKSTATE;
    }

    @Overwrite
    @Override
    public BlockState getBlockState(BlockPos pos) {
        return this.mfh$getBlock(pos.getX(), pos.getY(), pos.getZ());
    }

    @Unique
    private BlockState mfh$debugBlock(int x, int y, int z) {
        if (y == 60) {
            return Blocks.BARRIER.defaultBlockState();
        }
        if (y == 70) {
            BlockState state = DebugLevelSource.getBlockStateFor(x, z);
            return state == null ? AIR_BLOCKSTATE : state;
        }
        return AIR_BLOCKSTATE;
    }

    @Override
    public final BlockState mfh$getBlock(int x, int y, int z) {
        if (this.mfh$debug) {
            return this.mfh$debugBlock(x, y, z);
        }

        int sectionY = (y >> 4) - this.mfh$minSection;
        LevelChunkSection[] sections = this.sections;
        if (sectionY < 0 || sectionY >= sections.length) {
            return this.mfh$outOfBoundsState;
        }

        LevelChunkSection section = sections[sectionY];
        if (section.hasOnlyAir()) {
            return this.mfh$outOfBoundsState;
        }

        int index = (x & 15) | ((z & 15) << 4) | ((y & 15) << 8);
        return mfh$states(section).mfh$get(index);
    }

    @Overwrite
    public FluidState getFluidState(int x, int y, int z) {
        int sectionY = (y >> 4) - this.mfh$minSection;
        LevelChunkSection[] sections = this.sections;
        if (sectionY < 0 || sectionY >= sections.length) {
            return AIR_FLUIDSTATE;
        }

        LevelChunkSection section = sections[sectionY];
        if (section.hasOnlyAir()) {
            return AIR_FLUIDSTATE;
        }

        int index = (x & 15) | ((z & 15) << 4) | ((y & 15) << 8);
        return mfh$states(section).mfh$get(index).getFluidState();
    }

    @SuppressWarnings("unchecked")
    @Unique
    private static PalettedContainerGetInvoker<BlockState> mfh$states(LevelChunkSection section) {
        return (PalettedContainerGetInvoker<BlockState>) (Object) ((LevelChunkSectionStatesAccessor) (Object) section).mfh$getStates();
    }
}
