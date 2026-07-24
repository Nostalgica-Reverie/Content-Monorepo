package net.nostalgica.modernica.common.mixin.perf.deduplicate_advancement_predicates;

import net.minecraft.advancements.Advancement;
import net.minecraft.advancements.Criterion;
import net.minecraft.advancements.criterion.InventoryChangeTrigger;
import net.minecraft.advancements.criterion.ItemPredicate;
import net.minecraft.resources.Identifier;
import net.minecraft.server.ServerAdvancementManager;
import net.minecraft.server.packs.resources.ResourceManager;
import net.minecraft.util.profiling.ProfilerFiller;
import net.nostalgica.modernica.common.advancement.StackSizeThresholds;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Map;

@Mixin(ServerAdvancementManager.class)
public abstract class MixinServerAdvancementManager {
    @Inject(method = "apply", at = @At("RETURN"))
    private void mfh$collectStackSizeThresholds(Map<Identifier, Advancement> preparations, ResourceManager manager, ProfilerFiller profiler, CallbackInfo ci) {
        StackSizeThresholds.clear();

        for (Advancement advancement : preparations.values()) {
            for (Criterion<?> criterion : advancement.criteria().values()) {
                if (!(criterion.triggerInstance() instanceof InventoryChangeTrigger.TriggerInstance trigger)) {
                    continue;
                }

                for (ItemPredicate predicate : trigger.items()) {
                    predicate.count().min().ifPresent(min -> {
                        if (min > 1) {
                            StackSizeThresholds.add(min);
                        }
                    });
                }
            }
        }
    }
}
