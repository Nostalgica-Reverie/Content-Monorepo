//! Operational transform over a flat offset space.
//!
//! This is the smallest correct thing that makes two people typing in one
//! buffer converge, and it is deliberately the first module written: a bug
//! here is invisible until it has already corrupted somebody's file, so it
//! gets tested on its own before anything is wired to an editor.
//!
//! # Why transform rather than a CRDT
//!
//! There is exactly one authority (the host) and a small number of peers. The
//! host serialises every operation, so the only case to handle is "a guest
//! sent an op based on a revision the host has since moved past" — transform
//! the incoming op against the ops applied in between. That is one function
//! and no new dependency. A CRDT would live in Rust while the buffers live in
//! TypeScript inside Monaco, forcing every keystroke through two marshalling
//! hops to buy robustness this session shape does not need.
//!
//! # The offset space
//!
//! Offsets are UTF-16 code units, because that is what Monaco's `ITextModel`
//! counts in and converting on every op would be both slow and a second place
//! to get emoji wrong.

use serde::{Deserialize, Serialize};

/// A single edit at an offset.
///
/// Retain is deliberately absent: with a single authority the ops are always
/// absolute-offset insert/delete pairs, and a retain-based encoding would make
/// every op depend on document length, which is exactly the coupling that
/// makes classic OT hard to reason about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TextOp {
	Insert { offset: usize, text: String },
	Delete { offset: usize, length: usize },
}

impl TextOp {
	/// Where the op starts.
	const fn offset(&self) -> usize {
		match self {
			Self::Insert { offset, .. } | Self::Delete { offset, .. } => *offset,
		}
	}

	/// How many code units the op spans in the *pre*-op document.
	const fn span(&self) -> usize {
		match self {
			Self::Insert { .. } => 0,
			Self::Delete { length, .. } => *length,
		}
	}

	/// How the document length changes when this is applied.
	fn delta(&self) -> isize {
		match self {
			Self::Insert { text, .. } => text.encode_utf16().count() as isize,
			Self::Delete { length, .. } => -(*length as isize),
		}
	}
}

/// Rebases `incoming` so it can be applied after `applied` already was.
///
/// # Why this returns a list
///
/// A delete whose range spans a remote insert cannot be expressed as one
/// operation. Deleting `[1,7)` of `"hello world"` while the other peer inserts
/// `"A"` at offset 3 has to remove two disjoint ranges in the new document,
/// because the inserted character sits between them and is *not* ours to
/// delete. Returning a single op forces a choice between swallowing the other
/// peer's text and dropping part of our own deletion — and dropping part of it
/// makes the two documents diverge, which the convergence tests below catch.
///
/// An empty list means the op was annihilated: a delete whose range was
/// already removed by the other side. That is a real outcome, not an error —
/// both peers deleted the same text and the second delete has nothing left to
/// do.
///
/// **The returned ops are ordered for sequential application** — highest
/// offset first, so each is valid against the document the previous one left.
///
/// `insert_wins_tie` breaks the ambiguous case of two inserts at the same
/// offset, where the text could legitimately land in either order. Both peers
/// must pass opposite values for the same pair, which is what makes them agree
/// rather than each preferring itself.
pub fn transform(incoming: &TextOp, applied: &TextOp, insert_wins_tie: bool) -> Vec<TextOp> {
	match (incoming, applied) {
		(
			TextOp::Insert { offset, text },
			TextOp::Insert {
				offset: applied_offset,
				..
			},
		) => {
			let shift = applied.delta().max(0) as usize;
			let moved =
				if *offset > *applied_offset || (*offset == *applied_offset && !insert_wins_tie) {
					offset + shift
				} else {
					*offset
				};
			vec![TextOp::Insert {
				offset: moved,
				text: text.clone(),
			}]
		}

		(TextOp::Insert { offset, text }, TextOp::Delete { .. }) => {
			let start = applied.offset();
			let end = start + applied.span();
			// An insert inside deleted text collapses to the deletion point
			// rather than being dropped: the typed characters are the user's
			// and survive, they just have nowhere else to go.
			let moved = if *offset <= start {
				*offset
			} else if *offset >= end {
				offset - applied.span()
			} else {
				start
			};
			vec![TextOp::Insert {
				offset: moved,
				text: text.clone(),
			}]
		}

		(TextOp::Delete { offset, length }, TextOp::Insert { .. }) => {
			let inserted = applied.delta().max(0) as usize;
			let applied_offset = applied.offset();
			if applied_offset <= *offset {
				vec![TextOp::Delete {
					offset: offset + inserted,
					length: *length,
				}]
			} else if applied_offset >= offset + length {
				vec![TextOp::Delete {
					offset: *offset,
					length: *length,
				}]
			} else {
				// The insert landed inside the range being deleted, so the
				// deletion splits around it. Emitted high offset first so both
				// remain valid when applied in order.
				let head = applied_offset - offset;
				vec![
					TextOp::Delete {
						offset: applied_offset + inserted,
						length: length - head,
					},
					TextOp::Delete {
						offset: *offset,
						length: head,
					},
				]
			}
		}

		(TextOp::Delete { offset, length }, TextOp::Delete { .. }) => {
			let start = *offset;
			let end = start + length;
			let applied_start = applied.offset();
			let applied_end = applied_start + applied.span();
			// Overlap is removed from this delete; whatever the other side
			// already took is no longer ours to take.
			let overlap = end
				.min(applied_end)
				.saturating_sub(start.max(applied_start));
			let remaining = length - overlap;
			if remaining == 0 {
				return Vec::new();
			}
			let moved = if applied_start < start {
				start - (start - applied_start).min(applied.span())
			} else {
				start
			};
			vec![TextOp::Delete {
				offset: moved,
				length: remaining,
			}]
		}
	}
}

/// Rebases an op across every op applied since its base revision.
pub fn transform_all(incoming: &TextOp, applied: &[TextOp], insert_wins_tie: bool) -> Vec<TextOp> {
	applied
		.iter()
		.fold(vec![incoming.clone()], |pending, next| {
			pending
				.iter()
				.flat_map(|operation| transform(operation, next, insert_wins_tie))
				.collect()
		})
}

/// Applies an op to a string, for tests and for the host's own bookkeeping.
///
/// Operates on UTF-16 code units to match Monaco. Out-of-range offsets are
/// clamped rather than panicking: a peer is untrusted input, and a malformed
/// offset must not be able to abort the host's session thread.
pub fn apply(text: &str, operation: &TextOp) -> String {
	let units: Vec<u16> = text.encode_utf16().collect();
	let mut next = Vec::with_capacity(units.len() + 16);
	match operation {
		TextOp::Insert { offset, text: new } => {
			let at = (*offset).min(units.len());
			next.extend_from_slice(&units[..at]);
			next.extend(new.encode_utf16());
			next.extend_from_slice(&units[at..]);
		}
		TextOp::Delete { offset, length } => {
			let at = (*offset).min(units.len());
			let end = (at + length).min(units.len());
			next.extend_from_slice(&units[..at]);
			next.extend_from_slice(&units[end..]);
		}
	}
	String::from_utf16_lossy(&next)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn insert(offset: usize, text: &str) -> TextOp {
		TextOp::Insert {
			offset,
			text: text.to_owned(),
		}
	}

	const fn delete(offset: usize, length: usize) -> TextOp {
		TextOp::Delete { offset, length }
	}

	/// The property that matters: applying `a` then the rebased `b` must give
	/// the same document as applying `b` then the rebased `a`. If this fails,
	/// two people typing produce two different files.
	fn assert_converges(base: &str, left: &TextOp, right: &TextOp) {
		let apply_all = |text: &str, ops: &[TextOp]| {
			ops.iter().fold(text.to_owned(), |current, operation| {
				apply(&current, operation)
			})
		};
		let left_first = {
			let text = apply(base, left);
			apply_all(&text, &transform(right, left, false))
		};
		let right_first = {
			let text = apply(base, right);
			apply_all(&text, &transform(left, right, true))
		};
		assert_eq!(
			left_first, right_first,
			"diverged: {left:?} vs {right:?} on {base:?}"
		);
	}

	#[test]
	fn insert_insert_converges_at_every_relative_position() {
		assert_converges("hello world", &insert(0, "A"), &insert(5, "B"));
		assert_converges("hello world", &insert(5, "A"), &insert(0, "B"));
		assert_converges("hello world", &insert(11, "A"), &insert(11, "B"));
	}

	/// The ambiguous case. Both orderings are defensible; what is not
	/// defensible is the two peers choosing differently.
	#[test]
	fn two_inserts_at_the_same_offset_converge() {
		assert_converges("hello", &insert(2, "A"), &insert(2, "B"));
	}

	#[test]
	fn insert_delete_converges() {
		assert_converges("hello world", &insert(0, "A"), &delete(5, 6));
		assert_converges("hello world", &insert(11, "A"), &delete(0, 5));
		// Insert inside the deleted range.
		assert_converges("hello world", &insert(3, "A"), &delete(1, 6));
	}

	#[test]
	fn delete_delete_converges() {
		assert_converges("hello world", &delete(0, 5), &delete(6, 5));
		assert_converges("hello world", &delete(6, 5), &delete(0, 5));
		// Partial overlap in both directions.
		assert_converges("hello world", &delete(0, 7), &delete(4, 7));
		assert_converges("hello world", &delete(4, 7), &delete(0, 7));
	}

	/// Both peers deleting the same text is a real thing users do, and the
	/// second delete must vanish rather than eat its neighbours.
	#[test]
	fn a_fully_superseded_delete_is_annihilated() {
		assert!(transform(&delete(2, 3), &delete(2, 3), false).is_empty());
		assert!(transform(&delete(3, 1), &delete(2, 3), false).is_empty());
		assert_converges("hello world", &delete(2, 3), &delete(2, 3));
	}

	/// Offsets are UTF-16 code units because Monaco counts in those. An emoji
	/// outside the BMP is two units, and treating it as one silently splits it.
	#[test]
	fn offsets_are_utf16_code_units() {
		let base = "a🙂b";
		assert_eq!(base.encode_utf16().count(), 4);
		assert_eq!(apply(base, &insert(3, "X")), "a🙂Xb");
		assert_eq!(apply(base, &delete(1, 2)), "ab");
	}

	/// A peer is untrusted input; a bad offset must not panic the host thread.
	#[test]
	fn out_of_range_offsets_are_clamped_not_panicked() {
		assert_eq!(apply("abc", &insert(99, "X")), "abcX");
		assert_eq!(apply("abc", &delete(99, 5)), "abc");
		assert_eq!(apply("abc", &delete(1, 99)), "a");
	}

	/// The case that forced `transform` to return a list. A single op here
	/// either swallows the other peer's insert or drops part of the deletion,
	/// and the second makes the documents diverge.
	#[test]
	fn a_delete_spanning_a_remote_insert_splits_into_two() {
		let split = transform(&delete(1, 6), &insert(3, "A"), false);
		assert_eq!(split, [delete(4, 4), delete(1, 2)]);
		let text = apply("hello world", &insert(3, "A"));
		let result = split
			.iter()
			.fold(text, |current, operation| apply(&current, operation));
		assert_eq!(result, "hAorld");
	}

	#[test]
	fn transform_all_rebases_across_a_run_of_ops() {
		let applied = [insert(0, "xx"), delete(5, 1)];
		let rebased = transform_all(&insert(3, "Y"), &applied, false);
		assert_eq!(rebased, [insert(5, "Y")]);
	}
}
