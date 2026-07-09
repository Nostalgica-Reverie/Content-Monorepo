package link.infra.packwiz.installer.bootstrap;

import java.io.FileNotFoundException;
import java.io.FileReader;
import java.io.IOException;
import java.util.Collections;
import java.util.List;
import java.util.Properties;

/**
 * Main class+method for the bootstrapper.
 * Has two modes:
 * 1. Detects packwiz.chainload.class property in packwiz-installer-bootstrap.properties or Java System Properties
 * 	 - Then starts itself as a subprocess (required to not initialise AWT in the current process)
 * 	 - Then invokes the main method of the class specified in packwiz.chainload.class
 * 2. Parses command line arguments, and starts an AWT window (if GUI is enabled)
 * TODO: need to relocate minimal-json and commons-cli
 */
public class Main {
	public static void main(String[] args) {
		// Check for chainload in Java System Properties
		if (attemptChainload(System.getProperties(), args)) { return; }

		// Try looking in packwiz-installer-bootstrap.properties
		try (FileReader reader = new FileReader("packwiz-installer-bootstrap.properties")) {
			Properties props = new Properties();
			props.load(reader);
			if (attemptChainload(props, args)) {
				return;
			} else {
				throw new RuntimeException("packwiz-installer-bootstrap.properties is invalid");
			}
		} catch (FileNotFoundException ignored) {
			// Ignored - continue trying to start without chainloading
		} catch (IOException e) {
			throw new RuntimeException("Failed to read packwiz-installer-bootstrap.properties", e);
		}

		Bootstrap.init(args);
	}

	private static boolean attemptChainload(Properties props, String[] args) {
		// Check for chainload class
		String chainloadClass = props.getProperty("packwiz.chainload.class");

		// Check for chainload jar
		String chainloadJar = props.getProperty("packwiz.chainload.jar");

		if (chainloadClass != null || chainloadJar != null) {
			// TODO: parse args (packwiz.chainload.args.0?)
			List<String> bootstrapArgs = Collections.emptyList();

			if (chainloadClass != null) {
				ChainloadHandler.startChainloadClass(chainloadClass, bootstrapArgs, args);
			} else {
				ChainloadHandler.startChainloadJar(chainloadJar, bootstrapArgs, args);
			}
			return true;
		}
		return false;
	}
}
