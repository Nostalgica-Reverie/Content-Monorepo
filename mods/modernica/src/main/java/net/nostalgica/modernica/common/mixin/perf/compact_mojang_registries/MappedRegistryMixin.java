package net.nostalgica.modernica.common.mixin.perf.compact_mojang_registries;

import net.minecraft.core.MappedRegistry;
import net.minecraft.core.RegistrationInfo;
import net.minecraft.resources.ResourceKey;
import net.nostalgica.modernica.annotation.IgnoreOutsideDev;
import net.nostalgica.modernica.registry.LifecycleMap;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Map;

@Mixin(MappedRegistry.class)
@IgnoreOutsideDev
public abstract class MappedRegistryMixin<T> {
    @Shadow
    @Final
    @Mutable
    private Map<ResourceKey<T>, RegistrationInfo> registrationInfos;

    @Inject(method = "<init>", at = @At("RETURN"))
    private void replaceStorage(CallbackInfo ci) {
        this.registrationInfos = new LifecycleMap<>();
    }
}
