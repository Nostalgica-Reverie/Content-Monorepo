//! Bounded parallelism for Packwand's batch operations.
//!
//! Packwand's heavy commands are lists of independent units of work: hash
//! every file in a pack, resolve every mod, generate checksums for every pack
//! subdir. The units are wildly uneven — a 40 MB jar next to a 200-byte JSON
//! file — so work is stolen from a shared pool rather than sliced into fixed
//! chunks. A worker that draws a big item does not stall the others.
//!
//! This used to be a hand-rolled queue over [`std::thread::scope`], kept
//! dependency-free on purpose. It is now backed by Rayon: the queue was
//! duplicating Rayon's work-stealing with a coarser strategy (one global
//! `Mutex` per result slot), and Rayon is already in this workspace's
//! dependency graph via the Packeater tree, so it costs nothing new to link.
//!
//! The public surface is unchanged, and the two properties call sites depend
//! on still hold:
//!
//! * **Results stay in input order.** Several callers hash a sequence of
//!   file contents into one stream and would produce a different digest if
//!   the order moved. `par_iter().map(..).collect::<Vec<_>>()` is
//!   order-preserving.
//! * **Width follows [`Jobs`], not the machine.** Rayon's *global* pool
//!   ignores `--jobs`, so this builds its own pool per distinct worker count
//!   and runs inside it. Pools are cached: building one spawns threads, and
//!   the batching callers in `packwand-diagnostics` call in a loop.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{LazyLock, Mutex, OnceLock};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// How many units of work may run at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Jobs(NonZeroUsize);

impl Jobs {
    /// Resolves the worker count. `0` (the CLI default for `--jobs`) means
    /// "decide for me", which is the machine's parallelism capped at 16 —
    /// beyond that these workloads are bound by disk or by a provider's
    /// request budget, not by cores, and the extra threads only add contention.
    #[must_use]
    pub fn new(requested: usize) -> Self {
        if let Some(explicit) = NonZeroUsize::new(requested) {
            return Self(explicit);
        }
        let available = std::thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1);
        Self(NonZeroUsize::new(available.clamp(1, 16)).expect("clamped to at least 1"))
    }

    #[must_use]
    pub fn get(self) -> usize {
        self.0.get()
    }
}

impl Default for Jobs {
    fn default() -> Self {
        Self::new(0)
    }
}

static CONFIGURED: OnceLock<Jobs> = OnceLock::new();

/// Records the worker count for the whole process, so `--jobs` reaches library
/// code without threading a parameter through every call site. First call
/// wins; later ones are ignored.
pub fn configure(jobs: Jobs) {
    let _ = CONFIGURED.set(jobs);
}

/// The configured worker count, or the automatic default when the host never
/// called [`configure`] — as the GUI and tests do not.
#[must_use]
pub fn configured() -> Jobs {
    CONFIGURED.get().copied().unwrap_or_default()
}

/// Pools keyed by worker count. A pool owns OS threads, so building one per
/// call would cost more than the work it parallelizes — `packwand-diagnostics`
/// calls [`try_map`] once per batch of files.
static POOLS: LazyLock<Mutex<HashMap<usize, &'static rayon::ThreadPool>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The pool for `jobs` workers, built on first use.
///
/// Leaked deliberately: the pool lives for the rest of the process either way
/// (the map holds it forever), and a `&'static` lets callers borrow from the
/// caller's stack inside `install` without an `Arc` clone on every call.
fn pool_for(jobs: Jobs) -> &'static rayon::ThreadPool {
    let mut pools = POOLS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    pools.entry(jobs.get()).or_insert_with(|| {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs.get())
            .thread_name(move |index| format!("packwand-{index}"))
            .build()
            .expect("a thread pool with a non-zero worker count is always buildable");
        Box::leak(Box::new(pool))
    })
}

/// Applies `f` to every item, up to `jobs` at a time, returning the results in
/// input order.
///
/// `f` runs on other threads, so it takes `&T` and must be `Sync`. Panics in
/// `f` propagate once the scope joins, as they would sequentially.
pub fn map<T, R, F>(items: &[T], jobs: Jobs, f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync + Send,
{
    // One worker, or nothing to do: skip the pool entirely so small
    // invocations pay nothing for the machinery.
    if jobs.get() <= 1 || items.len() <= 1 {
        return items.iter().map(f).collect();
    }
    // `collect` on an indexed parallel iterator writes by position, so results
    // stay in input order regardless of which worker finishes first.
    pool_for(jobs).install(|| items.par_iter().map(f).collect())
}

/// Runs `f` for every item, up to `jobs` at a time, discarding results.
pub fn for_each<T, F>(items: &[T], jobs: Jobs, f: F)
where
    T: Sync,
    F: Fn(&T) + Sync + Send,
{
    if jobs.get() <= 1 || items.len() <= 1 {
        items.iter().for_each(f);
        return;
    }
    pool_for(jobs).install(|| items.par_iter().for_each(f));
}

/// Applies a fallible `f` to every item, returning results in input order.
/// Every item is attempted — one failure does not cancel the rest, which is
/// what batch commands want so a single bad pack still reports the others.
pub fn try_map<T, R, E, F>(items: &[T], jobs: Jobs, f: F) -> Vec<Result<R, E>>
where
    T: Sync,
    R: Send,
    E: Send,
    F: Fn(&T) -> Result<R, E> + Sync + Send,
{
    map(items, jobs, f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

    #[test]
    fn zero_requests_a_sensible_default() {
        let jobs = Jobs::new(0);
        assert!(jobs.get() >= 1 && jobs.get() <= 16);
    }

    #[test]
    fn an_explicit_count_is_honored_even_above_the_cap() {
        assert_eq!(Jobs::new(1).get(), 1);
        assert_eq!(Jobs::new(32).get(), 32);
    }

    #[test]
    fn results_keep_input_order() {
        let items: Vec<usize> = (0..500).collect();
        let doubled = map(&items, Jobs::new(8), |value| value * 2);
        assert_eq!(
            doubled,
            items.iter().map(|value| value * 2).collect::<Vec<_>>()
        );
    }

    #[test]
    fn every_item_runs_exactly_once() {
        let items: Vec<usize> = (0..1_000).collect();
        let seen = AtomicU32::new(0);
        for_each(&items, Jobs::new(8), |_| {
            seen.fetch_add(1, Ordering::Relaxed);
        });
        assert_eq!(seen.load(Ordering::Relaxed), 1_000);
    }

    #[test]
    fn an_empty_input_does_no_work() {
        let items: Vec<usize> = Vec::new();
        assert!(map(&items, Jobs::new(8), |value| *value).is_empty());
    }

    #[test]
    fn uneven_work_still_completes() {
        // One deliberately heavy item among cheap ones: a static split would
        // leave a worker idle, the shared queue must not.
        let items: Vec<usize> = (0..64).map(|i| if i == 7 { 50_000 } else { 1 }).collect();
        let totals = map(&items, Jobs::new(4), |count| (0..*count).sum::<usize>());
        assert_eq!(totals.len(), 64);
        assert_eq!(totals[7], (0..50_000).sum::<usize>());
    }

    /// The reason this crate builds its own pool instead of using
    /// `par_iter` directly: Rayon's global pool is sized from the machine and
    /// would silently ignore `--jobs`.
    #[test]
    fn concurrency_never_exceeds_the_requested_worker_count() {
        let items: Vec<usize> = (0..256).collect();
        let live = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        for_each(&items, Jobs::new(3), |_| {
            let current = live.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(current, Ordering::SeqCst);
            // Long enough that workers genuinely overlap rather than each
            // finishing before the next one starts.
            std::thread::sleep(std::time::Duration::from_micros(200));
            live.fetch_sub(1, Ordering::SeqCst);
        });
        let peak = peak.load(Ordering::SeqCst);
        assert!(peak > 1, "expected real parallelism, saw {peak}");
        assert!(peak <= 3, "exceeded the requested worker count: {peak}");
    }

    /// Pools own OS threads; the batching callers in `packwand-diagnostics`
    /// call once per batch, so a fresh pool per call would dominate the work.
    #[test]
    fn pools_are_reused_across_calls_with_the_same_width() {
        let jobs = Jobs::new(4);
        assert!(
            std::ptr::eq(pool_for(jobs), pool_for(jobs)),
            "the same worker count must map to one pool"
        );
        assert!(!std::ptr::eq(
            pool_for(Jobs::new(4)),
            pool_for(Jobs::new(5))
        ));
    }

    #[test]
    fn failures_do_not_cancel_the_rest() {
        let items: Vec<usize> = (0..100).collect();
        let results = try_map(&items, Jobs::new(4), |value| {
            if value % 10 == 0 {
                Err(*value)
            } else {
                Ok(*value)
            }
        });
        assert_eq!(results.len(), 100);
        assert_eq!(results.iter().filter(|result| result.is_err()).count(), 10);
    }
}
