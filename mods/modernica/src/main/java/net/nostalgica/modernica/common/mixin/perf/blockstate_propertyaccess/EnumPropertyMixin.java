package net.nostalgica.modernica.common.mixin.perf.blockstate_propertyaccess;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.util.StringRepresentable;
import net.minecraft.world.level.block.state.properties.EnumProperty;
import net.minecraft.world.level.block.state.properties.Property;
import net.nostalgica.modernica.perf.blockstate_propertyaccess.PropertyAccess;

import java.lang.reflect.Array;
import java.util.Arrays;
import java.util.Collection;

@Mixin(EnumProperty.class)
abstract class EnumPropertyMixin<T extends Enum<T> & StringRepresentable> extends Property<T> implements PropertyAccess<T> {

    protected EnumPropertyMixin(String name, Class<T> type) {
        super(name, type);
    }

    @Unique
    private int[] mfh$idByOrdinal;

    @Override
    public final int mfh$getIdFor(T value) {
        Class<T> target = this.getValueClass();
        if (value.getClass() != target && value.getDeclaringClass() != target) {
            return -1;
        }
        return this.mfh$idByOrdinal[value.ordinal()];
    }

    @SuppressWarnings("unchecked")
    @Inject(method = "<init>", at = @At("RETURN"))
    private void mfh$init(CallbackInfo ci) {
        Collection<T> values = this.getPossibleValues();
        Class<T> clazz = this.getValueClass();

        this.mfh$idByOrdinal = new int[clazz.getEnumConstants().length];
        Arrays.fill(this.mfh$idByOrdinal, -1);
        T[] byId = (T[]) Array.newInstance(clazz, values.size());

        int id = 0;
        for (T value : values) {
            this.mfh$idByOrdinal[value.ordinal()] = id;
            byId[id] = value;
            id++;
        }

        this.mfh$setById(byId);
    }

    @Overwrite
    @Override
    public boolean equals(Object obj) {
        return this == obj;
    }
}
