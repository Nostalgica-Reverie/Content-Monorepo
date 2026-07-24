package net.nostalgica.modernica.common.mixin.perf.fast_palette;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import net.minecraft.world.level.chunk.PalettedContainer;
import net.nostalgica.modernica.perf.fast_palette.FastPalette;
import net.nostalgica.modernica.perf.fast_palette.FastPaletteData;

/**
 * {@code PalettedContainer#get}/{@code #getAndSet} resolve a storage index to a value via
 * {@code palette.valueFor(int)}, a virtual call whose cost varies by palette implementation (a bounds
 * check plus, for {@code HashMapPalette}, walking its bimap). Every palette type this can help
 * ({@code LinearPalette}, {@code SingleValuePalette}, {@code HashMapPalette} - see the {@code fast_palette}
 * package's other mixins) already has, or can cheaply expose, a flat {@code T[]} array mirroring that
 * mapping; this caches a pointer to that array on the container's {@code Data} record (refreshed on
 * construction, resize, and read - the three points where the palette or its backing array can change)
 * so {@code get}/{@code getAndSet} can index straight into it. {@code GlobalPalette} (the direct/global
 * fallback, used once a section has enough distinct states that a small local palette isn't worth it)
 * doesn't opt in, so it keeps using the vanilla path automatically.
 */
@Mixin(PalettedContainer.class)
abstract class PalettedContainerMixin<T> {

    @Shadow
    public volatile PalettedContainer.Data<T> data;

    @Unique
    @SuppressWarnings("unchecked")
    private void mfh$refreshCachedPalette(PalettedContainer.Data<T> data) {
        if (data == null) {
            return;
        }
        FastPaletteData<T> dataAccess = (FastPaletteData<T>) (Object) data;
        FastPalette<T> palette = (FastPalette<T>) data.palette();
        dataAccess.mfh$setCachedPalette(palette.mfh$getRawPalette(dataAccess));
    }

    @Inject(method = "<init>*", at = @At("RETURN"), require = 3)
    private void mfh$onConstruct(CallbackInfo ci) {
        this.mfh$refreshCachedPalette(this.data);
    }

    @Inject(method = "onResize", at = @At("RETURN"))
    private void mfh$onResize(CallbackInfoReturnable<Integer> cir) {
        this.mfh$refreshCachedPalette(this.data);
    }

    @Inject(method = "read", at = @At("RETURN"))
    private void mfh$onRead(CallbackInfo ci) {
        this.mfh$refreshCachedPalette(this.data);
    }

    @Unique
    @SuppressWarnings("unchecked")
    private T mfh$resolve(PalettedContainer.Data<T> data, int paletteIndex) {
        T[] cached = ((FastPaletteData<T>) (Object) data).mfh$getCachedPalette();
        if (cached == null) {
            return data.palette().valueFor(paletteIndex);
        }
        T value = cached[paletteIndex];
        if (value == null) {
            throw new IllegalArgumentException("Palette index out of bounds");
        }
        return value;
    }

    @Overwrite
    public T getAndSet(int index, T value) {
        PalettedContainer.Data<T> data = this.data;
        int paletteIndex = data.palette().idFor(value, (net.minecraft.world.level.chunk.PaletteResize<T>) (Object) this);
        int previous = data.storage().getAndSet(index, paletteIndex);
        return this.mfh$resolve(data, previous);
    }

    @Overwrite
    public T get(int index) {
        PalettedContainer.Data<T> data = this.data;
        return this.mfh$resolve(data, data.storage().get(index));
    }
}
