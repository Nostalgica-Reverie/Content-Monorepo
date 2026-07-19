package claritymod.creepereater.server;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
//? if fabric {
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
//?} else {
/*import net.neoforged.neoforge.common.NeoForge;*/
/*import net.neoforged.neoforge.event.server.ServerStartedEvent;*/
//?}
import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;
import net.minecraft.server.MinecraftServer;

public final class ServerReadyNotifier {
	private static final long ANNOUNCEMENT_DELAY_SECONDS = 5L;

	private ServerReadyNotifier() {
	}

	public static void register() {
		//? if fabric {
		ServerLifecycleEvents.SERVER_STARTED.register(ServerReadyNotifier::scheduleAnnouncement);
		//?} else {
		/*NeoForge.EVENT_BUS.addListener(ServerReadyNotifier::onServerStarted);*/
		//?}
	}

	//? if neoforge {
	/*private static void onServerStarted(ServerStartedEvent event) {*/
		/*scheduleAnnouncement(event.getServer());*/
	/*}*/
	//?}

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
