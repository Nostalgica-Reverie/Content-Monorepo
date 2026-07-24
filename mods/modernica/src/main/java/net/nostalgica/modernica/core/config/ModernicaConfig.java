package net.nostalgica.modernica.core.config;

import me.fzzyhmstrs.fzzy_config.annotations.Action;
import me.fzzyhmstrs.fzzy_config.annotations.RequiresAction;
import me.fzzyhmstrs.fzzy_config.annotations.TomlHeaderComment;
import me.fzzyhmstrs.fzzy_config.annotations.Translation;
import me.fzzyhmstrs.fzzy_config.config.Config;
import me.fzzyhmstrs.fzzy_config.config.ConfigSection;
import me.fzzyhmstrs.fzzy_config.util.EnumTranslatable;
import me.fzzyhmstrs.fzzy_config.validation.number.ValidatedInt;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.resources.Identifier;
import org.jetbrains.annotations.NotNull;

/** Most fields gate a Mixin transform, so changes always require a restart. */
@RequiresAction(action = Action.RESTART)
@TomlHeaderComment(text = "Modernica: merged Modernica + Hydrogen performance/bugfix mixin catalog.")
@TomlHeaderComment(text = "All settings require a restart to take effect - see the mod's README for why.")
@Translation(prefix = "modernica.config")
public class ModernicaConfig extends Config {

    public ModernicaConfig() {
        // namespace must match fabric.mod.json's "id" or ModMenu's config button silently disappears
        super(Identifier.fromNamespaceAndPath("modernica", "config"), "modernica");
    }

    public StabilityLevel stabilityLevel = StabilityLevel.GA;
    public boolean devEnvironmentMixins = FabricLoader.getInstance().isDevelopmentEnvironment();

    public PerformanceSection performance = new PerformanceSection();
    public TroubleshootingSection troubleshooting = new TroubleshootingSection();
    public ExpertOnlySection expertOnly = new ExpertOnlySection();

    public enum StabilityLevel implements EnumTranslatable {
        GA,
        BETA;

        public boolean isAtLeast(StabilityLevel required) {
            return this.ordinal() >= required.ordinal();
        }

        @NotNull
        @Override
        public String prefix() {
            return "modernica.config.stability_level";
        }
    }

    public static class PerformanceSection extends ConfigSection {
        public boolean perfDynamicResources = false; // perf.dynamic_resources
        public boolean perfReleaseProtochunks = true; // perf.release_protochunks

        public boolean perfNetworkOptimizations = true; // perf.network_optimizations (from Krypton)
        public boolean perfNetworkEnhancements = true; // perf.network_enhancements
        /** Max particle broadcast distance in blocks; only applies with {@link #perfNetworkEnhancements}. */
        public ValidatedInt particleTrackingRangeBlocks = new ValidatedInt(48, 128, 0);
        public boolean featureFastIpPing = true; // feature.fast_ip_ping
        /** Off by default: reintroduces a vanilla bug where you can briefly fall through the world on a slow connection. */
        public boolean featureForceCloseLoadingScreen = false; // feature.force_close_loading_screen
    }

    public static class TroubleshootingSection extends ConfigSection {
        public boolean spamThreadDump = false;
        public boolean sparkProfileLaunch = false;
        public boolean integratedServerWatchdog = true;
        public boolean clearMixinClassinfo = false;
    }

    public static class ExpertOnlySection extends ConfigSection {
        public Bugfixes bugfixes = new Bugfixes();
        public Perf perf = new Perf();
        public Misc misc = new Misc();

        public static class Bugfixes extends ConfigSection {
            public boolean bugfixChunkDeadlock = true; // bugfix.chunk_deadlock
            public boolean bugfixCofhCoreCrash = true; // bugfix.cofh_core_crash
            public boolean bugfixConcurrency = true; // bugfix.concurrency
            public boolean bugfixEndIslandOverflow = true; // bugfix.end_island_overflow
            public boolean bugfixExtraExperimentalScreen = true; // bugfix.extra_experimental_screen
            public boolean bugfixMissingBlockEntities = false; // bugfix.missing_block_entities
            public boolean bugfixPaperChunkPatches = true; // bugfix.paper_chunk_patches
            public boolean bugfixRecipeBookTypeDesync = true; // bugfix.recipe_book_type_desync
            public boolean bugfixRestoreOldDragonMovement = false; // bugfix.restore_old_dragon_movement
            public boolean bugfixSingleplayerKeepaliveKick = true; // bugfix.singleplayer_keepalive_kick
            public boolean bugfixUnsafeModdedShapeCaches = true; // bugfix.unsafe_modded_shape_caches
            public boolean bugfixWorldLeaks = true; // bugfix.world_leaks
            public boolean bugfixWorldScreenSkipped = true; // bugfix.world_screen_skipped
        }

        public static class Perf extends ConfigSection {
            public boolean perfAttributeSupplierDedup = true; // perf.attribute_supplier_dedup
            public boolean perfBlockCounting = true; // perf.block_counting
            public boolean perfBlockstatePropertyaccess = true; // perf.blockstate_propertyaccess
            public boolean perfCacheBlockstateCacheArrays = true; // perf.cache_blockstate_cache_arrays
            public boolean perfCacheProfileTextureUrl = true; // perf.cache_profile_texture_url
            public boolean perfCacheStrongholds = true; // perf.cache_strongholds
            public boolean perfChunkMeshing = true; // perf.chunk_meshing
            public boolean perfCompactBitStorage = true; // perf.compact_bit_storage
            public boolean perfCompactEntityModels = true; // perf.compact_entity_models
            public boolean perfCompactImposterprotochunks = true; // perf.compact_imposterprotochunks
            public boolean perfCompactMojangRegistries = true; // perf.compact_mojang_registries
            public boolean perfCompressUnihexFont = true; // perf.compress_unihex_font
            public boolean perfDatapackReloadExceptions = true; // perf.datapack_reload_exceptions
            public boolean perfDedicatedReloadExecutor = true; // perf.dedicated_reload_executor
            public boolean perfDeduplicateAdvancementPredicates = true; // perf.deduplicate_advancement_predicates (from Hydrogen)
            public boolean perfDeduplicateClimateParameters = false; // perf.deduplicate_climate_parameters
            public boolean perfDeduplicateLocation = false; // perf.deduplicate_location
            public boolean perfDeduplicateNbtStrings = true; // perf.deduplicate_nbt_strings (from Hydrogen)
            public boolean perfDeduplicateWallShapes = true; // perf.deduplicate_wall_shapes
            public boolean perfDynamicDfu = true; // perf.dynamic_dfu
            public boolean perfDynamicEntityRenderers = false; // perf.dynamic_entity_renderers
            public boolean perfDynamicLanguages = true; // perf.dynamic_languages
            public boolean perfDynamicSounds = true; // perf.dynamic_sounds
            public boolean perfDynamicStructureManager = true; // perf.dynamic_structure_manager
            public boolean perfEncoderCacheLeak = true; // perf.encoder_cache_leak
            public boolean perfFastBitstorage = true; // perf.fast_bitstorage
            public boolean perfFastBlockEntityRemoval = true; // perf.fast_block_entity_removal
            public boolean perfFastPalette = true; // perf.fast_palette
            public boolean perfFasterCommandSuggestions = true; // perf.faster_command_suggestions
            public boolean perfFasterItemRendering = true; // perf.faster_item_rendering
            public boolean perfFasterTextureStitching = true; // perf.faster_texture_stitching
            public boolean perfGameThreadPriority = true; // perf.game_thread_priority (from Hydrogen)
            public boolean perfGetblock = true; // perf.getblock
            public boolean perfLazySearchTreeRegistry = true; // perf.lazy_search_tree_registry
            public boolean perfMemoizeCreativeTabBuild = true; // perf.memoize_creative_tab_build
            public boolean perfModelOptimizations = true; // perf.model_optimizations
            public boolean perfMobSpawning = true; // perf.mob_spawning
            public boolean perfPoiLookup = true; // perf.poi_lookup
            public boolean perfMojangRegistrySize = true; // perf.mojang_registry_size
            public boolean perfRandomTicking = true; // perf.random_ticking (requires perf.block_counting)
            public boolean perfRemoveBiomeTemperatureCache = true; // perf.remove_biome_temperature_cache
            public boolean perfResourcepacks = true; // perf.resourcepacks
            public boolean perfStateDefinitionConstruct = true; // perf.state_definition_construct
            public boolean perfTagIdCaching = true; // perf.tag_id_caching
            public boolean perfThreadPriorities = true; // perf.thread_priorities
            public boolean perfThreadUnsafeRandom = true; // perf.thread_unsafe_random
            public boolean perfTickingChunkAlloc = true; // perf.ticking_chunk_alloc
            public boolean perfWorldgenAllocation = true; // perf.worldgen_allocation
        }

        public static class Misc extends ConfigSection {
            public boolean core = true; // core
            public boolean featureBlockentityIncorrectThread = false; // feature.blockentity_incorrect_thread
            public boolean featureBranding = true; // feature.branding
            public boolean featureCauseLagByDisablingThreads = false; // feature.cause_lag_by_disabling_threads
            public boolean featureDirectStackTrace = false; // feature.direct_stack_trace
            public boolean featureMcfunctionProfiling = true; // feature.mcfunction_profiling
            public boolean featureMeasureTime = true; // feature.measure_time
            public boolean featureRemoveChatSigning = false; // feature.remove_chat_signing
            public boolean featureRemoveTelemetry = true; // feature.remove_telemetry
            public boolean featureSparkProfileWorldJoin = true; // feature.spark_profile_world_join
            public boolean featureStalledChunkLoadDetection = false; // feature.stalled_chunk_load_detection
            public boolean featureSuppressNarratorStacktrace = true; // feature.suppress_narrator_stacktrace
            public boolean snapshotEasterEgg = true;
        }
    }
}
