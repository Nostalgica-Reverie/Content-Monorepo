package net.nostalgica.modernica.common.mixin.perf.worldgen_allocation;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.world.level.biome.Biome;
import net.nostalgica.modernica.world.gen.PositionalBiomeGetter;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;

import java.util.function.Function;

@Mixin(targets = {"net/minecraft/world/level/levelgen/SurfaceRules$Context"}, priority = 100)
public class SurfaceRulesContextMixin {
    @Shadow private long lastUpdateY;

    @Shadow private int blockX;

    @Shadow private int blockZ;

    @Shadow private int blockY;

    @Shadow private int waterHeight;

    @Shadow private int stoneDepthBelow;

    @Shadow private int stoneDepthAbove;

    @Shadow private Holder<Biome> biome;

    @Shadow @Final private Function<BlockPos, Holder<Biome>> biomeGetter;

    @Shadow @Final private BlockPos.MutableBlockPos pos;

    @Unique
    private PositionalBiomeGetter modernica$biomeCache;

    /**
     * @author embeddedt
     * @reason Reuse supplier object instead of creating new ones every time
     */
    @Overwrite
    protected void updateY(int stoneDepthAbove, int stoneDepthBelow, int waterHeight, int blockY) {
        ++this.lastUpdateY;

        var getter = this.modernica$biomeCache;
        if(getter == null) {
            this.modernica$biomeCache = getter = new PositionalBiomeGetter(this.biomeGetter, this.pos);
        }

        getter.update(this.blockX, blockY, this.blockZ);
        this.biome = null;
        this.blockY = blockY;
        this.waterHeight = waterHeight;
        this.stoneDepthBelow = stoneDepthBelow;
        this.stoneDepthAbove = stoneDepthAbove;
    }

    /**
     * @author coredex-source
     * @reason Reuse a single positional getter object for biome lookups
     */
    @Overwrite
    public Holder<Biome> getBiome() {
        var biome = this.biome;
        if(biome == null) {
            var getter = this.modernica$biomeCache;
            if(getter == null) {
                this.modernica$biomeCache = getter = new PositionalBiomeGetter(this.biomeGetter, this.pos);
            }

            getter.update(this.blockX, this.blockY, this.blockZ);
            this.biome = biome = getter.get();
        }

        return biome;
    }
}
