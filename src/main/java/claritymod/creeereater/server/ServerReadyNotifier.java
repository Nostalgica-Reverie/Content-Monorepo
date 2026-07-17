package claritymod.creeereater.server;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;

public final class ServerReadyNotifier {
	private static final long ANNOUNCEMENT_DELAY_SECONDS = 5L;

	private ServerReadyNotifier() {
	}

	public static void register() {
		ServerLifecycleEvents.SERVER_STARTED.register(ServerReadyNotifier::scheduleAnnouncement);
	}

	private static void scheduleAnnouncement(MinecraftServer server) {
		if (!server.isDedicatedServer()) {
			return;
		}

		CompletableFuture.delayedExecutor(ANNOUNCEMENT_DELAY_SECONDS, TimeUnit.SECONDS).execute(() -> {
			if (!server.isRunning() || server.isStopped()) {
				return;
			}

			server.execute(() -> {
				if (!server.isRunning() || server.isStopped()) {
					return;
				}

				server.getPlayerList().broadcastSystemMessage(
					Component.translatableWithFallback(
						"claritymod.server.ready",
						"Server is online and ready to join!"
					).withStyle(ChatFormatting.GREEN),
					false
				);
			});
		});
	}
}
