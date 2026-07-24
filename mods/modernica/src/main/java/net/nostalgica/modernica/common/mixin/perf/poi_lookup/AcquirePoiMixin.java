package net.nostalgica.modernica.common.mixin.perf.poi_lookup;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

import com.mojang.datafixers.util.Pair;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.world.entity.ai.behavior.AcquirePoi;
import net.minecraft.world.entity.ai.village.poi.PoiManager;
import net.minecraft.world.entity.ai.village.poi.PoiType;
import net.nostalgica.modernica.perf.poi_lookup.PoiSearch;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Predicate;
import java.util.stream.Stream;

/** The AI behavior only ever keeps the closest 5 candidates it finds - route it through
 * {@link PoiSearch}'s ranked search directly instead of building the full unranked stream first. */
@Mixin(AcquirePoi.class)
abstract class AcquirePoiMixin {

    @Redirect(
            method = {"lambda$create$3"},
            at = @At(
                    target = "Lnet/minecraft/world/entity/ai/village/poi/PoiManager;findAllClosestFirstWithType(Ljava/util/function/Predicate;Ljava/util/function/Predicate;Lnet/minecraft/core/BlockPos;ILnet/minecraft/world/entity/ai/village/poi/PoiManager$Occupancy;)Ljava/util/stream/Stream;",
                    value = "INVOKE",
                    ordinal = 0
            )
    )
    private static Stream<Pair<Holder<PoiType>, BlockPos>> mfh$rankedSearch(PoiManager poiManager, Predicate<Holder<PoiType>> predicate,
                                                                             Predicate<BlockPos> filter, BlockPos center, int radius,
                                                                             PoiManager.Occupancy occupancy) {
        List<Pair<Holder<PoiType>, BlockPos>> ret = new ArrayList<>();
        int minSectionY = ((PoiManagerMixin) (Object) poiManager).levelHeightAccessor.getMinY() >> 4;
        int maxSectionY = ((PoiManagerMixin) (Object) poiManager).levelHeightAccessor.getMaxY() >> 4;
        PoiSearch.findNearestPoiPositions(poiManager, predicate, filter, center, radius, Double.MAX_VALUE, occupancy,
                PoiSearch.LOAD_FOR_SEARCHING, minSectionY, maxSectionY, 5, ret);
        return ret.stream();
    }
}
