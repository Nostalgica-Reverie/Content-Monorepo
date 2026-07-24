package net.nostalgica.modernica.perf.poi_lookup;

import com.mojang.datafixers.util.Pair;
import it.unimi.dsi.fastutil.doubles.Double2ObjectMap;
import it.unimi.dsi.fastutil.doubles.Double2ObjectRBTreeMap;
import it.unimi.dsi.fastutil.longs.LongArrayFIFOQueue;
import it.unimi.dsi.fastutil.longs.LongOpenHashSet;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.SectionPos;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.ai.village.poi.PoiManager;
import net.minecraft.world.entity.ai.village.poi.PoiRecord;
import net.minecraft.world.entity.ai.village.poi.PoiSection;
import net.minecraft.world.entity.ai.village.poi.PoiType;
import net.nostalgica.modernica.common.mixin.perf.poi_lookup.PoiSectionByTypeAccessor;
import net.nostalgica.modernica.common.mixin.perf.poi_lookup.SectionStorageInvoker;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.function.BiPredicate;
import java.util.function.Predicate;

/**
 * Vanilla's POI search methods are built from chained {@code Stream} operations over every loaded POI
 * section in the search's bounding box - for wide-radius searches (villager gossip/work lookups can
 * range out to 48+ blocks) that's a lot of stream overhead for what's fundamentally a 3D grid walk.
 * These reimplement the same searches as plain loops over the section grid, with two kinds of pruning
 * depending on what's being searched for:
 * <ul>
 *   <li>{@code findAny*} (used for "does anything match" / take-first searches): iterates the bounding
 *   box directly and returns as soon as {@code max} results are found - vanilla's own iteration order
 *   (increasing Z, then X, then Y within a chunk) is preserved exactly since callers can depend on it.</li>
 *   <li>{@code findNearest*}/{@code findClosest*} (ranked "closest N" searches): expands outward from the
 *   source position one section-ring at a time via a FIFO queue, pruning any section whose closest
 *   possible point is already farther than the current worst kept result - this can finish long before
 *   the whole bounding box is walked. Because the search order no longer matches vanilla's raster order,
 *   results are re-sorted by that same raster order afterward wherever ties need breaking the way vanilla
 *   would.</li>
 * </ul>
 * All of this only changes how the search is carried out - every method here returns exactly what
 * vanilla's original implementation would for the same inputs.
 */
public final class PoiSearch {
    private PoiSearch() {}

    public static final boolean LOAD_FOR_SEARCHING = true;

    @SuppressWarnings("unchecked")
    private static Optional<PoiSection> section(PoiManager poiStorage, long key, boolean load) {
        SectionStorageInvoker<PoiSection> storage = (SectionStorageInvoker<PoiSection>) (Object) poiStorage;
        return load ? storage.mfh$getOrLoad(key) : storage.mfh$get(key);
    }

    public static int compareDistances(double d1, double d2) {
        return (int) Math.signum(d1 - d2);
    }

    private static long distSq(BlockPos a, BlockPos b) {
        long dx = (long) a.getX() - b.getX();
        long dy = (long) a.getY() - b.getY();
        long dz = (long) a.getZ() - b.getZ();
        return dx * dx + dy * dy + dz * dz;
    }

    public static int compareDistances(BlockPos center, BlockPos p1, BlockPos p2) {
        return Long.compareUnsigned(distSq(p1, center), distSq(p2, center));
    }

    private static double clamp(double val, double min, double max) {
        return val < min ? min : Math.min(val, max);
    }

    /** Closest possible squared distance from {@code (px,py,pz)} to the given axis-aligned box, 0 if inside. */
    private static double closestDistSq(double minX, double minY, double minZ, double maxX, double maxY, double maxZ,
                                         double px, double py, double pz) {
        if (px >= minX && px <= maxX && py >= minY && py <= maxY && pz >= minZ && pz <= maxZ) {
            return 0.0;
        }
        double halfX = (maxX - minX) / 2.0;
        double halfY = (maxY - minY) / 2.0;
        double halfZ = (maxZ - minZ) / 2.0;
        double centerX = (minX + maxX) / 2.0;
        double centerY = (minY + maxY) / 2.0;
        double centerZ = (minZ + maxZ) / 2.0;

        double dx = px - (clamp(px - centerX, -halfX, halfX) + centerX);
        double dy = py - (clamp(py - centerY, -halfY, halfY) + centerY);
        double dz = pz - (clamp(pz - centerZ, -halfZ, halfZ) + centerZ);
        return dx * dx + dy * dy + dz * dz;
    }

    /** Re-orders results to match vanilla's exact section-then-record iteration order (ascending chunk Z,
     * then chunk X, then section Y) wherever the caller's own comparator considers two results tied. */
    private static void sortLikeVanillaOrder(List<PoiRecord> records) {
        records.sort((r1, r2) -> {
            BlockPos p1 = r1.getPos();
            BlockPos p2 = r2.getPos();
            int cz1 = p1.getZ() >> 4;
            int cz2 = p2.getZ() >> 4;
            if (cz1 != cz2) {
                return Integer.compare(cz1, cz2);
            }
            int cx1 = p1.getX() >> 4;
            int cx2 = p2.getX() >> 4;
            if (cx1 != cx2) {
                return Integer.compare(cx1, cx2);
            }
            return Integer.compare(p1.getY() >> 4, p2.getY() >> 4);
        });
    }

    private static void forEachNeighborRing(int x, int y, int z, LongOpenHashSet seen, LongArrayFIFOQueue queue) {
        for (int dz = -1; dz <= 1; dz++) {
            for (int dx = -1; dx <= 1; dx++) {
                for (int dy = -1; dy <= 1; dy++) {
                    // only cardinal (single-axis) neighbors, matching a von Neumann neighborhood
                    if ((dx & 1) + (dy & 1) + (dz & 1) != 1) {
                        continue;
                    }
                    long key = SectionPos.asLong(x + dx, y + dy, z + dz);
                    if (seen.add(key)) {
                        queue.enqueue(key);
                    }
                }
            }
        }
    }

    /** Ranked "N closest" section-ring search shared by {@link #findClosestPoiDataRecords} and
     * {@link #findNearestPoiRecords}. */
    private interface SectionVisitor {
        /** Return the updated "can't beat this" threshold (squared distance) after considering this
         * section's records, or unchanged if nothing here helped. */
        double visit(PoiSection section, double currentWorst);
    }

    private static void expandingSectionSearch(PoiManager storage, BlockPos source, int range, int minSectionY, int maxSectionY,
                                                boolean load, double initialWorst, SectionVisitor visitor) {
        int lowerX = Mth.floor(source.getX() - range) >> 4;
        int lowerZ = Mth.floor(source.getZ() - range) >> 4;
        int upperX = Mth.floor(source.getX() + range) >> 4;
        int upperZ = Mth.floor(source.getZ() + range) >> 4;

        int centerX = source.getX() >> 4;
        int centerY = Mth.clamp(source.getY() >> 4, minSectionY, maxSectionY);
        int centerZ = source.getZ() >> 4;

        LongArrayFIFOQueue queue = new LongArrayFIFOQueue();
        LongOpenHashSet seen = new LongOpenHashSet();
        long centerKey = SectionPos.asLong(centerX, centerY, centerZ);
        seen.add(centerKey);
        queue.enqueue(centerKey);

        double worst = initialWorst;

        while (!queue.isEmpty()) {
            long key = queue.dequeueLong();
            int sx = SectionPos.x(key);
            int sy = SectionPos.y(key);
            int sz = SectionPos.z(key);

            if (sx < lowerX || sx > upperX || sy < minSectionY || sy > maxSectionY || sz < lowerZ || sz > upperZ) {
                continue;
            }

            double sectionDistSq = closestDistSq(sx << 4, sy << 4, sz << 4, (sx << 4) | 15, (sy << 4) | 15, (sz << 4) | 15,
                    source.getX(), source.getY(), source.getZ());
            if (sectionDistSq > worst) {
                continue;
            }

            forEachNeighborRing(sx, sy, sz, seen, queue);

            Optional<PoiSection> optional = section(storage, key, load);
            if (optional == null || optional.isEmpty()) {
                continue;
            }

            worst = visitor.visit(optional.get(), worst);
        }
    }

    // === "closest, exact match, no cap" - used by PortalForcerMixin ===

    public static void findClosestPoiDataRecords(PoiManager poiStorage, Predicate<Holder<PoiType>> typePredicate,
                                           BiPredicate<Holder<PoiType>, BlockPos> positionPredicate, BlockPos source,
                                           int range, double maxDistanceSquared, PoiManager.Occupancy occupancy,
                                           boolean load, int minSectionY, int maxSectionY, List<PoiRecord> ret) {
        Predicate<? super PoiRecord> occupancyFilter = occupancy.getTest();
        List<PoiRecord> closest = new ArrayList<>();
        double[] closestDistSq = {maxDistanceSquared};

        expandingSectionSearch(poiStorage, source, range, minSectionY, maxSectionY, load, maxDistanceSquared, (section, worst) -> {
            Map<Holder<PoiType>, Set<PoiRecord>> byType = ((PoiSectionByTypeAccessor) section).mfh$getByType();
            if (byType.isEmpty()) {
                return worst;
            }
            for (Map.Entry<Holder<PoiType>, Set<PoiRecord>> entry : byType.entrySet()) {
                if (!typePredicate.test(entry.getKey())) {
                    continue;
                }
                for (PoiRecord record : entry.getValue()) {
                    if (!occupancyFilter.test(record)) {
                        continue;
                    }
                    BlockPos pos = record.getPos();
                    if (Math.abs(pos.getX() - source.getX()) > range || Math.abs(pos.getZ() - source.getZ()) > range) {
                        continue;
                    }
                    double dist = pos.distSqr(source);
                    if (dist > closestDistSq[0]) {
                        continue;
                    }
                    if (positionPredicate != null && !positionPredicate.test(record.getPoiType(), pos)) {
                        continue;
                    }
                    if (dist < closestDistSq[0]) {
                        closest.clear();
                        closestDistSq[0] = dist;
                    }
                    closest.add(record);
                }
            }
            return closestDistSq[0];
        });

        sortLikeVanillaOrder(closest);
        ret.addAll(closest);
    }

    // === "closest N, ranked" - used by AcquirePoiMixin ===

    public static BlockPos findNearestPoiPosition(PoiManager poiStorage, Predicate<Holder<PoiType>> typePredicate, Predicate<BlockPos> positionPredicate,
                                            BlockPos source, int range, double maxDistanceSquared, PoiManager.Occupancy occupancy,
                                            boolean load, int minSectionY, int maxSectionY) {
        PoiRecord record = findNearestPoiRecord(poiStorage, typePredicate, positionPredicate, source, range, maxDistanceSquared, occupancy, load, minSectionY, maxSectionY);
        return record == null ? null : record.getPos();
    }

    public static void findNearestPoiPositions(PoiManager poiStorage, Predicate<Holder<PoiType>> typePredicate, Predicate<BlockPos> positionPredicate,
                                         BlockPos source, int range, double maxDistanceSquared, PoiManager.Occupancy occupancy, boolean load,
                                         int minSectionY, int maxSectionY, int max, List<Pair<Holder<PoiType>, BlockPos>> ret) {
        Set<BlockPos> seen = new HashSet<>();
        Predicate<BlockPos> dedupe = pos -> (positionPredicate == null || positionPredicate.test(pos)) && seen.add(pos.immutable());

        List<PoiRecord> found = new ArrayList<>();
        findNearestPoiRecords(poiStorage, typePredicate, dedupe, source, range, maxDistanceSquared, occupancy, load, minSectionY, maxSectionY, max, found);
        for (PoiRecord record : found) {
            ret.add(Pair.of(record.getPoiType(), record.getPos()));
        }
    }

    public static PoiRecord findNearestPoiRecord(PoiManager poiStorage, Predicate<Holder<PoiType>> typePredicate, Predicate<BlockPos> positionPredicate,
                                           BlockPos source, int range, double maxDistanceSquared, PoiManager.Occupancy occupancy, boolean load,
                                           int minSectionY, int maxSectionY) {
        List<PoiRecord> ret = new ArrayList<>();
        findNearestPoiRecords(poiStorage, typePredicate, positionPredicate, source, range, maxDistanceSquared, occupancy, load, minSectionY, maxSectionY, 1, ret);
        return ret.isEmpty() ? null : ret.get(0);
    }

    public static void findNearestPoiRecords(PoiManager poiStorage, Predicate<Holder<PoiType>> typePredicate, Predicate<BlockPos> positionPredicate,
                                       BlockPos source, int range, double maxDistanceSquared, PoiManager.Occupancy occupancy, boolean load,
                                       int minSectionY, int maxSectionY, int max, List<PoiRecord> ret) {
        Predicate<? super PoiRecord> occupancyFilter = occupancy.getTest();

        Double2ObjectRBTreeMap<List<PoiRecord>> byDistance = new Double2ObjectRBTreeMap<>();
        int[] total = {0};
        double[] worstKept = {maxDistanceSquared};

        expandingSectionSearch(poiStorage, source, range, minSectionY, maxSectionY, load,
                maxDistanceSquared, (section, currentWorst) -> {
            Map<Holder<PoiType>, Set<PoiRecord>> byType = ((PoiSectionByTypeAccessor) section).mfh$getByType();
            if (byType.isEmpty()) {
                return currentWorst;
            }
            for (Map.Entry<Holder<PoiType>, Set<PoiRecord>> entry : byType.entrySet()) {
                if (!typePredicate.test(entry.getKey())) {
                    continue;
                }
                for (PoiRecord record : entry.getValue()) {
                    if (!occupancyFilter.test(record)) {
                        continue;
                    }
                    BlockPos pos = record.getPos();
                    if (Math.abs(pos.getX() - source.getX()) > range || Math.abs(pos.getZ() - source.getZ()) > range) {
                        continue;
                    }
                    double dist = pos.distSqr(source);
                    if (dist > maxDistanceSquared) {
                        continue;
                    }
                    if (dist > worstKept[0] && total[0] >= max) {
                        continue;
                    }
                    if (positionPredicate != null && !positionPredicate.test(pos)) {
                        continue;
                    }
                    if (dist > worstKept[0]) {
                        worstKept[0] = dist;
                    }
                    byDistance.computeIfAbsent(dist, unused -> new ArrayList<>()).add(record);
                    if (++total[0] >= max && byDistance.size() >= 2) {
                        // trim the farthest distance bucket once every remaining slot is already
                        // spoken for by closer entries
                        int keptSoFar = 0;
                        Iterator<Double2ObjectMap.Entry<List<PoiRecord>>> it = byDistance.double2ObjectEntrySet().iterator();
                        double nextWorst = 0.0;
                        for (int i = 0, len = byDistance.size() - 1; i < len; i++) {
                            Double2ObjectMap.Entry<List<PoiRecord>> bucket = it.next();
                            keptSoFar += bucket.getValue().size();
                            nextWorst = bucket.getDoubleKey();
                        }
                        if (keptSoFar >= max) {
                            Double2ObjectMap.Entry<List<PoiRecord>> farthest = it.next();
                            total[0] -= farthest.getValue().size();
                            it.remove();
                            worstKept[0] = nextWorst;
                        }
                    }
                }
            }
            return total[0] >= max ? worstKept[0] : maxDistanceSquared;
        });

        List<PoiRecord> flattened = new ArrayList<>();
        for (List<PoiRecord> bucket : byDistance.values()) {
            flattened.addAll(bucket);
        }
        sortLikeVanillaOrder(flattened);
        for (int i = flattened.size() - 1; i >= max; i--) {
            flattened.remove(i);
        }
        ret.addAll(flattened);
    }

    // === "any match, take first N" - used by PoiManager's own routed methods ===

    public static BlockPos findAnyPoiPosition(PoiManager poiStorage, Predicate<Holder<PoiType>> typePredicate, Predicate<BlockPos> positionPredicate,
                                        BlockPos source, int range, PoiManager.Occupancy occupancy, boolean load, int minSectionY, int maxSectionY) {
        PoiRecord record = findAnyPoiRecord(poiStorage, typePredicate, positionPredicate, source, range, (double) ((long) range * range), occupancy, load, minSectionY, maxSectionY);
        return record == null ? null : record.getPos();
    }

    public static void findAnyPoiPositions(PoiManager poiStorage, Predicate<Holder<PoiType>> typePredicate, Predicate<BlockPos> positionPredicate,
                                     BlockPos source, int range, PoiManager.Occupancy occupancy, boolean load, int minSectionY, int maxSectionY,
                                     int max, List<Pair<Holder<PoiType>, BlockPos>> ret) {
        Set<BlockPos> seen = new HashSet<>();
        Predicate<BlockPos> dedupe = pos -> (positionPredicate == null || positionPredicate.test(pos)) && seen.add(pos.immutable());

        List<PoiRecord> found = new ArrayList<>();
        findAnyPoiRecords(poiStorage, typePredicate, dedupe, source, range, (double) ((long) range * range), occupancy, load, minSectionY, maxSectionY, max, found);
        for (PoiRecord record : found) {
            ret.add(Pair.of(record.getPoiType(), record.getPos()));
        }
    }

    public static PoiRecord findAnyPoiRecord(PoiManager poiStorage, Predicate<Holder<PoiType>> typePredicate, Predicate<BlockPos> positionPredicate,
                                       BlockPos source, int range, double maxDistanceSquared, PoiManager.Occupancy occupancy, boolean load,
                                       int minSectionY, int maxSectionY) {
        List<PoiRecord> ret = new ArrayList<>();
        findAnyPoiRecords(poiStorage, typePredicate, positionPredicate, source, range, maxDistanceSquared, occupancy, load, minSectionY, maxSectionY, 1, ret);
        return ret.isEmpty() ? null : ret.get(0);
    }

    public static void findAnyPoiRecords(PoiManager poiStorage, Predicate<Holder<PoiType>> typePredicate, Predicate<BlockPos> positionPredicate,
                                   BlockPos source, int range, double maxDistanceSquared, PoiManager.Occupancy occupancy, boolean load,
                                   int minSectionY, int maxSectionY, int max, List<PoiRecord> ret) {
        Predicate<? super PoiRecord> occupancyFilter = occupancy.getTest();
        int[] added = {0};

        int lowerX = Mth.floor(source.getX() - range) >> 4;
        int lowerY = Math.max(minSectionY, Mth.floor(source.getY() - range) >> 4);
        int lowerZ = Mth.floor(source.getZ() - range) >> 4;
        int upperX = Mth.floor(source.getX() + range) >> 4;
        int upperY = Math.min(maxSectionY, Mth.floor(source.getY() + range) >> 4);
        int upperZ = Mth.floor(source.getZ() + range) >> 4;

        // matches vanilla's own raster order: Z outermost, then X, then Y
        for (int cz = lowerZ; cz <= upperZ; cz++) {
            for (int cx = lowerX; cx <= upperX; cx++) {
                for (int cy = lowerY; cy <= upperY; cy++) {
                    long key = SectionPos.asLong(cx, cy, cz);
                    Optional<PoiSection> optional = section(poiStorage, key, load);
                    PoiSection section = optional == null ? null : optional.orElse(null);
                    if (section == null) {
                        continue;
                    }

                    Map<Holder<PoiType>, Set<PoiRecord>> byType = ((PoiSectionByTypeAccessor) section).mfh$getByType();
                    if (byType.isEmpty()) {
                        continue;
                    }

                    for (Map.Entry<Holder<PoiType>, Set<PoiRecord>> entry : byType.entrySet()) {
                        if (!typePredicate.test(entry.getKey())) {
                            continue;
                        }
                        for (PoiRecord record : entry.getValue()) {
                            if (!occupancyFilter.test(record)) {
                                continue;
                            }
                            BlockPos pos = record.getPos();
                            if (Math.abs(pos.getX() - source.getX()) > range || Math.abs(pos.getZ() - source.getZ()) > range) {
                                continue;
                            }
                            if (pos.distSqr(source) > maxDistanceSquared) {
                                continue;
                            }
                            if (positionPredicate != null && !positionPredicate.test(pos)) {
                                continue;
                            }
                            ret.add(record);
                            if (++added[0] >= max) {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    public static PoiRecord findAnyPoiRecord(PoiManager poiStorage, Predicate<Holder<PoiType>> typePredicate, BiPredicate<Holder<PoiType>, BlockPos> positionPredicate,
                                       BlockPos source, int range, double maxDistanceSquared, PoiManager.Occupancy occupancy, boolean load,
                                       int minSectionY, int maxSectionY) {
        List<PoiRecord> ret = new ArrayList<>();
        findAnyPoiRecords(poiStorage, typePredicate, positionPredicate, source, range, maxDistanceSquared, occupancy, load, minSectionY, maxSectionY, 1, ret);
        return ret.isEmpty() ? null : ret.get(0);
    }

    public static void findAnyPoiRecords(PoiManager poiStorage, Predicate<Holder<PoiType>> typePredicate, BiPredicate<Holder<PoiType>, BlockPos> positionPredicate,
                                   BlockPos source, int range, double maxDistanceSquared, PoiManager.Occupancy occupancy, boolean load,
                                   int minSectionY, int maxSectionY, int max, List<PoiRecord> ret) {
        Predicate<? super PoiRecord> occupancyFilter = occupancy.getTest();
        int[] added = {0};

        int lowerX = Mth.floor(source.getX() - range) >> 4;
        int lowerY = Math.max(minSectionY, Mth.floor(source.getY() - range) >> 4);
        int lowerZ = Mth.floor(source.getZ() - range) >> 4;
        int upperX = Mth.floor(source.getX() + range) >> 4;
        int upperY = Math.min(maxSectionY, Mth.floor(source.getY() + range) >> 4);
        int upperZ = Mth.floor(source.getZ() + range) >> 4;

        for (int cz = lowerZ; cz <= upperZ; cz++) {
            for (int cx = lowerX; cx <= upperX; cx++) {
                for (int cy = lowerY; cy <= upperY; cy++) {
                    long key = SectionPos.asLong(cx, cy, cz);
                    Optional<PoiSection> optional = section(poiStorage, key, load);
                    PoiSection section = optional == null ? null : optional.orElse(null);
                    if (section == null) {
                        continue;
                    }

                    Map<Holder<PoiType>, Set<PoiRecord>> byType = ((PoiSectionByTypeAccessor) section).mfh$getByType();
                    if (byType.isEmpty()) {
                        continue;
                    }

                    for (Map.Entry<Holder<PoiType>, Set<PoiRecord>> entry : byType.entrySet()) {
                        if (!typePredicate.test(entry.getKey())) {
                            continue;
                        }
                        for (PoiRecord record : entry.getValue()) {
                            if (!occupancyFilter.test(record)) {
                                continue;
                            }
                            BlockPos pos = record.getPos();
                            if (Math.abs(pos.getX() - source.getX()) > range || Math.abs(pos.getZ() - source.getZ()) > range) {
                                continue;
                            }
                            if (pos.distSqr(source) > maxDistanceSquared) {
                                continue;
                            }
                            if (positionPredicate != null && !positionPredicate.test(record.getPoiType(), pos)) {
                                continue;
                            }
                            ret.add(record);
                            if (++added[0] >= max) {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}
