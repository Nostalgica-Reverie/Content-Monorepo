package net.nostalgica.modernica.common.mixin.perf.getblock;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

import net.minecraft.world.level.chunk.PalettedContainer;

@Mixin(PalettedContainer.class)
public interface PalettedContainerGetInvoker<T> {
    @Invoker("get")
    T mfh$get(int index);
}
