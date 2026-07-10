import * as $json from "../../../gleam_json/gleam/json.mjs";
import * as $dynamic from "../../../gleam_stdlib/gleam/dynamic.mjs";
import { identity as coerce } from "../../../gleam_stdlib/gleam/function.mjs";
import * as $list from "../../../gleam_stdlib/gleam/list.mjs";
import * as $string from "../../../gleam_stdlib/gleam/string.mjs";
import * as $string_tree from "../../../gleam_stdlib/gleam/string_tree.mjs";
import * as $houdini from "../../../houdini/houdini.mjs";
import {
  toList,
  Empty as $Empty,
  prepend as listPrepend,
  CustomType as $CustomType,
} from "../../gleam.mjs";
import * as $json_object_builder from "../../lustre/internals/json_object_builder.mjs";
import * as $mutable_map from "../../lustre/internals/mutable_map.mjs";
import * as $ref from "../../lustre/internals/ref.mjs";
import * as $vattr from "../../lustre/vdom/vattr.mjs";

export class Fragment extends $CustomType {
  constructor(kind, key, children, keyed_children) {
    super();
    this.kind = kind;
    this.key = key;
    this.children = children;
    this.keyed_children = keyed_children;
  }
}
export const Element$Fragment = (kind, key, children, keyed_children) =>
  new Fragment(kind, key, children, keyed_children);
export const Element$isFragment = (value) => value instanceof Fragment;
export const Element$Fragment$kind = (value) => value.kind;
export const Element$Fragment$0 = (value) => value.kind;
export const Element$Fragment$key = (value) => value.key;
export const Element$Fragment$1 = (value) => value.key;
export const Element$Fragment$children = (value) => value.children;
export const Element$Fragment$2 = (value) => value.children;
export const Element$Fragment$keyed_children = (value) => value.keyed_children;
export const Element$Fragment$3 = (value) => value.keyed_children;

export class Element extends $CustomType {
  constructor(kind, key, namespace, tag, attributes, children, keyed_children, self_closing, void$) {
    super();
    this.kind = kind;
    this.key = key;
    this.namespace = namespace;
    this.tag = tag;
    this.attributes = attributes;
    this.children = children;
    this.keyed_children = keyed_children;
    this.self_closing = self_closing;
    this.void = void$;
  }
}
export const Element$Element = (kind, key, namespace, tag, attributes, children, keyed_children, self_closing, void$) =>
  new Element(kind,
  key,
  namespace,
  tag,
  attributes,
  children,
  keyed_children,
  self_closing,
  void$);
export const Element$isElement = (value) => value instanceof Element;
export const Element$Element$kind = (value) => value.kind;
export const Element$Element$0 = (value) => value.kind;
export const Element$Element$key = (value) => value.key;
export const Element$Element$1 = (value) => value.key;
export const Element$Element$namespace = (value) => value.namespace;
export const Element$Element$2 = (value) => value.namespace;
export const Element$Element$tag = (value) => value.tag;
export const Element$Element$3 = (value) => value.tag;
export const Element$Element$attributes = (value) => value.attributes;
export const Element$Element$4 = (value) => value.attributes;
export const Element$Element$children = (value) => value.children;
export const Element$Element$5 = (value) => value.children;
export const Element$Element$keyed_children = (value) => value.keyed_children;
export const Element$Element$6 = (value) => value.keyed_children;
export const Element$Element$self_closing = (value) => value.self_closing;
export const Element$Element$7 = (value) => value.self_closing;
export const Element$Element$void = (value) => value.void;
export const Element$Element$8 = (value) => value.void;

export class Text extends $CustomType {
  constructor(kind, key, content) {
    super();
    this.kind = kind;
    this.key = key;
    this.content = content;
  }
}
export const Element$Text = (kind, key, content) =>
  new Text(kind, key, content);
export const Element$isText = (value) => value instanceof Text;
export const Element$Text$kind = (value) => value.kind;
export const Element$Text$0 = (value) => value.kind;
export const Element$Text$key = (value) => value.key;
export const Element$Text$1 = (value) => value.key;
export const Element$Text$content = (value) => value.content;
export const Element$Text$2 = (value) => value.content;

export class UnsafeInnerHtml extends $CustomType {
  constructor(kind, key, namespace, tag, attributes, inner_html) {
    super();
    this.kind = kind;
    this.key = key;
    this.namespace = namespace;
    this.tag = tag;
    this.attributes = attributes;
    this.inner_html = inner_html;
  }
}
export const Element$UnsafeInnerHtml = (kind, key, namespace, tag, attributes, inner_html) =>
  new UnsafeInnerHtml(kind, key, namespace, tag, attributes, inner_html);
export const Element$isUnsafeInnerHtml = (value) =>
  value instanceof UnsafeInnerHtml;
export const Element$UnsafeInnerHtml$kind = (value) => value.kind;
export const Element$UnsafeInnerHtml$0 = (value) => value.kind;
export const Element$UnsafeInnerHtml$key = (value) => value.key;
export const Element$UnsafeInnerHtml$1 = (value) => value.key;
export const Element$UnsafeInnerHtml$namespace = (value) => value.namespace;
export const Element$UnsafeInnerHtml$2 = (value) => value.namespace;
export const Element$UnsafeInnerHtml$tag = (value) => value.tag;
export const Element$UnsafeInnerHtml$3 = (value) => value.tag;
export const Element$UnsafeInnerHtml$attributes = (value) => value.attributes;
export const Element$UnsafeInnerHtml$4 = (value) => value.attributes;
export const Element$UnsafeInnerHtml$inner_html = (value) => value.inner_html;
export const Element$UnsafeInnerHtml$5 = (value) => value.inner_html;

export class Map extends $CustomType {
  constructor(kind, key, mapper, child) {
    super();
    this.kind = kind;
    this.key = key;
    this.mapper = mapper;
    this.child = child;
  }
}
export const Element$Map = (kind, key, mapper, child) =>
  new Map(kind, key, mapper, child);
export const Element$isMap = (value) => value instanceof Map;
export const Element$Map$kind = (value) => value.kind;
export const Element$Map$0 = (value) => value.kind;
export const Element$Map$key = (value) => value.key;
export const Element$Map$1 = (value) => value.key;
export const Element$Map$mapper = (value) => value.mapper;
export const Element$Map$2 = (value) => value.mapper;
export const Element$Map$child = (value) => value.child;
export const Element$Map$3 = (value) => value.child;

export class Memo extends $CustomType {
  constructor(kind, key, dependencies, view) {
    super();
    this.kind = kind;
    this.key = key;
    this.dependencies = dependencies;
    this.view = view;
  }
}
export const Element$Memo = (kind, key, dependencies, view) =>
  new Memo(kind, key, dependencies, view);
export const Element$isMemo = (value) => value instanceof Memo;
export const Element$Memo$kind = (value) => value.kind;
export const Element$Memo$0 = (value) => value.kind;
export const Element$Memo$key = (value) => value.key;
export const Element$Memo$1 = (value) => value.key;
export const Element$Memo$dependencies = (value) => value.dependencies;
export const Element$Memo$2 = (value) => value.dependencies;
export const Element$Memo$view = (value) => value.view;
export const Element$Memo$3 = (value) => value.view;

export const Element$key = (value) => value.key;
export const Element$kind = (value) => value.kind;

export const fragment_kind = 0;

export const element_kind = 1;

export const text_kind = 2;

export const unsafe_inner_html_kind = 3;

export const map_kind = 4;

export const memo_kind = 5;

export function fragment(key, children, keyed_children) {
  return new Fragment(fragment_kind, key, children, keyed_children);
}

export function element(
  key,
  namespace,
  tag,
  attributes,
  children,
  keyed_children,
  self_closing,
  void$
) {
  return new Element(
    element_kind,
    key,
    namespace,
    tag,
    $vattr.prepare(attributes),
    children,
    keyed_children,
    self_closing,
    void$,
  );
}

export function is_void_html_element(tag, namespace) {
  if (namespace === "") {
    if (tag === "area") {
      return true;
    } else if (tag === "base") {
      return true;
    } else if (tag === "br") {
      return true;
    } else if (tag === "col") {
      return true;
    } else if (tag === "embed") {
      return true;
    } else if (tag === "hr") {
      return true;
    } else if (tag === "img") {
      return true;
    } else if (tag === "input") {
      return true;
    } else if (tag === "link") {
      return true;
    } else if (tag === "meta") {
      return true;
    } else if (tag === "param") {
      return true;
    } else if (tag === "source") {
      return true;
    } else if (tag === "track") {
      return true;
    } else if (tag === "wbr") {
      return true;
    } else {
      return false;
    }
  } else {
    return false;
  }
}

export function text(key, content) {
  return new Text(text_kind, key, content);
}

export function unsafe_inner_html(key, namespace, tag, attributes, inner_html) {
  return new UnsafeInnerHtml(
    unsafe_inner_html_kind,
    key,
    namespace,
    tag,
    $vattr.prepare(attributes),
    inner_html,
  );
}

export function map(element, mapper) {
  if (element instanceof Map) {
    let child_mapper = element.mapper;
    return new Map(
      map_kind,
      element.key,
      (handler) => { return coerce(mapper)(child_mapper(handler)); },
      coerce(element.child),
    );
  } else {
    return new Map(map_kind, element.key, coerce(mapper), coerce(element));
  }
}

export function memo(key, dependencies, view) {
  return new Memo(memo_kind, key, dependencies, view);
}

export function to_keyed(key, node) {
  if (node instanceof Fragment) {
    return new Fragment(node.kind, key, node.children, node.keyed_children);
  } else if (node instanceof Element) {
    return new Element(
      node.kind,
      key,
      node.namespace,
      node.tag,
      node.attributes,
      node.children,
      node.keyed_children,
      node.self_closing,
      node.void,
    );
  } else if (node instanceof Text) {
    return new Text(node.kind, key, node.content);
  } else if (node instanceof UnsafeInnerHtml) {
    return new UnsafeInnerHtml(
      node.kind,
      key,
      node.namespace,
      node.tag,
      node.attributes,
      node.inner_html,
    );
  } else if (node instanceof Map) {
    let child = node.child;
    return new Map(node.kind, key, node.mapper, to_keyed(key, child));
  } else {
    let view = node.view;
    return new Memo(
      node.kind,
      key,
      node.dependencies,
      () => { return to_keyed(key, view()); },
    );
  }
}

function unsafe_inner_html_to_json(
  kind,
  key,
  namespace,
  tag,
  attributes,
  inner_html
) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.string(_pipe, "key", key);
  let _pipe$2 = $json_object_builder.string(_pipe$1, "namespace", namespace);
  let _pipe$3 = $json_object_builder.string(_pipe$2, "tag", tag);
  let _pipe$4 = $json_object_builder.list(
    _pipe$3,
    "attributes",
    attributes,
    $vattr.to_json,
  );
  let _pipe$5 = $json_object_builder.string(_pipe$4, "inner_html", inner_html);
  return $json_object_builder.build(_pipe$5);
}

function text_to_json(kind, key, content) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.string(_pipe, "key", key);
  let _pipe$2 = $json_object_builder.string(_pipe$1, "content", content);
  return $json_object_builder.build(_pipe$2);
}

function memo_to_json(view, memos) {
  let child = $mutable_map.get_or_compute(memos, view, view);
  return to_json(child, memos);
}

function map_to_json(kind, key, child, memos) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.string(_pipe, "key", key);
  let _pipe$2 = $json_object_builder.json(
    _pipe$1,
    "child",
    to_json(child, memos),
  );
  return $json_object_builder.build(_pipe$2);
}

function element_to_json(kind, key, namespace, tag, attributes, children, memos) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.string(_pipe, "key", key);
  let _pipe$2 = $json_object_builder.string(_pipe$1, "namespace", namespace);
  let _pipe$3 = $json_object_builder.string(_pipe$2, "tag", tag);
  let _pipe$4 = $json_object_builder.list(
    _pipe$3,
    "attributes",
    attributes,
    $vattr.to_json,
  );
  let _pipe$5 = $json_object_builder.list(
    _pipe$4,
    "children",
    children,
    (_capture) => { return to_json(_capture, memos); },
  );
  return $json_object_builder.build(_pipe$5);
}

function fragment_to_json(kind, key, children, memos) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.string(_pipe, "key", key);
  let _pipe$2 = $json_object_builder.list(
    _pipe$1,
    "children",
    children,
    (_capture) => { return to_json(_capture, memos); },
  );
  return $json_object_builder.build(_pipe$2);
}

export function to_json(node, memos) {
  if (node instanceof Fragment) {
    let kind = node.kind;
    let key = node.key;
    let children = node.children;
    return fragment_to_json(kind, key, children, memos);
  } else if (node instanceof Element) {
    let kind = node.kind;
    let key = node.key;
    let namespace = node.namespace;
    let tag = node.tag;
    let attributes = node.attributes;
    let children = node.children;
    return element_to_json(
      kind,
      key,
      namespace,
      tag,
      attributes,
      children,
      memos,
    );
  } else if (node instanceof Text) {
    let kind = node.kind;
    let key = node.key;
    let content = node.content;
    return text_to_json(kind, key, content);
  } else if (node instanceof UnsafeInnerHtml) {
    let kind = node.kind;
    let key = node.key;
    let namespace = node.namespace;
    let tag = node.tag;
    let attributes = node.attributes;
    let inner_html = node.inner_html;
    return unsafe_inner_html_to_json(
      kind,
      key,
      namespace,
      tag,
      attributes,
      inner_html,
    );
  } else if (node instanceof Map) {
    let kind = node.kind;
    let key = node.key;
    let child = node.child;
    return map_to_json(kind, key, child, memos);
  } else {
    let view = node.view;
    return memo_to_json(view, memos);
  }
}

function marker_comment(label, key) {
  if (key === "") {
    return $string_tree.from_string(("<!-- " + label) + " -->");
  } else {
    let _pipe = $string_tree.from_string(("<!-- " + label) + " key=\"");
    let _pipe$1 = $string_tree.append(_pipe, $houdini.escape(key));
    return $string_tree.append(_pipe$1, "\" -->");
  }
}

function children_to_string_tree(html, children, namespace) {
  return $list.fold(
    children,
    html,
    (html, child) => {
      return $string_tree.append_tree(html, to_string_tree(child, namespace));
    },
  );
}

export function to_string_tree(node, parent_namespace) {
  if (node instanceof Fragment) {
    let key = node.key;
    let children = node.children;
    let _pipe = marker_comment("lustre:fragment", key);
    let _pipe$1 = children_to_string_tree(_pipe, children, parent_namespace);
    return $string_tree.append_tree(
      _pipe$1,
      marker_comment("/lustre:fragment", ""),
    );
  } else if (node instanceof Element) {
    let self_closing = node.self_closing;
    if (self_closing) {
      let key = node.key;
      let namespace = node.namespace;
      let tag = node.tag;
      let attributes = node.attributes;
      let html = $string_tree.from_string("<" + tag);
      let attributes$1 = $vattr.to_string_tree(
        key,
        namespace,
        parent_namespace,
        attributes,
      );
      let _pipe = html;
      let _pipe$1 = $string_tree.append_tree(_pipe, attributes$1);
      return $string_tree.append(_pipe$1, "/>");
    } else {
      let void$ = node.void;
      if (void$) {
        let key = node.key;
        let namespace = node.namespace;
        let tag = node.tag;
        let attributes = node.attributes;
        let html = $string_tree.from_string("<" + tag);
        let attributes$1 = $vattr.to_string_tree(
          key,
          namespace,
          parent_namespace,
          attributes,
        );
        let _pipe = html;
        let _pipe$1 = $string_tree.append_tree(_pipe, attributes$1);
        return $string_tree.append(_pipe$1, ">");
      } else {
        let key = node.key;
        let namespace = node.namespace;
        let tag = node.tag;
        let attributes = node.attributes;
        let children = node.children;
        let html = $string_tree.from_string("<" + tag);
        let attributes$1 = $vattr.to_string_tree(
          key,
          namespace,
          parent_namespace,
          attributes,
        );
        let _pipe = html;
        let _pipe$1 = $string_tree.append_tree(_pipe, attributes$1);
        let _pipe$2 = $string_tree.append(_pipe$1, ">");
        let _pipe$3 = children_to_string_tree(_pipe$2, children, namespace);
        return $string_tree.append(_pipe$3, ("</" + tag) + ">");
      }
    }
  } else if (node instanceof Text) {
    let $ = node.content;
    if ($ === "") {
      return $string_tree.new$();
    } else {
      let content = $;
      return $string_tree.from_string($houdini.escape(content));
    }
  } else if (node instanceof UnsafeInnerHtml) {
    let key = node.key;
    let namespace = node.namespace;
    let tag = node.tag;
    let attributes = node.attributes;
    let inner_html = node.inner_html;
    let html = $string_tree.from_string("<" + tag);
    let attributes$1 = $vattr.to_string_tree(
      key,
      namespace,
      parent_namespace,
      attributes,
    );
    let _pipe = html;
    let _pipe$1 = $string_tree.append_tree(_pipe, attributes$1);
    let _pipe$2 = $string_tree.append(_pipe$1, ">");
    let _pipe$3 = $string_tree.append(_pipe$2, inner_html);
    return $string_tree.append(_pipe$3, ("</" + tag) + ">");
  } else if (node instanceof Map) {
    let key = node.key;
    let child = node.child;
    let _pipe = marker_comment("lustre:map", key);
    return $string_tree.append_tree(
      _pipe,
      to_string_tree(child, parent_namespace),
    );
  } else {
    let key = node.key;
    let view = node.view;
    let _pipe = marker_comment("lustre:memo", key);
    return $string_tree.append_tree(
      _pipe,
      to_string_tree(view(), parent_namespace),
    );
  }
}

export function to_string(node) {
  let _pipe = node;
  let _pipe$1 = to_string_tree(_pipe, "");
  return $string_tree.to_string(_pipe$1);
}

function children_to_snapshot_builder(
  loop$html,
  loop$children,
  loop$raw,
  loop$debug,
  loop$namespace,
  loop$indent
) {
  while (true) {
    let html = loop$html;
    let children = loop$children;
    let raw = loop$raw;
    let debug = loop$debug;
    let namespace = loop$namespace;
    let indent = loop$indent;
    if (children instanceof $Empty) {
      return html;
    } else {
      let $ = children.tail;
      if ($ instanceof $Empty) {
        let child = children.head;
        let rest = $;
        let _pipe = child;
        let _pipe$1 = do_to_snapshot_builder(
          _pipe,
          raw,
          debug,
          namespace,
          indent,
        );
        let _pipe$2 = $string_tree.append(_pipe$1, "\n");
        let _pipe$3 = $string_tree.prepend_tree(_pipe$2, html);
        loop$html = _pipe$3;
        loop$children = rest;
        loop$raw = raw;
        loop$debug = debug;
        loop$namespace = namespace;
        loop$indent = indent;
      } else {
        let $1 = children.head;
        if ($1 instanceof Text) {
          let $2 = $.head;
          if ($2 instanceof Text) {
            let rest = $.tail;
            let a = $1.content;
            let b = $2.content;
            loop$html = html;
            loop$children = listPrepend(new Text(text_kind, "", a + b), rest);
            loop$raw = raw;
            loop$debug = debug;
            loop$namespace = namespace;
            loop$indent = indent;
          } else {
            let child = $1;
            let rest = $;
            let _pipe = child;
            let _pipe$1 = do_to_snapshot_builder(
              _pipe,
              raw,
              debug,
              namespace,
              indent,
            );
            let _pipe$2 = $string_tree.append(_pipe$1, "\n");
            let _pipe$3 = $string_tree.prepend_tree(_pipe$2, html);
            loop$html = _pipe$3;
            loop$children = rest;
            loop$raw = raw;
            loop$debug = debug;
            loop$namespace = namespace;
            loop$indent = indent;
          }
        } else {
          let child = $1;
          let rest = $;
          let _pipe = child;
          let _pipe$1 = do_to_snapshot_builder(
            _pipe,
            raw,
            debug,
            namespace,
            indent,
          );
          let _pipe$2 = $string_tree.append(_pipe$1, "\n");
          let _pipe$3 = $string_tree.prepend_tree(_pipe$2, html);
          loop$html = _pipe$3;
          loop$children = rest;
          loop$raw = raw;
          loop$debug = debug;
          loop$namespace = namespace;
          loop$indent = indent;
        }
      }
    }
  }
}

function do_to_snapshot_builder(
  loop$node,
  loop$raw,
  loop$debug,
  loop$parent_namespace,
  loop$indent
) {
  while (true) {
    let node = loop$node;
    let raw = loop$raw;
    let debug = loop$debug;
    let parent_namespace = loop$parent_namespace;
    let indent = loop$indent;
    let spaces = $string.repeat("  ", indent);
    if (node instanceof Fragment) {
      if (debug) {
        let key = node.key;
        let children = node.children;
        let _pipe = marker_comment("lustre:fragment", key);
        let _pipe$1 = $string_tree.prepend(_pipe, spaces);
        let _pipe$2 = $string_tree.append(_pipe$1, "\n");
        let _pipe$3 = children_to_snapshot_builder(
          _pipe$2,
          children,
          raw,
          debug,
          parent_namespace,
          indent + 1,
        );
        let _pipe$4 = $string_tree.append(_pipe$3, spaces);
        return $string_tree.append_tree(
          _pipe$4,
          marker_comment("/lustre:fragment", ""),
        );
      } else {
        let children = node.children;
        return children_to_snapshot_builder(
          $string_tree.new$(),
          children,
          raw,
          debug,
          parent_namespace,
          indent,
        );
      }
    } else if (node instanceof Element) {
      let $ = node.self_closing;
      if ($) {
        let key = node.key;
        let namespace = node.namespace;
        let tag = node.tag;
        let attributes = node.attributes;
        let html = $string_tree.from_string("<" + tag);
        let attributes$1 = $vattr.to_string_tree(
          key,
          namespace,
          parent_namespace,
          attributes,
        );
        let _pipe = html;
        let _pipe$1 = $string_tree.prepend(_pipe, spaces);
        let _pipe$2 = $string_tree.append_tree(_pipe$1, attributes$1);
        return $string_tree.append(_pipe$2, "/>");
      } else {
        let $1 = node.void;
        if ($1) {
          let key = node.key;
          let namespace = node.namespace;
          let tag = node.tag;
          let attributes = node.attributes;
          let html = $string_tree.from_string("<" + tag);
          let attributes$1 = $vattr.to_string_tree(
            key,
            namespace,
            parent_namespace,
            attributes,
          );
          let _pipe = html;
          let _pipe$1 = $string_tree.prepend(_pipe, spaces);
          let _pipe$2 = $string_tree.append_tree(_pipe$1, attributes$1);
          return $string_tree.append(_pipe$2, ">");
        } else {
          let $2 = node.children;
          if ($2 instanceof $Empty) {
            let key = node.key;
            let namespace = node.namespace;
            let tag = node.tag;
            let attributes = node.attributes;
            let html = $string_tree.from_string("<" + tag);
            let attributes$1 = $vattr.to_string_tree(
              key,
              namespace,
              parent_namespace,
              attributes,
            );
            let _pipe = html;
            let _pipe$1 = $string_tree.prepend(_pipe, spaces);
            let _pipe$2 = $string_tree.append_tree(_pipe$1, attributes$1);
            let _pipe$3 = $string_tree.append(_pipe$2, ">");
            return $string_tree.append(_pipe$3, ("</" + tag) + ">");
          } else {
            let key = node.key;
            let namespace = node.namespace;
            let tag = node.tag;
            let attributes = node.attributes;
            let children = $2;
            let html = $string_tree.from_string("<" + tag);
            let attributes$1 = $vattr.to_string_tree(
              key,
              namespace,
              parent_namespace,
              attributes,
            );
            let _pipe = html;
            let _pipe$1 = $string_tree.prepend(_pipe, spaces);
            let _pipe$2 = $string_tree.append_tree(_pipe$1, attributes$1);
            let _pipe$3 = $string_tree.append(_pipe$2, ">\n");
            let _pipe$4 = children_to_snapshot_builder(
              _pipe$3,
              children,
              raw,
              debug,
              namespace,
              indent + 1,
            );
            let _pipe$5 = $string_tree.append(_pipe$4, spaces);
            return $string_tree.append(_pipe$5, ("</" + tag) + ">");
          }
        }
      }
    } else if (node instanceof Text) {
      let $ = node.content;
      if ($ === "") {
        return $string_tree.new$();
      } else if (raw) {
        let content = $;
        return $string_tree.from_strings(toList([spaces, content]));
      } else {
        let content = $;
        return $string_tree.from_strings(
          toList([spaces, $houdini.escape(content)]),
        );
      }
    } else if (node instanceof UnsafeInnerHtml) {
      let key = node.key;
      let namespace = node.namespace;
      let tag = node.tag;
      let attributes = node.attributes;
      let inner_html = node.inner_html;
      let html = $string_tree.from_string("<" + tag);
      let attributes$1 = $vattr.to_string_tree(
        key,
        namespace,
        parent_namespace,
        attributes,
      );
      let _pipe = html;
      let _pipe$1 = $string_tree.prepend(_pipe, spaces);
      let _pipe$2 = $string_tree.append_tree(_pipe$1, attributes$1);
      let _pipe$3 = $string_tree.append(_pipe$2, ">");
      let _pipe$4 = $string_tree.append(_pipe$3, inner_html);
      return $string_tree.append(_pipe$4, ("</" + tag) + ">");
    } else if (node instanceof Map) {
      if (debug) {
        let key = node.key;
        let child = node.child;
        let _pipe = marker_comment("lustre:map", key);
        let _pipe$1 = $string_tree.prepend(_pipe, spaces);
        let _pipe$2 = $string_tree.append(_pipe$1, "\n");
        return $string_tree.append_tree(
          _pipe$2,
          do_to_snapshot_builder(
            child,
            raw,
            debug,
            parent_namespace,
            indent + 1,
          ),
        );
      } else {
        let child = node.child;
        loop$node = child;
        loop$raw = raw;
        loop$debug = debug;
        loop$parent_namespace = parent_namespace;
        loop$indent = indent;
      }
    } else if (debug) {
      let key = node.key;
      let view = node.view;
      let _pipe = marker_comment("lustre:memo", key);
      let _pipe$1 = $string_tree.prepend(_pipe, spaces);
      let _pipe$2 = $string_tree.append(_pipe$1, "\n");
      return $string_tree.append_tree(
        _pipe$2,
        do_to_snapshot_builder(view(), raw, debug, parent_namespace, indent + 1),
      );
    } else {
      let view = node.view;
      loop$node = view();
      loop$raw = raw;
      loop$debug = debug;
      loop$parent_namespace = parent_namespace;
      loop$indent = indent;
    }
  }
}

export function to_snapshot(node, debug) {
  let _pipe = do_to_snapshot_builder(node, false, debug, "", 0);
  return $string_tree.to_string(_pipe);
}
