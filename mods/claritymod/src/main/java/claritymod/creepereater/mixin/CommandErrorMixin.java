package claritymod.creepereater.mixin;

import com.mojang.brigadier.exceptions.CommandSyntaxException;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
//? if >=26 {
import net.minecraft.server.permissions.Permissions;
//?}
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.ComponentUtils;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(Commands.class)
public abstract class CommandErrorMixin {
	@Redirect(
		method = "finishParsing",
		at = @At(
			value = "INVOKE",
			target = "Lnet/minecraft/commands/CommandSourceStack;sendFailure(Lnet/minecraft/network/chat/Component;)V"
		)
	)
	private static void claritymod$explainUnknownCommand(CommandSourceStack source, Component message) {
		Component unknownCommand = ComponentUtils.fromMessage(
			CommandSyntaxException.BUILT_IN_EXCEPTIONS.dispatcherUnknownCommand().create().getRawMessage()
		);
		//? if >=26 {
		boolean canUseAdminCommands = source.permissions().hasPermission(Permissions.COMMANDS_GAMEMASTER);
		//?} else {
		/*boolean canUseAdminCommands = source.hasPermission(2);*/
		//?}

		if (!canUseAdminCommands && message.equals(unknownCommand)) {
			message = Component.translatableWithFallback(
				"claritymod.command.unknown",
				"Unknown command, or you do not have permission to use it. Enable cheats or ask a server administrator for access."
			);
		}
		source.sendFailure(message);
	}
}
