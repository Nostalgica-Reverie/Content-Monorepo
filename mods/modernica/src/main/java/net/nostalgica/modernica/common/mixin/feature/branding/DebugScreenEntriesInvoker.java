package net.nostalgica.modernica.common.mixin.feature.branding;

import net.minecraft.client.gui.components.debug.DebugScreenEntries;
import net.minecraft.client.gui.components.debug.DebugScreenEntry;
import net.minecraft.resources.Identifier;
import net.nostalgica.modernica.annotation.ClientOnlyMixin;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;


@ClientOnlyMixin
@Mixin(DebugScreenEntries.class)
public interface DebugScreenEntriesInvoker {
    @Invoker("register")
    static Identifier mfix$register(Identifier name, DebugScreenEntry entry) {
        throw new RuntimeException("Invoker mixin didn't work?");
    };
}
