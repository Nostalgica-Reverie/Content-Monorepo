package net.nostalgica.modernica.common.mixin.perf.poi_lookup;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

import net.minecraft.world.entity.ai.village.poi.PoiRecord;

@Mixin(PoiRecord.class)
public interface PoiRecordInvoker {
    @Invoker("acquireTicket")
    void mfh$acquireTicket();
}
