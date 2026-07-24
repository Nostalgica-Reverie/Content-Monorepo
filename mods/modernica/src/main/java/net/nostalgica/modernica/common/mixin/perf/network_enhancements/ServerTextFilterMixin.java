package net.nostalgica.modernica.common.mixin.perf.network_enhancements;

import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import net.minecraft.server.network.ServerTextFilter;

/**
 * See {@link VarLongMixin}'s class doc for the independent-reimplementation context.
 * <p>
 * Vanilla's chat-filter worker pool ({@code createWorkerPool}) is a small, bounded platform-thread
 * pool whose threads spend almost all their time blocked on an HTTP call to a third-party moderation
 * endpoint per chat message - a textbook blocking-IO workload, and exactly what Java 21's virtual
 * threads exist for: a thread per in-flight request, no bound to size, no pool to exhaust under a
 * burst of chat activity. This is a standard, publicly documented virtual-thread migration pattern
 * (see JEP 444), not anything specific to a third-party mod.
 */
@Mixin(ServerTextFilter.class)
public class ServerTextFilterMixin {
    /**
     * @author modernica
     * @reason swap the bounded platform-thread pool for a virtual-thread-per-task executor; the
     * {@code workerCount} argument (vanilla's platform-thread bound) no longer applies
     */
    @Overwrite
    protected static ExecutorService createWorkerPool(int workerCount) {
        return Executors.newThreadPerTaskExecutor(Thread.ofVirtual().name("modernica-text-filter-", 0).factory());
    }
}
