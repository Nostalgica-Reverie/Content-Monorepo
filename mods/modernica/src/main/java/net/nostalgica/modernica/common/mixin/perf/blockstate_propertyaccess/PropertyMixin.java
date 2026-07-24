package net.nostalgica.modernica.common.mixin.perf.blockstate_propertyaccess;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.world.level.block.state.properties.Property;
import net.nostalgica.modernica.perf.blockstate_propertyaccess.PropertyAccess;

import java.util.concurrent.atomic.AtomicInteger;

@Mixin(Property.class)
abstract class PropertyMixin<T extends Comparable<T>> implements PropertyAccess<T> {

    @Unique
    private static final AtomicInteger MFH_ID_GENERATOR = new AtomicInteger();

    @Unique
    private int mfh$id;

    @Unique
    private T[] mfh$byId;

    @Override
    public final int mfh$getId() {
        return this.mfh$id;
    }

    @Override
    public final T mfh$getById(int id) {
        T[] byId = this.mfh$byId;
        return id < 0 || id >= byId.length ? null : byId[id];
    }

    @Override
    public final void mfh$setById(T[] byId) {
        if (this.mfh$byId != null) {
            throw new IllegalStateException("Already set");
        }
        this.mfh$byId = byId;
    }

    @Override
    public abstract int mfh$getIdFor(T value);

    @Inject(method = "<init>", at = @At("RETURN"))
    private void mfh$assignId(CallbackInfo ci) {
        this.mfh$id = MFH_ID_GENERATOR.getAndIncrement();
    }

    /** Every {@code Property} instance is a singleton owned by its block/fluid, so reference equality
     * already matches vanilla's actual usage - this just skips the (also singleton-equivalent, but
     * slower) generated equals. */
    @Overwrite
    @Override
    public boolean equals(Object obj) {
        return this == obj;
    }
}
