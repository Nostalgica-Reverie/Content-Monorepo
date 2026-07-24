package net.nostalgica.modernica.perf.mob_spawning;

/** Implemented by {@code EntityTypeMixin}: lets {@code NaturalSpawnerMixin} skip the (surprisingly not
 * free - a biome lookup plus two map gets) mob-spawn-cost check entirely for entity types that no biome
 * assigns a cost to in the first place, which is most of them. */
public interface MobSpawningEntityType {
    boolean mfh$hasAnyBiomeCost();

    void mfh$setHasBiomeCost();
}
