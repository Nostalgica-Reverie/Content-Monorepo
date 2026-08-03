package net.nostalgica.modernica.core.config;

import net.fabricmc.loader.api.FabricLoader;
import org.apache.logging.log4j.Logger;

import java.io.Reader;
import java.io.Writer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.Properties;

final class EarlyMixinOptions {
    private static final String PREFIX = "mixin.";
    private final Properties values;

    private EarlyMixinOptions(Properties values) {
        this.values = values;
    }

    static EarlyMixinOptions load(Logger logger, Map<String, Boolean> defaults) {
        Properties values = new Properties();
        Path file = FabricLoader.getInstance().getConfigDir().resolve("modernica-mixins.properties");
        try {
            if (Files.isRegularFile(file)) {
                try (Reader reader = Files.newBufferedReader(file)) {
                    values.load(reader);
                }
            } else {
                Files.createDirectories(file.getParent());
                for (Map.Entry<String, Boolean> entry : defaults.entrySet()) {
                    values.setProperty(PREFIX + entry.getKey(), entry.getValue().toString());
                }
                try (Writer writer = Files.newBufferedWriter(file)) {
                    values.store(writer, "Modernica mixin configuration. Restart Minecraft after editing.");
                }
                logger.info("Created Modernica's mixin configuration at {}", file);
            }
        } catch (Exception e) {
            logger.warn("Failed to load Modernica's mixin configuration; using compiled-in defaults.", e);
        }
        return new EarlyMixinOptions(values);
    }

    boolean resolveBoolean(String key, boolean defaultValue) {
        String value = values.getProperty(PREFIX + key);
        return value == null ? defaultValue : Boolean.parseBoolean(value.trim());
    }

    int resolveInt(String key, int defaultValue) {
        try {
            return Integer.parseInt(values.getProperty(PREFIX + key, Integer.toString(defaultValue)).trim());
        } catch (NumberFormatException ignored) {
            return defaultValue;
        }
    }
}
