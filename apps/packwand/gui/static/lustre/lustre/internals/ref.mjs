import { identity as from } from "../../../gleam_stdlib/gleam/function.mjs";
import { Empty as $Empty } from "../../gleam.mjs";
import { sameValueZero as equal } from "./ref.ffi.mjs";

export { equal, from };

/**
 *
 */
export function equal_lists(loop$xs, loop$ys) {
  while (true) {
    let xs = loop$xs;
    let ys = loop$ys;
    if (xs instanceof $Empty) {
      if (ys instanceof $Empty) {
        return true;
      } else {
        return false;
      }
    } else if (ys instanceof $Empty) {
      return false;
    } else {
      let x = xs.head;
      let xs$1 = xs.tail;
      let y = ys.head;
      let ys$1 = ys.tail;
      let $ = equal(x, y);
      if ($) {
        loop$xs = xs$1;
        loop$ys = ys$1;
      } else {
        return $;
      }
    }
  }
}
