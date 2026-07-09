import * as $json from "../../../gleam_json/gleam/json.mjs";
import * as $decode from "../../../gleam_stdlib/gleam/dynamic/decode.mjs";
import * as $list from "../../../gleam_stdlib/gleam/list.mjs";
import * as $order from "../../../gleam_stdlib/gleam/order.mjs";
import * as $string from "../../../gleam_stdlib/gleam/string.mjs";
import * as $string_tree from "../../../gleam_stdlib/gleam/string_tree.mjs";
import * as $houdini from "../../../houdini/houdini.mjs";
import { Empty as $Empty, prepend as listPrepend, CustomType as $CustomType } from "../../gleam.mjs";
import * as $constants from "../../lustre/internals/constants.mjs";
import * as $json_object_builder from "../../lustre/internals/json_object_builder.mjs";
import { compare } from "./vattr.ffi.mjs";

export { compare };

export class Attribute extends $CustomType {
  constructor(kind, name, value) {
    super();
    this.kind = kind;
    this.name = name;
    this.value = value;
  }
}
export const Attribute$Attribute = (kind, name, value) =>
  new Attribute(kind, name, value);
export const Attribute$isAttribute = (value) => value instanceof Attribute;
export const Attribute$Attribute$kind = (value) => value.kind;
export const Attribute$Attribute$0 = (value) => value.kind;
export const Attribute$Attribute$name = (value) => value.name;
export const Attribute$Attribute$1 = (value) => value.name;
export const Attribute$Attribute$value = (value) => value.value;
export const Attribute$Attribute$2 = (value) => value.value;

export class Property extends $CustomType {
  constructor(kind, name, value) {
    super();
    this.kind = kind;
    this.name = name;
    this.value = value;
  }
}
export const Attribute$Property = (kind, name, value) =>
  new Property(kind, name, value);
export const Attribute$isProperty = (value) => value instanceof Property;
export const Attribute$Property$kind = (value) => value.kind;
export const Attribute$Property$0 = (value) => value.kind;
export const Attribute$Property$name = (value) => value.name;
export const Attribute$Property$1 = (value) => value.name;
export const Attribute$Property$value = (value) => value.value;
export const Attribute$Property$2 = (value) => value.value;

export class Event extends $CustomType {
  constructor(kind, name, handler, include, prevent_default, stop_propagation, debounce, throttle) {
    super();
    this.kind = kind;
    this.name = name;
    this.handler = handler;
    this.include = include;
    this.prevent_default = prevent_default;
    this.stop_propagation = stop_propagation;
    this.debounce = debounce;
    this.throttle = throttle;
  }
}
export const Attribute$Event = (kind, name, handler, include, prevent_default, stop_propagation, debounce, throttle) =>
  new Event(kind,
  name,
  handler,
  include,
  prevent_default,
  stop_propagation,
  debounce,
  throttle);
export const Attribute$isEvent = (value) => value instanceof Event;
export const Attribute$Event$kind = (value) => value.kind;
export const Attribute$Event$0 = (value) => value.kind;
export const Attribute$Event$name = (value) => value.name;
export const Attribute$Event$1 = (value) => value.name;
export const Attribute$Event$handler = (value) => value.handler;
export const Attribute$Event$2 = (value) => value.handler;
export const Attribute$Event$include = (value) => value.include;
export const Attribute$Event$3 = (value) => value.include;
export const Attribute$Event$prevent_default = (value) => value.prevent_default;
export const Attribute$Event$4 = (value) => value.prevent_default;
export const Attribute$Event$stop_propagation = (value) =>
  value.stop_propagation;
export const Attribute$Event$5 = (value) => value.stop_propagation;
export const Attribute$Event$debounce = (value) => value.debounce;
export const Attribute$Event$6 = (value) => value.debounce;
export const Attribute$Event$throttle = (value) => value.throttle;
export const Attribute$Event$7 = (value) => value.throttle;

export const Attribute$kind = (value) => value.kind;
export const Attribute$name = (value) => value.name;

export class Handler extends $CustomType {
  constructor(prevent_default, stop_propagation, message) {
    super();
    this.prevent_default = prevent_default;
    this.stop_propagation = stop_propagation;
    this.message = message;
  }
}
export const Handler$Handler = (prevent_default, stop_propagation, message) =>
  new Handler(prevent_default, stop_propagation, message);
export const Handler$isHandler = (value) => value instanceof Handler;
export const Handler$Handler$prevent_default = (value) => value.prevent_default;
export const Handler$Handler$0 = (value) => value.prevent_default;
export const Handler$Handler$stop_propagation = (value) =>
  value.stop_propagation;
export const Handler$Handler$1 = (value) => value.stop_propagation;
export const Handler$Handler$message = (value) => value.message;
export const Handler$Handler$2 = (value) => value.message;

export class Never extends $CustomType {
  constructor(kind) {
    super();
    this.kind = kind;
  }
}
export const EventBehaviour$Never = (kind) => new Never(kind);
export const EventBehaviour$isNever = (value) => value instanceof Never;
export const EventBehaviour$Never$kind = (value) => value.kind;
export const EventBehaviour$Never$0 = (value) => value.kind;

export class Possible extends $CustomType {
  constructor(kind) {
    super();
    this.kind = kind;
  }
}
export const EventBehaviour$Possible = (kind) => new Possible(kind);
export const EventBehaviour$isPossible = (value) => value instanceof Possible;
export const EventBehaviour$Possible$kind = (value) => value.kind;
export const EventBehaviour$Possible$0 = (value) => value.kind;

export class Always extends $CustomType {
  constructor(kind) {
    super();
    this.kind = kind;
  }
}
export const EventBehaviour$Always = (kind) => new Always(kind);
export const EventBehaviour$isAlways = (value) => value instanceof Always;
export const EventBehaviour$Always$kind = (value) => value.kind;
export const EventBehaviour$Always$0 = (value) => value.kind;

export const EventBehaviour$kind = (value) => value.kind;

export const attribute_kind = 0;

export const property_kind = 1;

export const event_kind = 2;

export const never_kind = 0;

export const never = /* @__PURE__ */ new Never(never_kind);

export const possible_kind = 1;

export const possible = /* @__PURE__ */ new Possible(possible_kind);

export const always_kind = 2;

export const always = /* @__PURE__ */ new Always(always_kind);

export function attribute(name, value) {
  return new Attribute(attribute_kind, name, value);
}

export function property(name, value) {
  return new Property(property_kind, name, value);
}

export function event(
  name,
  handler,
  include,
  prevent_default,
  stop_propagation,
  debounce,
  throttle
) {
  return new Event(
    event_kind,
    name,
    handler,
    include,
    prevent_default,
    stop_propagation,
    debounce,
    throttle,
  );
}

export function merge(loop$attributes, loop$merged) {
  while (true) {
    let attributes = loop$attributes;
    let merged = loop$merged;
    if (attributes instanceof $Empty) {
      return merged;
    } else {
      let $ = attributes.head;
      if ($ instanceof Attribute) {
        let $1 = $.name;
        if ($1 === "") {
          let rest = attributes.tail;
          loop$attributes = rest;
          loop$merged = merged;
        } else if ($1 === "class") {
          let $2 = $.value;
          if ($2 === "") {
            let rest = attributes.tail;
            loop$attributes = rest;
            loop$merged = merged;
          } else {
            let $3 = attributes.tail;
            if ($3 instanceof $Empty) {
              let attribute$1 = $;
              let rest = $3;
              loop$attributes = rest;
              loop$merged = listPrepend(attribute$1, merged);
            } else {
              let $4 = $3.head;
              if ($4 instanceof Attribute) {
                let $5 = $4.name;
                if ($5 === "class") {
                  let kind = $.kind;
                  let class1 = $2;
                  let rest = $3.tail;
                  let class2 = $4.value;
                  let value = (class1 + " ") + class2;
                  let attribute$1 = new Attribute(kind, "class", value);
                  loop$attributes = listPrepend(attribute$1, rest);
                  loop$merged = merged;
                } else {
                  let attribute$1 = $;
                  let rest = $3;
                  loop$attributes = rest;
                  loop$merged = listPrepend(attribute$1, merged);
                }
              } else {
                let attribute$1 = $;
                let rest = $3;
                loop$attributes = rest;
                loop$merged = listPrepend(attribute$1, merged);
              }
            }
          }
        } else if ($1 === "style") {
          let $2 = $.value;
          if ($2 === "") {
            let rest = attributes.tail;
            loop$attributes = rest;
            loop$merged = merged;
          } else {
            let $3 = attributes.tail;
            if ($3 instanceof $Empty) {
              let attribute$1 = $;
              let rest = $3;
              loop$attributes = rest;
              loop$merged = listPrepend(attribute$1, merged);
            } else {
              let $4 = $3.head;
              if ($4 instanceof Attribute) {
                let $5 = $4.name;
                if ($5 === "style") {
                  let kind = $.kind;
                  let style1 = $2;
                  let rest = $3.tail;
                  let style2 = $4.value;
                  let value = (style1 + ";") + style2;
                  let attribute$1 = new Attribute(kind, "style", value);
                  loop$attributes = listPrepend(attribute$1, rest);
                  loop$merged = merged;
                } else {
                  let attribute$1 = $;
                  let rest = $3;
                  loop$attributes = rest;
                  loop$merged = listPrepend(attribute$1, merged);
                }
              } else {
                let attribute$1 = $;
                let rest = $3;
                loop$attributes = rest;
                loop$merged = listPrepend(attribute$1, merged);
              }
            }
          }
        } else {
          let attribute$1 = $;
          let rest = attributes.tail;
          loop$attributes = rest;
          loop$merged = listPrepend(attribute$1, merged);
        }
      } else {
        let attribute$1 = $;
        let rest = attributes.tail;
        loop$attributes = rest;
        loop$merged = listPrepend(attribute$1, merged);
      }
    }
  }
}

export function prepare(attributes) {
  if (attributes instanceof $Empty) {
    return attributes;
  } else {
    let $ = attributes.tail;
    if ($ instanceof $Empty) {
      return attributes;
    } else {
      let _pipe = attributes;
      let _pipe$1 = $list.sort(_pipe, (a, b) => { return compare(b, a); });
      return merge(_pipe$1, $constants.empty_list);
    }
  }
}

function event_behaviour_to_json_builder(behaviour) {
  if (behaviour instanceof Never) {
    let kind = behaviour.kind;
    return $json_object_builder.tagged(kind);
  } else if (behaviour instanceof Possible) {
    return $json_object_builder.tagged(never_kind);
  } else {
    let kind = behaviour.kind;
    return $json_object_builder.tagged(kind);
  }
}

function event_to_json(
  kind,
  name,
  include,
  prevent_default,
  stop_propagation,
  debounce,
  throttle
) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.string(_pipe, "name", name);
  let _pipe$2 = $json_object_builder.list(
    _pipe$1,
    "include",
    include,
    $json.string,
  );
  let _pipe$3 = $json_object_builder.object(
    _pipe$2,
    "prevent_default",
    event_behaviour_to_json_builder(prevent_default),
  );
  let _pipe$4 = $json_object_builder.object(
    _pipe$3,
    "stop_propagation",
    event_behaviour_to_json_builder(stop_propagation),
  );
  let _pipe$5 = $json_object_builder.int(_pipe$4, "debounce", debounce);
  let _pipe$6 = $json_object_builder.int(_pipe$5, "throttle", throttle);
  return $json_object_builder.build(_pipe$6);
}

function property_to_json(kind, name, value) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.string(_pipe, "name", name);
  let _pipe$2 = $json_object_builder.json(_pipe$1, "value", value);
  return $json_object_builder.build(_pipe$2);
}

function attribute_to_json(kind, name, value) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.string(_pipe, "name", name);
  let _pipe$2 = $json_object_builder.string(_pipe$1, "value", value);
  return $json_object_builder.build(_pipe$2);
}

export function to_json(attribute) {
  if (attribute instanceof Attribute) {
    let kind = attribute.kind;
    let name = attribute.name;
    let value = attribute.value;
    return attribute_to_json(kind, name, value);
  } else if (attribute instanceof Property) {
    let kind = attribute.kind;
    let name = attribute.name;
    let value = attribute.value;
    return property_to_json(kind, name, value);
  } else {
    let kind = attribute.kind;
    let name = attribute.name;
    let include = attribute.include;
    let prevent_default = attribute.prevent_default;
    let stop_propagation = attribute.stop_propagation;
    let debounce = attribute.debounce;
    let throttle = attribute.throttle;
    return event_to_json(
      kind,
      name,
      include,
      prevent_default,
      stop_propagation,
      debounce,
      throttle,
    );
  }
}

export function to_string_tree(key, namespace, parent_namespace, attributes) {
  let _block;
  let $ = key !== "";
  if ($) {
    _block = listPrepend(attribute("data-lustre-key", key), attributes);
  } else {
    _block = attributes;
  }
  let attributes$1 = _block;
  let _block$1;
  let $1 = namespace !== parent_namespace;
  if ($1) {
    if (namespace === "") {
      _block$1 = listPrepend(
        attribute("xmlns", "http://www.w3.org/1999/xhtml"),
        attributes$1,
      );
    } else {
      _block$1 = listPrepend(attribute("xmlns", namespace), attributes$1);
    }
  } else {
    _block$1 = attributes$1;
  }
  let attributes$2 = _block$1;
  return $list.fold(
    attributes$2,
    $string_tree.new$(),
    (html, attr) => {
      if (attr instanceof Attribute) {
        let $2 = attr.name;
        if ($2 === "virtual:defaultValue") {
          let value = attr.value;
          return $string_tree.append(
            html,
            (" value=\"" + $houdini.escape(value)) + "\"",
          );
        } else if ($2 === "virtual:defaultChecked") {
          return $string_tree.append(html, " checked");
        } else if ($2 === "virtual:defaultSelected") {
          return $string_tree.append(html, " selected");
        } else if ($2 === "") {
          return html;
        } else {
          let $3 = attr.value;
          if ($3 === "") {
            let name = $2;
            return $string_tree.append(html, " " + name);
          } else {
            let name = $2;
            let value = $3;
            return $string_tree.append(
              html,
              ((((" " + name) + "=\"") + $houdini.escape(value)) + "\""),
            );
          }
        }
      } else {
        return html;
      }
    },
  );
}
