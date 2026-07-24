package net.nostalgica.modernica.common.mixin.perf.mojang_registry_size;

import it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.Map;

@Mixin(ResourceKey.class)
public class ResourceKeyMixin<T> {
    private static final Map<Identifier, Map<Identifier, ResourceKey<?>>> INTERNING_MAP = new Object2ObjectOpenHashMap<>();
    @Inject(method = "create(Lnet/minecraft/resources/Identifier;Lnet/minecraft/resources/Identifier;)Lnet/minecraft/resources/ResourceKey;", at = @At("HEAD"), cancellable = true)
    private static <T> void createEfficient(Identifier parent, Identifier identifier, CallbackInfoReturnable<ResourceKey<T>> cir) {
        synchronized (ResourceKey.class) {
            Map<Identifier, ResourceKey<?>> keys = INTERNING_MAP.computeIfAbsent(parent, k -> new Object2ObjectOpenHashMap<>());
            ResourceKey<?> key = keys.get(identifier);
            if(key == null) {
                key = ResourceKeyInvoker.mfix$invokeCtor(parent, identifier);
                keys.put(identifier, key);
            }
            cir.setReturnValue((ResourceKey<T>) key);
        }
    }
}