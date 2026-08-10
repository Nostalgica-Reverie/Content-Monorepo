import * as $int from '../gleam_stdlib/gleam/int.mjs'
import * as $list from '../gleam_stdlib/gleam/list.mjs'
import * as $order from '../gleam_stdlib/gleam/order.mjs'
import * as $result from '../gleam_stdlib/gleam/result.mjs'
import * as $string from '../gleam_stdlib/gleam/string.mjs'
import { toList, Empty as $Empty } from './gleam.mjs'

function parse(version) {
	let _pipe = version
	let _pipe$1 = $string.split(_pipe, '.')
	return $list.map(_pipe$1, (part) => {
		let _pipe$2 = part
		let _pipe$3 = $int.parse(_pipe$2)
		return $result.unwrap(_pipe$3, 0)
	})
}

function do_compare(loop$a, loop$b) {
	while (true) {
		let a = loop$a
		let b = loop$b
		if (a instanceof $Empty) {
			if (b instanceof $Empty) {
				return 0
			} else {
				let y = b.head
				let rest = b.tail
				if (y === 0) {
					loop$a = toList([])
					loop$b = rest
				} else {
					return -1
				}
			}
		} else if (b instanceof $Empty) {
			let x = a.head
			let rest = a.tail
			if (x === 0) {
				loop$a = rest
				loop$b = toList([])
			} else {
				return 1
			}
		} else {
			let x = a.head
			let rest_a = a.tail
			let y = b.head
			let rest_b = b.tail
			let $ = $int.compare(x, y)
			if ($ instanceof $order.Lt) {
				return -1
			} else if ($ instanceof $order.Eq) {
				loop$a = rest_a
				loop$b = rest_b
			} else {
				return 1
			}
		}
	}
}

/**
 * Compares two dotted version strings numerically, the way Minecraft
 * versions sort (so "1.20.1" is newer than "1.9"). Returns a human verdict.
 */
export function compare_versions(a, b) {
	let $ = do_compare(parse(a), parse(b))
	if ($ === 0) {
		return a + ' and ' + b + ' are the same version'
	} else {
		let x = $
		if (x < 0) {
			return a + ' is older than ' + b
		} else {
			return a + ' is newer than ' + b
		}
	}
}

function trim_dashes(s) {
	let _block
	let $ = $string.starts_with(s, '-')
	if ($) {
		_block = $string.drop_start(s, 1)
	} else {
		_block = s
	}
	let s$1 = _block
	let $1 = $string.ends_with(s$1, '-')
	if ($1) {
		return $string.drop_end(s$1, 1)
	} else {
		return s$1
	}
}

function collapse_dashes(loop$s) {
	while (true) {
		let s = loop$s
		let $ = $string.contains(s, '--')
		if ($) {
			loop$s = $string.replace(s, '--', '-')
		} else {
			return s
		}
	}
}

/**
 * Normalises a display name into a pack slug, mirroring packwand's rules.
 */
export function slugify(name) {
	let _pipe = name
	let _pipe$1 = $string.lowercase(_pipe)
	let _pipe$2 = $string.to_graphemes(_pipe$1)
	let _pipe$3 = $list.map(_pipe$2, (g) => {
		let $ = $string.contains('abcdefghijklmnopqrstuvwxyz0123456789', g)
		if ($) {
			return g
		} else {
			return '-'
		}
	})
	let _pipe$4 = $string.join(_pipe$3, '')
	let _pipe$5 = collapse_dashes(_pipe$4)
	return trim_dashes(_pipe$5)
}
