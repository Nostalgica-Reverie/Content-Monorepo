package net.nostalgica.modernica;

import net.minecraft.client.Minecraft;
import net.minecraft.server.MinecraftServer;
import net.minecraft.util.MemoryReserve;
import net.nostalgica.modernica.api.constants.IntegrationConstants;
import net.nostalgica.modernica.api.entrypoint.ModernicaClientIntegration;
import net.nostalgica.modernica.core.ModernicaMixinPlugin;
import net.nostalgica.modernica.platform.ModernicaPlatformHooks;
import net.nostalgica.modernica.spark.SparkLaunchProfiler;
import net.nostalgica.modernica.util.ClassInfoManager;
import net.nostalgica.modernica.world.IntegratedWatchdog;

import java.lang.management.ManagementFactory;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;

public class ModernicaClient {
    public static ModernicaClient INSTANCE;
    public static long worldLoadStartTime = -1;
    private static int numRenderTicks;

    public static float gameStartTimeSeconds = -1;

    public static boolean recipesUpdated, tagsUpdated = false;

    public String brandingString = null;

    /**
     * The list of loaded client integrations.
     */
    public static List<ModernicaClientIntegration> CLIENT_INTEGRATIONS = new CopyOnWriteArrayList<>();

    public ModernicaClient() {
        INSTANCE = this;
        // clear reserve as it's not needed
        MemoryReserve.release();
        if(ModernicaMixinPlugin.instance.isOptionEnabled("feature.branding.F3Screen")) {
            brandingString = Modernica.NAME + " " + ModernicaPlatformHooks.INSTANCE.getVersionString();
        }
        for(String className : ModernicaPlatformHooks.INSTANCE.getCustomModOptions().get(IntegrationConstants.CLIENT_INTEGRATION_CLASS)) {
            try {
                CLIENT_INTEGRATIONS.add((ModernicaClientIntegration)Class.forName(className).getDeclaredConstructor().newInstance());
            } catch(ReflectiveOperationException | ClassCastException e) {
                Modernica.LOGGER.error("Could not instantiate integration {}", className, e);
            }
        }

        if(ModernicaMixinPlugin.instance.isOptionEnabled("perf.dynamic_resources.FireIntegrationHook")) {
            for(ModernicaClientIntegration integration : ModernicaClient.CLIENT_INTEGRATIONS) {
                integration.onDynamicResourcesStatusChange(true);
            }
        }
    }

    public void resetWorldLoadStateMachine() {
        numRenderTicks = 0;
        worldLoadStartTime = -1;
        recipesUpdated = false;
        tagsUpdated = false;
    }

    public void onGameLaunchFinish() {
        if(gameStartTimeSeconds >= 0)
            return;
        gameStartTimeSeconds = ManagementFactory.getRuntimeMXBean().getUptime() / 1000f;
        if(ModernicaMixinPlugin.instance.isOptionEnabled("feature.measure_time.GameLoad"))
            Modernica.LOGGER.warn("Game took " + gameStartTimeSeconds + " seconds to start");
        ModernicaPlatformHooks.INSTANCE.onLaunchComplete();
        ClassInfoManager.clear();
    }

    public void onRecipesUpdated() {
        recipesUpdated = true;
    }

    public void onTagsUpdated() {
        tagsUpdated = true;
    }

    public void onRenderTickEnd() {
        if(recipesUpdated
                && tagsUpdated
                && worldLoadStartTime != -1
                && Minecraft.getInstance().player != null
                && numRenderTicks++ >= 10) {
            float timeSpentLoading = ((float)(System.nanoTime() - worldLoadStartTime) / 1000000000f);
            if(ModernicaMixinPlugin.instance.isOptionEnabled("feature.measure_time.WorldLoad")) {
                Modernica.LOGGER.warn("Time from main menu to in-game was " + timeSpentLoading + " seconds");
                Modernica.LOGGER.warn("Total time to load game and open world was " + (timeSpentLoading + gameStartTimeSeconds) + " seconds");
            }
            if (ModernicaPlatformHooks.INSTANCE.modPresent("spark") && ModernicaMixinPlugin.instance.isOptionEnabled("feature.spark_profile_world_join.WorldJoin")) {
                SparkLaunchProfiler.stop("world_join");
            }
            resetWorldLoadStateMachine();
        }
    }

    public void onServerStarted(MinecraftServer server) {
        if(!ModernicaMixinPlugin.instance.isOptionEnabled("feature.integrated_server_watchdog.IntegratedWatchdog"))
            return;
        IntegratedWatchdog watchdog = new IntegratedWatchdog(server);
        watchdog.start();
    }
}
