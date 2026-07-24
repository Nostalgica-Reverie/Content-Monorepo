package net.nostalgica.modernica.core.config;

import net.fabricmc.loader.api.FabricLoader;
import net.peanuuutz.tomlkt.Toml;
import net.peanuuutz.tomlkt.TomlElement;
import net.peanuuutz.tomlkt.TomlElementKt;
import net.peanuuutz.tomlkt.TomlLiteral;
import net.peanuuutz.tomlkt.TomlTable;
import org.apache.logging.log4j.Logger;

import java.nio.file.Files;
import java.nio.file.Path;

/** Reads the config TOML directly, before it's safe to construct a real {@link ModernicaConfig} (its
 * Identifier would race other mods' mixins during Mixin's prepare phase). Never throws. */
final class EarlyMixinOptions {
    private final TomlTable root;

    private EarlyMixinOptions(TomlTable root) {
        this.root = root;
    }

    static EarlyMixinOptions load(Logger logger) {
        try {
            Path file = FabricLoader.getInstance().getConfigDir()
                    .resolve("modernica")
                    .resolve("modernica")
                    .resolve("config.toml");
            if (!Files.isRegularFile(file)) {
                return new EarlyMixinOptions(null);
            }
            String content = Files.readString(file);
            return new EarlyMixinOptions(Toml.Default.parseToTomlTable(content));
        } catch (Exception e) {
            logger.warn("Failed to pre-read Modernica's config for early mixin gating; using compiled-in defaults until the real config loads.", e);
            return new EarlyMixinOptions(null);
        }
    }

    boolean resolveBoolean(String section, String field, boolean defaultValue) {
        TomlTable scope = scope(section);
        if (scope == null) {
            return defaultValue;
        }
        try {
            Boolean value = TomlElementKt.getBooleanOrNull(scope, field);
            return value != null ? value : defaultValue;
        } catch (RuntimeException e) {
            return defaultValue;
        }
    }

    ModernicaConfig.StabilityLevel resolveStabilityLevel(ModernicaConfig.StabilityLevel defaultValue) {
        if (root == null) {
            return defaultValue;
        }
        try {
            TomlElement element = root.get("stabilityLevel");
            if (!(element instanceof TomlLiteral literal)) {
                return defaultValue;
            }
            return ModernicaConfig.StabilityLevel.valueOf(literal.getContent());
        } catch (RuntimeException e) {
            return defaultValue;
        }
    }

    private TomlTable scope(String section) {
        if (root == null) {
            return null;
        }
        if (section.isEmpty()) {
            return root;
        }
        TomlTable current = root;
        for (String part : section.split("\\.")) {
            TomlElement element = current.get(part);
            if (!(element instanceof TomlTable table)) {
                return null;
            }
            current = table;
        }
        return current;
    }
}
