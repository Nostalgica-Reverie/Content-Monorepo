package claritymod.creeereater.compat;

//? if fabric {
import net.fabricmc.loader.api.FabricLoader;
//?} else {
/*import net.neoforged.fml.ModList;*/
//?}

public final class ModLoaderCompat {
	private ModLoaderCompat() {
	}

	public static boolean isModLoaded(String modId) {
		//? if fabric {
		return FabricLoader.getInstance().isModLoaded(modId);
		//?} else {
		/*return ModList.get().isLoaded(modId);*/
		//?}
	}
}
