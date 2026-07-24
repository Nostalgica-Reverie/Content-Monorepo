package net.nostalgica.modernica.common.mixin.perf.mob_spawning;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import it.unimi.dsi.fastutil.objects.Object2IntMap;
import net.minecraft.world.entity.MobCategory;

/** {@code MobCategory} is a small fixed enum - an ordinal-indexed {@code int[]} is a strictly cheaper
 * and simpler stand-in for a hash map keyed by it. {@code MobCounts} is a private nested class, so it
 * has to be targeted by name rather than by class literal. */
@Mixin(targets = "net/minecraft/world/level/LocalMobCapCalculator$MobCounts")
abstract class LocalMobCapCalculatorMobCountsMixin {

    @Shadow
    @Mutable
    @Final
    private Object2IntMap<MobCategory> counts;

    @Unique
    private static final MobCategory[] MFH_CATEGORIES = MobCategory.values();

    @Unique
    private final int[] mfh$counts = new int[MFH_CATEGORIES.length];

    @Inject(method = "<init>", at = @At("RETURN"))
    private void mfh$dropVanillaMap(CallbackInfo ci) {
        this.counts = null;
    }

    @Overwrite
    public void add(MobCategory category) {
        this.mfh$counts[category.ordinal()]++;
    }

    @Overwrite
    public boolean canSpawn(MobCategory category) {
        return this.mfh$counts[category.ordinal()] < category.getMaxInstancesPerChunk();
    }
}
