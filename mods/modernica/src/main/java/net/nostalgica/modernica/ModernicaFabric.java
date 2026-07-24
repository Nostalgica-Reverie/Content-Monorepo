package net.nostalgica.modernica;

import net.fabricmc.api.ModInitializer;
import net.minecraft.server.MinecraftServer;
import net.nostalgica.modernica.core.ModernicaMixinPlugin;
import net.nostalgica.modernica.platform.ModernicaPlatformHooks;

import java.lang.ref.WeakReference;

public class ModernicaFabric implements ModInitializer {
    public static Modernica commonMod;
    public static WeakReference<MinecraftServer> theServer = new WeakReference<>(null);

    static {
        // From Krypton's KryptonSharedInitializer: by default, Netty allocates 16MiB arenas for the
        // PooledByteBufAllocator, far more than Minecraft needs (max packet size is 2MiB). Lower the
        // chunk size (pageSize << maxOrder) by reducing maxOrder from its default of 11 to 9, unless the
        // user already set their own value. Must run before Netty's allocator classes are first
        // touched, so it lives in a static initializer rather than onInitialize() - same reasoning
        // Krypton itself used.
        if (ModernicaMixinPlugin.instance.isOptionEnabled("perf.network_optimizations")
                && System.getProperty("io.netty.allocator.maxOrder") == null) {
            System.setProperty("io.netty.allocator.maxOrder", "9");
        }
    }

    @Override
    public void onInitialize() {
        ModernicaMixinPlugin.instance.loadRealConfig();

        commonMod = new Modernica();

        if (ModernicaMixinPlugin.instance.isOptionEnabled("perf.network_optimizations")
                && ModernicaPlatformHooks.INSTANCE.isDedicatedServer()) {
            Modernica.LOGGER.info("Network optimizations (from Krypton) are accelerating this server's networking stack");
        }

        // TODO: implement entity ID desync
    }


}
