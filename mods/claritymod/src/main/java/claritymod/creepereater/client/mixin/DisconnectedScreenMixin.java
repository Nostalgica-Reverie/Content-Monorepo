package claritymod.creepereater.client.mixin;

import java.util.Locale;
import net.minecraft.client.gui.screens.DisconnectedScreen;
import net.minecraft.network.DisconnectionDetails;
import net.minecraft.network.chat.Component;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

@Mixin(DisconnectedScreen.class)
public abstract class DisconnectedScreenMixin {
	@Redirect(
		method = {"init", "getNarrationMessage"},
		at = @At(
			value = "INVOKE",
			target = "Lnet/minecraft/network/DisconnectionDetails;reason()Lnet/minecraft/network/chat/Component;"
		)
	)
	private Component claritymod$replaceGetsockoptError(DisconnectionDetails details) {
		Component originalReason = details.reason();
		String reasonText = originalReason.getString().toLowerCase(Locale.ROOT);
		if (!reasonText.contains("getsockopt")) {
			return originalReason;
		}

		if (reasonText.contains("timed out")) {
			return Component.translatableWithFallback(
				"claritymod.disconnect.getsockopt_timeout",
				"Connection timed out. Check that the server is online and its address and port are correct. "
					+ "If you host the server, allow Java through the firewall and forward its Minecraft port. "
					+ "Otherwise, contact the server administrator."
			);
		}

		if (reasonText.contains("refused")) {
			return Component.translatableWithFallback(
				"claritymod.disconnect.getsockopt_refused",
				"Connection refused. The server may be offline, still starting, or not listening on that port. "
					+ "Check the address and port. If you host the server, start it and verify its firewall and port forwarding."
			);
		}

		return Component.translatableWithFallback(
			"claritymod.disconnect.getsockopt_generic",
			"Could not connect to the server. Check that it is online and that its address and port are correct. "
				+ "If you host the server, check its firewall and port-forwarding settings."
		);
	}
}
