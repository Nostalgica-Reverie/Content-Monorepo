import * as $int from "../../../gleam_stdlib/gleam/int.mjs";
import * as $string from "../../../gleam_stdlib/gleam/string.mjs";
import { Empty as $Empty, prepend as listPrepend, CustomType as $CustomType } from "../../gleam.mjs";
import * as $constants from "../../lustre/internals/constants.mjs";

class Root extends $CustomType {}

class Key extends $CustomType {
  constructor(key, parent) {
    super();
    this.key = key;
    this.parent = parent;
  }
}

class Index extends $CustomType {
  constructor(index, parent) {
    super();
    this.index = index;
    this.parent = parent;
  }
}

class Subtree extends $CustomType {
  constructor(parent) {
    super();
    this.parent = parent;
  }
}

/**
 *
 */
export const separator_subtree = "\r";

/**
 *
 */
export const separator_element = "\t";

/**
 *
 */
export const separator_event = "\n";

/**
 *
 */
export const root = /* @__PURE__ */ new Root();

function finish_to_string(acc) {
  if (acc instanceof $Empty) {
    return "";
  } else {
    let segments = acc.tail;
    return $string.concat(segments);
  }
}

function do_to_string(loop$full, loop$path, loop$acc) {
  while (true) {
    let full = loop$full;
    let path = loop$path;
    let acc = loop$acc;
    if (path instanceof Root) {
      return finish_to_string(acc);
    } else if (path instanceof Key) {
      let key = path.key;
      let parent = path.parent;
      loop$full = full;
      loop$path = parent;
      loop$acc = listPrepend(separator_element, listPrepend(key, acc));
    } else if (path instanceof Index) {
      let index = path.index;
      let parent = path.parent;
      let acc$1 = listPrepend(
        separator_element,
        listPrepend($int.to_string(index), acc),
      );
      loop$full = full;
      loop$path = parent;
      loop$acc = acc$1;
    } else if (!full) {
      return finish_to_string(acc);
    } else {
      let parent = path.parent;
      if (acc instanceof $Empty) {
        loop$full = full;
        loop$path = parent;
        loop$acc = acc;
      } else {
        let acc$1 = acc.tail;
        loop$full = full;
        loop$path = parent;
        loop$acc = listPrepend(separator_subtree, acc$1);
      }
    }
  }
}

/**
 * Convert a path to a full resolved string, including all memo barriers.
 */
export function to_string(path) {
  return do_to_string(true, path, $constants.empty_list);
}

function do_matches(loop$path, loop$candidates) {
  while (true) {
    let path = loop$path;
    let candidates = loop$candidates;
    if (candidates instanceof $Empty) {
      return false;
    } else {
      let candidate = candidates.head;
      let rest = candidates.tail;
      let $ = $string.starts_with(path, candidate);
      if ($) {
        return $;
      } else {
        loop$path = path;
        loop$candidates = rest;
      }
    }
  }
}

/**
 *
 */
export function matches(path, candidates) {
  if (candidates instanceof $Empty) {
    return false;
  } else {
    return do_matches(to_string(path), candidates);
  }
}

export function split_subtree_path(path) {
  return $string.split(path, separator_subtree);
}

/**
 *
 */
export function add(parent, index, key) {
  if (key === "") {
    return new Index(index, parent);
  } else {
    return new Key(key, parent);
  }
}

export function subtree(path) {
  return new Subtree(path);
}

/**
 * Convert a path to a resolved string with an event name appended to it.
 * This returns a partial path, up to the closest Memo barrier.
 */
export function event(path, event) {
  return do_to_string(
    false,
    path,
    listPrepend(separator_event, listPrepend(event, $constants.empty_list)),
  );
}

/**
 * Convert a path to a child tree to a resolved string.
 */
export function child(path) {
  return do_to_string(false, path, $constants.empty_list);
}
