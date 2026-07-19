package claritymod.creepereater.compat;

import java.util.Set;

public final class GraveModCompat {
	private static final Set<String> SUPPORTED_MOD_IDS = Set.of(
		"yigd",
		"universal-graves",
		"corpse"
	);

	private static final boolean SUPPORTED_GRAVE_MOD_LOADED = SUPPORTED_MOD_IDS.stream().anyMatch(ModLoaderCompat::isModLoaded);

	private GraveModCompat() {
	}

	public static boolean isSupportedGraveModLoaded() {
		return SUPPORTED_GRAVE_MOD_LOADED;
	}
}
