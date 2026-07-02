import * as $dynamic from "../../../gleam_stdlib/gleam/dynamic.mjs";
import * as $decode from "../../../gleam_stdlib/gleam/dynamic/decode.mjs";
import * as $function from "../../../gleam_stdlib/gleam/function.mjs";
import { identity as coerce } from "../../../gleam_stdlib/gleam/function.mjs";
import * as $list from "../../../gleam_stdlib/gleam/list.mjs";
import { Ok, Empty as $Empty, prepend as listPrepend, CustomType as $CustomType } from "../../gleam.mjs";
import * as $constants from "../../lustre/internals/constants.mjs";
import * as $mutable_map from "../../lustre/internals/mutable_map.mjs";
import * as $path from "../../lustre/vdom/path.mjs";
import * as $vattr from "../../lustre/vdom/vattr.mjs";
import { Event, Handler } from "../../lustre/vdom/vattr.mjs";
import * as $vnode from "../../lustre/vdom/vnode.mjs";
import { Element, Fragment, Map, Memo, Text, UnsafeInnerHtml } from "../../lustre/vdom/vnode.mjs";

class Cache extends $CustomType {
  constructor(events, vdoms, old_vdoms, dispatched_paths, next_dispatched_paths) {
    super();
    this.events = events;
    this.vdoms = vdoms;
    this.old_vdoms = old_vdoms;
    this.dispatched_paths = dispatched_paths;
    this.next_dispatched_paths = next_dispatched_paths;
  }
}

class Events extends $CustomType {
  constructor(handlers, children) {
    super();
    this.handlers = handlers;
    this.children = children;
  }
}

class Child extends $CustomType {
  constructor(mapper, events) {
    super();
    this.mapper = mapper;
    this.events = events;
  }
}

class AddedChildren extends $CustomType {
  constructor(handlers, children, vdoms) {
    super();
    this.handlers = handlers;
    this.children = children;
    this.vdoms = vdoms;
  }
}

class DecodedEvent extends $CustomType {
  constructor(path, handler) {
    super();
    this.path = path;
    this.handler = handler;
  }
}

class DispatchedEvent extends $CustomType {
  constructor(path) {
    super();
    this.path = path;
  }
}

export function compose_mapper(mapper, child_mapper) {
  return (message) => { return mapper(child_mapper(message)); };
}

export function new_events() {
  return new Events($mutable_map.new$(), $mutable_map.new$());
}

/**
 *
 */
export function new$() {
  return new Cache(
    new_events(),
    $mutable_map.new$(),
    $mutable_map.new$(),
    $constants.empty_list,
    $constants.empty_list,
  );
}

function do_add_event(handlers, path, name, handler) {
  return $mutable_map.insert(handlers, $path.event(path, name), handler);
}

function add_attributes(handlers, path, attributes) {
  return $list.fold(
    attributes,
    handlers,
    (events, attribute) => {
      if (attribute instanceof Event) {
        let name = attribute.name;
        let handler = attribute.handler;
        return do_add_event(events, path, name, handler);
      } else {
        return events;
      }
    },
  );
}

function do_add_children(
  loop$handlers,
  loop$children,
  loop$vdoms,
  loop$parent,
  loop$child_index,
  loop$nodes
) {
  while (true) {
    let handlers = loop$handlers;
    let children = loop$children;
    let vdoms = loop$vdoms;
    let parent = loop$parent;
    let child_index = loop$child_index;
    let nodes = loop$nodes;
    let next = child_index + 1;
    if (nodes instanceof $Empty) {
      return new AddedChildren(handlers, children, vdoms);
    } else {
      let $ = nodes.head;
      if ($ instanceof Fragment) {
        let rest = nodes.tail;
        let key = $.key;
        let nodes$1 = $.children;
        let path = $path.add(parent, child_index, key);
        let $1 = do_add_children(handlers, children, vdoms, path, 0, nodes$1);
        let handlers$1 = $1.handlers;
        let children$1 = $1.children;
        let vdoms$1 = $1.vdoms;
        loop$handlers = handlers$1;
        loop$children = children$1;
        loop$vdoms = vdoms$1;
        loop$parent = parent;
        loop$child_index = next;
        loop$nodes = rest;
      } else if ($ instanceof Element) {
        let rest = nodes.tail;
        let key = $.key;
        let attributes = $.attributes;
        let nodes$1 = $.children;
        let path = $path.add(parent, child_index, key);
        let handlers$1 = add_attributes(handlers, path, attributes);
        let $1 = do_add_children(handlers$1, children, vdoms, path, 0, nodes$1);
        let handlers$2 = $1.handlers;
        let children$1 = $1.children;
        let vdoms$1 = $1.vdoms;
        loop$handlers = handlers$2;
        loop$children = children$1;
        loop$vdoms = vdoms$1;
        loop$parent = parent;
        loop$child_index = next;
        loop$nodes = rest;
      } else if ($ instanceof Text) {
        let rest = nodes.tail;
        loop$handlers = handlers;
        loop$children = children;
        loop$vdoms = vdoms;
        loop$parent = parent;
        loop$child_index = next;
        loop$nodes = rest;
      } else if ($ instanceof UnsafeInnerHtml) {
        let rest = nodes.tail;
        let key = $.key;
        let attributes = $.attributes;
        let path = $path.add(parent, child_index, key);
        let handlers$1 = add_attributes(handlers, path, attributes);
        loop$handlers = handlers$1;
        loop$children = children;
        loop$vdoms = vdoms;
        loop$parent = parent;
        loop$child_index = next;
        loop$nodes = rest;
      } else if ($ instanceof Map) {
        let rest = nodes.tail;
        let key = $.key;
        let mapper = $.mapper;
        let child = $.child;
        let path = $path.add(parent, child_index, key);
        let added = do_add_children(
          $mutable_map.new$(),
          $mutable_map.new$(),
          vdoms,
          $path.subtree(path),
          0,
          $constants.singleton_list(child),
        );
        let vdoms$1 = added.vdoms;
        let child_events = new Events(added.handlers, added.children);
        let child$1 = new Child(mapper, child_events);
        let children$1 = $mutable_map.insert(
          children,
          $path.child(path),
          child$1,
        );
        loop$handlers = handlers;
        loop$children = children$1;
        loop$vdoms = vdoms$1;
        loop$parent = parent;
        loop$child_index = next;
        loop$nodes = rest;
      } else {
        let rest = nodes.tail;
        let view = $.view;
        let child_node = view();
        let vdoms$1 = $mutable_map.insert(vdoms, view, child_node);
        let next$1 = child_index;
        let rest$1 = listPrepend(child_node, rest);
        loop$handlers = handlers;
        loop$children = children;
        loop$vdoms = vdoms$1;
        loop$parent = parent;
        loop$child_index = next$1;
        loop$nodes = rest$1;
      }
    }
  }
}

/**
 *
 */
export function add_children(cache, events, path, child_index, nodes) {
  let vdoms = cache.vdoms;
  let handlers = events.handlers;
  let children = events.children;
  let $ = do_add_children(handlers, children, vdoms, path, child_index, nodes);
  let handlers$1 = $.handlers;
  let children$1 = $.children;
  let vdoms$1 = $.vdoms;
  return [
    new Cache(
      cache.events,
      vdoms$1,
      cache.old_vdoms,
      cache.dispatched_paths,
      cache.next_dispatched_paths,
    ),
    new Events(handlers$1, children$1),
  ];
}

export function add_child(cache, events, parent, index, child) {
  let children = $constants.singleton_list(child);
  return add_children(cache, events, parent, index, children);
}

export function from_node(root) {
  let cache = new$();
  let $ = add_child(cache, cache.events, $path.root, 0, root);
  let cache$1 = $[0];
  let events$1 = $[1];
  return new Cache(
    events$1,
    cache$1.vdoms,
    cache$1.old_vdoms,
    cache$1.dispatched_paths,
    cache$1.next_dispatched_paths,
  );
}

export function tick(cache) {
  return new Cache(
    cache.events,
    $mutable_map.new$(),
    cache.vdoms,
    cache.next_dispatched_paths,
    $constants.empty_list,
  );
}

export function events(cache) {
  return cache.events;
}

export function update_events(cache, events) {
  return new Cache(
    events,
    cache.vdoms,
    cache.old_vdoms,
    cache.dispatched_paths,
    cache.next_dispatched_paths,
  );
}

/**
 * Get a dictionary of all materialised Memo views.
 */
export function memos(cache) {
  return cache.vdoms;
}

/**
 *
 */
export function get_old_memo(cache, old, new$) {
  return $mutable_map.get_or_compute(cache.old_vdoms, old, new$);
}

/**
 * Reuses the cached element when dependencies are unchanged.
 */
export function keep_memo(cache, old, new$) {
  let node = $mutable_map.get_or_compute(cache.old_vdoms, old, new$);
  let vdoms = $mutable_map.insert(cache.vdoms, new$, node);
  return new Cache(
    cache.events,
    vdoms,
    cache.old_vdoms,
    cache.dispatched_paths,
    cache.next_dispatched_paths,
  );
}

/**
 * Caches a newly computed element when dependencies changed.
 */
export function add_memo(cache, new$, node) {
  let vdoms = $mutable_map.insert(cache.vdoms, new$, node);
  return new Cache(
    cache.events,
    vdoms,
    cache.old_vdoms,
    cache.dispatched_paths,
    cache.next_dispatched_paths,
  );
}

/**
 * Gets the isolated event subtree for a Map node.
 */
export function get_subtree(events, path, old_mapper) {
  let child = $mutable_map.get_or_compute(
    events.children,
    path,
    () => { return new Child(old_mapper, new_events()); },
  );
  return child.events;
}

/**
 * Updates the Map node's isolated event subtree after diffing its child.
 */
export function update_subtree(parent, path, mapper, events) {
  let new_child = new Child(mapper, events);
  let children = $mutable_map.insert(parent.children, path, new_child);
  return new Events(parent.handlers, children);
}

export function add_event(events, path, name, handler) {
  let handlers = do_add_event(events.handlers, path, name, handler);
  return new Events(handlers, events.children);
}

function do_remove_event(handlers, path, name) {
  return $mutable_map.delete$(handlers, $path.event(path, name));
}

export function remove_event(events, path, name) {
  let handlers = do_remove_event(events.handlers, path, name);
  return new Events(handlers, events.children);
}

function remove_attributes(handlers, path, attributes) {
  return $list.fold(
    attributes,
    handlers,
    (events, attribute) => {
      if (attribute instanceof Event) {
        let name = attribute.name;
        return do_remove_event(events, path, name);
      } else {
        return events;
      }
    },
  );
}

function do_remove_children(
  loop$handlers,
  loop$children,
  loop$vdoms,
  loop$parent,
  loop$index,
  loop$nodes
) {
  while (true) {
    let handlers = loop$handlers;
    let children = loop$children;
    let vdoms = loop$vdoms;
    let parent = loop$parent;
    let index = loop$index;
    let nodes = loop$nodes;
    let next = index + 1;
    if (nodes instanceof $Empty) {
      return new Events(handlers, children);
    } else {
      let $ = nodes.head;
      if ($ instanceof Fragment) {
        let rest = nodes.tail;
        let key = $.key;
        let nodes$1 = $.children;
        let path = $path.add(parent, index, key);
        let $1 = do_remove_children(handlers, children, vdoms, path, 0, nodes$1);
        let handlers$1 = $1.handlers;
        let children$1 = $1.children;
        loop$handlers = handlers$1;
        loop$children = children$1;
        loop$vdoms = vdoms;
        loop$parent = parent;
        loop$index = next;
        loop$nodes = rest;
      } else if ($ instanceof Element) {
        let rest = nodes.tail;
        let key = $.key;
        let attributes = $.attributes;
        let nodes$1 = $.children;
        let path = $path.add(parent, index, key);
        let handlers$1 = remove_attributes(handlers, path, attributes);
        let $1 = do_remove_children(
          handlers$1,
          children,
          vdoms,
          path,
          0,
          nodes$1,
        );
        let handlers$2 = $1.handlers;
        let children$1 = $1.children;
        loop$handlers = handlers$2;
        loop$children = children$1;
        loop$vdoms = vdoms;
        loop$parent = parent;
        loop$index = next;
        loop$nodes = rest;
      } else if ($ instanceof Text) {
        let rest = nodes.tail;
        loop$handlers = handlers;
        loop$children = children;
        loop$vdoms = vdoms;
        loop$parent = parent;
        loop$index = next;
        loop$nodes = rest;
      } else if ($ instanceof UnsafeInnerHtml) {
        let rest = nodes.tail;
        let key = $.key;
        let attributes = $.attributes;
        let path = $path.add(parent, index, key);
        let handlers$1 = remove_attributes(handlers, path, attributes);
        loop$handlers = handlers$1;
        loop$children = children;
        loop$vdoms = vdoms;
        loop$parent = parent;
        loop$index = next;
        loop$nodes = rest;
      } else if ($ instanceof Map) {
        let rest = nodes.tail;
        let key = $.key;
        let path = $path.add(parent, index, key);
        let children$1 = $mutable_map.delete$(children, $path.child(path));
        loop$handlers = handlers;
        loop$children = children$1;
        loop$vdoms = vdoms;
        loop$parent = parent;
        loop$index = next;
        loop$nodes = rest;
      } else {
        let rest = nodes.tail;
        let view = $.view;
        let $1 = $mutable_map.has_key(vdoms, view);
        if ($1) {
          let child = $mutable_map.unsafe_get(vdoms, view);
          let nodes$1 = listPrepend(child, rest);
          loop$handlers = handlers;
          loop$children = children;
          loop$vdoms = vdoms;
          loop$parent = parent;
          loop$index = index;
          loop$nodes = nodes$1;
        } else {
          loop$handlers = handlers;
          loop$children = children;
          loop$vdoms = vdoms;
          loop$parent = parent;
          loop$index = next;
          loop$nodes = rest;
        }
      }
    }
  }
}

export function remove_child(cache, events, parent, child_index, child) {
  return do_remove_children(
    events.handlers,
    events.children,
    cache.old_vdoms,
    parent,
    child_index,
    $constants.singleton_list(child),
  );
}

export function replace_child(cache, events, parent, child_index, prev, next) {
  let events$1 = remove_child(cache, events, parent, child_index, prev);
  return add_child(cache, events$1, parent, child_index, next);
}

function get_handler(loop$events, loop$path, loop$mapper) {
  while (true) {
    let events = loop$events;
    let path = loop$path;
    let mapper = loop$mapper;
    if (path instanceof $Empty) {
      return $constants.error_nil;
    } else {
      let $ = path.tail;
      if ($ instanceof $Empty) {
        let key = path.head;
        let $1 = $mutable_map.has_key(events.handlers, key);
        if ($1) {
          let handler = $mutable_map.unsafe_get(events.handlers, key);
          return new Ok(
            $decode.map(
              handler,
              (handler) => {
                return new Handler(
                  handler.prevent_default,
                  handler.stop_propagation,
                  coerce(mapper)(handler.message),
                );
              },
            ),
          );
        } else {
          return $constants.error_nil;
        }
      } else {
        let key = path.head;
        let path$1 = $;
        let $1 = $mutable_map.has_key(events.children, key);
        if ($1) {
          let child = $mutable_map.unsafe_get(events.children, key);
          let mapper$1 = compose_mapper(mapper, child.mapper);
          loop$events = child.events;
          loop$path = path$1;
          loop$mapper = mapper$1;
        } else {
          return $constants.error_nil;
        }
      }
    }
  }
}

export function decode(cache, path, name, event) {
  let parts = $path.split_subtree_path((path + $path.separator_event) + name);
  let $ = get_handler(cache.events, parts, $function.identity);
  if ($ instanceof Ok) {
    let handler = $[0];
    let $1 = $decode.run(event, handler);
    if ($1 instanceof Ok) {
      let handler$1 = $1[0];
      return new DecodedEvent(path, handler$1);
    } else {
      return new DispatchedEvent(path);
    }
  } else {
    return new DispatchedEvent(path);
  }
}

export function dispatch(cache, event) {
  let next_dispatched_paths = listPrepend(
    event.path,
    cache.next_dispatched_paths,
  );
  let cache$1 = new Cache(
    cache.events,
    cache.vdoms,
    cache.old_vdoms,
    cache.dispatched_paths,
    next_dispatched_paths,
  );
  if (event instanceof DecodedEvent) {
    let handler = event.handler;
    return [cache$1, new Ok(handler)];
  } else {
    return [cache$1, $constants.error_nil];
  }
}

/**
 *
 */
export function handle(cache, path, name, event) {
  let _pipe = decode(cache, path, name, event);
  return ((_capture) => { return dispatch(cache, _capture); })(_pipe);
}

export function has_dispatched_events(cache, path) {
  return $path.matches(path, cache.dispatched_paths);
}
