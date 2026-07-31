package net.nostalgica.modernica.dynresources;

import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.util.AbstractCollection;
import java.util.AbstractSet;
import java.util.Collection;
import java.util.Collections;
import java.util.LinkedHashSet;
import java.util.Iterator;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.function.Function;

/**
 * A map that behaves like Guava's Maps.asMap but allows for additional entries to be written that override the backing
 * map's entries.
 */
public final class DynamicRegistryMap<K, V> implements Map<K, V> {
    private static final Object NULL_OVERRIDE = new Object();
    private final Set<K> originalKeys;
    private final Function<K, V> fallbackGetter;
    private final ConcurrentHashMap<K, Object> overrides;
    private final EntrySet entrySet;

    public DynamicRegistryMap(Set<K> originalKeys, Function<K, V> fallbackGetter) {
        this.originalKeys = originalKeys;
        this.fallbackGetter = fallbackGetter;
        this.overrides = new ConcurrentHashMap<>();
        this.entrySet = new EntrySet();
    }

    @Override
    public int size() {
        return visibleKeys().size();
    }

    @Override
    public boolean isEmpty() {
        return visibleKeys().isEmpty();
    }

    @Override
    public boolean containsKey(Object o) {
        if (o == null) {
            return false;
        }
        var override = overrides.get(o);
        if (override == NULL_OVERRIDE) {
            return false;
        }
        return override != null || originalKeys.contains(o);
    }

    @Override
    public boolean containsValue(Object o) {
        if (o == null || o == NULL_OVERRIDE) {
            return false;
        }
        if (overrides.containsValue(o)) {
            return true;
        }
        for (K key : originalKeys) {
            if (!overrides.containsKey(key) && Objects.equals(fallbackGetter.apply(key), o)) {
                return true;
            }
        }
        return false;
    }

    @Override
    public V get(Object o) {
        Object value = overrides.get(o);
        if (value == NULL_OVERRIDE) {
            return null;
        } else if (value != null) {
            return (V) value;
        } else if (originalKeys.contains(o)) {
            return fallbackGetter.apply((K)o);
        }
        return null;
    }

    @Override
    public V getOrDefault(Object o, V defaultValue) {
        Object value = overrides.get(o);
        if (value == NULL_OVERRIDE) {
            return defaultValue;
        } else if (value != null) {
            return (V) value;
        } else if (originalKeys.contains(o)) {
            var fallback = fallbackGetter.apply((K)o);
            return fallback != null ? fallback : defaultValue;
        }
        return defaultValue;
    }

    @Override
    public @Nullable V put(K k, V v) {
        V oldValue = get(k);
        if (v == null) {
            remove(k);
            return oldValue;
        }
        overrides.put(k, v);
        return oldValue;
    }

    @Override
    public V remove(Object o) {
        V oldValue = get(o);
        if (originalKeys.contains(o)) {
            overrides.put((K)o, NULL_OVERRIDE);
        } else {
            overrides.remove(o);
        }
        return oldValue;
    }

    @Override
    public void putAll(@NotNull Map<? extends K, ? extends V> map) {
        map.forEach(this::put);
    }

    @Override
    public void clear() {
        overrides.clear();
        originalKeys.forEach(key -> overrides.put(key, NULL_OVERRIDE));
    }

    @Override
    public @NotNull Set<K> keySet() {
        return Collections.unmodifiableSet(visibleKeys());
    }

    @Override
    public @NotNull Collection<V> values() {
        return new AbstractCollection<>() {
            @Override
            public Iterator<V> iterator() {
                return visibleKeys().stream().map(DynamicRegistryMap.this::get).iterator();
            }

            @Override
            public int size() {
                return DynamicRegistryMap.this.size();
            }
        };
    }

    @Override
    public @NotNull Set<Entry<K, V>> entrySet() {
        return this.entrySet;
    }

    private class ModelEntry implements Map.Entry<K, V> {
        private final K key;

        private ModelEntry(K key) {
            this.key = key;
        }

        @Override
        public K getKey() {
            return key;
        }

        @Override
        public V getValue() {
            return get(key);
        }

        @Override
        public V setValue(V value) {
            return put(key, value);
        }
    }

    private class EntrySet extends AbstractSet<Map.Entry<K, V>> {
        @Override
        public Iterator<Entry<K, V>> iterator() {
            var iterator = visibleKeys().iterator();
            return new Iterator<>() {
                @Override
                public boolean hasNext() {
                    return iterator.hasNext();
                }

                @Override
                public Entry<K, V> next() {
                    return new ModelEntry(iterator.next());
                }
            };
        }

        @Override
        public int size() {
            return DynamicRegistryMap.this.size();
        }
    }

    private Set<K> visibleKeys() {
        Set<K> keys = new LinkedHashSet<>(originalKeys);
        for (Map.Entry<K, Object> entry : overrides.entrySet()) {
            if (entry.getValue() == NULL_OVERRIDE) {
                keys.remove(entry.getKey());
            } else {
                keys.add(entry.getKey());
            }
        }
        return keys;
    }
}
