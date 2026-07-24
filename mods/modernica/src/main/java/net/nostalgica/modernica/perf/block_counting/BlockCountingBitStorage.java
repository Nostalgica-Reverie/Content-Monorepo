package net.nostalgica.modernica.perf.block_counting;

import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.shorts.ShortArrayList;

/** Implemented by {@code SimpleBitStorageMixin}/{@code ZeroBitStorageMixin}: bulk-decodes every entry in
 * one pass into a palette-index -> "which slots hold it" histogram, instead of the naive per-slot
 * {@code get(int)} loop {@code LevelChunkSectionMixin#recalcBlockCounts} would otherwise need. */
public interface BlockCountingBitStorage {
    Int2ObjectOpenHashMap<ShortArrayList> mfh$countEntries();
}
