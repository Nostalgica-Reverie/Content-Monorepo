package net.nostalgica.modernica.common.mixin.perf.dynamic_resources;

import net.minecraft.client.resources.model.BlockStateDefinitions;
import net.minecraft.resources.Identifier;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.StateDefinition;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import java.util.Map;

@Mixin(BlockStateDefinitions.class)
@ClientOnlyMixin
public interface BlockStateDefinitionsAccessor {
    @Accessor("STATIC_DEFINITIONS")
    static Map<Identifier, StateDefinition<Block, BlockState>> getStaticDefinitions() {
        throw new AssertionError();
    }
}