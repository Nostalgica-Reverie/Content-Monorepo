import * as $json from "../../../gleam_json/gleam/json.mjs";
import * as $dict from "../../../gleam_stdlib/gleam/dict.mjs";
import * as $dynamic from "../../../gleam_stdlib/gleam/dynamic.mjs";
import * as $decode from "../../../gleam_stdlib/gleam/dynamic/decode.mjs";
import * as $function from "../../../gleam_stdlib/gleam/function.mjs";
import { toList, CustomType as $CustomType } from "../../gleam.mjs";
import * as $patch from "../../lustre/vdom/patch.mjs";
import * as $vnode from "../../lustre/vdom/vnode.mjs";

export class Mount extends $CustomType {
  constructor(kind, open_shadow_root, will_adopt_styles, observed_attributes, observed_properties, requested_contexts, provided_contexts, vdom, memos) {
    super();
    this.kind = kind;
    this.open_shadow_root = open_shadow_root;
    this.will_adopt_styles = will_adopt_styles;
    this.observed_attributes = observed_attributes;
    this.observed_properties = observed_properties;
    this.requested_contexts = requested_contexts;
    this.provided_contexts = provided_contexts;
    this.vdom = vdom;
    this.memos = memos;
  }
}
export const ClientMessage$Mount = (kind, open_shadow_root, will_adopt_styles, observed_attributes, observed_properties, requested_contexts, provided_contexts, vdom, memos) =>
  new Mount(kind,
  open_shadow_root,
  will_adopt_styles,
  observed_attributes,
  observed_properties,
  requested_contexts,
  provided_contexts,
  vdom,
  memos);
export const ClientMessage$isMount = (value) => value instanceof Mount;
export const ClientMessage$Mount$kind = (value) => value.kind;
export const ClientMessage$Mount$0 = (value) => value.kind;
export const ClientMessage$Mount$open_shadow_root = (value) =>
  value.open_shadow_root;
export const ClientMessage$Mount$1 = (value) => value.open_shadow_root;
export const ClientMessage$Mount$will_adopt_styles = (value) =>
  value.will_adopt_styles;
export const ClientMessage$Mount$2 = (value) => value.will_adopt_styles;
export const ClientMessage$Mount$observed_attributes = (value) =>
  value.observed_attributes;
export const ClientMessage$Mount$3 = (value) => value.observed_attributes;
export const ClientMessage$Mount$observed_properties = (value) =>
  value.observed_properties;
export const ClientMessage$Mount$4 = (value) => value.observed_properties;
export const ClientMessage$Mount$requested_contexts = (value) =>
  value.requested_contexts;
export const ClientMessage$Mount$5 = (value) => value.requested_contexts;
export const ClientMessage$Mount$provided_contexts = (value) =>
  value.provided_contexts;
export const ClientMessage$Mount$6 = (value) => value.provided_contexts;
export const ClientMessage$Mount$vdom = (value) => value.vdom;
export const ClientMessage$Mount$7 = (value) => value.vdom;
export const ClientMessage$Mount$memos = (value) => value.memos;
export const ClientMessage$Mount$8 = (value) => value.memos;

export class Reconcile extends $CustomType {
  constructor(kind, patch, memos) {
    super();
    this.kind = kind;
    this.patch = patch;
    this.memos = memos;
  }
}
export const ClientMessage$Reconcile = (kind, patch, memos) =>
  new Reconcile(kind, patch, memos);
export const ClientMessage$isReconcile = (value) => value instanceof Reconcile;
export const ClientMessage$Reconcile$kind = (value) => value.kind;
export const ClientMessage$Reconcile$0 = (value) => value.kind;
export const ClientMessage$Reconcile$patch = (value) => value.patch;
export const ClientMessage$Reconcile$1 = (value) => value.patch;
export const ClientMessage$Reconcile$memos = (value) => value.memos;
export const ClientMessage$Reconcile$2 = (value) => value.memos;

export class Emit extends $CustomType {
  constructor(kind, name, data) {
    super();
    this.kind = kind;
    this.name = name;
    this.data = data;
  }
}
export const ClientMessage$Emit = (kind, name, data) =>
  new Emit(kind, name, data);
export const ClientMessage$isEmit = (value) => value instanceof Emit;
export const ClientMessage$Emit$kind = (value) => value.kind;
export const ClientMessage$Emit$0 = (value) => value.kind;
export const ClientMessage$Emit$name = (value) => value.name;
export const ClientMessage$Emit$1 = (value) => value.name;
export const ClientMessage$Emit$data = (value) => value.data;
export const ClientMessage$Emit$2 = (value) => value.data;

export class Provide extends $CustomType {
  constructor(kind, key, value) {
    super();
    this.kind = kind;
    this.key = key;
    this.value = value;
  }
}
export const ClientMessage$Provide = (kind, key, value) =>
  new Provide(kind, key, value);
export const ClientMessage$isProvide = (value) => value instanceof Provide;
export const ClientMessage$Provide$kind = (value) => value.kind;
export const ClientMessage$Provide$0 = (value) => value.kind;
export const ClientMessage$Provide$key = (value) => value.key;
export const ClientMessage$Provide$1 = (value) => value.key;
export const ClientMessage$Provide$value = (value) => value.value;
export const ClientMessage$Provide$2 = (value) => value.value;

export class Subscribe extends $CustomType {
  constructor(kind, key) {
    super();
    this.kind = kind;
    this.key = key;
  }
}
export const ClientMessage$Subscribe = (kind, key) => new Subscribe(kind, key);
export const ClientMessage$isSubscribe = (value) => value instanceof Subscribe;
export const ClientMessage$Subscribe$kind = (value) => value.kind;
export const ClientMessage$Subscribe$0 = (value) => value.kind;
export const ClientMessage$Subscribe$key = (value) => value.key;
export const ClientMessage$Subscribe$1 = (value) => value.key;

export class Unsubscribe extends $CustomType {
  constructor(kind, key) {
    super();
    this.kind = kind;
    this.key = key;
  }
}
export const ClientMessage$Unsubscribe = (kind, key) =>
  new Unsubscribe(kind, key);
export const ClientMessage$isUnsubscribe = (value) =>
  value instanceof Unsubscribe;
export const ClientMessage$Unsubscribe$kind = (value) => value.kind;
export const ClientMessage$Unsubscribe$0 = (value) => value.kind;
export const ClientMessage$Unsubscribe$key = (value) => value.key;
export const ClientMessage$Unsubscribe$1 = (value) => value.key;

export const ClientMessage$kind = (value) => value.kind;

export class Batch extends $CustomType {
  constructor(kind, messages) {
    super();
    this.kind = kind;
    this.messages = messages;
  }
}
export const ServerMessage$Batch = (kind, messages) =>
  new Batch(kind, messages);
export const ServerMessage$isBatch = (value) => value instanceof Batch;
export const ServerMessage$Batch$kind = (value) => value.kind;
export const ServerMessage$Batch$0 = (value) => value.kind;
export const ServerMessage$Batch$messages = (value) => value.messages;
export const ServerMessage$Batch$1 = (value) => value.messages;

export class AttributeChanged extends $CustomType {
  constructor(kind, name, value) {
    super();
    this.kind = kind;
    this.name = name;
    this.value = value;
  }
}
export const ServerMessage$AttributeChanged = (kind, name, value) =>
  new AttributeChanged(kind, name, value);
export const ServerMessage$isAttributeChanged = (value) =>
  value instanceof AttributeChanged;
export const ServerMessage$AttributeChanged$kind = (value) => value.kind;
export const ServerMessage$AttributeChanged$0 = (value) => value.kind;
export const ServerMessage$AttributeChanged$name = (value) => value.name;
export const ServerMessage$AttributeChanged$1 = (value) => value.name;
export const ServerMessage$AttributeChanged$value = (value) => value.value;
export const ServerMessage$AttributeChanged$2 = (value) => value.value;

export class PropertyChanged extends $CustomType {
  constructor(kind, name, value) {
    super();
    this.kind = kind;
    this.name = name;
    this.value = value;
  }
}
export const ServerMessage$PropertyChanged = (kind, name, value) =>
  new PropertyChanged(kind, name, value);
export const ServerMessage$isPropertyChanged = (value) =>
  value instanceof PropertyChanged;
export const ServerMessage$PropertyChanged$kind = (value) => value.kind;
export const ServerMessage$PropertyChanged$0 = (value) => value.kind;
export const ServerMessage$PropertyChanged$name = (value) => value.name;
export const ServerMessage$PropertyChanged$1 = (value) => value.name;
export const ServerMessage$PropertyChanged$value = (value) => value.value;
export const ServerMessage$PropertyChanged$2 = (value) => value.value;

export class EventFired extends $CustomType {
  constructor(kind, path, name, event) {
    super();
    this.kind = kind;
    this.path = path;
    this.name = name;
    this.event = event;
  }
}
export const ServerMessage$EventFired = (kind, path, name, event) =>
  new EventFired(kind, path, name, event);
export const ServerMessage$isEventFired = (value) =>
  value instanceof EventFired;
export const ServerMessage$EventFired$kind = (value) => value.kind;
export const ServerMessage$EventFired$0 = (value) => value.kind;
export const ServerMessage$EventFired$path = (value) => value.path;
export const ServerMessage$EventFired$1 = (value) => value.path;
export const ServerMessage$EventFired$name = (value) => value.name;
export const ServerMessage$EventFired$2 = (value) => value.name;
export const ServerMessage$EventFired$event = (value) => value.event;
export const ServerMessage$EventFired$3 = (value) => value.event;

export class ContextProvided extends $CustomType {
  constructor(kind, key, value) {
    super();
    this.kind = kind;
    this.key = key;
    this.value = value;
  }
}
export const ServerMessage$ContextProvided = (kind, key, value) =>
  new ContextProvided(kind, key, value);
export const ServerMessage$isContextProvided = (value) =>
  value instanceof ContextProvided;
export const ServerMessage$ContextProvided$kind = (value) => value.kind;
export const ServerMessage$ContextProvided$0 = (value) => value.kind;
export const ServerMessage$ContextProvided$key = (value) => value.key;
export const ServerMessage$ContextProvided$1 = (value) => value.key;
export const ServerMessage$ContextProvided$value = (value) => value.value;
export const ServerMessage$ContextProvided$2 = (value) => value.value;

export const ServerMessage$kind = (value) => value.kind;

export const mount_kind = 0;

export const reconcile_kind = 1;

export const emit_kind = 2;

export const provide_kind = 3;

export const subscribe_kind = 4;

export const unsubscribe_kind = 5;

export const attribute_changed_kind = 0;

export const event_fired_kind = 1;

export const property_changed_kind = 2;

export const batch_kind = 3;

export const context_provided_kind = 4;

export function mount(
  open_shadow_root,
  will_adopt_styles,
  observed_attributes,
  observed_properties,
  requested_contexts,
  provided_contexts,
  vdom,
  memos
) {
  return new Mount(
    mount_kind,
    open_shadow_root,
    will_adopt_styles,
    observed_attributes,
    observed_properties,
    requested_contexts,
    provided_contexts,
    vdom,
    memos,
  );
}

export function reconcile(patch, memos) {
  return new Reconcile(reconcile_kind, patch, memos);
}

export function emit(name, data) {
  return new Emit(emit_kind, name, data);
}

export function provide(key, value) {
  return new Provide(provide_kind, key, value);
}

export function subscribe(key) {
  return new Subscribe(subscribe_kind, key);
}

export function unsubscribe(key) {
  return new Unsubscribe(unsubscribe_kind, key);
}

export function attribute_changed(name, value) {
  return new AttributeChanged(attribute_changed_kind, name, value);
}

export function event_fired(path, name, event) {
  return new EventFired(event_fired_kind, path, name, event);
}

export function property_changed(name, value) {
  return new PropertyChanged(property_changed_kind, name, value);
}

export function batch(messages) {
  return new Batch(batch_kind, messages);
}

export function context_provided(key, value) {
  return new ContextProvided(context_provided_kind, key, value);
}

function unsubscribe_to_json(kind, key) {
  return $json.object(
    toList([["kind", $json.int(kind)], ["key", $json.string(key)]]),
  );
}

function subscribe_to_json(kind, key) {
  return $json.object(
    toList([["kind", $json.int(kind)], ["key", $json.string(key)]]),
  );
}

function provide_to_json(kind, key, value) {
  return $json.object(
    toList([
      ["kind", $json.int(kind)],
      ["key", $json.string(key)],
      ["value", value],
    ]),
  );
}

function emit_to_json(kind, name, data) {
  return $json.object(
    toList([
      ["kind", $json.int(kind)],
      ["name", $json.string(name)],
      ["data", data],
    ]),
  );
}

function reconcile_to_json(kind, patch, memos) {
  return $json.object(
    toList([["kind", $json.int(kind)], ["patch", $patch.to_json(patch, memos)]]),
  );
}

function mount_to_json(
  kind,
  open_shadow_root,
  will_adopt_styles,
  observed_attributes,
  observed_properties,
  requested_contexts,
  provided_contexts,
  vdom,
  memos
) {
  return $json.object(
    toList([
      ["kind", $json.int(kind)],
      ["open_shadow_root", $json.bool(open_shadow_root)],
      ["will_adopt_styles", $json.bool(will_adopt_styles)],
      ["observed_attributes", $json.array(observed_attributes, $json.string)],
      ["observed_properties", $json.array(observed_properties, $json.string)],
      ["requested_contexts", $json.array(requested_contexts, $json.string)],
      [
        "provided_contexts",
        $json.dict(provided_contexts, $function.identity, $function.identity),
      ],
      ["vdom", $vnode.to_json(vdom, memos)],
    ]),
  );
}

export function client_message_to_json(message) {
  if (message instanceof Mount) {
    let kind = message.kind;
    let open_shadow_root = message.open_shadow_root;
    let will_adopt_styles = message.will_adopt_styles;
    let observed_attributes = message.observed_attributes;
    let observed_properties = message.observed_properties;
    let requested_contexts = message.requested_contexts;
    let provided_contexts = message.provided_contexts;
    let vdom = message.vdom;
    let memos = message.memos;
    return mount_to_json(
      kind,
      open_shadow_root,
      will_adopt_styles,
      observed_attributes,
      observed_properties,
      requested_contexts,
      provided_contexts,
      vdom,
      memos,
    );
  } else if (message instanceof Reconcile) {
    let kind = message.kind;
    let patch = message.patch;
    let memos = message.memos;
    return reconcile_to_json(kind, patch, memos);
  } else if (message instanceof Emit) {
    let kind = message.kind;
    let name = message.name;
    let data = message.data;
    return emit_to_json(kind, name, data);
  } else if (message instanceof Provide) {
    let kind = message.kind;
    let key = message.key;
    let value = message.value;
    return provide_to_json(kind, key, value);
  } else if (message instanceof Subscribe) {
    let kind = message.kind;
    let key = message.key;
    return subscribe_to_json(kind, key);
  } else {
    let kind = message.kind;
    let key = message.key;
    return unsubscribe_to_json(kind, key);
  }
}

function event_fired_decoder() {
  return $decode.field(
    "path",
    $decode.string,
    (path) => {
      return $decode.field(
        "name",
        $decode.string,
        (name) => {
          return $decode.field(
            "event",
            $decode.dynamic,
            (event) => {
              return $decode.success(event_fired(path, name, event));
            },
          );
        },
      );
    },
  );
}

function property_changed_decoder() {
  return $decode.field(
    "name",
    $decode.string,
    (name) => {
      return $decode.field(
        "value",
        $decode.dynamic,
        (value) => { return $decode.success(property_changed(name, value)); },
      );
    },
  );
}

function attribute_changed_decoder() {
  return $decode.field(
    "name",
    $decode.string,
    (name) => {
      return $decode.field(
        "value",
        $decode.string,
        (value) => { return $decode.success(attribute_changed(name, value)); },
      );
    },
  );
}

function batch_decoder() {
  return $decode.field(
    "messages",
    $decode.list(server_message_decoder()),
    (messages) => { return $decode.success(batch(messages)); },
  );
}

export function server_message_decoder() {
  return $decode.field(
    "kind",
    $decode.int,
    (kind) => {
      if (kind === 0) {
        return attribute_changed_decoder();
      } else if (kind === 2) {
        return property_changed_decoder();
      } else if (kind === 1) {
        return event_fired_decoder();
      } else if (kind === 3) {
        return batch_decoder();
      } else {
        return $decode.failure(batch(toList([])), "");
      }
    },
  );
}

export function context_provided_decoder() {
  return $decode.field(
    "key",
    $decode.string,
    (key) => {
      return $decode.field(
        "value",
        $decode.dynamic,
        (value) => { return $decode.success(context_provided(key, value)); },
      );
    },
  );
}
