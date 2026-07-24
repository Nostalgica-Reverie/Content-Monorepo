package net.nostalgica.modernica.common.mixin.perf.poi_lookup;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import net.minecraft.core.Holder;
import net.minecraft.world.entity.ai.village.poi.PoiRecord;
import net.minecraft.world.entity.ai.village.poi.PoiSection;
import net.minecraft.world.entity.ai.village.poi.PoiType;

import java.util.Map;
import java.util.Set;

@Mixin(PoiSection.class)
public interface PoiSectionByTypeAccessor {
    @Accessor("byType")
    Map<Holder<PoiType>, Set<PoiRecord>> mfh$getByType();
}
