package claritymod.creeereater.compat;

import java.lang.reflect.Method;
import net.fabricmc.loader.api.FabricLoader;

public final class ModLoaderCompat {
	private ModLoaderCompat() {
	}

	public static boolean isModLoaded(String modId) {
		if (FabricLoader.getInstance().isModLoaded(modId)) {
			return true;
		}

		try {
			Class<?> modListClass = Class.forName("net.neoforged.fml.ModList");
			Object modList = modListClass.getMethod("get").invoke(null);
			Method isLoaded = modListClass.getMethod("isLoaded", String.class);
			return Boolean.TRUE.equals(isLoaded.invoke(modList, modId));
		} catch (ReflectiveOperationException | LinkageError ignored) {
			return false;
		}
	}
}
