package net.nostalgica.modernica.fabric.mixin.feature.force_close_loading_screen;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Gui;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.client.gui.screens.multiplayer.ServerReconfigScreen;
import net.nostalgica.modernica.feature.forcecloseloadingscreen.ReconfigBridgeScreen;
import net.nostalgica.modernica.feature.forcecloseloadingscreen.TitleBridgeScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyVariable;

/** Ported from kennytv's forcecloseloadingscreen (MIT) */
//STONECUTTER_FCLS_GUI_MIXIN_TARGET
@Mixin(Minecraft.class)
public abstract class GuiMixin {

    @ModifyVariable(at = @At("HEAD"), method = "setScreen", argsOnly = true, name = "screen")
    public Screen setScreen(final Screen screen) {
        if (screen instanceof ServerReconfigScreen reconfigScreen) {
            return new ReconfigBridgeScreen(reconfigScreen.connection);
        } else if (screen instanceof TitleScreen) {
            return new TitleBridgeScreen();
        }
        return screen;
    }
}
