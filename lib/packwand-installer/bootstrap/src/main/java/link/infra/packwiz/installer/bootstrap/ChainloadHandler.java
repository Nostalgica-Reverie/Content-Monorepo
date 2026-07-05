package link.infra.packwiz.installer.bootstrap;

import java.io.IOException;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.URISyntaxException;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public class ChainloadHandler {
	private ChainloadHandler() {} // Who needs OOP anyway?

	public static void startChainloadClass(String chainloadClass, List<String> bootstrapArgs, String[] args) {
		System.out.println("[packwiz-installer-bootstrap] Invoking self (chainload class " + chainloadClass + ")");

		runSubprocessSelf(bootstrapArgs);
		runChainloadClass(Arrays.asList(args), chainloadClass);
	}

	public static void startChainloadJar(String chainloadJar, List<String> bootstrapArgs, String[] args) {
		System.out.println("[packwiz-installer-bootstrap] Invoking self (chainload jar " + chainloadJar + ")");

		// TODO: check chainload path validity? (shouldn't allow traversal)
		// TODO: threat model for this file??
		runSubprocessSelf(bootstrapArgs);
		runSubprocess(Arrays.asList(args), chainloadJar);
	}

	private static void runSubprocess(List<String> args, String jarPath) {
		List<String> command = new ArrayList<>();
		command.add(Paths.get(System.getProperty("java.home"), "bin", "java").toString());
		command.add("-jar");
		command.add(jarPath);
		command.addAll(args);

		ProcessBuilder pb = new ProcessBuilder();
		pb.command(command);
		pb.inheritIO();
		try {
			int res = pb.start().waitFor();
			if (res != 0) {
				System.exit(res);
			}
		} catch (InterruptedException | IOException e) {
			throw new RuntimeException("packwiz-installer-bootstrap failed to run subprocess: ", e);
		}
	}

	private static void runSubprocessSelf(List<String> args) {
		String jarPath;
		// Get current jar path
		try {
			jarPath = Paths.get(ChainloadHandler.class.getProtectionDomain().getCodeSource().getLocation().toURI()).toString();
		} catch (URISyntaxException e) {
			throw new RuntimeException("packwiz-installer-bootstrap failed to run self subprocess: ", e);
		}
		runSubprocess(args, jarPath);
	}

	private static void runChainloadClass(List<String> args, String mainClassName) {
		Class<?> mainClass;
		try {
			mainClass = ChainloadHandler.class.getClassLoader().loadClass(mainClassName);
		} catch (ClassNotFoundException e) {
			throw new RuntimeException("packwiz-installer-bootstrap failed chainload: ", e);
		}
		if (mainClass != null) {
			try {
				Method main = mainClass.getMethod("main", String[].class);
				main.invoke(null, new Object[] {args.toArray(new String[0])});
			} catch (NoSuchMethodException | IllegalAccessException e) {
				throw new RuntimeException("packwiz-installer-bootstrap failed chainload: ", e);
			} catch (InvocationTargetException e) {
				throw new RuntimeException(e.getTargetException());
			}
		}
	}
}
