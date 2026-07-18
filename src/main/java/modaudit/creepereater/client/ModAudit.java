package modaudit.creepereater.client;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.swing.SwingUtilities;
import java.awt.GraphicsEnvironment;
import java.nio.file.Path;
import java.util.function.Predicate;

public final class ModAudit {
	public static final String MOD_ID = "modaudit";
	static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

	private ModAudit() {}

	public static void initialize(Path configDirectory, Path modsDirectory, Predicate<String> isModLoaded) {
		var missingMods = ModAuditConfig.loadMissingMods(configDirectory, isModLoaded);
		LOGGER.info("Found {} missing configured mods", missingMods.size());
		if (!missingMods.isEmpty()) {
			System.setProperty("java.awt.headless", "false");
			if (GraphicsEnvironment.isHeadless()) {
				LOGGER.error("Cannot show the Mod Audit window in a headless environment");
				return;
			}
			SwingUtilities.invokeLater(() -> ModAuditWindow.open(missingMods, modsDirectory));
		}
	}
}
