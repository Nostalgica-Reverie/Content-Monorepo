package net.nostalgica.modernica.common.mixin.perf.random_ticking;

import com.llamalad7.mixinextras.sugar.Local;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

import it.unimi.dsi.fastutil.shorts.ShortArrayList;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.RegistryAccess;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.WorldGenLevel;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.dimension.DimensionType;
import net.minecraft.world.level.levelgen.RandomSupport;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.storage.WritableLevelData;
import net.nostalgica.modernica.perf.block_counting.BlockCountingChunkSection;
import net.nostalgica.modernica.util.FastIndexRandom;

/**
 * Vanilla's random tick loop rolls {@code tickSpeed} random positions per section and calls
 * {@code getBlockState} on each to see if it's even randomly-tickable - almost always a miss, since most
 * blocks in a typical section aren't. {@link BlockCountingChunkSection#mfh$getTickingBlockPositions}
 * (kept up to date incrementally, see its owner class) already knows exactly which positions qualify, so
 * this rolls an index into that list instead: a miss (index past the list's current size) is a cheap
 * bounds check instead of a wasted palette lookup, and a hit skips straight to a known-good position.
 * The block/fluid random-tick pair vanilla runs per hit is otherwise unchanged.
 */
@Mixin(ServerLevel.class)
abstract class ServerLevelMixin extends Level implements WorldGenLevel {

    protected ServerLevelMixin(WritableLevelData writableLevelData, ResourceKey<Level> resourceKey, RegistryAccess registryAccess,
                                Holder<DimensionType> holder, boolean bl, boolean bl2, long l, int i) {
        super(writableLevelData, resourceKey, registryAccess, holder, bl, bl2, l, i);
    }

    @Unique
    private static final LevelChunkSection[] MFH_NO_SECTIONS = new LevelChunkSection[0];

    @Unique
    private final FastIndexRandom mfh$tickRandom = new FastIndexRandom(RandomSupport.generateUniqueSeed());

    @Redirect(method = "tickChunk", at = @At(value = "INVOKE", target = "Lnet/minecraft/util/RandomSource;nextInt(I)I"))
    private int mfh$fastTickRandom(RandomSource instance, int bound) {
        return this.mfh$tickRandom.nextInt(bound);
    }

    @Redirect(
            method = "tickChunk",
            at = @At(value = "INVOKE", target = "Lnet/minecraft/world/level/chunk/LevelChunk;getSections()[Lnet/minecraft/world/level/chunk/LevelChunkSection;", ordinal = 0)
    )
    private LevelChunkSection[] mfh$fastRandomTick(LevelChunk chunk, @Local(ordinal = 0, argsOnly = true) int tickSpeed) {
        LevelChunkSection[] sections = chunk.getSections();
        int minSection = this.getMinY() >> 4;
        FastIndexRandom random = this.mfh$tickRandom;

        ChunkPos pos = chunk.getPos();
        int offsetX = pos.x() << 4;
        int offsetZ = pos.z() << 4;

        for (int sectionIndex = 0; sectionIndex < sections.length; sectionIndex++) {
            LevelChunkSection section = sections[sectionIndex];
            if (!section.isRandomlyTickingBlocks()) {
                continue;
            }

            int offsetY = (sectionIndex + minSection) << 4;
            PalettedContainerGetInvoker<BlockState> states = mfh$states(section);
            ShortArrayList tickPositions = ((BlockCountingChunkSection) section).mfh$getTickingBlockPositions();

            for (int i = 0; i < tickSpeed; i++) {
                int candidateCount = tickPositions.size();
                int index = random.nextInt() & ((16 * 16 * 16) - 1);
                if (index >= candidateCount) {
                    continue;
                }

                int packed = tickPositions.getShort(index) & 0xFFFF;
                BlockState state = states.mfh$get(packed);
                BlockPos blockPos = new BlockPos((packed & 15) | offsetX, ((packed >>> 8) & 15) | offsetY, ((packed >>> 4) & 15) | offsetZ);

                state.randomTick((ServerLevel) (Object) this, blockPos, random);

                FluidState fluidState = state.getFluidState();
                if (fluidState.isRandomlyTicking()) {
                    fluidState.randomTick((ServerLevel) (Object) this, blockPos, random);
                }
            }
        }

        return MFH_NO_SECTIONS;
    }

    @SuppressWarnings("unchecked")
    @Unique
    private static PalettedContainerGetInvoker<BlockState> mfh$states(LevelChunkSection section) {
        return (PalettedContainerGetInvoker<BlockState>) (Object) ((LevelChunkSectionStatesAccessor) (Object) section).mfh$getStates();
    }
}
