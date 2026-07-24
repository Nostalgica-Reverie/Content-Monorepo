package net.nostalgica.modernica.common.advancement;

import net.minecraft.world.item.ItemStack;

import java.util.SortedSet;
import java.util.TreeSet;

public class StackSizeThresholds {
    private static final SortedSet<Integer> thresholds = new TreeSet<>();

    public static void clear() {
        thresholds.clear();
        thresholds.add(1);
    }

    public static void add(int value) {
        thresholds.add(value);
    }

    public static boolean stackPassesThreshold(ItemStack stack) {
        int prevValue = ((ItemStackDataHolder) (Object) stack).mfh$getPreviousStackSize();
        int newValue = stack.getCount();

        for (int threshold : thresholds) {
            if (prevValue < threshold && newValue >= threshold) {
                return true;
            }
        }

        return false;
    }
}
