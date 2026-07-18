package claritymod.creeereater;

import claritymod.creeereater.server.ServerReadyNotifier;
//? if fabric {
import net.fabricmc.api.ModInitializer;
//?} else {
/*import net.neoforged.fml.common.Mod;*/
//?}
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

//? if neoforge
/*@Mod(Claritymod.MOD_ID)*/
public class Claritymod /*? if fabric {*/ implements ModInitializer /*?}*/ {
	public static final String MOD_ID = "claritymod";
	public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

	//? if fabric
	@Override
	public void onInitialize() {
		initialize();
	}

	//? if neoforge {
	/*public Claritymod() {*/
		/*initialize();*/
	/*}*/
	//?}

	private static void initialize() {
		ServerReadyNotifier.register();
		LOGGER.info("Clarity Mod initialized");
	}
}
