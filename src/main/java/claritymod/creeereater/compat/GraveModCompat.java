package claritymod.creeereater.compat;

import java.lang.reflect.Method;
import java.util.Set;
import net.fabricmc.loader.api.FabricLoader;

public final class GraveModCompat {
	private static final Set<String> SUPPORTED_MOD_IDS = Set.of(
		"yigd",
		"universal-graves",
		"corpse"
	);

	private static final boolean SUPPORTED_GRAVE_MOD_LOADED = SUPPORTED_MOD_IDS.stream().anyMatch(GraveModCompat::isModLoaded);

	private GraveModCompat() {
	}

	public static boolean isSupportedGraveModLoaded() {
		return SUPPORTED_GRAVE_MOD_LOADED;
	}

	private static boolean isModLoaded(String modId) {
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
