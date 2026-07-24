package net.nostalgica.modernica.common.mixin.perf.mob_spawning;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;

import net.minecraft.world.entity.EntityType;
import net.nostalgica.modernica.perf.mob_spawning.MobSpawningEntityType;

@Mixin(EntityType.class)
abstract class EntityTypeMixin implements MobSpawningEntityType {

    @Unique
    private boolean mfh$hasBiomeCost = false;

    @Override
    public final boolean mfh$hasAnyBiomeCost() {
        return this.mfh$hasBiomeCost;
    }

    @Override
    public final void mfh$setHasBiomeCost() {
        this.mfh$hasBiomeCost = true;
    }
}
