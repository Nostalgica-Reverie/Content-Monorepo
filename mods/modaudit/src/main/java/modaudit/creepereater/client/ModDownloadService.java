package modaudit.creepereater.client;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.URI;
import java.net.URLDecoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.security.DigestInputStream;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.Duration;
import java.util.HexFormat;
import java.util.Locale;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.regex.Pattern;

public final class ModDownloadService {
	private static final long MAX_DOWNLOAD_SIZE = 1_073_741_824L;
	private static final Pattern CONTENT_DISPOSITION_FILE_NAME = Pattern.compile("(?i)filename\\*?=(?:UTF-8''|\\\")?([^\\\";]+)");
	private static final HttpClient HTTP_CLIENT = HttpClient.newBuilder()
		.followRedirects(HttpClient.Redirect.NORMAL)
		.connectTimeout(Duration.ofSeconds(20))
		.build();

	private ModDownloadService() {}

	public static CompletableFuture<Path> download(ExternalMod mod, Path modsDirectory) {
		return CompletableFuture.supplyAsync(() -> {
			try {
				return downloadNow(mod, modsDirectory);
			} catch (IOException | InterruptedException exception) {
				if (exception instanceof InterruptedException) {
					Thread.currentThread().interrupt();
				}
				throw new CompletionException(exception);
			}
		});
	}

	private static Path downloadNow(ExternalMod mod, Path modsDirectory) throws IOException, InterruptedException {
		HttpRequest request = HttpRequest.newBuilder(URI.create(mod.url()))
			.timeout(Duration.ofMinutes(5))
			.header("User-Agent", "ModAudit/1.0.0")
			.GET()
			.build();
		HttpResponse<InputStream> response = HTTP_CLIENT.send(request, HttpResponse.BodyHandlers.ofInputStream());
		try (InputStream body = response.body()) {
			if (response.statusCode() < 200 || response.statusCode() >= 300) {
				throw new IOException("Download returned HTTP " + response.statusCode());
			}

			if (response.headers().firstValueAsLong("Content-Length").orElse(-1L) > MAX_DOWNLOAD_SIZE) {
				throw new IOException("Download is larger than 1 GB");
			}

			String fileName = resolveFileName(mod, response);
			Files.createDirectories(modsDirectory);
			Path target = modsDirectory.resolve(fileName);
			if (Files.exists(target)) {
				verifyHash(target, mod.sha256());
				return target;
			}

			Path temporary = Files.createTempFile(modsDirectory, "mod-audit-", ".download");
			try {
				copy(body, temporary);
				verifyHash(temporary, mod.sha256());
				return Files.move(temporary, target, StandardCopyOption.ATOMIC_MOVE);
			} finally {
				Files.deleteIfExists(temporary);
			}
		}
	}

	private static String resolveFileName(ExternalMod mod, HttpResponse<?> response) throws IOException {
		String configured = normalizeFileName(mod.fileName());
		if (configured != null) {
			return configured;
		}

		var matcher = CONTENT_DISPOSITION_FILE_NAME.matcher(
			response.headers().firstValue("Content-Disposition").orElse("")
		);
		if (matcher.find()) {
			String resolved = normalizeFileName(URLDecoder.decode(matcher.group(1).trim(), StandardCharsets.UTF_8));
			if (resolved != null) {
				return resolved;
			}
		}

		String path = response.uri().getPath();
		if (path != null) {
			String resolved = normalizeFileName(path.substring(path.lastIndexOf('/') + 1));
			if (resolved != null) {
				return resolved;
			}
		}

		throw new IOException("The download did not provide a JAR filename");
	}

	private static String normalizeFileName(String fileName) throws IOException {
		if (fileName == null || fileName.isBlank()) {
			return null;
		}

		String trimmed = fileName.trim();
		if (!trimmed.toLowerCase(Locale.ROOT).endsWith(".jar")
			|| trimmed.contains("/")
			|| trimmed.contains("\\")
			|| trimmed.contains("..")
			|| trimmed.chars().anyMatch(character -> Character.isISOControl(character) || "<>:\\|?*".indexOf(character) >= 0)) {
			throw new IOException("Invalid JAR filename");
		}

		return trimmed;
	}

	private static void copy(InputStream input, Path target) throws IOException {
		try (var output = Files.newOutputStream(target)) {
			byte[] buffer = new byte[65_536];
			long total = 0L;
			for (int read; (read = input.read(buffer)) >= 0;) {
				total += read;
				if (total > MAX_DOWNLOAD_SIZE) {
					throw new IOException("Download is larger than 1 GB");
				}
				output.write(buffer, 0, read);
			}
		}
	}

	private static void verifyHash(Path file, String expectedHash) throws IOException {
		if (expectedHash == null || expectedHash.isBlank()) {
			return;
		}

		try {
			MessageDigest digest = MessageDigest.getInstance("SHA-256");
			try (var input = new DigestInputStream(Files.newInputStream(file), digest)) {
				input.transferTo(OutputStream.nullOutputStream());
			}

			if (!HexFormat.of().formatHex(digest.digest()).equalsIgnoreCase(expectedHash)) {
				throw new IOException("SHA-256 verification failed");
			}
		} catch (NoSuchAlgorithmException exception) {
			throw new IOException("SHA-256 is unavailable", exception);
		}
	}
}
