package net.nostalgica.modernica.common.mixin.perf.fast_palette;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.core.IdMap;
import net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap;
import net.nostalgica.modernica.perf.fast_palette.FastPalette;
import net.nostalgica.modernica.perf.fast_palette.FastPaletteData;

/**
 * {@code byId} backs {@code CrudeIncrementalIntIdentityHashBiMap} the same way {@code LinearPalette}
 * backs itself, except {@code grow()} replaces the array wholesale when it needs more room - so whoever
 * cached the old array reference (the owning {@link PalettedContainerDataMixin}) needs to be told about
 * the new one, or it'll keep reading a stale, undersized array.
 */
@Mixin(CrudeIncrementalIntIdentityHashBiMap.class)
abstract class CrudeIncrementalIntIdentityHashBiMapMixin<K> implements IdMap<K>, FastPalette<K> {

    @Shadow
    private K[] byId;

    @Unique
    private FastPaletteData<K> mfh$owner;

    @Override
    public final K[] mfh$getRawPalette(FastPaletteData<K> owner) {
        this.mfh$owner = owner;
        return this.byId;
    }

    @Inject(method = "grow", at = @At("RETURN"))
    private void mfh$notifyOwnerOnGrow(CallbackInfo ci) {
        FastPaletteData<K> owner = this.mfh$owner;
        if (owner != null) {
            owner.mfh$setCachedPalette(this.byId);
        }
    }
}
