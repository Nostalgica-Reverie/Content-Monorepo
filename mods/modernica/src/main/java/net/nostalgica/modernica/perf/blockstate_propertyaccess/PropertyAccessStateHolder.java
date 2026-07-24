package net.nostalgica.modernica.perf.blockstate_propertyaccess;

import java.util.Collection;

/** Implemented by {@code StateHolderMixin}. */
public interface PropertyAccessStateHolder<O, S> {
    long mfh$getTableIndex();

    void mfh$init(Collection<S> states);
}
