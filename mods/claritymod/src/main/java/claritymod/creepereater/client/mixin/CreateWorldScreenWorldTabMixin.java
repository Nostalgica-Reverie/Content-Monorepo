package claritymod.creepereater.client.mixin;

import claritymod.creepereater.compat.ModLoaderCompat;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.components.CycleButton;
import net.minecraft.client.gui.screens.ConfirmScreen;
import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import net.minecraft.client.gui.screens.worldselection.WorldCreationUiState;
import net.minecraft.network.chat.Component;
//? if <26 {
/*import org.spongepowered.asm.mixin.Final;*/
/*import org.spongepowered.asm.mixin.Shadow;*/
//?}
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(targets = "net.minecraft.client.gui.screens.worldselection.CreateWorldScreen$WorldTab")
public abstract class CreateWorldScreenWorldTabMixin {
	//? if <26
	/*@Shadow @Final private CreateWorldScreen this$0;*/

	@Inject(
		//? if >=26 {
		method = "lambda$new$0(Lnet/minecraft/client/gui/screens/worldselection/CreateWorldScreen;Lnet/minecraft/client/gui/components/CycleButton;Lnet/minecraft/client/gui/screens/worldselection/WorldCreationUiState$WorldTypeEntry;)V",
		//?} else {
		/*method = "lambda$new$0(Lnet/minecraft/client/gui/components/CycleButton;Lnet/minecraft/client/gui/screens/worldselection/WorldCreationUiState$WorldTypeEntry;)V",*/
		//?}
		at = @At("HEAD"),
		cancellable = true
	)
	private /*? if >=26 {*/ static /*?}*/ void claritymod$confirmWorldTypeChange(
		//? if >=26
		CreateWorldScreen outerScreen,
		CycleButton<WorldCreationUiState.WorldTypeEntry> typeButton,
		WorldCreationUiState.WorldTypeEntry newWorldType,
		CallbackInfo callback
	) {
		if (!ModLoaderCompat.isModLoaded("betterend") && !ModLoaderCompat.isModLoaded("betternether")) {
			return;
		}

		CreateWorldScreen createWorldScreen =
			//? if >=26 {
			outerScreen;
			//?} else {
			/*this.this$0;*/
			//?}
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
