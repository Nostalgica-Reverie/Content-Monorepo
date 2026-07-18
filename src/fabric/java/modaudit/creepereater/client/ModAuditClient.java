package modaudit.creepereater.client;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.loader.api.FabricLoader;

public final class ModAuditClient implements ClientModInitializer {
	@Override
	public void onInitializeClient() {
		FabricLoader loader = FabricLoader.getInstance();
		ModAudit.initialize(loader.getConfigDir(), loader.getGameDir().resolve("mods"), loader::isModLoaded);
	}
}
