package net.nostalgica.modernica.common.mixin.perf.poi_lookup;

import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.ai.village.poi.PoiManager;
import net.minecraft.world.entity.ai.village.poi.PoiRecord;
import net.minecraft.world.entity.ai.village.poi.PoiType;
import net.minecraft.world.entity.ai.village.poi.PoiTypes;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.level.border.WorldBorder;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.levelgen.BelowZeroRetrogen;
import net.minecraft.world.level.portal.PortalForcer;
import net.nostalgica.modernica.perf.poi_lookup.PoiSearch;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

@Mixin(PortalForcer.class)
abstract class PortalForcerMixin {

    @Shadow
    @Final
    private ServerLevel level;

    @Overwrite
    public Optional<BlockPos> findClosestPortalPosition(BlockPos approximateExitPos, boolean toNether, WorldBorder worldBorder) {
        PoiManager poiManager = this.level.getPoiManager();
        int radius = toNether ? 16 : 128;
        int minSectionY = this.level.getMinY() >> 4;
        int maxSectionY = this.level.getMaxY() >> 4;

        List<PoiRecord> records = new ArrayList<>();
        PoiSearch.findClosestPoiDataRecords(
                poiManager, type -> type.is(PoiTypes.NETHER_PORTAL),
                (Holder<PoiType> type, BlockPos pos) -> {
                    if (!worldBorder.isWithinBounds(pos)) {
                        return false;
                    }

                    ChunkAccess lowest = this.level.getChunk(pos.getX() >> 4, pos.getZ() >> 4, ChunkStatus.EMPTY);

                    BelowZeroRetrogen belowZeroRetrogen;
                    if (!lowest.getPersistedStatus().isOrAfter(ChunkStatus.FULL)
                            && ((belowZeroRetrogen = lowest.getBelowZeroRetrogen()) == null || !belowZeroRetrogen.targetStatus().isOrAfter(ChunkStatus.SPAWN))) {
                        return false;
                    }

                    return lowest.getBlockState(pos).hasProperty(BlockStateProperties.HORIZONTAL_AXIS);
                },
                approximateExitPos, radius, Double.MAX_VALUE, PoiManager.Occupancy.ANY, true, minSectionY, maxSectionY, records
        );

        // PoiSearch already narrows to the closest distance bucket, but vanilla additionally biases
        // toward the lowest Y among ties at that distance
        PoiRecord lowestY = null;
        for (PoiRecord record : records) {
            if (lowestY == null || lowestY.getPos().getY() > record.getPos().getY()) {
                lowestY = record;
            }
        }
        return Optional.ofNullable(lowestY == null ? null : lowestY.getPos());
    }
}
