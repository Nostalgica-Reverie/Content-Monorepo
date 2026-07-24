package net.nostalgica.modernica.common.mixin.perf.deduplicate_nbt_strings;

import it.unimi.dsi.fastutil.objects.Object2ObjectMap;
import it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.Tag;
import net.nostalgica.modernica.dedup.NbtCaches;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Map;

/**
 * Ported from Hydrogen's {@code mixin.nbt.MixinNbtCompound} (no ModernFix equivalent).
 */
@Mixin(CompoundTag.class)
public class MixinNbtCompound {
    @Mutable
    @Shadow
    @Final
    private Map<String, Tag> tags;

    @Inject(method = "<init>(Ljava/util/Map;)V", at = @At("RETURN"))
    private void reinit(Map<String, Tag> tags, CallbackInfo ci) {
        if (tags instanceof Object2ObjectMap) {
            this.tags = tags;
            return;
        }

        Object2ObjectOpenHashMap<String, Tag> deduped = new Object2ObjectOpenHashMap<>(tags.size());

        for (Map.Entry<String, Tag> entry : tags.entrySet()) {
            deduped.put(NbtCaches.KEYS.deduplicate(entry.getKey()), entry.getValue());
        }

        this.tags = deduped;
    }
}
