package net.nostalgica.modernica.duck.reuse_datapacks;

import net.minecraft.server.ReloadableServerResources;

import java.util.Collection;

public interface ICachingResourceClient {
    void setCachedResources(ReloadableServerResources r);
    void setCachedDataPackConfig(Collection<String> c);
}
