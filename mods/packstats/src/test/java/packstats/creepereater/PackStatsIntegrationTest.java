package packstats.creepereater;

import java.lang.reflect.Method;
import java.net.URI;

public final class PackStatsIntegrationTest {
	private PackStatsIntegrationTest() {
	}

	public static void main(String[] args) throws Exception {
		Method safeEndpoint = PackStats.class.getDeclaredMethod("safeEndpoint", URI.class);
		safeEndpoint.setAccessible(true);

		assertEndpoint(safeEndpoint, "https://stats.example.com/events", true);
		assertEndpoint(safeEndpoint, "http://localhost:8080/events", true);
		assertEndpoint(safeEndpoint, "http://127.0.0.1/events", true);
		assertEndpoint(safeEndpoint, "http://[::1]/events", true);
		assertEndpoint(safeEndpoint, "http://stats.example.com/events", false);
		assertEndpoint(safeEndpoint, "https://user@stats.example.com/events", false);
		assertEndpoint(safeEndpoint, "https://stats.example.com/events#fragment", false);
	}

	private static void assertEndpoint(Method safeEndpoint, String value, boolean expected) throws Exception {
		boolean actual = (boolean) safeEndpoint.invoke(null, URI.create(value));
		if (actual != expected) {
			throw new AssertionError("safeEndpoint(" + value + ") = " + actual + ", expected " + expected);
		}
	}
}
