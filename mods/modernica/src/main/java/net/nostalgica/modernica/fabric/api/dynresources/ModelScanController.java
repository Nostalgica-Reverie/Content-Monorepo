package net.nostalgica.modernica.fabric.api.dynresources;

import net.minecraft.resources.Identifier;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Predicate;

public class ModelScanController {
    public static final List<Predicate<Identifier>> SCAN_PREDICATES = new ArrayList<>();
    public static boolean shouldScanAndTestWrapping(Identifier location) {
        if(SCAN_PREDICATES.size() > 0) {
            for(Predicate<Identifier> predicate : SCAN_PREDICATES) {
                if(!predicate.test(location))
                    return false;
            }
        }
        return true;
    }
}
