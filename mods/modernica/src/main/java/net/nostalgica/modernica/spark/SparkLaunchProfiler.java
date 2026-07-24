package net.nostalgica.modernica.spark;

import com.google.common.util.concurrent.ThreadFactoryBuilder;
import it.unimi.dsi.fastutil.objects.Object2ReferenceOpenHashMap;
import me.lucko.spark.common.SparkPlatform;
import me.lucko.spark.common.SparkPlugin;
import me.lucko.spark.common.command.sender.CommandSender;
import me.lucko.spark.common.platform.PlatformInfo;
import me.lucko.spark.common.sampler.Sampler;
import me.lucko.spark.common.sampler.SamplerSettings;
import me.lucko.spark.common.sampler.ThreadDumper;
import me.lucko.spark.common.sampler.ThreadGrouper;
import me.lucko.spark.common.sampler.async.AsyncSampler;
import me.lucko.spark.common.sampler.async.SampleCollector;
import me.lucko.spark.common.sampler.java.JavaSampler;
import me.lucko.spark.common.sampler.java.MergeStrategy;
import me.lucko.spark.lib.adventure.text.Component;
import me.lucko.spark.proto.SparkSamplerProtos;
import net.minecraft.SharedConstants;
import net.nostalgica.modernica.core.ModernicaMixinPlugin;
import net.nostalgica.modernica.platform.ModernicaPlatformHooks;

import java.nio.file.Path;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.logging.Level;
import java.util.stream.Stream;

/* Inspired by CensoredASM */
public class SparkLaunchProfiler {
    private static PlatformInfo platformInfo = new ModernicaPlatformInfo();
    private static CommandSender commandSender = new ModernicaCommandSender();
    private static Map<String, Sampler> ongoingSamplers = new Object2ReferenceOpenHashMap<>();
    private static ExecutorService executor = Executors.newSingleThreadScheduledExecutor(new ThreadFactoryBuilder().setDaemon(true).setNameFormat("spark-modernica-async-worker").build());
    private static final SparkPlatform platform = new SparkPlatform(new ModernicaSparkPlugin());

    private static final String ALLOW_SPARK_PROFILING_PROP = "modernica.allowSparkProfiling";
    private static final boolean USE_JAVA_SAMPLER_FOR_LAUNCH = !Boolean.getBoolean("modernica.profileWithAsyncSampler");
    private static final boolean ALLOW_SPARK_PROFILING = Boolean.getBoolean(ALLOW_SPARK_PROFILING_PROP);
    private static final int SAMPLING_INTERVAL = Integer.getInteger("modernica.profileSamplingIntervalMicroseconds", 4000);
    private static final String THREAD_GROUPER = System.getProperty("modernica.profileSamplingThreadGrouper", "by-pool");

    private static boolean checkSparkProfilingAllowed() {
        if (!ALLOW_SPARK_PROFILING) {
            ModernicaMixinPlugin.instance.logger.fatal("To reduce excessive load on the Spark servers, you must set " +
                    "-D{}=true in your JVM arguments for profiling to proceed. Please do " +
                    "this and relaunch the game.", ALLOW_SPARK_PROFILING_PROP);
            return false;
        }
        return true;
    }

    public static void start(String key) {
        if (!checkSparkProfilingAllowed()) {
            return;
        }
        if (!ongoingSamplers.containsKey(key)) {
            Sampler sampler;
            SamplerSettings settings = new SamplerSettings(SAMPLING_INTERVAL, ThreadDumper.ALL, ThreadGrouper.parseConfigSetting(THREAD_GROUPER).get(), -1, false, true);
            try {
                if(USE_JAVA_SAMPLER_FOR_LAUNCH) {
                    throw new UnsupportedOperationException();
                }
                sampler = new AsyncSampler(platform, settings, new SampleCollector.Execution(SAMPLING_INTERVAL));
            } catch (UnsupportedOperationException e) {
                sampler = new JavaSampler(platform, settings);
            }
            ongoingSamplers.put(key, sampler);
            ModernicaMixinPlugin.instance.logger.warn("Profiler has started for stage [{}]...", key);
            sampler.start();
        }
    }

    public static void stop(String key) {
        Sampler sampler = ongoingSamplers.remove(key);
        if (sampler != null) {
            sampler.stop(true);
            output(key, sampler);
        }
    }

    private static void output(String key, Sampler sampler) {
        executor.execute(() -> {
            ModernicaMixinPlugin.instance.logger.warn("Stage [{}] profiler has stopped! Uploading results...", key);
            SparkSamplerProtos.SamplerData output = sampler.toProto(platform, new Sampler.ExportProps()
                    .creator(new CommandSender.Data(commandSender.getName(), commandSender.getUniqueId()))
                    .comment("Stage: " + key)
                    .mergeStrategy(MergeStrategy.SAME_METHOD)
                    .classSourceLookup(platform::createClassSourceLookup));
            try {
                String urlKey = platform.getBytebinClient().postContent(output, "application/x-spark-sampler").key();
                String url = "https://spark.lucko.me/" + urlKey;
                ModernicaMixinPlugin.instance.logger.warn("Profiler results for Stage [{}]: {}", key, url);
            } catch (Exception e) {
                ModernicaMixinPlugin.instance.logger.fatal("An error occurred whilst uploading the results.", e);
            }
        });
    }

    static class ModernicaPlatformInfo implements PlatformInfo {

        @Override
        public Type getType() {
            return ModernicaPlatformHooks.INSTANCE.isClient() ? Type.CLIENT : Type.SERVER;
        }

        @Override
        public String getName() {
            return ModernicaPlatformHooks.INSTANCE.getPlatformName();
        }

        @Override
        public String getBrand() {
            return this.getName();
        }

        @Override
        public String getVersion() {
            return ModernicaPlatformHooks.INSTANCE.getVersionString();
        }

        @Override
        public String getMinecraftVersion() {
            return SharedConstants.getCurrentVersion().name();
        }

    }

    public static class ModernicaCommandSender implements CommandSender {

        private final UUID uuid = UUID.randomUUID();
        private final String name;

        public ModernicaCommandSender() {
            this.name = "Modernica";
        }

        @Override
        public String getName() {
            return name;
        }

        @Override
        public UUID getUniqueId() {
            return uuid;
        }

        @Override
        public boolean hasPermission(String s) {
            return true;
        }

        @Override
        public void sendMessage(Component component) {

        }
    }

    static class ModernicaSparkPlugin implements SparkPlugin {

        @Override
        public String getVersion() {
            return "1.0";
        }

        @Override
        public Path getPluginDirectory() {
            return ModernicaPlatformHooks.INSTANCE.getGameDirectory().resolve("spark-modernica");
        }

        @Override
        public String getCommandName() {
            return "spark-modernica";
        }

        @Override
        public Stream<? extends CommandSender> getCommandSenders() {
            return Stream.of();
        }

        @Override
        public void executeAsync(Runnable runnable) {
            executor.execute(runnable);
        }

        @Override
        public void log(Level level, String s) {
            ModernicaMixinPlugin.instance.logger.warn(s);
        }

        @Override
        public void log(Level level, String s, Throwable t) {
            ModernicaMixinPlugin.instance.logger.warn(s, t);
        }

        @Override
        public PlatformInfo getPlatformInfo() {
            return platformInfo;
        }
    }
}
