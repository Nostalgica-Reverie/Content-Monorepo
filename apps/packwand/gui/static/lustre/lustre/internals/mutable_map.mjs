import {
  empty as new$,
  get as unsafe_get,
  get_or_compute,
  has_key,
  insert,
  remove as delete$,
  size,
} from "./mutable_map.ffi.mjs";

export { delete$, get_or_compute, has_key, insert, new$, size, unsafe_get };

/**
 *
 */
export function is_empty(map) {
  return size(map) === 0;
}
