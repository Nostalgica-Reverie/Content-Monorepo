package net.nostalgica.modernica.common.mixin.feature.branding;

import net.minecraft.client.gui.components.debug.DebugScreenEntries;
import net.minecraft.client.gui.components.debug.DebugScreenEntryStatus;
import net.minecraft.resources.Identifier;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import net.nostalgica.modernica.debug.ModernicaDebugScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

import java.util.Map;
import java.util.stream.Collectors;
import java.util.stream.Stream;

@ClientOnlyMixin
@Mixin(DebugScreenEntries.class)
public class DebugScreenEntriesMixin {
    @ModifyVariable(method = "<clinit>", at = @At(value = "STORE"), ordinal = 0)
    private static Map<Identifier, DebugScreenEntryStatus> insertIntoDefaultProfile(Map<Identifier, DebugScreenEntryStatus> map) {
        // Map is immutable, so we need to build a new one with the added entry
        var stream = map.entrySet().stream();
        var new_map = Map.of(ModernicaDebugScreen.MODERNICA_ENTRY, DebugScreenEntryStatus.IN_OVERLAY).entrySet().stream();

        return Stream.concat(stream, new_map).collect(Collectors.toMap(Map.Entry::getKey, Map.Entry::getValue));
    }
}
