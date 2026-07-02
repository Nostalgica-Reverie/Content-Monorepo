package link.infra.packwiz.installer.bootstrap;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

public class RollbackHandler {
	
	private final Path rollbackPath;
	private byte[] storage = null;
	private boolean hasRollback = false;
	
	public RollbackHandler(String path) {
		rollbackPath = Paths.get(path);
		try {
			storage = Files.readAllBytes(rollbackPath);
			hasRollback = true;
		// Ignore errors, it probably doesn't exist!
		} catch (IOException ignored) {}
	}

	public void rollback() throws IOException {
		if (hasRollback) {
			Files.write(rollbackPath, storage);
		}
	}
	
}
