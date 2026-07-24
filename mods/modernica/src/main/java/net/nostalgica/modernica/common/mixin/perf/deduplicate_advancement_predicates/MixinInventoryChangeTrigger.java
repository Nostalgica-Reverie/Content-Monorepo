package net.nostalgica.modernica.common.mixin.perf.deduplicate_advancement_predicates;

import net.minecraft.advancements.criterion.InventoryChangeTrigger;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.player.Inventory;
import net.minecraft.world.item.ItemStack;
import net.nostalgica.modernica.common.advancement.ItemStackDataHolder;
import net.nostalgica.modernica.common.advancement.StackSizeThresholds;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.HashMap;
import java.util.Map;
import java.util.UUID;

@Mixin(InventoryChangeTrigger.class)
public abstract class MixinInventoryChangeTrigger {
    @Unique
    private final Map<UUID, Map<String, Integer>> mfh$skipTicks = new HashMap<>();

    @Inject(method = "trigger(Lnet/minecraft/server/level/ServerPlayer;Lnet/minecraft/world/entity/player/Inventory;Lnet/minecraft/world/item/ItemStack;)V",
            at = @At("HEAD"), cancellable = true)
    private void mfh$skipIrrelevantTriggers(ServerPlayer player, Inventory inventory, ItemStack itemStack, CallbackInfo ci) {
        Map<String, Integer> skipTickMap = this.mfh$skipTicks.computeIfAbsent(player.getUUID(), k -> new HashMap<>());
        String itemName = itemStack.getItem().toString();
        int skipTicks = skipTickMap.getOrDefault(itemName, 5);

        if (skipTicks < 4) {
            skipTickMap.put(itemName, skipTicks + 1);
            ci.cancel();
            return;
        }

        skipTickMap.put(itemName, 0);

        int prevSize = ((ItemStackDataHolder) (Object) itemStack).mfh$getPreviousStackSize();

        if (itemStack.isEmpty()
                || itemStack.getCount() < prevSize
                || !StackSizeThresholds.stackPassesThreshold(itemStack)) {
            ci.cancel();
        }
    }
}
