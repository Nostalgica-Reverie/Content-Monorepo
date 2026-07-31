package net.nostalgica.modernica.perf;

import net.minecraft.client.model.geom.ModelPart;

import java.util.List;
import java.util.concurrent.ConcurrentHashMap;
import java.util.function.Supplier;

/** Resource-reload scoped shared cubes for equivalent entity model definitions. */
public final class CompactEntityModelCache {
    private static final ConcurrentHashMap<List<Object>, ModelPart.Cube> CUBES = new ConcurrentHashMap<>();

    private CompactEntityModelCache() {}

    public static ModelPart.Cube getOrCreate(List<Object> key, Supplier<ModelPart.Cube> factory) {
        return CUBES.computeIfAbsent(key, ignored -> factory.get());
    }

    public static void clear() {
        CUBES.clear();
    }
}
