import type { TextOp } from './invoke/collab'

const length = (operation: TextOp) =>
	operation.kind === 'insert' ? [...operation.text].join('').length : operation.length

/** Rebase one UTF-16 offset operation across one already-applied operation. */
export function transform(incoming: TextOp, applied: TextOp, insertWinsTie: boolean): TextOp[] {
	if (incoming.kind === 'insert' && applied.kind === 'insert') {
		const shift = applied.text.length
		const offset =
			incoming.offset > applied.offset || (incoming.offset === applied.offset && !insertWinsTie)
				? incoming.offset + shift
				: incoming.offset
		return [{ ...incoming, offset }]
	}
	if (incoming.kind === 'insert' && applied.kind === 'delete') {
		const end = applied.offset + applied.length
		const offset =
			incoming.offset <= applied.offset
				? incoming.offset
				: incoming.offset >= end
					? incoming.offset - applied.length
					: applied.offset
		return [{ ...incoming, offset }]
	}
	if (incoming.kind === 'delete' && applied.kind === 'insert') {
		const inserted = length(applied)
		if (applied.offset <= incoming.offset) {
			return [{ ...incoming, offset: incoming.offset + inserted }]
		}
		if (applied.offset >= incoming.offset + incoming.length) return [incoming]
		const head = applied.offset - incoming.offset
		return [
			{ kind: 'delete', offset: applied.offset + inserted, length: incoming.length - head },
			{ kind: 'delete', offset: incoming.offset, length: head },
		]
	}
	if (incoming.kind === 'delete' && applied.kind === 'delete') {
		const end = incoming.offset + incoming.length
		const appliedEnd = applied.offset + applied.length
		const overlap = Math.max(
			0,
			Math.min(end, appliedEnd) - Math.max(incoming.offset, applied.offset),
		)
		const remaining = incoming.length - overlap
		if (!remaining) return []
		const offset =
			applied.offset < incoming.offset
				? incoming.offset - Math.min(incoming.offset - applied.offset, applied.length)
				: incoming.offset
		return [{ kind: 'delete', offset, length: remaining }]
	}
	return [incoming]
}

export function transformAll(
	incoming: TextOp,
	applied: TextOp[],
	insertWinsTie: boolean,
): TextOp[] {
	return applied.reduce<TextOp[]>(
		(pending, operation) =>
			pending.flatMap((candidate) => transform(candidate, operation, insertWinsTie)),
		[incoming],
	)
}

export const transformMany = (incoming: TextOp[], applied: TextOp[], insertWinsTie: boolean) =>
	incoming.flatMap((operation) => transformAll(operation, applied, insertWinsTie))
