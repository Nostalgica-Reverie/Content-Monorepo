package net.nostalgica.modernica.common.mixin.perf.block_counting;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import net.minecraft.world.level.chunk.PalettedContainer;

@Mixin(PalettedContainer.class)
public interface PalettedContainerDataAccessor<T> {
    @Accessor("data")
    PalettedContainer.Data<T> mfh$getData();
}
