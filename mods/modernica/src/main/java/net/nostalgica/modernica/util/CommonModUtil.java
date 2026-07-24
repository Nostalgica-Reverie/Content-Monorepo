package net.nostalgica.modernica.util;

import net.nostalgica.modernica.core.ModernicaMixinPlugin;

public class CommonModUtil {
    @FunctionalInterface
    public interface SafeRunnable {
        void run() throws Throwable;
    }

    /**
     * Avoid using this, it's bad practice but cleanest way of suppressing errors for nonessential mod-dependent
     * functionality.
     */
    public static void runWithoutCrash(SafeRunnable r, String errorMsg) {
        try {
            r.run();
        } catch(Throwable e) {
            ModernicaMixinPlugin.instance.logger.error(errorMsg, e);
        }
    }
}
