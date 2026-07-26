package net.nostalgica.modernica.core.config;

import net.fabricmc.loader.api.FabricLoader;
import org.apache.logging.log4j.Logger;

import java.util.LinkedHashMap;
import java.util.Map;
import java.util.function.BiConsumer;
import java.util.function.BooleanSupplier;
import java.util.function.Consumer;
import java.util.function.Function;

/** Maps a mixin's dotted key to its {@link ModernicaConfig} field. Gating decisions always read from
 * {@link #earlyValues} (raw TOML, no Identifier touched - see {@link EarlyMixinOptions}); the real
 * config is bound later purely for the GUI/sync/save experience via {@link #bindRealConfig}. */
public final class MixinGate {
    private MixinGate() {}

    private record Toggle(String section, String field, boolean defaultValue, BooleanSupplier lateGetter, Consumer<Boolean> lateSetter) {}

    private static final Map<String, Toggle> REGISTRY = new LinkedHashMap<>();
    private static Map<String, Boolean> earlyValues = Map.of();
    /** Never {@link ModernicaConfig.StabilityLevel} - see {@link EarlyStabilityLevel} for why touching it
     * from the Mixin plugin's constructor breaks other mods' mixins. */
    private static EarlyStabilityLevel earlyStabilityLevel = EarlyStabilityLevel.GA;
    private static ModernicaConfig config;

    private static void register(String key, String section, String field, boolean defaultValue, BooleanSupplier lateGetter, Consumer<Boolean> lateSetter) {
        if (REGISTRY.put(key, new Toggle(section, field, defaultValue, lateGetter, lateSetter)) != null) {
            throw new IllegalStateException("Duplicate mixin gate registered for '" + key + "'");
        }
    }

    /** Must be called once, from the Mixin plugin's constructor, before any {@link #isEnabled} call. */
    public static void registerAll(Logger logger) {
        REGISTRY.clear();

        register("bugfix.chunk_deadlock", "expertOnly.bugfixes", "bugfixChunkDeadlock", true, () -> config.expertOnly.bugfixes.bugfixChunkDeadlock, v -> config.expertOnly.bugfixes.bugfixChunkDeadlock = v);
        register("bugfix.cofh_core_crash", "expertOnly.bugfixes", "bugfixCofhCoreCrash", true, () -> config.expertOnly.bugfixes.bugfixCofhCoreCrash, v -> config.expertOnly.bugfixes.bugfixCofhCoreCrash = v);
        register("bugfix.concurrency", "expertOnly.bugfixes", "bugfixConcurrency", true, () -> config.expertOnly.bugfixes.bugfixConcurrency, v -> config.expertOnly.bugfixes.bugfixConcurrency = v);
        register("bugfix.end_island_overflow", "expertOnly.bugfixes", "bugfixEndIslandOverflow", true, () -> config.expertOnly.bugfixes.bugfixEndIslandOverflow, v -> config.expertOnly.bugfixes.bugfixEndIslandOverflow = v);
        register("bugfix.extra_experimental_screen", "expertOnly.bugfixes", "bugfixExtraExperimentalScreen", true, () -> config.expertOnly.bugfixes.bugfixExtraExperimentalScreen, v -> config.expertOnly.bugfixes.bugfixExtraExperimentalScreen = v);
        register("bugfix.missing_block_entities", "expertOnly.bugfixes", "bugfixMissingBlockEntities", false, () -> config.expertOnly.bugfixes.bugfixMissingBlockEntities, v -> config.expertOnly.bugfixes.bugfixMissingBlockEntities = v);
        register("bugfix.paper_chunk_patches", "expertOnly.bugfixes", "bugfixPaperChunkPatches", true, () -> config.expertOnly.bugfixes.bugfixPaperChunkPatches, v -> config.expertOnly.bugfixes.bugfixPaperChunkPatches = v);
        register("bugfix.recipe_book_type_desync", "expertOnly.bugfixes", "bugfixRecipeBookTypeDesync", true, () -> config.expertOnly.bugfixes.bugfixRecipeBookTypeDesync, v -> config.expertOnly.bugfixes.bugfixRecipeBookTypeDesync = v);
        register("bugfix.restore_old_dragon_movement", "expertOnly.bugfixes", "bugfixRestoreOldDragonMovement", false, () -> config.expertOnly.bugfixes.bugfixRestoreOldDragonMovement, v -> config.expertOnly.bugfixes.bugfixRestoreOldDragonMovement = v);
        register("bugfix.singleplayer_keepalive_kick", "expertOnly.bugfixes", "bugfixSingleplayerKeepaliveKick", true, () -> config.expertOnly.bugfixes.bugfixSingleplayerKeepaliveKick, v -> config.expertOnly.bugfixes.bugfixSingleplayerKeepaliveKick = v);
        register("bugfix.unsafe_modded_shape_caches", "expertOnly.bugfixes", "bugfixUnsafeModdedShapeCaches", true, () -> config.expertOnly.bugfixes.bugfixUnsafeModdedShapeCaches, v -> config.expertOnly.bugfixes.bugfixUnsafeModdedShapeCaches = v);
        register("bugfix.world_leaks", "expertOnly.bugfixes", "bugfixWorldLeaks", true, () -> config.expertOnly.bugfixes.bugfixWorldLeaks, v -> config.expertOnly.bugfixes.bugfixWorldLeaks = v);
        register("bugfix.world_screen_skipped", "expertOnly.bugfixes", "bugfixWorldScreenSkipped", true, () -> config.expertOnly.bugfixes.bugfixWorldScreenSkipped, v -> config.expertOnly.bugfixes.bugfixWorldScreenSkipped = v);
        register("core", "expertOnly.misc", "core", true, () -> config.expertOnly.misc.core, v -> config.expertOnly.misc.core = v);
        register("devenv", "", "devEnvironmentMixins", FabricLoader.getInstance().isDevelopmentEnvironment(), () -> config.devEnvironmentMixins, v -> config.devEnvironmentMixins = v);
        register("feature.blockentity_incorrect_thread", "expertOnly.misc", "featureBlockentityIncorrectThread", false, () -> config.expertOnly.misc.featureBlockentityIncorrectThread, v -> config.expertOnly.misc.featureBlockentityIncorrectThread = v);
        register("feature.branding", "expertOnly.misc", "featureBranding", true, () -> config.expertOnly.misc.featureBranding, v -> config.expertOnly.misc.featureBranding = v);
        register("feature.cause_lag_by_disabling_threads", "expertOnly.misc", "featureCauseLagByDisablingThreads", false, () -> config.expertOnly.misc.featureCauseLagByDisablingThreads, v -> config.expertOnly.misc.featureCauseLagByDisablingThreads = v);
        register("feature.direct_stack_trace", "expertOnly.misc", "featureDirectStackTrace", false, () -> config.expertOnly.misc.featureDirectStackTrace, v -> config.expertOnly.misc.featureDirectStackTrace = v);
        register("feature.mcfunction_profiling", "expertOnly.misc", "featureMcfunctionProfiling", true, () -> config.expertOnly.misc.featureMcfunctionProfiling, v -> config.expertOnly.misc.featureMcfunctionProfiling = v);
        register("feature.measure_time", "expertOnly.misc", "featureMeasureTime", true, () -> config.expertOnly.misc.featureMeasureTime, v -> config.expertOnly.misc.featureMeasureTime = v);
        register("feature.remove_chat_signing", "expertOnly.misc", "featureRemoveChatSigning", false, () -> config.expertOnly.misc.featureRemoveChatSigning, v -> config.expertOnly.misc.featureRemoveChatSigning = v);
        register("feature.remove_telemetry", "expertOnly.misc", "featureRemoveTelemetry", true, () -> config.expertOnly.misc.featureRemoveTelemetry, v -> config.expertOnly.misc.featureRemoveTelemetry = v);
        register("feature.spark_profile_world_join", "expertOnly.misc", "featureSparkProfileWorldJoin", true, () -> config.expertOnly.misc.featureSparkProfileWorldJoin, v -> config.expertOnly.misc.featureSparkProfileWorldJoin = v);
        register("feature.stalled_chunk_load_detection", "expertOnly.misc", "featureStalledChunkLoadDetection", false, () -> config.expertOnly.misc.featureStalledChunkLoadDetection, v -> config.expertOnly.misc.featureStalledChunkLoadDetection = v);
        register("feature.suppress_narrator_stacktrace", "expertOnly.misc", "featureSuppressNarratorStacktrace", true, () -> config.expertOnly.misc.featureSuppressNarratorStacktrace, v -> config.expertOnly.misc.featureSuppressNarratorStacktrace = v);
        register("perf.attribute_supplier_dedup", "expertOnly.perf", "perfAttributeSupplierDedup", true, () -> config.expertOnly.perf.perfAttributeSupplierDedup, v -> config.expertOnly.perf.perfAttributeSupplierDedup = v);
        register("perf.block_counting", "expertOnly.perf", "perfBlockCounting", true, () -> config.expertOnly.perf.perfBlockCounting, v -> config.expertOnly.perf.perfBlockCounting = v);
        register("perf.blockstate_propertyaccess", "expertOnly.perf", "perfBlockstatePropertyaccess", true, () -> config.expertOnly.perf.perfBlockstatePropertyaccess, v -> config.expertOnly.perf.perfBlockstatePropertyaccess = v);
        register("perf.cache_blockstate_cache_arrays", "expertOnly.perf", "perfCacheBlockstateCacheArrays", true, () -> config.expertOnly.perf.perfCacheBlockstateCacheArrays, v -> config.expertOnly.perf.perfCacheBlockstateCacheArrays = v);
        register("perf.cache_profile_texture_url", "expertOnly.perf", "perfCacheProfileTextureUrl", true, () -> config.expertOnly.perf.perfCacheProfileTextureUrl, v -> config.expertOnly.perf.perfCacheProfileTextureUrl = v);
        register("perf.cache_strongholds", "expertOnly.perf", "perfCacheStrongholds", true, () -> config.expertOnly.perf.perfCacheStrongholds, v -> config.expertOnly.perf.perfCacheStrongholds = v);
        register("perf.chunk_meshing", "expertOnly.perf", "perfChunkMeshing", true, () -> config.expertOnly.perf.perfChunkMeshing, v -> config.expertOnly.perf.perfChunkMeshing = v);
        register("perf.compact_bit_storage", "expertOnly.perf", "perfCompactBitStorage", true, () -> config.expertOnly.perf.perfCompactBitStorage, v -> config.expertOnly.perf.perfCompactBitStorage = v);
        register("perf.compact_entity_models", "expertOnly.perf", "perfCompactEntityModels", true, () -> config.expertOnly.perf.perfCompactEntityModels, v -> config.expertOnly.perf.perfCompactEntityModels = v);
        register("perf.compact_imposterprotochunks", "expertOnly.perf", "perfCompactImposterprotochunks", true, () -> config.expertOnly.perf.perfCompactImposterprotochunks, v -> config.expertOnly.perf.perfCompactImposterprotochunks = v);
        register("perf.compact_mojang_registries", "expertOnly.perf", "perfCompactMojangRegistries", true, () -> config.expertOnly.perf.perfCompactMojangRegistries, v -> config.expertOnly.perf.perfCompactMojangRegistries = v);
        register("perf.compress_unihex_font", "expertOnly.perf", "perfCompressUnihexFont", true, () -> config.expertOnly.perf.perfCompressUnihexFont, v -> config.expertOnly.perf.perfCompressUnihexFont = v);
        register("perf.datapack_reload_exceptions", "expertOnly.perf", "perfDatapackReloadExceptions", true, () -> config.expertOnly.perf.perfDatapackReloadExceptions, v -> config.expertOnly.perf.perfDatapackReloadExceptions = v);
        register("perf.dedicated_reload_executor", "expertOnly.perf", "perfDedicatedReloadExecutor", true, () -> config.expertOnly.perf.perfDedicatedReloadExecutor, v -> config.expertOnly.perf.perfDedicatedReloadExecutor = v);
        register("perf.deduplicate_advancement_predicates", "expertOnly.perf", "perfDeduplicateAdvancementPredicates", true, () -> config.expertOnly.perf.perfDeduplicateAdvancementPredicates, v -> config.expertOnly.perf.perfDeduplicateAdvancementPredicates = v);
        register("perf.deduplicate_climate_parameters", "expertOnly.perf", "perfDeduplicateClimateParameters", false, () -> config.expertOnly.perf.perfDeduplicateClimateParameters, v -> config.expertOnly.perf.perfDeduplicateClimateParameters = v);
        register("perf.deduplicate_location", "expertOnly.perf", "perfDeduplicateLocation", false, () -> config.expertOnly.perf.perfDeduplicateLocation, v -> config.expertOnly.perf.perfDeduplicateLocation = v);
        register("perf.deduplicate_nbt_strings", "expertOnly.perf", "perfDeduplicateNbtStrings", true, () -> config.expertOnly.perf.perfDeduplicateNbtStrings, v -> config.expertOnly.perf.perfDeduplicateNbtStrings = v);
        register("perf.deduplicate_wall_shapes", "expertOnly.perf", "perfDeduplicateWallShapes", true, () -> config.expertOnly.perf.perfDeduplicateWallShapes, v -> config.expertOnly.perf.perfDeduplicateWallShapes = v);
        register("perf.dynamic_dfu", "expertOnly.perf", "perfDynamicDfu", true, () -> config.expertOnly.perf.perfDynamicDfu, v -> config.expertOnly.perf.perfDynamicDfu = v);
        register("perf.dynamic_entity_renderers", "expertOnly.perf", "perfDynamicEntityRenderers", false, () -> config.expertOnly.perf.perfDynamicEntityRenderers, v -> config.expertOnly.perf.perfDynamicEntityRenderers = v);
        register("perf.dynamic_languages", "expertOnly.perf", "perfDynamicLanguages", true, () -> config.expertOnly.perf.perfDynamicLanguages, v -> config.expertOnly.perf.perfDynamicLanguages = v);
        register("perf.dynamic_resources", "performance", "perfDynamicResources", false, () -> config.performance.perfDynamicResources, v -> config.performance.perfDynamicResources = v);
        register("perf.dynamic_sounds", "expertOnly.perf", "perfDynamicSounds", true, () -> config.expertOnly.perf.perfDynamicSounds, v -> config.expertOnly.perf.perfDynamicSounds = v);
        register("perf.dynamic_structure_manager", "expertOnly.perf", "perfDynamicStructureManager", true, () -> config.expertOnly.perf.perfDynamicStructureManager, v -> config.expertOnly.perf.perfDynamicStructureManager = v);
        register("perf.encoder_cache_leak", "expertOnly.perf", "perfEncoderCacheLeak", true, () -> config.expertOnly.perf.perfEncoderCacheLeak, v -> config.expertOnly.perf.perfEncoderCacheLeak = v);
        register("perf.fast_bitstorage", "expertOnly.perf", "perfFastBitstorage", true, () -> config.expertOnly.perf.perfFastBitstorage, v -> config.expertOnly.perf.perfFastBitstorage = v);
        register("perf.fast_block_entity_removal", "expertOnly.perf", "perfFastBlockEntityRemoval", true, () -> config.expertOnly.perf.perfFastBlockEntityRemoval, v -> config.expertOnly.perf.perfFastBlockEntityRemoval = v);
        register("perf.fast_palette", "expertOnly.perf", "perfFastPalette", true, () -> config.expertOnly.perf.perfFastPalette, v -> config.expertOnly.perf.perfFastPalette = v);
        register("perf.faster_command_suggestions", "expertOnly.perf", "perfFasterCommandSuggestions", true, () -> config.expertOnly.perf.perfFasterCommandSuggestions, v -> config.expertOnly.perf.perfFasterCommandSuggestions = v);
        register("perf.faster_item_rendering", "expertOnly.perf", "perfFasterItemRendering", true, () -> config.expertOnly.perf.perfFasterItemRendering, v -> config.expertOnly.perf.perfFasterItemRendering = v);
        register("perf.faster_texture_stitching", "expertOnly.perf", "perfFasterTextureStitching", true, () -> config.expertOnly.perf.perfFasterTextureStitching, v -> config.expertOnly.perf.perfFasterTextureStitching = v);
        register("perf.game_thread_priority", "expertOnly.perf", "perfGameThreadPriority", true, () -> config.expertOnly.perf.perfGameThreadPriority, v -> config.expertOnly.perf.perfGameThreadPriority = v);
        register("perf.getblock", "expertOnly.perf", "perfGetblock", true, () -> config.expertOnly.perf.perfGetblock, v -> config.expertOnly.perf.perfGetblock = v);
        register("perf.lazy_search_tree_registry", "expertOnly.perf", "perfLazySearchTreeRegistry", true, () -> config.expertOnly.perf.perfLazySearchTreeRegistry, v -> config.expertOnly.perf.perfLazySearchTreeRegistry = v);
        register("perf.memoize_creative_tab_build", "expertOnly.perf", "perfMemoizeCreativeTabBuild", true, () -> config.expertOnly.perf.perfMemoizeCreativeTabBuild, v -> config.expertOnly.perf.perfMemoizeCreativeTabBuild = v);
        register("perf.model_optimizations", "expertOnly.perf", "perfModelOptimizations", true, () -> config.expertOnly.perf.perfModelOptimizations, v -> config.expertOnly.perf.perfModelOptimizations = v);
        register("perf.mob_spawning", "expertOnly.perf", "perfMobSpawning", true, () -> config.expertOnly.perf.perfMobSpawning, v -> config.expertOnly.perf.perfMobSpawning = v);
        register("perf.poi_lookup", "expertOnly.perf", "perfPoiLookup", true, () -> config.expertOnly.perf.perfPoiLookup, v -> config.expertOnly.perf.perfPoiLookup = v);
        register("perf.mojang_registry_size", "expertOnly.perf", "perfMojangRegistrySize", true, () -> config.expertOnly.perf.perfMojangRegistrySize, v -> config.expertOnly.perf.perfMojangRegistrySize = v);
        register("perf.random_ticking", "expertOnly.perf", "perfRandomTicking", true, () -> config.expertOnly.perf.perfRandomTicking, v -> config.expertOnly.perf.perfRandomTicking = v);
        register("perf.network_optimizations", "performance", "perfNetworkOptimizations", true, () -> config.performance.perfNetworkOptimizations, v -> config.performance.perfNetworkOptimizations = v);
        register("perf.network_enhancements", "performance", "perfNetworkEnhancements", true, () -> config.performance.perfNetworkEnhancements, v -> config.performance.perfNetworkEnhancements = v);
        register("perf.release_protochunks", "performance", "perfReleaseProtochunks", true, () -> config.performance.perfReleaseProtochunks, v -> config.performance.perfReleaseProtochunks = v);
        register("feature.fast_ip_ping", "performance", "featureFastIpPing", true, () -> config.performance.featureFastIpPing, v -> config.performance.featureFastIpPing = v);
        register("feature.force_close_loading_screen", "performance", "featureForceCloseLoadingScreen", false, () -> config.performance.featureForceCloseLoadingScreen, v -> config.performance.featureForceCloseLoadingScreen = v);
        register("perf.remove_biome_temperature_cache", "expertOnly.perf", "perfRemoveBiomeTemperatureCache", true, () -> config.expertOnly.perf.perfRemoveBiomeTemperatureCache, v -> config.expertOnly.perf.perfRemoveBiomeTemperatureCache = v);
        register("perf.resourcepacks", "expertOnly.perf", "perfResourcepacks", true, () -> config.expertOnly.perf.perfResourcepacks, v -> config.expertOnly.perf.perfResourcepacks = v);
        register("perf.state_definition_construct", "expertOnly.perf", "perfStateDefinitionConstruct", true, () -> config.expertOnly.perf.perfStateDefinitionConstruct, v -> config.expertOnly.perf.perfStateDefinitionConstruct = v);
        register("perf.tag_id_caching", "expertOnly.perf", "perfTagIdCaching", true, () -> config.expertOnly.perf.perfTagIdCaching, v -> config.expertOnly.perf.perfTagIdCaching = v);
        register("perf.thread_priorities", "expertOnly.perf", "perfThreadPriorities", true, () -> config.expertOnly.perf.perfThreadPriorities, v -> config.expertOnly.perf.perfThreadPriorities = v);
        register("perf.thread_unsafe_random", "expertOnly.perf", "perfThreadUnsafeRandom", true, () -> config.expertOnly.perf.perfThreadUnsafeRandom, v -> config.expertOnly.perf.perfThreadUnsafeRandom = v);
        register("perf.ticking_chunk_alloc", "expertOnly.perf", "perfTickingChunkAlloc", true, () -> config.expertOnly.perf.perfTickingChunkAlloc, v -> config.expertOnly.perf.perfTickingChunkAlloc = v);
        register("perf.worldgen_allocation", "expertOnly.perf", "perfWorldgenAllocation", true, () -> config.expertOnly.perf.perfWorldgenAllocation, v -> config.expertOnly.perf.perfWorldgenAllocation = v);

        register("feature.spam_thread_dump", "troubleshooting", "spamThreadDump", false, () -> config.troubleshooting.spamThreadDump, v -> config.troubleshooting.spamThreadDump = v);
        register("feature.spark_profile_launch", "troubleshooting", "sparkProfileLaunch", false, () -> config.troubleshooting.sparkProfileLaunch, v -> config.troubleshooting.sparkProfileLaunch = v);
        register("feature.snapshot_easter_egg", "expertOnly.misc", "snapshotEasterEgg", true, () -> config.expertOnly.misc.snapshotEasterEgg, v -> config.expertOnly.misc.snapshotEasterEgg = v);
        register("feature.integrated_server_watchdog", "troubleshooting", "integratedServerWatchdog", true, () -> config.troubleshooting.integratedServerWatchdog, v -> config.troubleshooting.integratedServerWatchdog = v);
        register("perf.clear_mixin_classinfo", "troubleshooting", "clearMixinClassinfo", false, () -> config.troubleshooting.clearMixinClassinfo, v -> config.troubleshooting.clearMixinClassinfo = v);

        EarlyMixinOptions early = EarlyMixinOptions.load(logger);
        earlyStabilityLevel = early.resolveStabilityLevel(EarlyStabilityLevel.GA);

        Map<String, Boolean> values = new LinkedHashMap<>();
        for (Map.Entry<String, Toggle> entry : REGISTRY.entrySet()) {
            Toggle toggle = entry.getValue();
            values.put(entry.getKey(), early.resolveBoolean(toggle.section(), toggle.field(), toggle.defaultValue()));
        }
        applyModCompat(logger, values::get, values::put);
        applyJvmPropertyOverrides(logger, values::get, values::put);
        enforceDependencies(logger, values::get, values::put);
        earlyValues = values;
    }

    /** Called once, later, from normal mod init - rebinds mod-compat/JVM overrides against the real
     * config so the GUI/save stay consistent. Cannot change which mixins already applied. */
    public static void bindRealConfig(ModernicaConfig cfg, Logger logger) {
        config = cfg;
        Function<String, Boolean> getter = key -> {
            Toggle toggle = REGISTRY.get(key);
            return toggle == null ? null : toggle.lateGetter().getAsBoolean();
        };
        BiConsumer<String, Boolean> setter = (key, value) -> {
            Toggle toggle = REGISTRY.get(key);
            if (toggle != null) {
                toggle.lateSetter().accept(value);
            }
        };
        applyModCompat(logger, getter, setter);
        applyJvmPropertyOverrides(logger, getter, setter);
        enforceDependencies(logger, getter, setter);
        ModernicaConfig.verifyEarlyStabilityLevelInSync(logger);
    }

    /** {@code perf.random_ticking}'s fast tick-position lookup only exists because
     * {@code perf.block_counting}'s {@code LevelChunkSectionMixin} implements the interface it casts to -
     * without that mixin applied, the cast throws a {@link ClassCastException} the first time a chunk
     * random-ticks. */
    private static void enforceDependencies(Logger logger, Function<String, Boolean> getter, BiConsumer<String, Boolean> setter) {
        Boolean blockCounting = getter.apply("perf.block_counting");
        Boolean randomTicking = getter.apply("perf.random_ticking");
        if (Boolean.TRUE.equals(randomTicking) && !Boolean.TRUE.equals(blockCounting)) {
            setter.accept("perf.random_ticking", false);
            logger.warn("Disabled 'perf.random_ticking': it requires 'perf.block_counting', which is disabled");
        }
    }

    private static void applyModCompat(Logger logger, Function<String, Boolean> getter, BiConsumer<String, Boolean> setter) {
        disableIfModPresent(logger, getter, setter, "perf.thread_priorities", "smoothboot", "threadtweak");
        disableIfModPresent(logger, getter, setter, "bugfix.chunk_deadlock", "c2me", "dimthread");
        disableIfModPresent(logger, getter, setter, "perf.release_protochunks", "c2me", "moonrise");
        disableIfModPresent(logger, getter, setter, "perf.faster_texture_stitching", "optifine");
        disableIfModPresent(logger, getter, setter, "perf.datapack_reload_exceptions", "cyanide");
        disableIfModPresent(logger, getter, setter, "feature.remove_chat_signing", "nochatreports");
        disableIfModPresent(logger, getter, setter, "perf.deduplicate_wall_shapes", "dashloader");
        disableIfModPresent(logger, getter, setter, "perf.cache_strongholds", "littletiles", "c2me", "flashback");
        disableIfModPresent(logger, getter, setter, "perf.dynamic_dfu", "litematica");
        // the real Moonrise mixins these same vanilla classes; our independent reimplementations
        // of the same techniques would otherwise silently fight over @Overwrite on the same methods
        disableIfModPresent(logger, getter, setter, "bugfix.end_island_overflow", "moonrise");
        disableIfModPresent(logger, getter, setter, "bugfix.singleplayer_keepalive_kick", "moonrise");
        // Lithium's chunk.no_validation.SimpleBitStorageMixin/ZeroBitStorageMixin do the same
        // get/set/getAndSet @Overwrite
        disableIfModPresent(logger, getter, setter, "perf.fast_bitstorage", "moonrise", "lithium");
        // Lithium's world.block_entity_ticking.sleeping.* rewrites the same tickBlockEntities loop
        disableIfModPresent(logger, getter, setter, "perf.fast_block_entity_removal", "moonrise", "lithium");
        // Lithium's alloc.chunk_random.LevelMixin/ServerLevelMixin already replaces Level/Entity's
        // random field the same way - this is what caused the "Scanned 0 target(s)" failure
        disableIfModPresent(logger, getter, setter, "perf.thread_unsafe_random", "moonrise", "lithium");
        // Lithium ships several of its own PalettedContainer mixins (locking, serialization,
        // block-ticking, debug) - too much shared surface with our get/getAndSet @Overwrite to trust
        disableIfModPresent(logger, getter, setter, "perf.fast_palette", "moonrise", "lithium");
        // Lithium's own util.block_tracking.LevelChunkSectionMixin also rewrites recalcBlockCounts(),
        // and our @Overwrite removes the injection point it depends on
        disableIfModPresent(logger, getter, setter, "perf.block_counting", "moonrise", "lithium");
        // Lithium's world.chunk_ticking.random_block_ticking.* does the same optimization directly
        disableIfModPresent(logger, getter, setter, "perf.random_ticking", "moonrise", "lithium");
        disableIfModPresent(logger, getter, setter, "perf.mob_spawning", "moonrise");
        // both rewrite StateHolder's property storage from scratch; FerriteCore's FastMapStateHolderMixin
        // injects into getNullableValue's original body, which our @Overwrite replaces outright
        disableIfModPresent(logger, getter, setter, "perf.blockstate_propertyaccess", "moonrise", "ferritecore");
        // Lithium's entire ai.poi.* package (PoiManagerMixin, PortalForcerMixin, AcquirePoiMixin, ...)
        // already reimplements this exact search - both @Overwrite the same PoiManager methods
        disableIfModPresent(logger, getter, setter, "perf.poi_lookup", "moonrise", "lithium");
        // matches the compat note in getblock's own LevelMixin: Lithium ships an equivalent
        // Level-height-caching mixin, so ours would be redundant (priority already defers to it)
        disableIfModPresent(logger, getter, setter, "perf.getblock", "lithium");
    }

    private static void disableIfModPresent(Logger logger, Function<String, Boolean> getter, BiConsumer<String, Boolean> setter, String key, String... modIds) {
        for (String modId : modIds) {
            if (FabricLoader.getInstance().isModLoaded(modId)) {
                Boolean current = getter.apply(key);
                if (current != null && current) {
                    setter.accept(key, false);
                    logger.warn("Disabled mixin group '{}' for compatibility with detected mod '{}'", key, modId);
                }
            }
        }
    }

    private static void applyJvmPropertyOverrides(Logger logger, Function<String, Boolean> getter, BiConsumer<String, Boolean> setter) {
        for (String key : REGISTRY.keySet()) {
            String value = System.getProperty("modernica.config." + key);
            if (value == null || value.isEmpty()) {
                continue;
            }
            if (value.equalsIgnoreCase("true") || value.equalsIgnoreCase("false")) {
                setter.accept(key, Boolean.parseBoolean(value));
                logger.info("Configured '{}' to '{}' via JVM property.", key, value);
            } else {
                logger.warn("Invalid value '{}' for JVM property 'modernica.config.{}', ignoring", value, key);
            }
        }
    }

    /** Walks the dotted path from longest to shortest prefix, returns the first registered ancestor's
     * value, or false (fail-closed) if nothing matches. */
    public static boolean isEnabled(String dottedPath, Logger logger, boolean devEnv) {
        String path = dottedPath;
        while (true) {
            Boolean value = earlyValues.get(path);
            if (value != null) {
                return value;
            }
            int lastDot = path.lastIndexOf('.');
            if (lastDot <= 0) {
                break;
            }
            path = path.substring(0, lastDot);
        }
        String msg = "No rules matched '{}', treating as foreign and disabling!";
        if (devEnv) {
            logger.error(msg, dottedPath);
        } else {
            logger.debug(msg, dottedPath);
        }
        return false;
    }

    private static final Map<String, Boolean> REQUIRES_BETA = Map.of(
            "perf.compact_entity_models", true,
            "perf.dynamic_languages", true,
            "perf.faster_item_rendering", true
    );

    public static boolean meetsFeatureLevel(String mixinGroup) {
        Boolean requiresBeta = REQUIRES_BETA.get(mixinGroup);
        return requiresBeta == null || earlyStabilityLevel.isAtLeast(EarlyStabilityLevel.BETA);
    }

    /** spark_profile_world_join entries: spark is compileOnly, and SparkLaunchProfiler's statics
     * eagerly touch it, so without this gate world creation crashes for anyone without Spark. */
    private static final Map<String, String> REQUIRES_MOD = Map.of(
            "bugfix.cofh_core_crash.FlagManagerMixin", "cofh_core",
            "bugfix.paper_chunk_patches.SortedArraySetMixin", "!moonrise",
            "bugfix.unsafe_modded_shape_caches.ShapeCacheCyclicMixin", "cyclic",
            "bugfix.unsafe_modded_shape_caches.ShapeCacheRSMixin", "refinedstorage",
            "perf.chunk_meshing.RebuildTaskMixin", "!fluidlogged",
            "perf.state_definition_construct.StateDefinitionMixin", "ferritecore",
            "feature.spark_profile_world_join.WorldLoaderMixin", "spark",
            "feature.spark_profile_world_join.MinecraftMixin", "spark"
    );

    public static boolean meetsModRequirement(String fullMixinPath) {
        String rule = REQUIRES_MOD.get(fullMixinPath);
        if (rule == null) {
            return true;
        }
        boolean negated = rule.startsWith("!");
        String modId = negated ? rule.substring(1) : rule;
        boolean present = FabricLoader.getInstance().isModLoaded(modId);
        return negated != present;
    }
}
