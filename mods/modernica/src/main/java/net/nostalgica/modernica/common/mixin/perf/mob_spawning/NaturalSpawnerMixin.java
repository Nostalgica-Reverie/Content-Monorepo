package net.nostalgica.modernica.common.mixin.perf.mob_spawning;

import com.llamalad7.mixinextras.sugar.Local;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

import net.minecraft.core.BlockPos;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.level.NaturalSpawner;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.MobSpawnSettings;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.LevelChunk;
import net.nostalgica.modernica.perf.mob_spawning.MobSpawningEntityType;

/**
 * Computing a mob-spawn cost involves resolving the rough biome at the position and then two map
 * lookups, for every candidate mob every spawn attempt - even though most entity types never appear in
 * any biome's cost table at all and are guaranteed {@code null}. Checking
 * {@link MobSpawningEntityType#mfh$hasAnyBiomeCost} first (a single boolean read) lets that whole chain
 * be skipped for the common case without changing the result.
 */
@Mixin(NaturalSpawner.class)
abstract class NaturalSpawnerMixin {

    @Shadow
    static Biome getRoughBiome(BlockPos pos, ChunkAccess chunk) {
        return null;
    }

    @Redirect(method = {"lambda$createState$0"}, at = @At(value = "INVOKE", target = "Lnet/minecraft/world/level/NaturalSpawner;getRoughBiome(Lnet/minecraft/core/BlockPos;Lnet/minecraft/world/level/chunk/ChunkAccess;)Lnet/minecraft/world/level/biome/Biome;"))
    private static Biome mfh$delayRoughBiome(BlockPos pos, ChunkAccess chunk) {
        return null;
    }

    @Redirect(method = {"lambda$createState$0"}, at = @At(value = "INVOKE", target = "Lnet/minecraft/world/level/biome/Biome;getMobSettings()Lnet/minecraft/world/level/biome/MobSpawnSettings;"))
    private static MobSpawnSettings mfh$delayMobSpawnSettings(Biome biome) {
        return null;
    }

    @Redirect(method = {"lambda$createState$0"}, at = @At(value = "INVOKE", target = "Lnet/minecraft/world/level/biome/MobSpawnSettings;getMobSpawnCost(Lnet/minecraft/world/entity/EntityType;)Lnet/minecraft/world/level/biome/MobSpawnSettings$MobSpawnCost;"))
    private static MobSpawnSettings.MobSpawnCost mfh$avoidLookupIfNoCost(MobSpawnSettings alwaysNull, EntityType<?> type,
                                                                          @Local(ordinal = 0, argsOnly = true) BlockPos pos,
                                                                          @Local(ordinal = 0, argsOnly = true) LevelChunk chunk) {
        if (!((MobSpawningEntityType) type).mfh$hasAnyBiomeCost()) {
            return null;
        }
        return getRoughBiome(pos, chunk).getMobSettings().getMobSpawnCost(type);
    }
}
