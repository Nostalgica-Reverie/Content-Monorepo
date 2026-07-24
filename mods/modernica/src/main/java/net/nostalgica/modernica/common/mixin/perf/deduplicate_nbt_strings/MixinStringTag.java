package net.nostalgica.modernica.common.mixin.perf.deduplicate_nbt_strings;

import net.minecraft.nbt.StringTag;
import net.nostalgica.modernica.dedup.NbtCaches;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Ported from Hydrogen's {@code mixin.nbt.MixinStringTag} (no ModernFix equivalent).
 */
@Mixin(StringTag.class)
public class MixinStringTag {
    @Inject(method = "valueOf", at = @At("RETURN"), cancellable = true)
    private static void dedupe(String data, CallbackInfoReturnable<StringTag> cir) {
        cir.setReturnValue(NbtCaches.STRING_TAGS.deduplicate(cir.getReturnValue()));
    }
}
