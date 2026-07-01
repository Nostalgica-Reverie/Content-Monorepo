import * as $json from "../../../gleam_json/gleam/json.mjs";
import { Empty as $Empty, prepend as listPrepend, CustomType as $CustomType } from "../../gleam.mjs";
import * as $constants from "../../lustre/internals/constants.mjs";
import * as $json_object_builder from "../../lustre/internals/json_object_builder.mjs";
import * as $vattr from "../../lustre/vdom/vattr.mjs";
import * as $vnode from "../../lustre/vdom/vnode.mjs";

export class Patch extends $CustomType {
  constructor(index, path, removed, changes, children) {
    super();
    this.index = index;
    this.path = path;
    this.removed = removed;
    this.changes = changes;
    this.children = children;
  }
}
export const Patch$Patch = (index, path, removed, changes, children) =>
  new Patch(index, path, removed, changes, children);
export const Patch$isPatch = (value) => value instanceof Patch;
export const Patch$Patch$index = (value) => value.index;
export const Patch$Patch$0 = (value) => value.index;
export const Patch$Patch$path = (value) => value.path;
export const Patch$Patch$1 = (value) => value.path;
export const Patch$Patch$removed = (value) => value.removed;
export const Patch$Patch$2 = (value) => value.removed;
export const Patch$Patch$changes = (value) => value.changes;
export const Patch$Patch$3 = (value) => value.changes;
export const Patch$Patch$children = (value) => value.children;
export const Patch$Patch$4 = (value) => value.children;

export class ReplaceText extends $CustomType {
  constructor(kind, content) {
    super();
    this.kind = kind;
    this.content = content;
  }
}
export const Change$ReplaceText = (kind, content) =>
  new ReplaceText(kind, content);
export const Change$isReplaceText = (value) => value instanceof ReplaceText;
export const Change$ReplaceText$kind = (value) => value.kind;
export const Change$ReplaceText$0 = (value) => value.kind;
export const Change$ReplaceText$content = (value) => value.content;
export const Change$ReplaceText$1 = (value) => value.content;

export class ReplaceInnerHtml extends $CustomType {
  constructor(kind, inner_html) {
    super();
    this.kind = kind;
    this.inner_html = inner_html;
  }
}
export const Change$ReplaceInnerHtml = (kind, inner_html) =>
  new ReplaceInnerHtml(kind, inner_html);
export const Change$isReplaceInnerHtml = (value) =>
  value instanceof ReplaceInnerHtml;
export const Change$ReplaceInnerHtml$kind = (value) => value.kind;
export const Change$ReplaceInnerHtml$0 = (value) => value.kind;
export const Change$ReplaceInnerHtml$inner_html = (value) => value.inner_html;
export const Change$ReplaceInnerHtml$1 = (value) => value.inner_html;

export class Update extends $CustomType {
  constructor(kind, added, removed) {
    super();
    this.kind = kind;
    this.added = added;
    this.removed = removed;
  }
}
export const Change$Update = (kind, added, removed) =>
  new Update(kind, added, removed);
export const Change$isUpdate = (value) => value instanceof Update;
export const Change$Update$kind = (value) => value.kind;
export const Change$Update$0 = (value) => value.kind;
export const Change$Update$added = (value) => value.added;
export const Change$Update$1 = (value) => value.added;
export const Change$Update$removed = (value) => value.removed;
export const Change$Update$2 = (value) => value.removed;

/**
 * Move a keyed child so that it is before the child at the given index.
 */
export class Move extends $CustomType {
  constructor(kind, key, before) {
    super();
    this.kind = kind;
    this.key = key;
    this.before = before;
  }
}
export const Change$Move = (kind, key, before) => new Move(kind, key, before);
export const Change$isMove = (value) => value instanceof Move;
export const Change$Move$kind = (value) => value.kind;
export const Change$Move$0 = (value) => value.kind;
export const Change$Move$key = (value) => value.key;
export const Change$Move$1 = (value) => value.key;
export const Change$Move$before = (value) => value.before;
export const Change$Move$2 = (value) => value.before;

/**
 * Replace a node at the given index with a new vnode.
 */
export class Replace extends $CustomType {
  constructor(kind, index, with$) {
    super();
    this.kind = kind;
    this.index = index;
    this.with = with$;
  }
}
export const Change$Replace = (kind, index, with$) =>
  new Replace(kind, index, with$);
export const Change$isReplace = (value) => value instanceof Replace;
export const Change$Replace$kind = (value) => value.kind;
export const Change$Replace$0 = (value) => value.kind;
export const Change$Replace$index = (value) => value.index;
export const Change$Replace$1 = (value) => value.index;
export const Change$Replace$with = (value) => value.with;
export const Change$Replace$2 = (value) => value.with;

/**
 * Remove a child at the given index.
 */
export class Remove extends $CustomType {
  constructor(kind, index) {
    super();
    this.kind = kind;
    this.index = index;
  }
}
export const Change$Remove = (kind, index) => new Remove(kind, index);
export const Change$isRemove = (value) => value instanceof Remove;
export const Change$Remove$kind = (value) => value.kind;
export const Change$Remove$0 = (value) => value.kind;
export const Change$Remove$index = (value) => value.index;
export const Change$Remove$1 = (value) => value.index;

/**
 * Insert one or multiple children before the child with the given index.
 */
export class Insert extends $CustomType {
  constructor(kind, children, before) {
    super();
    this.kind = kind;
    this.children = children;
    this.before = before;
  }
}
export const Change$Insert = (kind, children, before) =>
  new Insert(kind, children, before);
export const Change$isInsert = (value) => value instanceof Insert;
export const Change$Insert$kind = (value) => value.kind;
export const Change$Insert$0 = (value) => value.kind;
export const Change$Insert$children = (value) => value.children;
export const Change$Insert$1 = (value) => value.children;
export const Change$Insert$before = (value) => value.before;
export const Change$Insert$2 = (value) => value.before;

export const Change$kind = (value) => value.kind;

export const replace_text_kind = 0;

export const replace_inner_html_kind = 1;

export const update_kind = 2;

export const move_kind = 3;

export const remove_kind = 4;

export const replace_kind = 5;

export const insert_kind = 6;

export function new$(index, removed, changes, children) {
  return new Patch(index, $constants.empty_list, removed, changes, children);
}

export function replace_text(content) {
  return new ReplaceText(replace_text_kind, content);
}

export function replace_inner_html(inner_html) {
  return new ReplaceInnerHtml(replace_inner_html_kind, inner_html);
}

export function update(added, removed) {
  return new Update(update_kind, added, removed);
}

export function move(key, before) {
  return new Move(move_kind, key, before);
}

export function remove(index) {
  return new Remove(remove_kind, index);
}

export function replace(index, with$) {
  return new Replace(replace_kind, index, with$);
}

export function insert(children, before) {
  return new Insert(insert_kind, children, before);
}

export function is_empty(patch) {
  let $ = patch.changes;
  if ($ instanceof $Empty) {
    let $1 = patch.children;
    if ($1 instanceof $Empty) {
      let $2 = patch.removed;
      if ($2 === 0) {
        return true;
      } else {
        return false;
      }
    } else {
      return false;
    }
  } else {
    return false;
  }
}

export function add_parent(child, index) {
  return new Patch(
    index,
    listPrepend(child.index, child.path),
    child.removed,
    child.changes,
    child.children,
  );
}

function insert_to_json(kind, children, before, memos) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.int(_pipe, "before", before);
  let _pipe$2 = $json_object_builder.list(
    _pipe$1,
    "children",
    children,
    (_capture) => { return $vnode.to_json(_capture, memos); },
  );
  return $json_object_builder.build(_pipe$2);
}

function replace_to_json(kind, index, with$, memos) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.int(_pipe, "index", index);
  let _pipe$2 = $json_object_builder.json(
    _pipe$1,
    "with",
    $vnode.to_json(with$, memos),
  );
  return $json_object_builder.build(_pipe$2);
}

function remove_to_json(kind, index) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.int(_pipe, "index", index);
  return $json_object_builder.build(_pipe$1);
}

function move_to_json(kind, key, before) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.string(_pipe, "key", key);
  let _pipe$2 = $json_object_builder.int(_pipe$1, "before", before);
  return $json_object_builder.build(_pipe$2);
}

function update_to_json(kind, added, removed) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.list(_pipe, "added", added, $vattr.to_json);
  let _pipe$2 = $json_object_builder.list(
    _pipe$1,
    "removed",
    removed,
    $vattr.to_json,
  );
  return $json_object_builder.build(_pipe$2);
}

function replace_inner_html_to_json(kind, inner_html) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.string(_pipe, "inner_html", inner_html);
  return $json_object_builder.build(_pipe$1);
}

function replace_text_to_json(kind, content) {
  let _pipe = $json_object_builder.tagged(kind);
  let _pipe$1 = $json_object_builder.string(_pipe, "content", content);
  return $json_object_builder.build(_pipe$1);
}

function change_to_json(change, memos) {
  if (change instanceof ReplaceText) {
    let kind = change.kind;
    let content = change.content;
    return replace_text_to_json(kind, content);
  } else if (change instanceof ReplaceInnerHtml) {
    let kind = change.kind;
    let inner_html = change.inner_html;
    return replace_inner_html_to_json(kind, inner_html);
  } else if (change instanceof Update) {
    let kind = change.kind;
    let added = change.added;
    let removed = change.removed;
    return update_to_json(kind, added, removed);
  } else if (change instanceof Move) {
    let kind = change.kind;
    let key = change.key;
    let before = change.before;
    return move_to_json(kind, key, before);
  } else if (change instanceof Replace) {
    let kind = change.kind;
    let index = change.index;
    let with$ = change.with;
    return replace_to_json(kind, index, with$, memos);
  } else if (change instanceof Remove) {
    let kind = change.kind;
    let index = change.index;
    return remove_to_json(kind, index);
  } else {
    let kind = change.kind;
    let children = change.children;
    let before = change.before;
    return insert_to_json(kind, children, before, memos);
  }
}

export function to_json(patch, memos) {
  let _pipe = $json_object_builder.new$();
  let _pipe$1 = $json_object_builder.list(_pipe, "path", patch.path, $json.int);
  let _pipe$2 = $json_object_builder.int(_pipe$1, "index", patch.index);
  let _pipe$3 = $json_object_builder.int(_pipe$2, "removed", patch.removed);
  let _pipe$4 = $json_object_builder.list(
    _pipe$3,
    "changes",
    patch.changes,
    (change) => { return change_to_json(change, memos); },
  );
  let _pipe$5 = $json_object_builder.list(
    _pipe$4,
    "children",
    patch.children,
    (child) => { return to_json(child, memos); },
  );
  return $json_object_builder.build(_pipe$5);
}
