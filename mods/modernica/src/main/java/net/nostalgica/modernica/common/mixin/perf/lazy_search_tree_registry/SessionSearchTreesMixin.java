package net.nostalgica.modernica.common.mixin.perf.lazy_search_tree_registry;

import com.google.common.base.Stopwatch;
import com.llamalad7.mixinextras.injector.wrapmethod.WrapMethod;
import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import net.minecraft.client.gui.screens.recipebook.RecipeCollection;
import net.minecraft.client.multiplayer.SessionSearchTrees;
import net.minecraft.client.searchtree.SearchTree;
import net.nostalgica.modernica.Modernica;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;

import java.util.concurrent.CompletableFuture;
import java.util.function.Supplier;

@Mixin(SessionSearchTrees.class)
@ClientOnlyMixin
public class SessionSearchTreesMixin {
    @Shadow private CompletableFuture<SearchTree<RecipeCollection>> recipeSearch;
    private Supplier<SearchTree<RecipeCollection>> mfix$deferredSearchTreeSupplier;

    @ModifyArg(method = { "lambda$updateRecipes$0" }, at = @At(value = "INVOKE", target = "Ljava/util/concurrent/CompletableFuture;supplyAsync(Ljava/util/function/Supplier;Ljava/util/concurrent/Executor;)Ljava/util/concurrent/CompletableFuture;"))
    private Supplier<SearchTree<RecipeCollection>> mfix$deferProcessing(Supplier<SearchTree<RecipeCollection>> supplier) {
        this.mfix$deferredSearchTreeSupplier = supplier;
        return SearchTree::empty;
    }

    @WrapMethod(method = "recipes")
    private SearchTree<RecipeCollection> mfix$processDeferredBuild(Operation<SearchTree<RecipeCollection>> original) {
        synchronized (this) {
            if (mfix$deferredSearchTreeSupplier != null) {
                Stopwatch watch = Stopwatch.createStarted();
                this.recipeSearch = CompletableFuture.completedFuture(mfix$deferredSearchTreeSupplier.get());
                watch.stop();
                Modernica.LOGGER.info("Building recipe book search tree took {}", watch);
                mfix$deferredSearchTreeSupplier = null;
            }
            return original.call();
        }
    }
}
