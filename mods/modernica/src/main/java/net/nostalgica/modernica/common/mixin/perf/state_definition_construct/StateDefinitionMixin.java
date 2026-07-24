package net.nostalgica.modernica.common.mixin.perf.state_definition_construct;

import net.minecraft.world.level.block.state.StateDefinition;
import net.minecraft.world.level.block.state.StateHolder;
import net.minecraft.world.level.block.state.properties.Property;
import net.nostalgica.modernica.annotation.RequiresMod;
import net.nostalgica.modernica.blockstate.FakeStateMap;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

import java.util.List;
import java.util.Map;

// This optimization requires FerriteCore to be worthwhile, otherwise the FakeStateMap degrades to hash internally
@Mixin(StateDefinition.class)
@RequiresMod("ferritecore")
public class StateDefinitionMixin<O, S extends StateHolder<O, S>> {
    /**
     * @author embeddedt
     * @reason write states into a custom array map for fast iteration by FerriteCore, no need to waste time hashing
     * and growing
     */
    @ModifyVariable(method = "createMultiPropertyStates", at = @At(value = "STORE", ordinal = 0), index = 6)
    private static <O, S extends StateHolder<O, S>> Map<List<Comparable<?>>, S> useArrayMap(Map<List<Comparable<?>>, S> in) {
        return new FakeStateMap<>(64);
    }
}
