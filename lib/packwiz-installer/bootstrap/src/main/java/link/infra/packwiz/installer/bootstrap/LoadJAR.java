package link.infra.packwiz.installer.bootstrap;

import org.apache.commons.cli.Options;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.lang.reflect.Method;
import java.net.MalformedURLException;
import java.net.URL;
import java.net.URLClassLoader;
import java.util.jar.JarInputStream;
import java.util.jar.Manifest;

public class LoadJAR {
	
	private static Class<?> mainClass = null;
	
	private static void loadClass(String path) throws MalformedURLException, ClassNotFoundException {
		if (mainClass != null) {
			return;
		}
		
		if (path == null) {
			path = Bootstrap.JAR_NAME;
		}
		
		URLClassLoader child = new URLClassLoader(new URL[] { new File(path).toURI().toURL() },
				LoadJAR.class.getClassLoader());
		mainClass = Class.forName("link.infra.packwiz.installer.Main", true, child);
	}
	
	public static boolean addOptions(Options options, String path) {
		try {
			loadClass(path);
			
			Method method = mainClass.getDeclaredMethod("addNonBootstrapOptions", Options.class);
			method.invoke(null, options);
		} catch (Exception e) {
			// Woah that was a lot of errors!
			// Just ignore them, it doesn't matter if there aren't any more options added anyway - it's just a help command.
			return false;
		}
		return true;
	}
	
	public static void start(String[] args, String path) throws Exception {
		loadClass(path);
		
		// Must be casted to Object (not array) because varargs
		mainClass.getConstructor(String[].class).newInstance((Object)args);
	}
	
	public static String getVersion(String path) {
		JarInputStream jarStream;
		try {
			jarStream = new JarInputStream(new FileInputStream(path));
		} catch (IOException e) {
			return null;
		}
		String version = null;
		// Aggressive try/catching - what if there isn't a manifest?
		try {
			Manifest mf = jarStream.getManifest();
			version = mf.getMainAttributes().getValue("Implementation-Version");
		} catch (Exception ignored) {}
		
		// Clean up
		try {
			jarStream.close();
		} catch (IOException ignored) {}
		return version;
	}
}
