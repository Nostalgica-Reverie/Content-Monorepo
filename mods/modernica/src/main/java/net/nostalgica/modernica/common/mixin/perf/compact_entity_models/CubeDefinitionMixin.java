package net.nostalgica.modernica.common.mixin.perf.compact_entity_models;

import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import net.minecraft.client.model.geom.ModelPart;
import net.minecraft.client.model.geom.builders.CubeDefinition;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import net.nostalgica.modernica.perf.CompactEntityModelCache;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;

import java.util.List;
import java.util.Set;

@Mixin(CubeDefinition.class)
@ClientOnlyMixin
public class CubeDefinitionMixin {
    /**
     * @author embeddedt
     * @reason deduplicate creation of Cube objects
     */
    @WrapOperation(method = "bake", at = @At(value = "NEW", target = "(IIFFFFFFFFFZFFLjava/util/Set;)Lnet/minecraft/client/model/geom/ModelPart$Cube;"))
    private ModelPart.Cube modernica$deduplicateCube(int texCoordU, int texCoordV, float originX, float originY, float originZ,
                                                     float dimensionX, float dimensionY, float dimensionZ, float gtowX,
                                                     float growY, float growZ, boolean mirror, float texScaleU,
                                                     float texScaleV, Set visibleFaces,
                                                     Operation<ModelPart.Cube> original) {
        // CubeDefinition's face set is not part of Modernica's ownership. Snapshot it so a caller cannot
        // mutate a key after insertion and make the cache entry unreachable.
        List<Object> cacheKey = List.of(texCoordU, texCoordV, originX, originY, originZ, dimensionX, dimensionY, dimensionZ, gtowX, growY, growZ, mirror, texScaleU, texScaleV, Set.copyOf(visibleFaces));
        return CompactEntityModelCache.getOrCreate(cacheKey, () -> original.call((Object[]) cacheKey.toArray()));
    }
}
