package net.nostalgica.modernica.perf.block_counting;

import it.unimi.dsi.fastutil.shorts.ShortArrayList;

/** Implemented by {@code LevelChunkSectionMixin}. The returned list holds each randomly-tickable block's
 * position packed as {@code x | z<<4 | y<<8}, so random ticking can pick straight from it instead of
 * rolling positions blind and checking each one. */
public interface BlockCountingChunkSection {
    ShortArrayList mfh$getTickingBlockPositions();
}
