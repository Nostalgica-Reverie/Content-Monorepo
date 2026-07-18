package packstats.creepereater;

import com.google.gson.Gson;
//? if fabric {
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.loader.api.FabricLoader;
//?} else {
/*import net.neoforged.api.distmarker.Dist;*/
/*import net.neoforged.fml.common.Mod;*/
/*import net.neoforged.fml.loading.FMLPaths;*/
/*import net.neoforged.neoforge.client.event.ClientPlayerNetworkEvent;*/
/*import net.neoforged.neoforge.client.event.ClientTickEvent;*/
/*import net.neoforged.neoforge.common.NeoForge;*/
/*import net.neoforged.neoforge.event.GameShuttingDownEvent;*/
//?}
import net.minecraft.client.Minecraft;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.file.Files;
import java.time.Duration;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;

//? if neoforge
/*@Mod(value = "packstats", dist = Dist.CLIENT)*/
public final class PackStats /*? if fabric {*/ implements ClientModInitializer /*?}*/ {
	private static final Logger LOGGER = LoggerFactory.getLogger("packstats");
	private static final Gson GSON = new Gson();
	private static final long INTERVAL = Duration.ofMinutes(10).toNanos();
	private static final String DEFAULT_CONFIG = """
		{
		  "endpoint": "",
		  "pack_id": "",
		  "pack_version": ""
		}
		""";

	private Config config;
	private HttpClient http;
	private String sessionId;
	private long nextReport;
	private CompletableFuture<Void> pending = CompletableFuture.completedFuture(null);

	//? if fabric {
	@Override
	public void onInitializeClient() {
		initialize();
	}
	//?} else {
	/*public PackStats() {*/
		/*initialize();*/
	/*}*/
	//?}

	private void initialize() {
		config = loadConfig();
		if (config == null) return;
		http = HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(3))
			.followRedirects(HttpClient.Redirect.NEVER).build();
		//? if fabric {
		ClientTickEvents.END_CLIENT_TICK.register(this::tick);
		ClientLifecycleEvents.CLIENT_STOPPING.register(client -> shutdown());
		//?} else {
		/*NeoForge.EVENT_BUS.addListener(this::neoTick);*/
		/*NeoForge.EVENT_BUS.addListener(this::neoLogout);*/
		/*NeoForge.EVENT_BUS.addListener((GameShuttingDownEvent ignored) -> shutdown());*/
		//?}
	}

	//? if neoforge {
	/*private void neoTick(ClientTickEvent.Post ignored) {*/
		/*tick(Minecraft.getInstance());*/
	/*}*/

	/*private void neoLogout(ClientPlayerNetworkEvent.LoggingOut ignored) {*/
		/*offline();*/
	/*}*/
	//?}

	private void tick(Minecraft client) {
		if (client.player == null) {
			offline();
			return;
		}
		long now = System.nanoTime();
		if (sessionId == null) sessionId = UUID.randomUUID().toString();
		else if (now < nextReport) return;
		nextReport = now + INTERVAL;
		send("heartbeat");
	}

	private void offline() {
		if (sessionId == null) return;
		send("offline");
		sessionId = null;
		nextReport = 0;
	}

	private void shutdown() {
		try {
			if (sessionId != null) offline();
			pending.get(6, TimeUnit.SECONDS);
		} catch (Exception ignored) {
		}
	}

	private CompletableFuture<Void> send(String event) {
		var body = ("{\"schema\":1,\"event\":%s,\"session_id\":%s,\"pack_id\":%s,\"pack_version\":%s}")
			.formatted(GSON.toJson(event), GSON.toJson(sessionId), GSON.toJson(config.pack_id()),
				GSON.toJson(config.pack_version()));
		var request = HttpRequest.newBuilder(URI.create(config.endpoint())).timeout(Duration.ofSeconds(3))
			.header("Content-Type", "application/json").POST(HttpRequest.BodyPublishers.ofString(body)).build();
		return pending = pending.exceptionally(ignored -> null)
			.thenCompose(ignored -> http.sendAsync(request, HttpResponse.BodyHandlers.discarding()))
			.thenAccept(ignored -> {});
	}

	private static Config loadConfig() {
		//? if fabric {
		var path = FabricLoader.getInstance().getConfigDir().resolve("packstats.json");
		//?} else {
		/*var path = FMLPaths.CONFIGDIR.get().resolve("packstats.json");*/
		//?}
		try {
			Files.createDirectories(path.getParent());
			if (Files.notExists(path)) Files.writeString(path, DEFAULT_CONFIG);
			var config = GSON.fromJson(Files.readString(path), Config.class);
			if (config.endpoint().isBlank()) return null;
			if (!safeEndpoint(URI.create(config.endpoint())))
				throw new IllegalArgumentException("endpoint must use HTTPS (or localhost HTTP)");
			return config;
		} catch (Exception exception) {
			LOGGER.warn("PackStats disabled: invalid config.");
			return null;
		}
	}

	private static boolean safeEndpoint(URI uri) {
		if (uri.getHost() == null || uri.getUserInfo() != null || uri.getFragment() != null) return false;
		if ("https".equalsIgnoreCase(uri.getScheme())) return true;
		var host = uri.getHost();
		return "http".equalsIgnoreCase(uri.getScheme())
			&& (host.equalsIgnoreCase("localhost") || host.equals("127.0.0.1") || host.equals("[::1]"));
	}

	private record Config(String endpoint, String pack_id, String pack_version) {
	}
}
