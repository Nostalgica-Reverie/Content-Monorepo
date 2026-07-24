package net.nostalgica.modernica.resources;

import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.PackResources;
import net.minecraft.server.packs.resources.IoSupplier;

import java.io.InputStream;
import java.util.Collection;
import java.util.function.Function;

public class NewResourcePackAdapter {
    public static void sendToOutput(Function<Identifier, IoSupplier<InputStream>> streamCreator, PackResources.ResourceOutput output, Collection<Identifier> locations) {
        for(Identifier rl : locations) {
            output.accept(rl, streamCreator.apply(rl));
        }
    }
}
