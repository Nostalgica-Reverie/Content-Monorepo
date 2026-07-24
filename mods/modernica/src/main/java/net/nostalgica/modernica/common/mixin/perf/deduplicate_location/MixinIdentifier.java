package net.nostalgica.modernica.common.mixin.perf.deduplicate_location;

import net.minecraft.resources.Identifier;
import net.nostalgica.modernica.dedup.IdentifierCaches;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Identifier.class)
public class MixinIdentifier {
    @Mutable
    @Shadow
    @Final
    private String namespace;

    @Mutable
    @Shadow
    @Final
    private String path;

    @Inject(method = "<init>", at = @At("RETURN"))
    private void reinit(String string, String string2, CallbackInfo ci) {
        this.namespace = IdentifierCaches.NAMESPACES.deduplicate(this.namespace);
        this.path = IdentifierCaches.PATH.deduplicate(this.path);
    }
}
