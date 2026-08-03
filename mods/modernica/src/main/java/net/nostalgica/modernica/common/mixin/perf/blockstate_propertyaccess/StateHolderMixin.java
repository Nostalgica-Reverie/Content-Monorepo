package net.nostalgica.modernica.common.mixin.perf.blockstate_propertyaccess;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.world.level.block.state.StateHolder;
import net.minecraft.world.level.block.state.properties.Property;
import net.nostalgica.modernica.perf.blockstate_propertyaccess.PropertyAccessStateHolder;
import net.nostalgica.modernica.perf.blockstate_propertyaccess.StateIndexTable;

import java.util.Collection;
import java.util.stream.Stream;

/** Stores block state properties in a shared lookup table. */
@Mixin(StateHolder.class)
abstract class StateHolderMixin<O, S> implements PropertyAccessStateHolder<O, S> {

    @Shadow
    @Final
    protected O owner;

    @Shadow
    @Final
    @Mutable
    public Property<?>[] propertyKeys;

    @Shadow
    @Final
    @Mutable
    public Comparable<?>[] propertyValues;

    @Shadow
    private S[][] neighbors;

    @Shadow
    private static <T extends Comparable<T>> Property.Value<T> createValue(Property<T> propertyKey, Comparable<?> propertyValue) {
        throw new UnsupportedOperationException();
    }

    @Unique
    private StateIndexTable<O, S> mfh$table;

    @Unique
    private long mfh$tableIndex;

    @Override
    public final long mfh$getTableIndex() {
        return this.mfh$tableIndex;
    }

    @Override
    @SuppressWarnings("unchecked")
    public final void mfh$init(Collection<S> states) {
        this.mfh$table.loadInTable(states);

        for (S sibling : states) {
            StateHolderMixin<O, S> mixin = (StateHolderMixin<O, S>) (Object) (StateHolder<O, S>) sibling;
            mixin.mfh$table = this.mfh$table;
            // Drop duplicate vanilla data
            mixin.propertyKeys = null;
            mixin.propertyValues = null;
            mixin.neighbors = null;
        }
    }

    @Inject(method = "<init>", at = @At("RETURN"))
    private void mfh$buildTable(O owner, Property<?>[] propertyKeys, Comparable<?>[] propertyValues, CallbackInfo ci) {
        this.mfh$table = new StateIndexTable<>(propertyKeys);
        this.mfh$tableIndex = this.mfh$table.getIndex(propertyKeys, propertyValues);
    }

    @Overwrite
    public Collection<Property<?>> getProperties() {
        return this.mfh$table.getProperties();
    }

    @Overwrite
    public boolean isSingletonState() {
        return this.mfh$table.isSingletonState();
    }

    @Overwrite
    public Stream<Property.Value<?>> getValues() {
        return this.mfh$table.getProperties().stream()
                .map(prop -> createValue(prop, ((StateHolder<O, S>) (Object) this).getValue(prop)));
    }

    @Overwrite
    public <T extends Comparable<T>, V extends T> S setValue(Property<T> property, V value) {
        S result = this.mfh$table.set(this.mfh$tableIndex, property, value);
        if (result != null) {
            return result;
        }
        throw new IllegalArgumentException("Cannot set property " + property + " to " + value + " on " + this.owner);
    }

    @Overwrite
    public <T extends Comparable<T>, V extends T> S trySetValue(Property<T> property, V value) {
        if (property == null) {
            return (S) (Object) this;
        }
        S result = this.mfh$table.trySet(this.mfh$tableIndex, property, value, (S) (Object) this);
        if (result != null) {
            return result;
        }
        throw new IllegalArgumentException("Cannot set property " + property + " to " + value + " on " + this.owner);
    }

    @Overwrite
    public <T extends Comparable<T>> T getNullableValue(Property<T> property) {
        return property == null ? null : this.mfh$table.get(this.mfh$tableIndex, property);
    }

    @Overwrite
    public <T extends Comparable<T>> T getValue(Property<T> property) {
        T result = this.mfh$table.get(this.mfh$tableIndex, property);
        if (result != null) {
            return result;
        }
        throw new IllegalArgumentException("Cannot get property " + property + " as it does not exist in " + this.owner);
    }

    @Overwrite
    public <T extends Comparable<T>> boolean hasProperty(Property<T> property) {
        return property != null && this.mfh$table.hasProperty(property);
    }
}
