package net.nostalgica.modernica.common.mixin.perf.release_protochunks;

import net.minecraft.server.level.GenerationChunkHolder;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import java.util.concurrent.atomic.AtomicReference;
import java.util.concurrent.atomic.AtomicReferenceArray;

@Mixin(GenerationChunkHolder.class)
public interface GenerationChunkHolderAccessor {
    @Accessor("futures")
    AtomicReferenceArray<Object> mfix$getFutures();

    @Accessor("startedWork")
    AtomicReference<Object> mfix$getStartedWork();
}