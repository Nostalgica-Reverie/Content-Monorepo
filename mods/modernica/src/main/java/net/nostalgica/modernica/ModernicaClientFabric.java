package net.nostalgica.modernica;

import me.fzzyhmstrs.fzzy_config.api.ConfigApiJava;
import me.fzzyhmstrs.fzzy_config.api.RegisterType;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.loader.api.FabricLoader;
import net.nostalgica.modernica.core.ModernicaMixinPlugin;
import net.nostalgica.modernica.core.config.ModernicaConfig;
import net.nostalgica.modernica.fabric.datagen.RuntimeDatagen;

public class ModernicaClientFabric implements ClientModInitializer {
    public static ModernicaClient commonMod;

    @Override
    public void onInitializeClient() {
        commonMod = new ModernicaClient();

        ConfigApiJava.registerConfig(ModernicaMixinPlugin.instance.config, ModernicaConfig::new, RegisterType.CLIENT);

        if (ModernicaMixinPlugin.instance.isOptionEnabled("perf.network_optimizations")) {
            Modernica.LOGGER.info("Network optimizations (from Krypton) are accelerating this client's networking stack "
                    + "- note these are most effective on servers, not the client");
        }

        if(FabricLoader.getInstance().isModLoaded("fabric-data-generation-api-v1")) {
            RuntimeDatagen.init();
        }
    }
}
