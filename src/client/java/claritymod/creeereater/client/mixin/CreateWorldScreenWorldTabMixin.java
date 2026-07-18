package claritymod.creeereater.client.mixin;

import claritymod.creeereater.compat.ModLoaderCompat;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.components.CycleButton;
import net.minecraft.client.gui.screens.ConfirmScreen;
import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import net.minecraft.client.gui.screens.worldselection.WorldCreationUiState;
import net.minecraft.network.chat.Component;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(targets = "net.minecraft.client.gui.screens.worldselection.CreateWorldScreen$WorldTab")
public abstract class CreateWorldScreenWorldTabMixin {
	@Inject(
		method = "lambda$new$0(Lnet/minecraft/client/gui/screens/worldselection/CreateWorldScreen;Lnet/minecraft/client/gui/components/CycleButton;Lnet/minecraft/client/gui/screens/worldselection/WorldCreationUiState$WorldTypeEntry;)V",
		at = @At("HEAD"),
		cancellable = true
	)
	private static void claritymod$confirmWorldTypeChange(
		CreateWorldScreen createWorldScreen,
		CycleButton<WorldCreationUiState.WorldTypeEntry> typeButton,
		WorldCreationUiState.WorldTypeEntry newWorldType,
		CallbackInfo callback
	) {
		if (!ModLoaderCompat.isModLoaded("betterend") && !ModLoaderCompat.isModLoaded("betternether")) {
			return;
		}

		Minecraft minecraft = Minecraft.getInstance();
		minecraft.setScreen(new ConfirmScreen(
			confirmed -> {
				if (confirmed) {
					createWorldScreen.getUiState().setWorldType(newWorldType);
				}

				minecraft.setScreen(createWorldScreen);
				typeButton.setValue(createWorldScreen.getUiState().getWorldType());
			},
			Component.translatableWithFallback(
				"claritymod.betterx.confirm.title",
				"Changing World Type May Break World Generation!"
			),
			Component.translatableWithFallback(
				"claritymod.betterx.confirm.message",
				"BetterEnd or BetterNether is installed. Changing the world type can permanently damage world generation. Are you sure you want to continue?"
			),
			Component.translatableWithFallback("claritymod.betterx.confirm.accept", "Change World Type"),
			Component.translatableWithFallback("claritymod.betterx.confirm.cancel", "Cancel")
		));
		callback.cancel();
	}
}
