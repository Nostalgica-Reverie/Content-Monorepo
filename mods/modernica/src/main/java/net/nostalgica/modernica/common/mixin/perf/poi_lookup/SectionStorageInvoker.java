package net.nostalgica.modernica.common.mixin.perf.poi_lookup;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

import net.minecraft.world.level.chunk.storage.SectionStorage;

import java.util.Optional;

@Mixin(SectionStorage.class)
public interface SectionStorageInvoker<R> {
    @Invoker("get")
    Optional<R> mfh$get(long key);

    @Invoker("getOrLoad")
    Optional<R> mfh$getOrLoad(long key);
}
