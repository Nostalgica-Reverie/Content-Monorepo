package net.nostalgica.modernica.dedup;

import net.minecraft.nbt.StringTag;

public class NbtCaches {
    public static final DeduplicationCache<String> KEYS = new DeduplicationCache<>();
    public static final DeduplicationCache<StringTag> STRING_TAGS = new DeduplicationCache<>();
}
