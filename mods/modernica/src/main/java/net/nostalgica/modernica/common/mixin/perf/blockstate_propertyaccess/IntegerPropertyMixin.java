package net.nostalgica.modernica.common.mixin.perf.blockstate_propertyaccess;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.world.level.block.state.properties.IntegerProperty;
import net.minecraft.world.level.block.state.properties.Property;
import net.nostalgica.modernica.perf.blockstate_propertyaccess.PropertyAccess;

@Mixin(IntegerProperty.class)
abstract class IntegerPropertyMixin extends Property<Integer> implements PropertyAccess<Integer> {

    @Shadow
    @Final
    private int min;

    @Shadow
    @Final
    private int max;

    protected IntegerPropertyMixin(String name, Class<Integer> type) {
        super(name, type);
    }

    @Override
    public final int mfh$getIdFor(Integer value) {
        int val = value.intValue();
        if (val < this.min || val > this.max) {
            return -1;
        }
        return val - this.min;
    }

    @Overwrite
    @Override
    public boolean equals(Object obj) {
        return this == obj;
    }

    @Inject(method = "<init>", at = @At("RETURN"))
    private void mfh$init(CallbackInfo ci) {
        int min = this.min;
        int max = this.max;
        Integer[] byId = new Integer[max - min + 1];
        for (int i = min; i <= max; i++) {
            byId[i - min] = i;
        }
        this.mfh$setById(byId);
    }
}
