package modaudit.creepereater.client;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonParseException;

import java.io.IOException;
import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashSet;
import java.util.List;
import java.util.function.Predicate;

public final class ModAuditConfig {
	private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

	private ModAuditConfig() {}

	public static List<ExternalMod> loadMissingMods(Path configDirectory, Predicate<String> isModLoaded) {
		Path path = configDirectory.resolve("mod-audit.json");

		try {
			if (Files.notExists(path)) {
				Files.createDirectories(path.getParent());
				Files.writeString(path, GSON.toJson(new Config(List.of(new ExternalMod("", "", "", "", "")))));
				return List.of();
			}

			Config config = GSON.fromJson(Files.readString(path), Config.class);
			if (config == null || config.mods() == null) {
				return List.of();
			}

			HashSet<String> ids = new HashSet<>();
			return config.mods().stream()
				.filter(ModAuditConfig::isValid)
				.filter(mod -> ids.add(mod.id()))
				.filter(mod -> !isModLoaded.test(mod.id()))
				.toList();
		} catch (IOException | JsonParseException exception) {
			ModAudit.LOGGER.error("Failed to load {}", path, exception);
			return List.of();
		}
	}

	private static boolean isValid(ExternalMod mod) {
		if (mod == null || isBlank(mod.id()) || isBlank(mod.name()) || isBlank(mod.url())) {
			return false;
		}

		try {
			String scheme = URI.create(mod.url()).getScheme();
			return ("https".equalsIgnoreCase(scheme) || "http".equalsIgnoreCase(scheme))
				&& (isBlank(mod.sha256()) || mod.sha256().matches("(?i)[0-9a-f]{64}"));
		} catch (IllegalArgumentException exception) {
			return false;
		}
	}

	private static boolean isBlank(String value) {
		return value == null || value.isBlank();
	}

	private record Config(List<ExternalMod> mods) {}
}
