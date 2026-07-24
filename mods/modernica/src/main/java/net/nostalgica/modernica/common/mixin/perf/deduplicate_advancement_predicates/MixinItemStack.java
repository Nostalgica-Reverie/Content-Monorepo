package net.nostalgica.modernica.common.mixin.perf.deduplicate_advancement_predicates;

import net.minecraft.world.item.ItemStack;
import net.nostalgica.modernica.common.advancement.ItemStackDataHolder;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;

@Mixin(ItemStack.class)
public class MixinItemStack implements ItemStackDataHolder {
    @Unique
    private int mfh$previousStackSize;

    @Override
    public void mfh$setPreviousStackSize(int value) {
        this.mfh$previousStackSize = value;
    }

    @Override
    public int mfh$getPreviousStackSize() {
        return this.mfh$previousStackSize;
    }
}
