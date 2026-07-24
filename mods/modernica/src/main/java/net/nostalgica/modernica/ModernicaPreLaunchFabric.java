package net.nostalgica.modernica;

import net.fabricmc.loader.api.FabricLoader;
import net.fabricmc.loader.api.entrypoint.PreLaunchEntrypoint;
import net.fabricmc.loader.impl.gui.FabricGuiEntry;
import net.fabricmc.loader.impl.gui.FabricStatusTree;
import net.nostalgica.modernica.core.ModernicaMixinPlugin;
import net.nostalgica.modernica.spark.SparkLaunchProfiler;
import net.nostalgica.modernica.util.CommonModUtil;

public class ModernicaPreLaunchFabric implements PreLaunchEntrypoint {
    @Override
    public void onPreLaunch() {
        if(ModernicaMixinPlugin.instance == null) {
            System.err.println("Mixin plugin not loaded yet");
            return;
        }
        if(ModernicaMixinPlugin.instance.isOptionEnabled("feature.spark_profile_launch.OnFabric")
                && FabricLoader.getInstance().isModLoaded("spark")) {
            CommonModUtil.runWithoutCrash(() -> SparkLaunchProfiler.start("launch"), "Failed to start profiler");
        }

        // Prevent launching with Continuity when dynamic resources is on
        if(false && ModernicaMixinPlugin.instance.isOptionEnabled("perf.dynamic_resources.ContinuityCheck")
                && FabricLoader.getInstance().isModLoaded("continuity")) {
            CommonModUtil.runWithoutCrash(() -> {
                FabricGuiEntry.displayError("Compatibility warning", null, tree -> {
                    FabricStatusTree.FabricStatusTab crashTab = tree.addTab("Warning");
                    crashTab.node.addMessage("Continuity and Modernica's dynamic resources option are not compatible before Minecraft 1.19.4.", FabricStatusTree.FabricTreeWarningLevel.ERROR);
                    crashTab.node.addMessage("Remove Continuity or disable dynamic resources in the Modernica config.", FabricStatusTree.FabricTreeWarningLevel.ERROR);
                    tree.tabs.removeIf(tab -> tab != crashTab);
                }, true);
            }, "display Continuity warning");
        }
    }
}
