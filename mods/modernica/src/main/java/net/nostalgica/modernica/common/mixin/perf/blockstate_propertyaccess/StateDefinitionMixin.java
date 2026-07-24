package net.nostalgica.modernica.common.mixin.perf.blockstate_propertyaccess;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import com.google.common.collect.ImmutableList;
import net.minecraft.world.level.block.state.StateDefinition;
import net.minecraft.world.level.block.state.StateHolder;
import net.nostalgica.modernica.perf.blockstate_propertyaccess.PropertyAccessStateHolder;

/** Once every state in a family has been built, wires the first one's {@link StateIndexTable} up and
 * lets {@link StateHolderMixin#mfh$init} de-duplicate the rest against it. */
@Mixin(StateDefinition.class)
abstract class StateDefinitionMixin {

    @Inject(
            at = @At("RETURN"),
            method = {
                    "createSingletonState",
                    "createSinglePropertyStates(Ljava/lang/Object;Lnet/minecraft/world/level/block/state/StateDefinition$Factory;Lnet/minecraft/world/level/block/state/properties/Property;)Lcom/google/common/collect/ImmutableList;",
                    "createMultiPropertyStates"
            }
    )
    private static <O, S extends StateHolder<O, S>> void mfh$initStateTable(CallbackInfoReturnable<ImmutableList<S>> cir) {
        ImmutableList<S> states = cir.getReturnValue();
        if (!states.isEmpty()) {
            ((PropertyAccessStateHolder<O, S>) (StateHolder<O, S>) states.get(0)).mfh$init(states);
        }
    }
}
