package net.nostalgica.modernica.perf.getblock;

import net.minecraft.world.level.block.state.BlockState;

/** Implemented by {@code LevelChunkMixin}; lets {@code getBlockState} share one lookup path. */
public interface GetBlockChunk {
    BlockState mfh$getBlock(int x, int y, int z);
}
