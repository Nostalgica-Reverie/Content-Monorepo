package net.nostalgica.modernica.perf.fast_palette;

/** Implemented by {@code PalettedContainerDataMixin}: caches the owning palette's flat {@code T[]}
 * (from {@link FastPalette#mfh$getRawPalette}) so it doesn't need to be re-fetched on every read. */
public interface FastPaletteData<T> {
    T[] mfh$getCachedPalette();

    void mfh$setCachedPalette(T[] palette);
}
