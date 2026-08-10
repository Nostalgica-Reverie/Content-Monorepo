import { describe, expect, test } from 'bun:test'

import { transform, transformMany } from '../src/helpers/collab-ot'

describe('collaboration OT reconciliation', () => {
	test('equal-offset inserts use opposite tie directions', () => {
		expect(
			transform(
				{ kind: 'insert', offset: 2, text: 'G' },
				{ kind: 'insert', offset: 2, text: 'H' },
				false,
			),
		).toEqual([{ kind: 'insert', offset: 3, text: 'G' }])
		expect(
			transform(
				{ kind: 'insert', offset: 2, text: 'H' },
				{ kind: 'insert', offset: 2, text: 'G' },
				true,
			),
		).toEqual([{ kind: 'insert', offset: 2, text: 'H' }])
	})

	test('a delete spanning a remote insert splits around it', () => {
		expect(
			transform(
				{ kind: 'delete', offset: 1, length: 6 },
				{ kind: 'insert', offset: 3, text: 'A' },
				false,
			),
		).toEqual([
			{ kind: 'delete', offset: 4, length: 4 },
			{ kind: 'delete', offset: 1, length: 2 },
		])
	})

	test('a pending run is rebased across an accepted remote run', () => {
		expect(
			transformMany(
				[{ kind: 'insert', offset: 3, text: 'Y' }],
				[
					{ kind: 'insert', offset: 0, text: 'xx' },
					{ kind: 'delete', offset: 5, length: 1 },
				],
				false,
			),
		).toEqual([{ kind: 'insert', offset: 5, text: 'Y' }])
	})
})
