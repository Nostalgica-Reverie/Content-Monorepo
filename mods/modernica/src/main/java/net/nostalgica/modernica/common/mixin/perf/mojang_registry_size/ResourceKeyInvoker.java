package net.nostalgica.modernica.common.mixin.perf.mojang_registry_size;

import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

@Mixin(ResourceKey.class)
public interface ResourceKeyInvoker {
    @Invoker("<init>")
    static ResourceKey mfix$invokeCtor(Identifier parent, Identifier identifier) {
        throw new IllegalStateException("Invoker mixin failed?");
    }
}