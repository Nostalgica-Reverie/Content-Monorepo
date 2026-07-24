package net.nostalgica.modernica.perf.blockstate_propertyaccess;

import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.objects.ReferenceArrayList;

import net.minecraft.world.level.block.state.StateHolder;
import net.minecraft.world.level.block.state.properties.Property;
import net.nostalgica.modernica.util.IntegerUtil;

import java.util.Arrays;
import java.util.Collection;
import java.util.Collections;

/**
 * Packs every property of a block/fluid state family into a single {@code long} index: each state's
 * {@code getValue}/{@code setValue} becomes a couple of integer-arithmetic ops on that index instead of
 * a linear scan of the state's property/value arrays (vanilla's actual representation, one pair of
 * parallel arrays per state).
 * <p>
 * Each property is assigned a "stride" (the product of every other property's value count that sorts
 * before it by property id - a mixed-radix positional numbering system, the same idea as row-major array
 * indexing generalized to unequal digit widths). Recovering one property's value from a composite index
 * is then a divide-by-stride followed by a mod-by-value-count - both done via the widening reciprocal
 * multiply in {@link IntegerUtil#unsignedFloorDiv}, so no property lookup ever pays for an actual integer
 * division. {@code set} is symmetric: subtract out the old digit, add in the new one.
 */
public final class StateIndexTable<O, S> {

    private record Indexer(int totalValues, long stride, long strideMagic, long totalValuesMagic) {}

    private final Int2ObjectOpenHashMap<Indexer> byPropertyId;
    private S[] statesByIndex;
    private final Collection<Property<?>> properties;

    @SuppressWarnings("unchecked")
    public StateIndexTable(Property<?>[] properties) {
        this.byPropertyId = new Int2ObjectOpenHashMap<>(properties.length);
        this.properties = ReferenceArrayList.wrap(properties.clone());

        Property<?>[] sortedById = properties.clone();
        // every state family sharing the same property set must agree on stride assignment order,
        // regardless of which order the properties were declared in - sort by the stable id every
        // Property gets in PropertyMixin
        Arrays.sort(sortedById, (p1, p2) -> Integer.compare(((PropertyAccess<?>) p1).mfh$getId(), ((PropertyAccess<?>) p2).mfh$getId()));

        long stride = 1L;
        for (Property<?> property : sortedById) {
            int totalValues = property.getPossibleValues().size();
            this.byPropertyId.put(
                    ((PropertyAccess<?>) property).mfh$getId(),
                    new Indexer(totalValues, stride, IntegerUtil.getUnsignedDivisorMagic64(stride), IntegerUtil.getUnsignedDivisorMagic64(totalValues))
            );
            stride *= totalValues;
        }
    }

    private static long digitOf(long index, Indexer indexer) {
        long afterStride = IntegerUtil.unsignedFloorDiv(index, indexer.strideMagic());
        long strideCount = IntegerUtil.unsignedFloorDiv(afterStride, indexer.totalValuesMagic());
        return afterStride - strideCount * indexer.totalValues();
    }

    public <T extends Comparable<T>> boolean hasProperty(Property<T> property) {
        return this.byPropertyId.containsKey(((PropertyAccess<T>) property).mfh$getId());
    }

    @SuppressWarnings("unchecked")
    public long getIndex(Property<?>[] keys, Comparable<?>[] values) {
        long index = 0L;
        for (int i = 0; i < keys.length; i++) {
            Property<?> property = keys[i];
            Indexer indexer = this.byPropertyId.get(((PropertyAccess<?>) property).mfh$getId());
            index += ((long) ((PropertyAccess) property).mfh$getIdFor(values[i])) * indexer.stride();
        }
        return index;
    }

    public boolean isLoaded() {
        return this.statesByIndex != null;
    }

    @SuppressWarnings("unchecked")
    public void loadInTable(Collection<S> states) {
        if (this.statesByIndex != null) {
            throw new IllegalStateException("Already loaded");
        }
        this.statesByIndex = (S[]) new StateHolder[states.size()];
        for (S state : states) {
            if (state == null) {
                continue;
            }
            this.statesByIndex[(int) ((PropertyAccessStateHolder<O, S>) state).mfh$getTableIndex()] = state;
        }
        for (S state : this.statesByIndex) {
            if (state == null) {
                throw new IllegalStateException("Incomplete state index table");
            }
        }
    }

    @SuppressWarnings("unchecked")
    public <T extends Comparable<T>> T get(long index, Property<T> property) {
        Indexer indexer = this.byPropertyId.get(((PropertyAccess<T>) property).mfh$getId());
        if (indexer == null) {
            return null;
        }
        return ((PropertyAccess<T>) property).mfh$getById((int) digitOf(index, indexer));
    }

    @SuppressWarnings("unchecked")
    public <T extends Comparable<T>> S set(long index, Property<T> property, T with) {
        Indexer indexer = this.byPropertyId.get(((PropertyAccess<T>) property).mfh$getId());
        if (indexer == null) {
            return null;
        }
        int newValueId = ((PropertyAccess<T>) property).mfh$getIdFor(with);
        if (newValueId < 0) {
            return null;
        }
        long oldDigit = digitOf(index, indexer);
        long newIndex = index + ((long) newValueId - oldDigit) * indexer.stride();
        return this.statesByIndex[(int) newIndex];
    }

    /** Like {@link #set}, but a property this state doesn't have returns {@code dfl} (a silent no-op)
     * instead of {@code null} - matching {@code StateHolder#trySetValue}'s "unknown property is fine,
     * bogus value for a known property is not" contract. */
    @SuppressWarnings("unchecked")
    public <T extends Comparable<T>> S trySet(long index, Property<T> property, T with, S dfl) {
        Indexer indexer = this.byPropertyId.get(((PropertyAccess<T>) property).mfh$getId());
        if (indexer == null) {
            return dfl;
        }
        int newValueId = ((PropertyAccess<T>) property).mfh$getIdFor(with);
        if (newValueId < 0) {
            return null;
        }
        long oldDigit = digitOf(index, indexer);
        long newIndex = index + ((long) newValueId - oldDigit) * indexer.stride();
        return this.statesByIndex[(int) newIndex];
    }

    public boolean isSingletonState() {
        return this.properties.isEmpty();
    }

    public Collection<Property<?>> getProperties() {
        return Collections.unmodifiableCollection(this.properties);
    }
}
