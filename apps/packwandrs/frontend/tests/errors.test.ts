import { describe, expect, test } from 'bun:test'

import { normalizeBridgeError } from '../src/helpers/errors'

describe('normalizeBridgeError', () => {
	test('preserves serializable Rust errors', () => {
		expect(normalizeBridgeError({ kind: 'io', message: 'disk unavailable' })).toEqual({
			kind: 'io',
			message: 'disk unavailable',
		})
	})

	test('normalizes ordinary errors', () => {
		expect(normalizeBridgeError(new Error('broken'))).toEqual({
			kind: 'frontend',
			message: 'broken',
		})
	})
})
