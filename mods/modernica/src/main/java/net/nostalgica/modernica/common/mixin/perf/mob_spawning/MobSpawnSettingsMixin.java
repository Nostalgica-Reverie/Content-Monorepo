package net.nostalgica.modernica.common.mixin.perf.mob_spawning;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.world.entity.EntityType;
import net.minecraft.world.level.biome.MobSpawnSettings;
import net.nostalgica.modernica.perf.mob_spawning.MobSpawningEntityType;

import java.util.Map;

/** Flags, on each {@code EntityType} that appears in any biome's spawn-cost table, that it has one -
 * see {@link NaturalSpawnerMixin}. */
@Mixin(MobSpawnSettings.class)
abstract class MobSpawnSettingsMixin {

    @Shadow
    @Final
    private Map<EntityType<?>, MobSpawnSettings.MobSpawnCost> mobSpawnCosts;

    @Inject(method = "<init>", at = @At("RETURN"))
    private void mfh$flagBiomeCosts(CallbackInfo ci) {
        for (EntityType<?> type : this.mobSpawnCosts.keySet()) {
            ((MobSpawningEntityType) type).mfh$setHasBiomeCost();
        }
    }
}
