package net.nostalgica.modernica.core;

import me.fzzyhmstrs.fzzy_config.api.ConfigApiJava;
import net.nostalgica.modernica.core.config.MixinGate;
import net.nostalgica.modernica.core.config.ModernicaConfig;
import net.nostalgica.modernica.platform.ModernicaPlatformHooks;
import net.nostalgica.modernica.world.ThreadDumper;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.objectweb.asm.tree.ClassNode;
import org.spongepowered.asm.mixin.extensibility.IMixinConfigPlugin;
import org.spongepowered.asm.mixin.extensibility.IMixinInfo;

import java.util.List;
import java.util.Properties;
import java.util.Set;
import java.util.regex.Pattern;

public class ModernicaMixinPlugin implements IMixinConfigPlugin {
    private static final Pattern PLATFORM_PREFIX = Pattern.compile("(fabric|common)\\.");
    private static final String MIXIN_PACKAGE_ROOT = "net.nostalgica.modernica.mixin.";

    public final Logger logger = LogManager.getLogger("Modernica");
    public ModernicaConfig config = null;
    public static ModernicaMixinPlugin instance;

    private static String sanitize(String mixinClassName) {
        return PLATFORM_PREFIX.matcher(mixinClassName).replaceFirst("");
    }

    public ModernicaMixinPlugin() {
        boolean firstConfig = instance == null;
        if (firstConfig) {
            instance = this;
            MixinGate.registerAll(this.logger);

            try {
                Class.forName("sun.misc.Unsafe").getDeclaredMethod("defineAnonymousClass", Class.class, byte[].class, Object[].class);
            } catch (ReflectiveOperationException | NullPointerException e) {
                this.logger.info("Applying Nashorn fix");
                Properties properties = System.getProperties();
                properties.setProperty("nashorn.args", properties.getProperty("nashorn.args", "") + " --anonymous-classes=false");
            }

            /* We abuse the constructor of a mixin plugin as a safe location to start modifying the classloader */
            ModernicaPlatformHooks.INSTANCE.injectPlatformSpecificHacks();

            if (isOptionEnabled("feature.spam_thread_dump.ThreadDumper")) {
                // run once to trigger classloading
                ThreadDumper.obtainThreadDump();
                Thread t = new Thread() {
                    public void run() {
                        while (true) {
                            try {
                                Thread.sleep(60000);
                                logger.error("------ DEBUG THREAD DUMP (occurs every 60 seconds) ------");
                                logger.error(ThreadDumper.obtainThreadDump());
                            } catch (InterruptedException | RuntimeException e) {
                            }
                        }
                    }
                };
                t.setDaemon(true);
                t.start();
            }

            if (ModernicaPlatformHooks.INSTANCE.isClient() && isOptionEnabled("perf.thread_priorities.AdjustThreadCount")) {
                computeBetterThreadCount();
            }
        }
    }

    /** Loads the real, GUI-facing config. Must only be called once it's safe to construct an Identifier -
     * i.e. from normal mod init, never from this plugin's constructor. Idempotent. */
    public void loadRealConfig() {
        if (this.config != null) {
            return;
        }
        try {
            this.config = ConfigApiJava.readOrCreateAndValidate(ModernicaConfig::new);
        } catch (Exception e) {
            throw new RuntimeException("Could not load configuration file for Modernica", e);
        }
        MixinGate.bindRealConfig(this.config, this.logger);

        this.logger.info("Loaded configuration for Modernica {}", ModernicaPlatformHooks.INSTANCE.getVersionString());
        if (this.config.stabilityLevel != ModernicaConfig.StabilityLevel.GA) {
            this.logger.warn("Modernica stability level is set to {}. Features at this level may be unstable or cause crashes.",
                    this.config.stabilityLevel);
        }
    }

    private void computeBetterThreadCount() {
        // Allow user-provided thread count to take precedence
        if (System.getProperty("max.bg.threads") != null) {
            return;
        }
        // Server thread + client thread + GC thread
        int reservedCores = 3;
        int availableBackgroundCores = Math.max(1, Runtime.getRuntime().availableProcessors() - reservedCores);
        logger.info("Configuring Minecraft's max.bg.threads option with {} threads", availableBackgroundCores);
        System.setProperty("max.bg.threads", String.valueOf(availableBackgroundCores));
    }

    @Override
    public void onLoad(String mixinPackage) {
    }

    @Override
    public String getRefMapperConfig() {
        return "modernica.refmap.json";
    }

    @Override
    public boolean shouldApplyMixin(String targetClassName, String mixinClassName) {
        String sanitized = sanitize(mixinClassName);
        if (!sanitized.startsWith(MIXIN_PACKAGE_ROOT)) {
            this.logger.error("Expected mixin '{}' to start with package root '{}', treating as foreign and " +
                    "disabling!", sanitized, MIXIN_PACKAGE_ROOT);
            return false;
        }

        String mixin = sanitized.substring(MIXIN_PACKAGE_ROOT.length());
        if (!isOptionEnabled(mixin)) {
            this.logger.debug("Skipping mixin {}: disabled by configuration", mixin);
            return false;
        }
        // strip the trailing ".ClassSimpleName" to get the group key for the feature-level check
        int lastDot = mixin.lastIndexOf('.');
        String group = lastDot > 0 ? mixin.substring(0, lastDot) : mixin;
        if (!MixinGate.meetsFeatureLevel(group)) {
            this.logger.debug("Skipping mixin {}: requires BETA stability level", mixin);
            return false;
        }
        if (!MixinGate.meetsModRequirement(mixin)) {
            this.logger.debug("Skipping mixin {}: mod compatibility requirement not met", mixin);
            return false;
        }
        this.logger.debug("Applying mixin {}", mixin);
        return true;
    }

    public boolean isOptionEnabled(String dottedPath) {
        return MixinGate.isEnabled(dottedPath, this.logger, ModernicaPlatformHooks.INSTANCE.isDevEnv());
    }

    @Override
    public void acceptTargets(Set<String> myTargets, Set<String> otherTargets) {
    }

    @Override
    public List<String> getMixins() {
        return null;
    }

    @Override
    public void preApply(String targetClassName, ClassNode targetClass, String mixinClassName, IMixinInfo mixinInfo) {
    }

    @Override
    public void postApply(String targetClassName, ClassNode targetClass, String mixinClassName, IMixinInfo mixinInfo) {
        ModernicaPlatformHooks.INSTANCE.applyASMTransformers(mixinClassName, targetClass);
    }
}
