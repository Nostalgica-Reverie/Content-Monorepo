package net.nostalgica.modernica.common.mixin.perf.random_ticking;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.PalettedContainer;

@Mixin(LevelChunkSection.class)
public interface LevelChunkSectionStatesAccessor {
    @Accessor("states")
    PalettedContainer<BlockState> mfh$getStates();
}
