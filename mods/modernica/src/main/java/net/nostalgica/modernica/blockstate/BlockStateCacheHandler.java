package net.nostalgica.modernica.blockstate;

import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockBehaviour;
import net.minecraft.world.level.block.state.BlockState;
import net.nostalgica.modernica.duck.IBlockState;

public class BlockStateCacheHandler {
    public static void invalidateCache() {
        synchronized (BlockBehaviour.BlockStateBase.class) {
            for (BlockState blockState : Block.BLOCK_STATE_REGISTRY) {
                ((IBlockState)blockState).clearCache();
            }
        }
    }
}
