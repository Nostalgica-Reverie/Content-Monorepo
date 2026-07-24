package net.nostalgica.modernica.dedup;


import net.nostalgica.modernica.Modernica;

public class IdentifierCaches {
    public static final DeduplicationCache<String> NAMESPACES = new DeduplicationCache<>();
    public static final DeduplicationCache<String> PATH = new DeduplicationCache<>();
    public static final DeduplicationCache<String> PROPERTY = new DeduplicationCache<>();

    public static void printDebug() {
        Modernica.LOGGER.info("[[[ Identifier de-duplication statistics ]]]");
        Modernica.LOGGER.info("Namespace cache: {}", NAMESPACES);
        Modernica.LOGGER.info("Path cache: {}", PATH);
    }
}
