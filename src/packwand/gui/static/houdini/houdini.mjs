import { makeError, toBitArray, bitArraySlice, stringBits } from "./gleam.mjs";
import { escape } from "./houdini.ffi.mjs";

export { escape };

const FILEPATH = "src\\houdini.gleam";

function unsafe_bit_array_to_string(_) {
  throw makeError(
    "panic",
    FILEPATH,
    "houdini",
    181,
    "unsafe_bit_array_to_string",
    "usafe_bit_array_to_string: this shouldn't be needed on the JavaScript target",
    {}
  )
}

function slice(_, _1, _2) {
  throw makeError(
    "panic",
    FILEPATH,
    "houdini",
    186,
    "slice",
    "slice: this shouldn't be needed on the JavaScript target",
    {}
  )
}

function do_escape_normal(
  loop$bin,
  loop$skip,
  loop$original,
  loop$acc,
  loop$len
) {
  while (true) {
    let bin = loop$bin;
    let skip = loop$skip;
    let original = loop$original;
    let acc = loop$acc;
    let len = loop$len;
    if (bin.bitSize >= 8) {
      if (bin.byteAt(0) === 60) {
        let rest = bitArraySlice(bin, 8);
        let acc$1 = toBitArray([
          acc,
          slice(original, skip, len),
          stringBits("&lt;"),
        ]);
        return do_escape(rest, (skip + len) + 1, original, acc$1);
      } else if (bin.byteAt(0) === 62) {
        let rest = bitArraySlice(bin, 8);
        let acc$1 = toBitArray([
          acc,
          slice(original, skip, len),
          stringBits("&gt;"),
        ]);
        return do_escape(rest, (skip + len) + 1, original, acc$1);
      } else if (bin.byteAt(0) === 38) {
        let rest = bitArraySlice(bin, 8);
        let acc$1 = toBitArray([
          acc,
          slice(original, skip, len),
          stringBits("&amp;"),
        ]);
        return do_escape(rest, (skip + len) + 1, original, acc$1);
      } else if (bin.byteAt(0) === 34) {
        let rest = bitArraySlice(bin, 8);
        let acc$1 = toBitArray([
          acc,
          slice(original, skip, len),
          stringBits("&quot;"),
        ]);
        return do_escape(rest, (skip + len) + 1, original, acc$1);
      } else if (bin.byteAt(0) === 39) {
        let rest = bitArraySlice(bin, 8);
        let acc$1 = toBitArray([
          acc,
          slice(original, skip, len),
          stringBits("&#39;"),
        ]);
        return do_escape(rest, (skip + len) + 1, original, acc$1);
      } else if (
        bin.bitSize >= 16 &&
        bin.bitSize >= 24 &&
        bin.bitSize >= 32 &&
        bin.bitSize >= 40 &&
        bin.bitSize >= 48 &&
        bin.bitSize >= 56 &&
        bin.bitSize >= 64
      ) {
        let b = bin.byteAt(1);
        let c = bin.byteAt(2);
        let d = bin.byteAt(3);
        let e = bin.byteAt(4);
        let f = bin.byteAt(5);
        let g = bin.byteAt(6);
        let h = bin.byteAt(7);
        if (
          ((((((((((b !== 34) && (b !== 38)) && (b !== 39)) && (b !== 60)) && (b !== 62)) && (((((c !== 34) && (c !== 38)) && (c !== 39)) && (c !== 60)) && (c !== 62))) && (((((d !== 34) && (d !== 38)) && (d !== 39)) && (d !== 60)) && (d !== 62))) && (((((e !== 34) && (e !== 38)) && (e !== 39)) && (e !== 60)) && (e !== 62))) && (((((f !== 34) && (f !== 38)) && (f !== 39)) && (f !== 60)) && (f !== 62))) && (((((g !== 34) && (g !== 38)) && (g !== 39)) && (g !== 60)) && (g !== 62))) && (((((h !== 34) && (h !== 38)) && (h !== 39)) && (h !== 60)) && (h !== 62))
        ) {
          let rest = bitArraySlice(bin, 64);
          loop$bin = rest;
          loop$skip = skip;
          loop$original = original;
          loop$acc = acc;
          loop$len = len + 8;
        } else {
          let rest = bitArraySlice(bin, 8);
          loop$bin = rest;
          loop$skip = skip;
          loop$original = original;
          loop$acc = acc;
          loop$len = len + 1;
        }
      } else {
        let rest = bitArraySlice(bin, 8);
        loop$bin = rest;
        loop$skip = skip;
        loop$original = original;
        loop$acc = acc;
        loop$len = len + 1;
      }
    } else if (bin.bitSize === 0) {
      if (skip === 0) {
        return original;
      } else {
        return toBitArray([acc, slice(original, skip, len)]);
      }
    } else {
      throw makeError(
        "panic",
        FILEPATH,
        "houdini",
        175,
        "do_escape_normal",
        "do_escape_normal: non byte aligned string, all strings should be byte aligned",
        {}
      )
    }
  }
}

function do_escape(loop$bin, loop$skip, loop$original, loop$acc) {
  while (true) {
    let bin = loop$bin;
    let skip = loop$skip;
    let original = loop$original;
    let acc = loop$acc;
    if (bin.bitSize >= 8) {
      if (bin.byteAt(0) === 60) {
        let rest = bitArraySlice(bin, 8);
        let acc$1 = toBitArray([acc, stringBits("&lt;")]);
        loop$bin = rest;
        loop$skip = skip + 1;
        loop$original = original;
        loop$acc = acc$1;
      } else if (bin.byteAt(0) === 62) {
        let rest = bitArraySlice(bin, 8);
        let acc$1 = toBitArray([acc, stringBits("&gt;")]);
        loop$bin = rest;
        loop$skip = skip + 1;
        loop$original = original;
        loop$acc = acc$1;
      } else if (bin.byteAt(0) === 38) {
        let rest = bitArraySlice(bin, 8);
        let acc$1 = toBitArray([acc, stringBits("&amp;")]);
        loop$bin = rest;
        loop$skip = skip + 1;
        loop$original = original;
        loop$acc = acc$1;
      } else if (bin.byteAt(0) === 34) {
        let rest = bitArraySlice(bin, 8);
        let acc$1 = toBitArray([acc, stringBits("&quot;")]);
        loop$bin = rest;
        loop$skip = skip + 1;
        loop$original = original;
        loop$acc = acc$1;
      } else if (bin.byteAt(0) === 39) {
        let rest = bitArraySlice(bin, 8);
        let acc$1 = toBitArray([acc, stringBits("&#39;")]);
        loop$bin = rest;
        loop$skip = skip + 1;
        loop$original = original;
        loop$acc = acc$1;
      } else {
        let rest = bitArraySlice(bin, 8);
        return do_escape_normal(rest, skip, original, acc, 1);
      }
    } else if (bin.bitSize === 0) {
      return acc;
    } else {
      throw makeError(
        "panic",
        FILEPATH,
        "houdini",
        79,
        "do_escape",
        "do_escape: non byte aligned string, all strings should be byte aligned",
        {}
      )
    }
  }
}
