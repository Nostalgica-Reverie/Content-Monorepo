package net.nostalgica.modernica.perf.block_counting;

import it.unimi.dsi.fastutil.shorts.ShortArrayList;

/** Implemented by {@code LevelChunkSectionMixin}. The returned list holds every position whose block or
 * fluid can random-tick, packed as {@code x | z<<4 | y<<8}. Random ticking can therefore pick straight
 * from relevant positions instead of rolling blindly, while still preserving vanilla fluid ticks. */
public interface BlockCountingChunkSection {
    ShortArrayList mfh$getTickingBlockPositions();
}
