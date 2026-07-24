package net.nostalgica.modernica.blockstate;

import com.google.common.collect.Iterators;
import it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.util.*;

/**
 * Fake "map" implementation used to hold the states.
 *
 * Intentionally throws on methods that would be inefficient so that we know
 * if an incompatible mod is present.
 */
public class FakeStateMap<K, V> implements Map<K, V> {
    private Object[] keys;
    private Map<K, V> fastLookup;
    private Object[] values;
    private int usedSlots;
    public FakeStateMap(int numStates) {
        this.keys = new Object[numStates];
        this.values = new Object[numStates];
        this.usedSlots = 0;
    }

    @Override
    public int size() {
        return usedSlots;
    }

    @Override
    public boolean isEmpty() {
        return size() == 0;
    }

    @Override
    public boolean containsKey(Object o) {
        return getFastLookup().containsKey(o);
    }

    @Override
    public boolean containsValue(Object o) {
        return getFastLookup().containsValue(o);
    }

    @SuppressWarnings("unchecked")
    private Map<K, V> getFastLookup() {
        if(fastLookup == null) {
            var map = new Object2ObjectOpenHashMap<K, V>(usedSlots);
            Object[] keys = this.keys;
            Object[] values = this.values;
            for(int i = 0; i < usedSlots; i++) {
                map.put((K)keys[i], (V)values[i]);
            }
            fastLookup = map;
        }
        return fastLookup;
    }

    @Override
    public V get(Object o) {
        return getFastLookup().get(o);
    }

    @Nullable
    @Override
    public V put(K key, V value) {
        if(fastLookup != null) {
            throw new IllegalStateException("Cannot populate map after fast lookup is built");
        }
        if(usedSlots == keys.length) {
            int newLen = keys.length + (keys.length >> 1);
            keys = Arrays.copyOf(keys, newLen);
            values = Arrays.copyOf(values, newLen);
        }
        keys[usedSlots] = key;
        values[usedSlots] = value;
        usedSlots++;
        return null;
    }

    @Override
    public V remove(Object o) {
        throw new UnsupportedOperationException();
    }

    @Override
    public void putAll(@NotNull Map<? extends K, ? extends V> map) {
        for(Entry<? extends K, ? extends V> entry : map.entrySet()) {
            this.put(entry.getKey(), entry.getValue());
        }
    }

    @Override
    public void clear() {
        for(int i = 0; i < usedSlots; i++) {
            this.keys[i] = null;
            this.values[i] = null;
        }
        this.usedSlots = 0;
    }

    @SuppressWarnings("unchecked")
    private <T> List<T> asList(Object[] array) {
        List<T> list = (List<T>)Arrays.asList(array);
        if(usedSlots < array.length) {
            list = list.subList(0, usedSlots);
        }
        return list;
    }

    @NotNull
    @Override
    @SuppressWarnings("unchecked")
    public Set<K> keySet() {
        return new AbstractSet<>() {
            @Override
            public Iterator<K> iterator() {
                return keys.length == usedSlots ? Iterators.forArray((K[])keys) : ((List<K>)asList(keys)).iterator();
            }

            @Override
            public int size() {
                return usedSlots;
            }
        };
    }

    @NotNull
    @Override
    @SuppressWarnings("unchecked")
    public Collection<V> values() {
        return (Collection<V>)asList(values);
    }

    @NotNull
    @Override
    @SuppressWarnings("unchecked")
    public Set<Entry<K, V>> entrySet() {
        return new AbstractSet<>() {
            @Override
            public int size() {
                return usedSlots;
            }

            @NotNull
            @Override
            public Iterator<Entry<K, V>> iterator() {
                return new Iterator<>() {
                    int currentIdx = 0;

                    @Override
                    public boolean hasNext() {
                        return currentIdx < usedSlots;
                    }

                    @Override
                    public Entry<K, V> next() {
                        if (currentIdx >= usedSlots)
                            throw new IndexOutOfBoundsException();
                        Entry<K, V> entry = new AbstractMap.SimpleImmutableEntry<>((K)keys[currentIdx], (V)values[currentIdx]);
                        currentIdx++;
                        return entry;
                    }
                };
            }
        };
    }
}
