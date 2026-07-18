package modaudit.creepereater.client;

import net.neoforged.api.distmarker.Dist;
import net.neoforged.fml.ModList;
import net.neoforged.fml.common.Mod;
import net.neoforged.fml.loading.FMLPaths;

@Mod(value = ModAudit.MOD_ID, dist = Dist.CLIENT)
public final class ModAuditClient {
	public ModAuditClient() {
		ModAudit.initialize(FMLPaths.CONFIGDIR.get(), FMLPaths.MODSDIR.get(), ModList.get()::isLoaded);
	}
}
