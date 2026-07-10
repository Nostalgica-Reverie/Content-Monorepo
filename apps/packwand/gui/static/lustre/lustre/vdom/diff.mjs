import * as $json from "../../../gleam_json/gleam/json.mjs";
import * as $order from "../../../gleam_stdlib/gleam/order.mjs";
import { Eq, Gt, Lt } from "../../../gleam_stdlib/gleam/order.mjs";
import { Empty as $Empty, prepend as listPrepend, CustomType as $CustomType } from "../../gleam.mjs";
import * as $constants from "../../lustre/internals/constants.mjs";
import * as $mutable_map from "../../lustre/internals/mutable_map.mjs";
import * as $ref from "../../lustre/internals/ref.mjs";
import * as $cache from "../../lustre/vdom/cache.mjs";
import * as $patch from "../../lustre/vdom/patch.mjs";
import { Patch } from "../../lustre/vdom/patch.mjs";
import * as $path from "../../lustre/vdom/path.mjs";
import * as $vattr from "../../lustre/vdom/vattr.mjs";
import { Attribute, Event, Property } from "../../lustre/vdom/vattr.mjs";
import * as $vnode from "../../lustre/vdom/vnode.mjs";
import { Element, Fragment, Map, Memo, Text, UnsafeInnerHtml } from "../../lustre/vdom/vnode.mjs";
import { isEqual as property_value_equal } from "../internals/equals.ffi.mjs";
import { is_browser } from "../runtime/client/runtime.ffi.mjs";

export class Diff extends $CustomType {
  constructor(patch, cache) {
    super();
    this.patch = patch;
    this.cache = cache;
  }
}
export const Diff$Diff = (patch, cache) => new Diff(patch, cache);
export const Diff$isDiff = (value) => value instanceof Diff;
export const Diff$Diff$patch = (value) => value.patch;
export const Diff$Diff$0 = (value) => value.patch;
export const Diff$Diff$cache = (value) => value.cache;
export const Diff$Diff$1 = (value) => value.cache;

class PartialDiff extends $CustomType {
  constructor(patch, cache, events) {
    super();
    this.patch = patch;
    this.cache = cache;
    this.events = events;
  }
}

class AttributeChange extends $CustomType {
  constructor(added, removed, events) {
    super();
    this.added = added;
    this.removed = removed;
    this.events = events;
  }
}

function diff_attributes(
  loop$controlled,
  loop$path,
  loop$events,
  loop$old,
  loop$new,
  loop$added,
  loop$removed
) {
  while (true) {
    let controlled = loop$controlled;
    let path = loop$path;
    let events = loop$events;
    let old = loop$old;
    let new$ = loop$new;
    let added = loop$added;
    let removed = loop$removed;
    if (old instanceof $Empty) {
      if (new$ instanceof $Empty) {
        return new AttributeChange(added, removed, events);
      } else {
        let $ = new$.head;
        if ($ instanceof Event) {
          let next = $;
          let new$1 = new$.tail;
          let name = $.name;
          let handler = $.handler;
          let events$1 = $cache.add_event(events, path, name, handler);
          let added$1 = listPrepend(next, added);
          loop$controlled = controlled;
          loop$path = path;
          loop$events = events$1;
          loop$old = old;
          loop$new = new$1;
          loop$added = added$1;
          loop$removed = removed;
        } else {
          let next = $;
          let new$1 = new$.tail;
          let added$1 = listPrepend(next, added);
          loop$controlled = controlled;
          loop$path = path;
          loop$events = events;
          loop$old = old;
          loop$new = new$1;
          loop$added = added$1;
          loop$removed = removed;
        }
      }
    } else if (new$ instanceof $Empty) {
      let $ = old.head;
      if ($ instanceof Event) {
        let prev = $;
        let old$1 = old.tail;
        let name = $.name;
        let events$1 = $cache.remove_event(events, path, name);
        let removed$1 = listPrepend(prev, removed);
        loop$controlled = controlled;
        loop$path = path;
        loop$events = events$1;
        loop$old = old$1;
        loop$new = new$;
        loop$added = added;
        loop$removed = removed$1;
      } else {
        let prev = $;
        let old$1 = old.tail;
        let removed$1 = listPrepend(prev, removed);
        loop$controlled = controlled;
        loop$path = path;
        loop$events = events;
        loop$old = old$1;
        loop$new = new$;
        loop$added = added;
        loop$removed = removed$1;
      }
    } else {
      let prev = old.head;
      let remaining_old = old.tail;
      let next = new$.head;
      let remaining_new = new$.tail;
      let $ = $vattr.compare(prev, next);
      if ($ instanceof Lt) {
        if (prev instanceof Event) {
          let name = prev.name;
          loop$controlled = controlled;
          loop$path = path;
          loop$events = $cache.remove_event(events, path, name);
          loop$old = remaining_old;
          loop$new = new$;
          loop$added = added;
          loop$removed = listPrepend(prev, removed);
        } else {
          loop$controlled = controlled;
          loop$path = path;
          loop$events = events;
          loop$old = remaining_old;
          loop$new = new$;
          loop$added = added;
          loop$removed = listPrepend(prev, removed);
        }
      } else if ($ instanceof Eq) {
        if (prev instanceof Attribute) {
          if (next instanceof Attribute) {
            let _block;
            let $1 = next.name;
            if ($1 === "value") {
              _block = controlled || (prev.value !== next.value);
            } else if ($1 === "checked") {
              _block = controlled || (prev.value !== next.value);
            } else if ($1 === "selected") {
              _block = controlled || (prev.value !== next.value);
            } else {
              _block = prev.value !== next.value;
            }
            let has_changes = _block;
            let _block$1;
            if (has_changes) {
              _block$1 = listPrepend(next, added);
            } else {
              _block$1 = added;
            }
            let added$1 = _block$1;
            loop$controlled = controlled;
            loop$path = path;
            loop$events = events;
            loop$old = remaining_old;
            loop$new = remaining_new;
            loop$added = added$1;
            loop$removed = removed;
          } else if (next instanceof Event) {
            let name = next.name;
            let handler = next.handler;
            loop$controlled = controlled;
            loop$path = path;
            loop$events = $cache.add_event(events, path, name, handler);
            loop$old = remaining_old;
            loop$new = remaining_new;
            loop$added = listPrepend(next, added);
            loop$removed = listPrepend(prev, removed);
          } else {
            loop$controlled = controlled;
            loop$path = path;
            loop$events = events;
            loop$old = remaining_old;
            loop$new = remaining_new;
            loop$added = listPrepend(next, added);
            loop$removed = listPrepend(prev, removed);
          }
        } else if (prev instanceof Property) {
          if (next instanceof Property) {
            let _block;
            let $1 = next.name;
            if ($1 === "scrollLeft") {
              _block = true;
            } else if ($1 === "scrollRight") {
              _block = true;
            } else if ($1 === "value") {
              _block = controlled || !property_value_equal(
                prev.value,
                next.value,
              );
            } else if ($1 === "checked") {
              _block = controlled || !property_value_equal(
                prev.value,
                next.value,
              );
            } else if ($1 === "selected") {
              _block = controlled || !property_value_equal(
                prev.value,
                next.value,
              );
            } else {
              _block = !property_value_equal(prev.value, next.value);
            }
            let has_changes = _block;
            let _block$1;
            if (has_changes) {
              _block$1 = listPrepend(next, added);
            } else {
              _block$1 = added;
            }
            let added$1 = _block$1;
            loop$controlled = controlled;
            loop$path = path;
            loop$events = events;
            loop$old = remaining_old;
            loop$new = remaining_new;
            loop$added = added$1;
            loop$removed = removed;
          } else if (next instanceof Event) {
            let name = next.name;
            let handler = next.handler;
            loop$controlled = controlled;
            loop$path = path;
            loop$events = $cache.add_event(events, path, name, handler);
            loop$old = remaining_old;
            loop$new = remaining_new;
            loop$added = listPrepend(next, added);
            loop$removed = listPrepend(prev, removed);
          } else {
            loop$controlled = controlled;
            loop$path = path;
            loop$events = events;
            loop$old = remaining_old;
            loop$new = remaining_new;
            loop$added = listPrepend(next, added);
            loop$removed = listPrepend(prev, removed);
          }
        } else if (next instanceof Event) {
          let name = next.name;
          let handler = next.handler;
          let has_changes = (((prev.prevent_default.kind !== next.prevent_default.kind) || (prev.stop_propagation.kind !== next.stop_propagation.kind)) || (prev.debounce !== next.debounce)) || (prev.throttle !== next.throttle);
          let _block;
          if (has_changes) {
            _block = listPrepend(next, added);
          } else {
            _block = added;
          }
          let added$1 = _block;
          loop$controlled = controlled;
          loop$path = path;
          loop$events = $cache.add_event(events, path, name, handler);
          loop$old = remaining_old;
          loop$new = remaining_new;
          loop$added = added$1;
          loop$removed = removed;
        } else {
          let name = prev.name;
          loop$controlled = controlled;
          loop$path = path;
          loop$events = $cache.remove_event(events, path, name);
          loop$old = remaining_old;
          loop$new = remaining_new;
          loop$added = listPrepend(next, added);
          loop$removed = listPrepend(prev, removed);
        }
      } else if (next instanceof Event) {
        let name = next.name;
        let handler = next.handler;
        loop$controlled = controlled;
        loop$path = path;
        loop$events = $cache.add_event(events, path, name, handler);
        loop$old = old;
        loop$new = remaining_new;
        loop$added = listPrepend(next, added);
        loop$removed = removed;
      } else {
        loop$controlled = controlled;
        loop$path = path;
        loop$events = events;
        loop$old = old;
        loop$new = remaining_new;
        loop$added = listPrepend(next, added);
        loop$removed = removed;
      }
    }
  }
}

function is_controlled(cache, namespace, tag, path) {
  if (tag === "input" && namespace === "") {
    return $cache.has_dispatched_events(cache, path);
  } else if (tag === "select" && namespace === "") {
    return $cache.has_dispatched_events(cache, path);
  } else if (tag === "textarea" && namespace === "") {
    return $cache.has_dispatched_events(cache, path);
  } else {
    return false;
  }
}

function do_diff(
  loop$old,
  loop$old_keyed,
  loop$new,
  loop$new_keyed,
  loop$moved,
  loop$moved_offset,
  loop$removed,
  loop$node_index,
  loop$patch_index,
  loop$changes,
  loop$children,
  loop$path,
  loop$cache,
  loop$events
) {
  while (true) {
    let old = loop$old;
    let old_keyed = loop$old_keyed;
    let new$ = loop$new;
    let new_keyed = loop$new_keyed;
    let moved = loop$moved;
    let moved_offset = loop$moved_offset;
    let removed = loop$removed;
    let node_index = loop$node_index;
    let patch_index = loop$patch_index;
    let changes = loop$changes;
    let children = loop$children;
    let path = loop$path;
    let cache = loop$cache;
    let events = loop$events;
    if (old instanceof $Empty) {
      if (new$ instanceof $Empty) {
        let _block;
        let $ = is_browser();
        if (changes instanceof $Empty) {
          if (children instanceof $Empty) {
            _block = $patch.new$(patch_index, removed, changes, children);
          } else if (!$) {
            let $1 = children.tail;
            if ($1 instanceof $Empty && removed === 0) {
              let child = children.head;
              _block = $patch.add_parent(child, patch_index);
            } else {
              _block = $patch.new$(patch_index, removed, changes, children);
            }
          } else {
            _block = $patch.new$(patch_index, removed, changes, children);
          }
        } else {
          _block = $patch.new$(patch_index, removed, changes, children);
        }
        let patch = _block;
        return new PartialDiff(patch, cache, events);
      } else {
        let $ = $cache.add_children(cache, events, path, node_index, new$);
        let cache$1 = $[0];
        let events$1 = $[1];
        let insert = $patch.insert(new$, node_index - moved_offset);
        let changes$1 = listPrepend(insert, changes);
        let patch = $patch.new$(patch_index, removed, changes$1, children);
        return new PartialDiff(patch, cache$1, events$1);
      }
    } else if (new$ instanceof $Empty) {
      let prev = old.head;
      let old$1 = old.tail;
      let $ = (prev.key === "") || !$mutable_map.has_key(moved, prev.key);
      if ($) {
        let events$1 = $cache.remove_child(
          cache,
          events,
          path,
          node_index,
          prev,
        );
        loop$old = old$1;
        loop$old_keyed = old_keyed;
        loop$new = new$;
        loop$new_keyed = new_keyed;
        loop$moved = moved;
        loop$moved_offset = moved_offset;
        loop$removed = removed + 1;
        loop$node_index = node_index;
        loop$patch_index = patch_index;
        loop$changes = changes;
        loop$children = children;
        loop$path = path;
        loop$cache = cache;
        loop$events = events$1;
      } else {
        loop$old = old$1;
        loop$old_keyed = old_keyed;
        loop$new = new$;
        loop$new_keyed = new_keyed;
        loop$moved = moved;
        loop$moved_offset = moved_offset;
        loop$removed = removed;
        loop$node_index = node_index;
        loop$patch_index = patch_index;
        loop$changes = changes;
        loop$children = children;
        loop$path = path;
        loop$cache = cache;
        loop$events = events;
      }
    } else {
      let prev = old.head;
      let next = new$.head;
      if (prev.key !== next.key) {
        let old_remaining = old.tail;
        let new_remaining = new$.tail;
        let next_did_exist = $mutable_map.has_key(old_keyed, next.key);
        let prev_does_exist = $mutable_map.has_key(new_keyed, prev.key);
        if (prev_does_exist) {
          if (next_did_exist) {
            let $ = $mutable_map.has_key(moved, prev.key);
            if ($) {
              loop$old = old_remaining;
              loop$old_keyed = old_keyed;
              loop$new = new$;
              loop$new_keyed = new_keyed;
              loop$moved = moved;
              loop$moved_offset = moved_offset - 1;
              loop$removed = removed;
              loop$node_index = node_index;
              loop$patch_index = patch_index;
              loop$changes = changes;
              loop$children = children;
              loop$path = path;
              loop$cache = cache;
              loop$events = events;
            } else {
              let match = $mutable_map.unsafe_get(old_keyed, next.key);
              let before = node_index - moved_offset;
              let changes$1 = listPrepend(
                $patch.move(next.key, before),
                changes,
              );
              let moved$1 = $mutable_map.insert(moved, next.key, undefined);
              loop$old = listPrepend(match, old);
              loop$old_keyed = old_keyed;
              loop$new = new$;
              loop$new_keyed = new_keyed;
              loop$moved = moved$1;
              loop$moved_offset = moved_offset + 1;
              loop$removed = removed;
              loop$node_index = node_index;
              loop$patch_index = patch_index;
              loop$changes = changes$1;
              loop$children = children;
              loop$path = path;
              loop$cache = cache;
              loop$events = events;
            }
          } else {
            let before = node_index - moved_offset;
            let $ = $cache.add_child(cache, events, path, node_index, next);
            let cache$1 = $[0];
            let events$1 = $[1];
            let insert = $patch.insert($constants.singleton_list(next), before);
            let changes$1 = listPrepend(insert, changes);
            loop$old = old;
            loop$old_keyed = old_keyed;
            loop$new = new_remaining;
            loop$new_keyed = new_keyed;
            loop$moved = moved;
            loop$moved_offset = moved_offset + 1;
            loop$removed = removed;
            loop$node_index = node_index + 1;
            loop$patch_index = patch_index;
            loop$changes = changes$1;
            loop$children = children;
            loop$path = path;
            loop$cache = cache$1;
            loop$events = events$1;
          }
        } else if (next_did_exist) {
          let index = node_index - moved_offset;
          let changes$1 = listPrepend($patch.remove(index), changes);
          let events$1 = $cache.remove_child(
            cache,
            events,
            path,
            node_index,
            prev,
          );
          loop$old = old_remaining;
          loop$old_keyed = old_keyed;
          loop$new = new$;
          loop$new_keyed = new_keyed;
          loop$moved = moved;
          loop$moved_offset = moved_offset - 1;
          loop$removed = removed;
          loop$node_index = node_index;
          loop$patch_index = patch_index;
          loop$changes = changes$1;
          loop$children = children;
          loop$path = path;
          loop$cache = cache;
          loop$events = events$1;
        } else {
          let change = $patch.replace(node_index - moved_offset, next);
          let $ = $cache.replace_child(
            cache,
            events,
            path,
            node_index,
            prev,
            next,
          );
          let cache$1 = $[0];
          let events$1 = $[1];
          loop$old = old_remaining;
          loop$old_keyed = old_keyed;
          loop$new = new_remaining;
          loop$new_keyed = new_keyed;
          loop$moved = moved;
          loop$moved_offset = moved_offset;
          loop$removed = removed;
          loop$node_index = node_index + 1;
          loop$patch_index = patch_index;
          loop$changes = listPrepend(change, changes);
          loop$children = children;
          loop$path = path;
          loop$cache = cache$1;
          loop$events = events$1;
        }
      } else {
        let $ = old.head;
        if ($ instanceof Fragment) {
          let $1 = new$.head;
          if ($1 instanceof Fragment) {
            let prev = $;
            let old$1 = old.tail;
            let next = $1;
            let new$1 = new$.tail;
            let $2 = do_diff(
              prev.children,
              prev.keyed_children,
              next.children,
              next.keyed_children,
              $mutable_map.new$(),
              0,
              0,
              0,
              node_index,
              $constants.empty_list,
              $constants.empty_list,
              $path.add(path, node_index, next.key),
              cache,
              events,
            );
            let patch = $2.patch;
            let cache$1 = $2.cache;
            let events$1 = $2.events;
            let _block;
            let $3 = patch.changes;
            if ($3 instanceof $Empty) {
              let $4 = patch.children;
              if ($4 instanceof $Empty) {
                let $5 = patch.removed;
                if ($5 === 0) {
                  _block = children;
                } else {
                  _block = listPrepend(patch, children);
                }
              } else {
                _block = listPrepend(patch, children);
              }
            } else {
              _block = listPrepend(patch, children);
            }
            let children$1 = _block;
            loop$old = old$1;
            loop$old_keyed = old_keyed;
            loop$new = new$1;
            loop$new_keyed = new_keyed;
            loop$moved = moved;
            loop$moved_offset = moved_offset;
            loop$removed = removed;
            loop$node_index = node_index + 1;
            loop$patch_index = patch_index;
            loop$changes = changes;
            loop$children = children$1;
            loop$path = path;
            loop$cache = cache$1;
            loop$events = events$1;
          } else {
            let prev = $;
            let old_remaining = old.tail;
            let next = $1;
            let new_remaining = new$.tail;
            let change = $patch.replace(node_index - moved_offset, next);
            let $2 = $cache.replace_child(
              cache,
              events,
              path,
              node_index,
              prev,
              next,
            );
            let cache$1 = $2[0];
            let events$1 = $2[1];
            loop$old = old_remaining;
            loop$old_keyed = old_keyed;
            loop$new = new_remaining;
            loop$new_keyed = new_keyed;
            loop$moved = moved;
            loop$moved_offset = moved_offset;
            loop$removed = removed;
            loop$node_index = node_index + 1;
            loop$patch_index = patch_index;
            loop$changes = listPrepend(change, changes);
            loop$children = children;
            loop$path = path;
            loop$cache = cache$1;
            loop$events = events$1;
          }
        } else if ($ instanceof Element) {
          let $1 = new$.head;
          if ($1 instanceof Element) {
            let prev = $;
            let next = $1;
            if ((prev.namespace === next.namespace) && (prev.tag === next.tag)) {
              let old$1 = old.tail;
              let new$1 = new$.tail;
              let child_path = $path.add(path, node_index, next.key);
              let controlled = is_controlled(
                cache,
                next.namespace,
                next.tag,
                child_path,
              );
              let $2 = diff_attributes(
                controlled,
                child_path,
                events,
                prev.attributes,
                next.attributes,
                $constants.empty_list,
                $constants.empty_list,
              );
              let added_attrs = $2.added;
              let removed_attrs = $2.removed;
              let events$1 = $2.events;
              let _block;
              if (
                added_attrs instanceof $Empty &&
                removed_attrs instanceof $Empty
              ) {
                _block = $constants.empty_list;
              } else {
                _block = $constants.singleton_list(
                  $patch.update(added_attrs, removed_attrs),
                );
              }
              let initial_child_changes = _block;
              let $3 = do_diff(
                prev.children,
                prev.keyed_children,
                next.children,
                next.keyed_children,
                $mutable_map.new$(),
                0,
                0,
                0,
                node_index,
                initial_child_changes,
                $constants.empty_list,
                child_path,
                cache,
                events$1,
              );
              let patch = $3.patch;
              let cache$1 = $3.cache;
              let events$2 = $3.events;
              let _block$1;
              let $4 = patch.changes;
              if ($4 instanceof $Empty) {
                let $5 = patch.children;
                if ($5 instanceof $Empty) {
                  let $6 = patch.removed;
                  if ($6 === 0) {
                    _block$1 = children;
                  } else {
                    _block$1 = listPrepend(patch, children);
                  }
                } else {
                  _block$1 = listPrepend(patch, children);
                }
              } else {
                _block$1 = listPrepend(patch, children);
              }
              let children$1 = _block$1;
              loop$old = old$1;
              loop$old_keyed = old_keyed;
              loop$new = new$1;
              loop$new_keyed = new_keyed;
              loop$moved = moved;
              loop$moved_offset = moved_offset;
              loop$removed = removed;
              loop$node_index = node_index + 1;
              loop$patch_index = patch_index;
              loop$changes = changes;
              loop$children = children$1;
              loop$path = path;
              loop$cache = cache$1;
              loop$events = events$2;
            } else {
              let prev = $;
              let old_remaining = old.tail;
              let next = $1;
              let new_remaining = new$.tail;
              let change = $patch.replace(node_index - moved_offset, next);
              let $2 = $cache.replace_child(
                cache,
                events,
                path,
                node_index,
                prev,
                next,
              );
              let cache$1 = $2[0];
              let events$1 = $2[1];
              loop$old = old_remaining;
              loop$old_keyed = old_keyed;
              loop$new = new_remaining;
              loop$new_keyed = new_keyed;
              loop$moved = moved;
              loop$moved_offset = moved_offset;
              loop$removed = removed;
              loop$node_index = node_index + 1;
              loop$patch_index = patch_index;
              loop$changes = listPrepend(change, changes);
              loop$children = children;
              loop$path = path;
              loop$cache = cache$1;
              loop$events = events$1;
            }
          } else {
            let prev = $;
            let old_remaining = old.tail;
            let next = $1;
            let new_remaining = new$.tail;
            let change = $patch.replace(node_index - moved_offset, next);
            let $2 = $cache.replace_child(
              cache,
              events,
              path,
              node_index,
              prev,
              next,
            );
            let cache$1 = $2[0];
            let events$1 = $2[1];
            loop$old = old_remaining;
            loop$old_keyed = old_keyed;
            loop$new = new_remaining;
            loop$new_keyed = new_keyed;
            loop$moved = moved;
            loop$moved_offset = moved_offset;
            loop$removed = removed;
            loop$node_index = node_index + 1;
            loop$patch_index = patch_index;
            loop$changes = listPrepend(change, changes);
            loop$children = children;
            loop$path = path;
            loop$cache = cache$1;
            loop$events = events$1;
          }
        } else if ($ instanceof Text) {
          let $1 = new$.head;
          if ($1 instanceof Text) {
            let prev = $;
            let next = $1;
            if (prev.content === next.content) {
              let old$1 = old.tail;
              let new$1 = new$.tail;
              loop$old = old$1;
              loop$old_keyed = old_keyed;
              loop$new = new$1;
              loop$new_keyed = new_keyed;
              loop$moved = moved;
              loop$moved_offset = moved_offset;
              loop$removed = removed;
              loop$node_index = node_index + 1;
              loop$patch_index = patch_index;
              loop$changes = changes;
              loop$children = children;
              loop$path = path;
              loop$cache = cache;
              loop$events = events;
            } else {
              let old$1 = old.tail;
              let next = $1;
              let new$1 = new$.tail;
              let child = $patch.new$(
                node_index,
                0,
                $constants.singleton_list($patch.replace_text(next.content)),
                $constants.empty_list,
              );
              loop$old = old$1;
              loop$old_keyed = old_keyed;
              loop$new = new$1;
              loop$new_keyed = new_keyed;
              loop$moved = moved;
              loop$moved_offset = moved_offset;
              loop$removed = removed;
              loop$node_index = node_index + 1;
              loop$patch_index = patch_index;
              loop$changes = changes;
              loop$children = listPrepend(child, children);
              loop$path = path;
              loop$cache = cache;
              loop$events = events;
            }
          } else {
            let prev = $;
            let old_remaining = old.tail;
            let next = $1;
            let new_remaining = new$.tail;
            let change = $patch.replace(node_index - moved_offset, next);
            let $2 = $cache.replace_child(
              cache,
              events,
              path,
              node_index,
              prev,
              next,
            );
            let cache$1 = $2[0];
            let events$1 = $2[1];
            loop$old = old_remaining;
            loop$old_keyed = old_keyed;
            loop$new = new_remaining;
            loop$new_keyed = new_keyed;
            loop$moved = moved;
            loop$moved_offset = moved_offset;
            loop$removed = removed;
            loop$node_index = node_index + 1;
            loop$patch_index = patch_index;
            loop$changes = listPrepend(change, changes);
            loop$children = children;
            loop$path = path;
            loop$cache = cache$1;
            loop$events = events$1;
          }
        } else if ($ instanceof UnsafeInnerHtml) {
          let $1 = new$.head;
          if ($1 instanceof UnsafeInnerHtml) {
            let prev = $;
            let old$1 = old.tail;
            let next = $1;
            let new$1 = new$.tail;
            let child_path = $path.add(path, node_index, next.key);
            let $2 = diff_attributes(
              false,
              child_path,
              events,
              prev.attributes,
              next.attributes,
              $constants.empty_list,
              $constants.empty_list,
            );
            let added_attrs = $2.added;
            let removed_attrs = $2.removed;
            let events$1 = $2.events;
            let _block;
            if (added_attrs instanceof $Empty && removed_attrs instanceof $Empty) {
              _block = $constants.empty_list;
            } else {
              _block = $constants.singleton_list(
                $patch.update(added_attrs, removed_attrs),
              );
            }
            let child_changes = _block;
            let _block$1;
            let $3 = prev.inner_html === next.inner_html;
            if ($3) {
              _block$1 = child_changes;
            } else {
              _block$1 = listPrepend(
                $patch.replace_inner_html(next.inner_html),
                child_changes,
              );
            }
            let child_changes$1 = _block$1;
            let _block$2;
            if (child_changes$1 instanceof $Empty) {
              _block$2 = children;
            } else {
              _block$2 = listPrepend(
                $patch.new$(
                  node_index,
                  0,
                  child_changes$1,
                  $constants.empty_list,
                ),
                children,
              );
            }
            let children$1 = _block$2;
            loop$old = old$1;
            loop$old_keyed = old_keyed;
            loop$new = new$1;
            loop$new_keyed = new_keyed;
            loop$moved = moved;
            loop$moved_offset = moved_offset;
            loop$removed = removed;
            loop$node_index = node_index + 1;
            loop$patch_index = patch_index;
            loop$changes = changes;
            loop$children = children$1;
            loop$path = path;
            loop$cache = cache;
            loop$events = events$1;
          } else {
            let prev = $;
            let old_remaining = old.tail;
            let next = $1;
            let new_remaining = new$.tail;
            let change = $patch.replace(node_index - moved_offset, next);
            let $2 = $cache.replace_child(
              cache,
              events,
              path,
              node_index,
              prev,
              next,
            );
            let cache$1 = $2[0];
            let events$1 = $2[1];
            loop$old = old_remaining;
            loop$old_keyed = old_keyed;
            loop$new = new_remaining;
            loop$new_keyed = new_keyed;
            loop$moved = moved;
            loop$moved_offset = moved_offset;
            loop$removed = removed;
            loop$node_index = node_index + 1;
            loop$patch_index = patch_index;
            loop$changes = listPrepend(change, changes);
            loop$children = children;
            loop$path = path;
            loop$cache = cache$1;
            loop$events = events$1;
          }
        } else if ($ instanceof Map) {
          let $1 = new$.head;
          if ($1 instanceof Map) {
            let prev = $;
            let old$1 = old.tail;
            let next = $1;
            let new$1 = new$.tail;
            let child_path = $path.add(path, node_index, next.key);
            let child_key = $path.child(child_path);
            let $2 = do_diff(
              $constants.singleton_list(prev.child),
              $mutable_map.new$(),
              $constants.singleton_list(next.child),
              $mutable_map.new$(),
              $mutable_map.new$(),
              0,
              0,
              0,
              node_index,
              $constants.empty_list,
              $constants.empty_list,
              $path.subtree(child_path),
              cache,
              $cache.get_subtree(events, child_key, prev.mapper),
            );
            let patch = $2.patch;
            let cache$1 = $2.cache;
            let child_events = $2.events;
            let events$1 = $cache.update_subtree(
              events,
              child_key,
              next.mapper,
              child_events,
            );
            let _block;
            let $3 = patch.changes;
            if ($3 instanceof $Empty) {
              let $4 = patch.children;
              if ($4 instanceof $Empty) {
                let $5 = patch.removed;
                if ($5 === 0) {
                  _block = children;
                } else {
                  _block = listPrepend(patch, children);
                }
              } else {
                _block = listPrepend(patch, children);
              }
            } else {
              _block = listPrepend(patch, children);
            }
            let children$1 = _block;
            loop$old = old$1;
            loop$old_keyed = old_keyed;
            loop$new = new$1;
            loop$new_keyed = new_keyed;
            loop$moved = moved;
            loop$moved_offset = moved_offset;
            loop$removed = removed;
            loop$node_index = node_index + 1;
            loop$patch_index = patch_index;
            loop$changes = changes;
            loop$children = children$1;
            loop$path = path;
            loop$cache = cache$1;
            loop$events = events$1;
          } else {
            let prev = $;
            let old_remaining = old.tail;
            let next = $1;
            let new_remaining = new$.tail;
            let change = $patch.replace(node_index - moved_offset, next);
            let $2 = $cache.replace_child(
              cache,
              events,
              path,
              node_index,
              prev,
              next,
            );
            let cache$1 = $2[0];
            let events$1 = $2[1];
            loop$old = old_remaining;
            loop$old_keyed = old_keyed;
            loop$new = new_remaining;
            loop$new_keyed = new_keyed;
            loop$moved = moved;
            loop$moved_offset = moved_offset;
            loop$removed = removed;
            loop$node_index = node_index + 1;
            loop$patch_index = patch_index;
            loop$changes = listPrepend(change, changes);
            loop$children = children;
            loop$path = path;
            loop$cache = cache$1;
            loop$events = events$1;
          }
        } else {
          let $1 = new$.head;
          if ($1 instanceof Memo) {
            let prev = $;
            let old$1 = old.tail;
            let next = $1;
            let new$1 = new$.tail;
            let $2 = $ref.equal_lists(prev.dependencies, next.dependencies);
            if ($2) {
              let cache$1 = $cache.keep_memo(cache, prev.view, next.view);
              loop$old = old$1;
              loop$old_keyed = old_keyed;
              loop$new = new$1;
              loop$new_keyed = new_keyed;
              loop$moved = moved;
              loop$moved_offset = moved_offset;
              loop$removed = removed;
              loop$node_index = node_index + 1;
              loop$patch_index = patch_index;
              loop$changes = changes;
              loop$children = children;
              loop$path = path;
              loop$cache = cache$1;
              loop$events = events;
            } else {
              let prev_node = $cache.get_old_memo(cache, prev.view, prev.view);
              let next_node = next.view();
              let cache$1 = $cache.add_memo(cache, next.view, next_node);
              loop$old = listPrepend(prev_node, old$1);
              loop$old_keyed = old_keyed;
              loop$new = listPrepend(next_node, new$1);
              loop$new_keyed = new_keyed;
              loop$moved = moved;
              loop$moved_offset = moved_offset;
              loop$removed = removed;
              loop$node_index = node_index;
              loop$patch_index = patch_index;
              loop$changes = changes;
              loop$children = children;
              loop$path = path;
              loop$cache = cache$1;
              loop$events = events;
            }
          } else {
            let prev = $;
            let old_remaining = old.tail;
            let next = $1;
            let new_remaining = new$.tail;
            let change = $patch.replace(node_index - moved_offset, next);
            let $2 = $cache.replace_child(
              cache,
              events,
              path,
              node_index,
              prev,
              next,
            );
            let cache$1 = $2[0];
            let events$1 = $2[1];
            loop$old = old_remaining;
            loop$old_keyed = old_keyed;
            loop$new = new_remaining;
            loop$new_keyed = new_keyed;
            loop$moved = moved;
            loop$moved_offset = moved_offset;
            loop$removed = removed;
            loop$node_index = node_index + 1;
            loop$patch_index = patch_index;
            loop$changes = listPrepend(change, changes);
            loop$children = children;
            loop$path = path;
            loop$cache = cache$1;
            loop$events = events$1;
          }
        }
      }
    }
  }
}

export function diff(cache, old, new$) {
  let cache$1 = $cache.tick(cache);
  let $ = do_diff(
    $constants.singleton_list(old),
    $mutable_map.new$(),
    $constants.singleton_list(new$),
    $mutable_map.new$(),
    $mutable_map.new$(),
    0,
    0,
    0,
    0,
    $constants.empty_list,
    $constants.empty_list,
    $path.root,
    cache$1,
    $cache.events(cache$1),
  );
  let patch = $.patch;
  let cache$2 = $.cache;
  let events = $.events;
  return new Diff(patch, $cache.update_events(cache$2, events));
}
