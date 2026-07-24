package net.nostalgica.modernica.common.mixin.perf.deduplicate_advancement_predicates;

import net.minecraft.advancements.criterion.ItemPredicate;
import net.minecraft.advancements.criterion.MinMaxBounds;
import net.minecraft.world.item.ItemInstance;
import net.minecraft.world.item.ItemStack;
import net.nostalgica.modernica.common.advancement.ItemPredicateDataHolder;
import net.nostalgica.modernica.common.advancement.ItemStackDataHolder;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

import java.util.Optional;

@Mixin(ItemPredicate.class)
public abstract class MixinItemPredicate implements ItemPredicateDataHolder {
    @Final
    @Shadow
    private MinMaxBounds.Ints count;

    @Shadow
    public abstract boolean test(ItemInstance itemStack);

    @Override
    public boolean mfh$fasterMatches(ItemInstance stack) {
        Optional<Integer> minThr = this.count.min();
        Optional<Integer> maxThr = this.count.max();
        int stackCount = stack.count();
        int prevStackCount = stack instanceof ItemStack itemStack
                ? ((ItemStackDataHolder) (Object) itemStack).mfh$getPreviousStackSize()
                : 0;

        if (minThr.map(min -> prevStackCount < min && min <= stackCount).orElseGet(() -> prevStackCount == 0)
                && (maxThr.isEmpty() || stackCount <= maxThr.get())) {
            return this.test(stack);
        }

        return false;
    }
}
