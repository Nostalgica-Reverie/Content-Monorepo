package net.nostalgica.modernica.common.mixin.perf.getblock;

import com.llamalad7.mixinextras.sugar.Local;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.LevelAccessor;
import net.minecraft.world.level.dimension.DimensionType;

/**
 * Vanilla's {@code LevelHeightAccessor} default methods (min/max Y, section count, section-index math)
 * re-derive everything from {@code dimensionType()} on every single call. A level's dimension type never
 * changes after construction, so all of that is cached here once instead.
 * <p>
 * Priority is raised above default so this applies after any other mod's own height-caching mixin on
 * {@code Level} (e.g. Lithium's), rather than racing it for which cached values win.
 */
@Mixin(value = Level.class, priority = 1100)
abstract class LevelMixin implements LevelAccessor, AutoCloseable {

    @Unique
    private int mfh$minY;
    @Unique
    private int mfh$height;
    @Unique
    private int mfh$maxY;
    @Unique
    private int mfh$minSectionY;
    @Unique
    private int mfh$maxSectionY;
    @Unique
    private int mfh$sectionsCount;

    @Inject(method = "<init>", at = @At("CTOR_HEAD"))
    private void mfh$cacheHeight(CallbackInfo ci, @Local(ordinal = 0, argsOnly = true) Holder<DimensionType> dimensionTypeHolder) {
        DimensionType dimType = dimensionTypeHolder.value();
        this.mfh$minY = dimType.minY();
        this.mfh$height = dimType.height();
        this.mfh$maxY = this.mfh$minY + this.mfh$height - 1;
        this.mfh$minSectionY = this.mfh$minY >> 4;
        this.mfh$maxSectionY = this.mfh$maxY >> 4;
        this.mfh$sectionsCount = this.mfh$maxSectionY - this.mfh$minSectionY + 1;
    }

    @Override
    public int getMinY() {
        return this.mfh$minY;
    }

    @Override
    public int getHeight() {
        return this.mfh$height;
    }

    @Override
    public int getMaxY() {
        return this.mfh$maxY;
    }

    @Override
    public int getSectionsCount() {
        return this.mfh$sectionsCount;
    }

    @Override
    public int getMinSectionY() {
        return this.mfh$minSectionY;
    }

    @Override
    public int getMaxSectionY() {
        return this.mfh$maxSectionY;
    }

    @Override
    public boolean isInsideBuildHeight(int blockY) {
        return blockY >= this.mfh$minY && blockY <= this.mfh$maxY;
    }

    @Override
    public boolean isOutsideBuildHeight(BlockPos pos) {
        return this.isOutsideBuildHeight(pos.getY());
    }

    @Override
    public boolean isOutsideBuildHeight(int blockY) {
        return blockY < this.mfh$minY || blockY > this.mfh$maxY;
    }

    @Override
    public int getSectionIndex(int blockY) {
        return (blockY >> 4) - this.mfh$minSectionY;
    }

    @Override
    public int getSectionIndexFromSectionY(int sectionY) {
        return sectionY - this.mfh$minSectionY;
    }

    @Override
    public int getSectionYFromSectionIndex(int sectionIndex) {
        return sectionIndex + this.mfh$minSectionY;
    }
}
