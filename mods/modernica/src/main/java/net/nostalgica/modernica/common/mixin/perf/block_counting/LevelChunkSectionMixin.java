package net.nostalgica.modernica.common.mixin.perf.block_counting;

import com.llamalad7.mixinextras.sugar.Local;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import it.unimi.dsi.fastutil.ints.Int2ObjectMap;
import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.shorts.ShortArrayList;
import net.minecraft.util.BitStorage;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.Palette;
import net.minecraft.world.level.chunk.PalettedContainer;
import net.minecraft.world.level.material.FluidState;
import net.nostalgica.modernica.perf.block_counting.BlockCountingBitStorage;
import net.nostalgica.modernica.perf.block_counting.BlockCountingChunkSection;

import java.util.Iterator;
import java.util.function.Predicate;

/**
 * Vanilla's {@code recalcBlockCounts} walks every one of a section's 4096 block positions individually
 * (via {@code PalettedContainer#count}) to rebuild {@code nonEmptyBlockCount}/{@code fluidCount}/
 * {@code tickingBlockCount}/{@code tickingFluidCount}. Since most sections have far fewer distinct
 * palette entries than 4096 positions, it's cheaper to ask the palette's backing storage for a
 * histogram of "which positions hold palette index N" ({@link BlockCountingBitStorage#mfh$countEntries})
 * and classify each palette entry (there are only as many of these as the palette is large) once.
 * <p>
 * That same histogram also lets random ticking (see {@code perf.random_ticking}) skip straight to the
 * positions worth rolling instead of picking blind - {@link #mfh$getTickingBlockPositions} is
 * incrementally kept in sync by {@link #mfh$onSetBlockState} rather than being rebuilt every tick.
 */
@Mixin(LevelChunkSection.class)
abstract class LevelChunkSectionMixin implements BlockCountingChunkSection {

    @Shadow
    @Final
    private PalettedContainer<BlockState> states;

    @Shadow
    private short nonEmptyBlockCount;

    @Shadow
    private short fluidCount;

    @Shadow
    private short tickingBlockCount;

    @Shadow
    private short tickingFluidCount;

    @Shadow
    public abstract boolean maybeHas(Predicate<BlockState> predicate);

    @Unique
    private static final ShortArrayList ALL_POSITIONS = new ShortArrayList(16 * 16 * 16);
    static {
        for (short i = 0; i < 16 * 16 * 16; i++) {
            ALL_POSITIONS.add(i);
        }
    }

    @Unique
    private final ShortArrayList mfh$tickingBlocks = new ShortArrayList();

    @Override
    public final ShortArrayList mfh$getTickingBlockPositions() {
        return this.mfh$tickingBlocks;
    }

    @Inject(
            method = "setBlockState(IIILnet/minecraft/world/level/block/state/BlockState;Z)Lnet/minecraft/world/level/block/state/BlockState;",
            at = @At("RETURN")
    )
    private void mfh$onSetBlockState(int x, int y, int z, BlockState newState, boolean lock,
                                      CallbackInfoReturnable<BlockState> cir, @Local(ordinal = 1) BlockState oldState) {
        if (oldState == newState) {
            return;
        }

        boolean oldTicking = oldState.isRandomlyTicking();
        boolean newTicking = newState.isRandomlyTicking();
        if (oldTicking != newTicking) {
            short position = (short) (x | (z << 4) | (y << 8));
            if (oldTicking) {
                this.mfh$tickingBlocks.rem(position);
            } else {
                this.mfh$tickingBlocks.add(position);
            }
        }
    }

    @Overwrite
    public void recalcBlockCounts() {
        this.nonEmptyBlockCount = 0;
        this.fluidCount = 0;
        this.tickingBlockCount = 0;
        this.tickingFluidCount = 0;
        this.mfh$tickingBlocks.clear();

        if (!this.maybeHas(state -> !state.isAir())) {
            return;
        }

        @SuppressWarnings("unchecked")
        PalettedContainer.Data<BlockState> data = ((PalettedContainerDataAccessor<BlockState>) (Object) this.states).mfh$getData();
        Palette<BlockState> palette = data.palette();
        BitStorage storage = data.storage();

        Int2ObjectOpenHashMap<ShortArrayList> counts;
        if (palette.getSize() == 1) {
            counts = new Int2ObjectOpenHashMap<>(1);
            counts.put(0, ALL_POSITIONS);
        } else {
            counts = ((BlockCountingBitStorage) storage).mfh$countEntries();
        }

        for (Iterator<Int2ObjectMap.Entry<ShortArrayList>> it = counts.int2ObjectEntrySet().fastIterator(); it.hasNext(); ) {
            Int2ObjectMap.Entry<ShortArrayList> entry = it.next();
            ShortArrayList positions = entry.getValue();
            int count = positions.size();

            BlockState state = palette.valueFor(entry.getIntKey());
            if (state.isAir()) {
                continue;
            }

            this.nonEmptyBlockCount += (short) count;
            if (state.isRandomlyTicking()) {
                this.tickingBlockCount += (short) count;
                this.mfh$tickingBlocks.addAll(positions);
            }

            FluidState fluid = state.getFluidState();
            if (!fluid.isEmpty()) {
                this.fluidCount += (short) count;
                if (fluid.isRandomlyTicking()) {
                    this.tickingFluidCount += (short) count;
                }
            }
        }
    }
}
