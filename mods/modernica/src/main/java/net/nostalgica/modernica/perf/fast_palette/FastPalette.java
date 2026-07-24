package net.nostalgica.modernica.perf.fast_palette;

/**
 * Implemented by every {@code Palette} type via {@code PaletteMixin}, and overridden by the specific
 * palette implementations that already keep (or can cheaply expose) a flat {@code T[]} backing their
 * id-to-value mapping. Types that don't override this (the default {@code null}) simply aren't
 * fast-pathed - {@code PalettedContainerMixin} falls back to the normal {@code palette.valueFor(int)}
 * call for them.
 */
public interface FastPalette<T> {
    default T[] mfh$getRawPalette(FastPaletteData<T> owner) {
        return null;
    }
}
