package net.nostalgica.modernica.platform;

import java.lang.reflect.Constructor;

class PlatformHookLoader {
    static ModernicaPlatformHooks findInstance() {
        String[] locations = new String[] { "neoforge", "fabric" };
        for(String location : locations) {
            try {
                Class<?> clz = Class.forName("net.nostalgica.modernica.platform." + location + ".ModernicaPlatformHooksImpl");
                Constructor<?> constructor = clz.getConstructor();
                constructor.setAccessible(true);
                return (ModernicaPlatformHooks)constructor.newInstance();
            } catch(ClassNotFoundException ignored) {
            } catch(ReflectiveOperationException | ClassCastException e) {
                e.printStackTrace();
            }
        }
        System.err.println("Modernica has failed to load platform hooks. It cannot function, the game will now close");
        Runtime.getRuntime().exit(1);
        throw new AssertionError("Somehow couldn't exit");
    }
}
