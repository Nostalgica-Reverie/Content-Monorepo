package net.nostalgica.modernica.common.mixin.perf.dynamic_resources;

import com.llamalad7.mixinextras.sugar.Local;
import com.mojang.serialization.JsonOps;
import net.minecraft.client.renderer.item.ClientItem;
import net.minecraft.client.resources.model.ClientItemInfoLoader;
import net.minecraft.core.RegistryAccess;
import net.minecraft.resources.FileToIdConverter;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.resources.Resource;
import net.minecraft.util.PlaceholderLookupProvider;
import net.minecraft.util.StrictJsonParser;
import net.nostalgica.modernica.Modernica;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import net.nostalgica.modernica.dynresources.DynamicModelSystem;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;

import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import java.util.function.Function;

@Mixin(ClientItemInfoLoader.class)
@ClientOnlyMixin
public abstract class MixinClientItemInfoLoader {
    @Unique
    private static final FileToIdConverter MFIX$ITEM_LISTER = FileToIdConverter.json("items");

    @Unique
    private static volatile boolean MFIX$DYNAMIC_CLIENT_ITEMS_ENABLED = true;

    @Unique
    private static volatile boolean MFIX$DYNAMIC_CLIENT_ITEMS_FAILURE_LOGGED = false;

    @Unique
    private static ClientItem mfix$loadSingleClientItemInfo(Identifier resourceFileId, Resource resource, RegistryAccess.Frozen staticRegistries) {
        Identifier itemId = MFIX$ITEM_LISTER.fileToId(resourceFileId);
        try (var reader = resource.openAsReader()) {
            PlaceholderLookupProvider placeholderLookupProvider = new PlaceholderLookupProvider(staticRegistries);
            var context = placeholderLookupProvider.createSerializationContext(JsonOps.INSTANCE);
            return ClientItem.CODEC.parse(context, StrictJsonParser.parse(reader))
                    .ifError(error -> Modernica.LOGGER.error("Couldn't parse item model '{}' from pack '{}': {}", itemId, resource.sourcePackId(), error.message()))
                    .result()
                    .map(clientItem -> placeholderLookupProvider.hasRegisteredPlaceholders() ? clientItem.withRegistrySwapper(placeholderLookupProvider.createSwapper()) : clientItem)
                    .orElse(null);
        } catch (Exception e) {
            Modernica.LOGGER.error("Failed to open item model {} from pack '{}'", resourceFileId, resource.sourcePackId(), e);
            return null;
        }
    }

    /**
     * @author embeddedt
     * @reason Load client item infos dynamically instead of all at once.
     */
    @ModifyArg(method = "scheduleLoad", at = @At(value = "INVOKE", target = "Ljava/util/concurrent/CompletableFuture;thenCompose(Ljava/util/function/Function;)Ljava/util/concurrent/CompletableFuture;"))
    private static Function<Map<Identifier, Resource>, ? extends CompletionStage<ClientItemInfoLoader.LoadedClientInfos>> skipAOTClientItemLoad(
            Function<Map<Identifier, Resource>, ? extends CompletionStage<ClientItemInfoLoader.LoadedClientInfos>> original,
            @Local(ordinal = 0) RegistryAccess.Frozen staticRegistries) {
        if (!MFIX$DYNAMIC_CLIENT_ITEMS_ENABLED) {
            return original;
        }
        return resourceMap -> CompletableFuture.completedFuture(DynamicModelSystem.createDynamicClientInfos(resourceMap, (resourceFileId, resource) -> {
            if (!MFIX$DYNAMIC_CLIENT_ITEMS_ENABLED) {
                return null;
            }
            try {
                return mfix$loadSingleClientItemInfo(resourceFileId, resource, staticRegistries);
            } catch (RuntimeException e) {
                MFIX$DYNAMIC_CLIENT_ITEMS_ENABLED = false;
                if (!MFIX$DYNAMIC_CLIENT_ITEMS_FAILURE_LOGGED) {
                    MFIX$DYNAMIC_CLIENT_ITEMS_FAILURE_LOGGED = true;
                    Modernica.LOGGER.warn("Disabling dynamic client item info loading due to runtime failure", e);
                }
                return null;
            }
        }));
    }
}