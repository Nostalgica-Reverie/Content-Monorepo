package net.nostalgica.modernica.common.mixin.perf.blockstate_propertyaccess;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.world.level.block.state.properties.BooleanProperty;
import net.minecraft.world.level.block.state.properties.Property;
import net.nostalgica.modernica.perf.blockstate_propertyaccess.PropertyAccess;

@Mixin(BooleanProperty.class)
abstract class BooleanPropertyMixin extends Property<Boolean> implements PropertyAccess<Boolean> {

    protected BooleanPropertyMixin(String name, Class<Boolean> type) {
        super(name, type);
    }

    @Unique
    private static final Boolean[] MFH_BY_ID = {Boolean.FALSE, Boolean.TRUE};

    @Override
    public final int mfh$getIdFor(Boolean value) {
        return value.booleanValue() ? 1 : 0;
    }

    @Inject(method = "<init>", at = @At("RETURN"))
    private void mfh$init(CallbackInfo ci) {
        this.mfh$setById(MFH_BY_ID);
    }
}
