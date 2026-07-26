//! Bounded parallelism for Packwand's batch operations.
//!
//! Packwand's heavy commands are lists of independent units of work: hash
//! every file in a pack, resolve every mod, generate checksums for every pack
//! subdir. The units are wildly uneven — a 40 MB jar next to a 200-byte JSON
//! file — so work is handed out from a shared queue rather than sliced into
//! fixed chunks. A worker that draws a big item does not stall the others.
//!
//! Deliberately dependency-free: this is a work queue over
//! [`std::thread::scope`], which lets workers borrow from the caller's stack
//! and keeps results in input order.

#![forbid(unsafe_code)]

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

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

/// Applies `f` to every item, up to `jobs` at a time, returning the results in
/// input order.
///
/// `f` runs on other threads, so it takes `&T` and must be `Sync`. Panics in
/// `f` propagate once the scope joins, as they would sequentially.
pub fn map<T, R, F>(items: &[T], jobs: Jobs, f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync,
{
    // One worker, or nothing to do: skip the threads entirely so small
    // invocations pay nothing for the machinery.
    let workers = jobs.get().min(items.len());
    if workers <= 1 {
        return items.iter().map(f).collect();
    }

    // Slots are filled by index, so results stay in input order regardless of
    // which worker finishes first.
    let slots: Vec<Mutex<Option<R>>> = items.iter().map(|_| Mutex::new(None)).collect();
    let next = AtomicUsize::new(0);

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(item) = items.get(index) else {
                        break;
                    };
                    let value = f(item);
                    *slots[index]
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(value);
                }
            });
        }
    });

    slots
        .into_iter()
        .map(|slot| {
            slot.into_inner()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .expect("every slot is filled before the scope joins")
        })
        .collect()
}

/// Runs `f` for every item, up to `jobs` at a time, discarding results.
pub fn for_each<T, F>(items: &[T], jobs: Jobs, f: F)
where
    T: Sync,
    F: Fn(&T) + Sync,
{
    map(items, jobs, |item| f(item));
}

/// Applies a fallible `f` to every item, returning results in input order.
/// Every item is attempted — one failure does not cancel the rest, which is
/// what batch commands want so a single bad pack still reports the others.
pub fn try_map<T, R, E, F>(items: &[T], jobs: Jobs, f: F) -> Vec<Result<R, E>>
where
    T: Sync,
    R: Send,
    E: Send,
    F: Fn(&T) -> Result<R, E> + Sync,
{
    map(items, jobs, f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

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
