package net.nostalgica.modernica.api.entrypoint;

/**
 * Implement this interface in a mod class and add it to "modernica:integration_v1" in your mod metadata file
 * to integrate with Modernica's features.
 */
public interface ModernicaClientIntegration {
    /**
     * Called when the dynamic resources status has changed during a model reload so mods know whether to run their
     * normal codepath or the dynamic version.
     *
     * @param enabled whether dynamic resources is enabled
     */
    default void onDynamicResourcesStatusChange(boolean enabled) {
    }
}
