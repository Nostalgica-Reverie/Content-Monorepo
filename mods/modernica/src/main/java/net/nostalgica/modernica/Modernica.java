package net.nostalgica.modernica;

import net.minecraft.SharedConstants;
import net.minecraft.TracingExecutor;
import net.minecraft.util.Util;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ChunkMap;
import net.minecraft.server.level.ServerLevel;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import net.nostalgica.modernica.command.ModernicaCommands;
import net.nostalgica.modernica.core.ModernicaMixinPlugin;
import net.nostalgica.modernica.platform.ModernicaPlatformHooks;
import net.nostalgica.modernica.resources.ReloadExecutor;
import net.nostalgica.modernica.util.ClassInfoManager;
import org.spongepowered.asm.mixin.MixinEnvironment;

import net.minecraft.client.Minecraft;
import java.lang.management.ManagementFactory;

// The value here should match an entry in the META-INF/mods.toml file
public class Modernica {

    // Directly reference a log4j logger.
    public static final Logger LOGGER = LogManager.getLogger("Modernica");

    public static final String MODID = "modernica";

    public static String NAME = "Modernica";

    public static Modernica INSTANCE;

    // Used to skip computing the blockstate caches twice
    public static boolean runningFirstInjection = false;

    private static TracingExecutor resourceReloadService = null;

    static {
        if(ModernicaMixinPlugin.instance.isOptionEnabled("perf.dedicated_reload_executor.ReloadExecutor")) {
            resourceReloadService = new TracingExecutor(ReloadExecutor.createCustomResourceReloadExecutor());
        } else {
            resourceReloadService = Util.backgroundExecutor();
        }
    }

    public static TracingExecutor resourceReloadExecutor() {
        return resourceReloadService;
    }

    public static void runAuditIfRequested() {
        boolean auditAndExit = Boolean.getBoolean("modernica.auditAndExit");
        if (auditAndExit || Boolean.getBoolean("modernica.auditMixinsAtStart")) {
            MixinEnvironment.getCurrentEnvironment().audit();
            if (auditAndExit) {
                // Prevents Crash Assistant from treating mixin audit as a crash
                Minecraft.getInstance().stop();
                System.exit(0);
            }
        }
    }

    public Modernica() {
        INSTANCE = this;
        if(ModernicaMixinPlugin.instance.isOptionEnabled("feature.snapshot_easter_egg.NameChange") && !SharedConstants.getCurrentVersion().stable())
            NAME = "PreemptiveFix";
        ModernicaPlatformHooks.INSTANCE.onServerCommandRegister(ModernicaCommands::register);
    }

    public void onServerStarted() {
        if(ModernicaPlatformHooks.INSTANCE.isDedicatedServer()) {
            float gameStartTime = ManagementFactory.getRuntimeMXBean().getUptime() / 1000f;
            if(ModernicaMixinPlugin.instance.isOptionEnabled("feature.measure_time.ServerLoad"))
                Modernica.LOGGER.warn("Dedicated server took " + gameStartTime + " seconds to load");
            ModernicaPlatformHooks.INSTANCE.onLaunchComplete();
        }
        ClassInfoManager.clear();
    }

    @SuppressWarnings("ConstantValue")
    public void onServerDead(MinecraftServer server) {
        /* Clear as much data from the integrated server as possible, in case a mod holds on to it */
        try {
            for(ServerLevel level : server.getAllLevels()) {
                ChunkMap chunkMap = level.getChunkSource().chunkMap;
                // Null check for mods that replace chunk system
                if(chunkMap.updatingChunkMap != null)
                    chunkMap.updatingChunkMap.clear();
                if(chunkMap.visibleChunkMap != null)
                    chunkMap.visibleChunkMap.clear();
                if(chunkMap.pendingUnloads != null)
                    chunkMap.pendingUnloads.clear();
            }
        } catch(RuntimeException e) {
            Modernica.LOGGER.error("Couldn't clear chunk data", e);
        }
    }
}
