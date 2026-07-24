package net.nostalgica.modernica.perf.blockstate_propertyaccess;

/** Implemented by {@code PropertyMixin} and specialized further by the boolean/enum/integer property
 * mixins: gives every {@code Property} a stable small integer id and an O(1) value<->id mapping, so
 * {@link StateIndexTable} can pack a state's whole property set into one integer index instead of
 * {@code Property}'s own {@code Comparable}-based lookup. */
public interface PropertyAccess<T> {
    int mfh$getId();

    int mfh$getIdFor(T value);

    T mfh$getById(int id);

    void mfh$setById(T[] values);
}
