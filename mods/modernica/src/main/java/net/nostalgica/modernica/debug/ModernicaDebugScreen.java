package net.nostalgica.modernica.debug;

import net.minecraft.client.gui.components.debug.DebugScreenDisplayer;
import net.minecraft.client.gui.components.debug.DebugScreenEntry;
import net.minecraft.resources.Identifier;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.chunk.LevelChunk;
import net.nostalgica.modernica.Modernica;
import net.nostalgica.modernica.ModernicaClient;
import net.nostalgica.modernica.common.mixin.feature.branding.DebugScreenEntriesInvoker;
import org.jetbrains.annotations.Nullable;

public class ModernicaDebugScreen {
    public static final Identifier MODERNICA_GROUP = Identifier.fromNamespaceAndPath(Modernica.MODID, "modernica_info");
    public static Identifier MODERNICA_ENTRY = DebugScreenEntriesInvoker.mfix$register(
            MODERNICA_GROUP,
            new ModernicaDebugEntry()
    );

    static class ModernicaDebugEntry implements DebugScreenEntry {
        @Override
        public void display(DebugScreenDisplayer displayer, @Nullable Level level, @Nullable LevelChunk clientChunk, @Nullable LevelChunk serverChunk) {
            displayer.addToGroup(MODERNICA_GROUP, ModernicaClient.INSTANCE.brandingString);
        }

        @Override
        public boolean isAllowed(boolean reducedDebugInfo) {
            return true;
        }
    }
}
