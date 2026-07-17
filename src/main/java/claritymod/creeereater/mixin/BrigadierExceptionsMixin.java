package claritymod.creeereater.mixin;

import com.mojang.brigadier.exceptions.SimpleCommandExceptionType;
import net.minecraft.commands.BrigadierExceptions;
import net.minecraft.network.chat.Component;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(BrigadierExceptions.class)
public abstract class BrigadierExceptionsMixin {
	@Inject(method = "dispatcherUnknownCommand", at = @At("RETURN"), cancellable = true)
	private void claritymod$replaceUnknownCommandMessage(
		CallbackInfoReturnable<SimpleCommandExceptionType> callback
	) {
		callback.setReturnValue(new SimpleCommandExceptionType(Component.translatableWithFallback(
			"claritymod.command.unknown",
			"Unknown command, or you do not have permission to use it. Enable cheats or ask a server administrator for access."
		)));
	}
}
