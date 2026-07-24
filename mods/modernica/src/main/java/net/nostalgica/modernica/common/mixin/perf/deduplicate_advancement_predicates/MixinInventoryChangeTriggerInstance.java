package net.nostalgica.modernica.common.mixin.perf.deduplicate_advancement_predicates;

import net.minecraft.advancements.criterion.InventoryChangeTrigger;
import net.minecraft.advancements.criterion.ItemPredicate;
import net.minecraft.world.entity.player.Inventory;
import net.minecraft.world.item.ItemInstance;
import net.minecraft.world.item.ItemStack;
import net.nostalgica.modernica.common.advancement.ItemPredicateDataHolder;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.List;

@Mixin(InventoryChangeTrigger.TriggerInstance.class)
public abstract class MixinInventoryChangeTriggerInstance {
    @Shadow
    @Final
    private List<ItemPredicate> items;

    @Inject(method = "matches(Lnet/minecraft/world/entity/player/Inventory;Lnet/minecraft/world/item/ItemStack;III)Z",
            at = @At(value = "INVOKE", target = "Lnet/minecraft/world/entity/player/Inventory;getContainerSize()I", ordinal = 0),
            cancellable = true)
    private void mfh$skipUnrelatedScan(Inventory inventory, ItemStack changedItem, int slotsFull, int slotsEmpty, int slotsOccupied,
                                             CallbackInfoReturnable<Boolean> cir) {
        if (this.items.stream().noneMatch(predicate -> predicate.test(changedItem))) {
            cir.setReturnValue(false);
        }
    }

    @Redirect(method = "matches(Lnet/minecraft/world/entity/player/Inventory;Lnet/minecraft/world/item/ItemStack;III)Z",
            at = @At(value = "INVOKE", target = "Lnet/minecraft/advancements/criterion/ItemPredicate;test(Lnet/minecraft/world/item/ItemInstance;)Z"))
    private boolean mfh$fasterSinglePredicateMatch(ItemPredicate itemPredicate, ItemInstance itemStack) {
        return ((ItemPredicateDataHolder) (Object) itemPredicate).mfh$fasterMatches(itemStack);
    }
}
