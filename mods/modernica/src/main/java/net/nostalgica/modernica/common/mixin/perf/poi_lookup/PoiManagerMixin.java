package net.nostalgica.modernica.common.mixin.perf.poi_lookup;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import com.mojang.datafixers.util.Pair;
import com.mojang.serialization.Codec;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.RegistryAccess;
import net.minecraft.util.RandomSource;
import net.minecraft.world.entity.ai.village.poi.PoiManager;
import net.minecraft.world.entity.ai.village.poi.PoiRecord;
import net.minecraft.world.entity.ai.village.poi.PoiSection;
import net.minecraft.world.entity.ai.village.poi.PoiType;
import net.minecraft.world.level.LevelHeightAccessor;
import net.minecraft.world.level.chunk.storage.ChunkIOErrorReporter;
import net.minecraft.world.level.chunk.storage.SectionStorage;
import net.minecraft.world.level.chunk.storage.SimpleRegionStorage;
import net.nostalgica.modernica.perf.poi_lookup.PoiSearch;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.BiFunction;
import java.util.function.BiPredicate;
import java.util.function.Function;
import java.util.function.Predicate;
import java.util.stream.Stream;

/** Routes every one of {@code PoiManager}'s stream-based search entry points through {@link PoiSearch}. */
@Mixin(PoiManager.class)
abstract class PoiManagerMixin extends SectionStorage<PoiSection, PoiSection.Packed> {

    public PoiManagerMixin(SimpleRegionStorage simpleRegionStorage, Codec<PoiSection.Packed> codec, Function<PoiSection, PoiSection.Packed> toPacked,
                            BiFunction<PoiSection.Packed, Runnable, PoiSection> fromPacked, Function<Runnable, PoiSection> factory,
                            RegistryAccess registryAccess, ChunkIOErrorReporter errorReporter, LevelHeightAccessor levelHeightAccessor) {
        super(simpleRegionStorage, codec, toPacked, fromPacked, factory, registryAccess, errorReporter, levelHeightAccessor);
    }

    @Shadow
    @Final
    protected LevelHeightAccessor levelHeightAccessor;

    @Overwrite
    public Stream<PoiRecord> getInSquare(Predicate<Holder<PoiType>> predicate, BlockPos center, int radius, PoiManager.Occupancy occupancy) {
        List<PoiRecord> ret = new ArrayList<>();
        PoiSearch.findAnyPoiRecords((PoiManager) (Object) this, predicate, (Predicate<BlockPos>) null, center, radius, Double.MAX_VALUE, occupancy,
                PoiSearch.LOAD_FOR_SEARCHING, this.levelHeightAccessor.getMinY() >> 4, this.levelHeightAccessor.getMaxY() >> 4, Integer.MAX_VALUE, ret);
        return ret.stream();
    }

    @Overwrite
    public Stream<PoiRecord> getInRange(Predicate<Holder<PoiType>> predicate, BlockPos center, int radius, PoiManager.Occupancy occupancy) {
        List<PoiRecord> ret = new ArrayList<>();
        PoiSearch.findAnyPoiRecords((PoiManager) (Object) this, predicate, (Predicate<BlockPos>) null, center, radius, (double) ((long) radius * radius), occupancy,
                PoiSearch.LOAD_FOR_SEARCHING, this.levelHeightAccessor.getMinY() >> 4, this.levelHeightAccessor.getMaxY() >> 4, Integer.MAX_VALUE, ret);
        return ret.stream();
    }

    @Overwrite
    public Stream<BlockPos> findAll(Predicate<Holder<PoiType>> predicate, Predicate<BlockPos> filter, BlockPos center, int radius, PoiManager.Occupancy occupancy) {
        List<PoiRecord> ret = new ArrayList<>();
        PoiSearch.findAnyPoiRecords((PoiManager) (Object) this, predicate, filter, center, radius, (double) ((long) radius * radius), occupancy,
                PoiSearch.LOAD_FOR_SEARCHING, this.levelHeightAccessor.getMinY() >> 4, this.levelHeightAccessor.getMaxY() >> 4, Integer.MAX_VALUE, ret);
        return ret.stream().map(PoiRecord::getPos);
    }

    @Overwrite
    public Stream<Pair<Holder<PoiType>, BlockPos>> findAllWithType(Predicate<Holder<PoiType>> predicate, Predicate<BlockPos> filter, BlockPos center, int radius, PoiManager.Occupancy occupancy) {
        List<PoiRecord> ret = new ArrayList<>();
        PoiSearch.findAnyPoiRecords((PoiManager) (Object) this, predicate, filter, center, radius, (double) ((long) radius * radius), occupancy,
                PoiSearch.LOAD_FOR_SEARCHING, this.levelHeightAccessor.getMinY() >> 4, this.levelHeightAccessor.getMaxY() >> 4, Integer.MAX_VALUE, ret);
        return ret.stream().map(record -> Pair.of(record.getPoiType(), record.getPos()));
    }

    @Overwrite
    public Stream<Pair<Holder<PoiType>, BlockPos>> findAllClosestFirstWithType(Predicate<Holder<PoiType>> predicate, Predicate<BlockPos> filter, BlockPos center, int radius, PoiManager.Occupancy occupancy) {
        List<PoiRecord> ret = new ArrayList<>();
        PoiSearch.findAnyPoiRecords((PoiManager) (Object) this, predicate, filter, center, radius, (double) ((long) radius * radius), occupancy,
                PoiSearch.LOAD_FOR_SEARCHING, this.levelHeightAccessor.getMinY() >> 4, this.levelHeightAccessor.getMaxY() >> 4, Integer.MAX_VALUE, ret);
        ret.sort((r1, r2) -> PoiSearch.compareDistances(center, r1.getPos(), r2.getPos()));
        return ret.stream().map(record -> Pair.of(record.getPoiType(), record.getPos()));
    }

    @Overwrite
    public Optional<BlockPos> find(Predicate<Holder<PoiType>> predicate, Predicate<BlockPos> filter, BlockPos center, int radius, PoiManager.Occupancy occupancy) {
        return Optional.ofNullable(PoiSearch.findAnyPoiPosition((PoiManager) (Object) this, predicate, filter, center, radius, occupancy,
                PoiSearch.LOAD_FOR_SEARCHING, this.levelHeightAccessor.getMinY() >> 4, this.levelHeightAccessor.getMaxY() >> 4));
    }

    @Overwrite
    public Optional<BlockPos> findClosest(Predicate<Holder<PoiType>> predicate, BlockPos center, int radius, PoiManager.Occupancy occupancy) {
        PoiRecord closest = PoiSearch.findNearestPoiRecord((PoiManager) (Object) this, predicate, null, center, radius, (double) ((long) radius * radius), occupancy,
                PoiSearch.LOAD_FOR_SEARCHING, this.levelHeightAccessor.getMinY() >> 4, this.levelHeightAccessor.getMaxY() >> 4);
        return closest == null ? Optional.empty() : Optional.of(closest.getPos());
    }

    @Overwrite
    public Optional<Pair<Holder<PoiType>, BlockPos>> findClosestWithType(Predicate<Holder<PoiType>> predicate, BlockPos center, int radius, PoiManager.Occupancy occupancy) {
        PoiRecord closest = PoiSearch.findNearestPoiRecord((PoiManager) (Object) this, predicate, null, center, radius, (double) ((long) radius * radius), occupancy,
                PoiSearch.LOAD_FOR_SEARCHING, this.levelHeightAccessor.getMinY() >> 4, this.levelHeightAccessor.getMaxY() >> 4);
        return closest == null ? Optional.empty() : Optional.of(Pair.of(closest.getPoiType(), closest.getPos()));
    }

    @Overwrite
    public Optional<BlockPos> findClosest(Predicate<Holder<PoiType>> predicate, Predicate<BlockPos> filter, BlockPos center, int radius, PoiManager.Occupancy occupancy) {
        PoiRecord closest = PoiSearch.findNearestPoiRecord((PoiManager) (Object) this, predicate, filter, center, radius, (double) ((long) radius * radius), occupancy,
                PoiSearch.LOAD_FOR_SEARCHING, this.levelHeightAccessor.getMinY() >> 4, this.levelHeightAccessor.getMaxY() >> 4);
        return closest == null ? Optional.empty() : Optional.of(closest.getPos());
    }

    @Overwrite
    public Optional<BlockPos> take(Predicate<Holder<PoiType>> predicate, BiPredicate<Holder<PoiType>, BlockPos> filter, BlockPos center, int radius) {
        PoiRecord record = PoiSearch.findAnyPoiRecord((PoiManager) (Object) this, predicate, filter, center, radius, (double) ((long) radius * radius), PoiManager.Occupancy.HAS_SPACE,
                PoiSearch.LOAD_FOR_SEARCHING, this.levelHeightAccessor.getMinY() >> 4, this.levelHeightAccessor.getMaxY() >> 4);
        if (record == null) {
            return Optional.empty();
        }
        ((PoiRecordInvoker) record).mfh$acquireTicket();
        return Optional.of(record.getPos());
    }

    @Overwrite
    public Optional<BlockPos> getRandom(Predicate<Holder<PoiType>> predicate, Predicate<BlockPos> filter, PoiManager.Occupancy occupancy, BlockPos center, int radius, RandomSource random) {
        List<PoiRecord> list = new ArrayList<>();
        PoiSearch.findAnyPoiRecords((PoiManager) (Object) this, predicate, filter, center, radius, (double) ((long) radius * radius), occupancy,
                PoiSearch.LOAD_FOR_SEARCHING, this.levelHeightAccessor.getMinY() >> 4, this.levelHeightAccessor.getMaxY() >> 4, Integer.MAX_VALUE, list);
        if (list.isEmpty()) {
            return Optional.empty();
        }
        // the position predicate is already folded into the search above, so a plain random pick here
        // is equivalent to vanilla's shuffle-then-find-first
        return Optional.ofNullable(list.get(random.nextInt(list.size())).getPos());
    }
}
