import * as $process from "../../../../gleam_erlang/gleam/erlang/process.mjs";
import * as $json from "../../../../gleam_json/gleam/json.mjs";
import * as $actor from "../../../../gleam_otp/gleam/otp/actor.mjs";
import * as $dict from "../../../../gleam_stdlib/gleam/dict.mjs";
import * as $decode from "../../../../gleam_stdlib/gleam/dynamic/decode.mjs";
import * as $option from "../../../../gleam_stdlib/gleam/option.mjs";
import * as $set from "../../../../gleam_stdlib/gleam/set.mjs";
import { Error, CustomType as $CustomType } from "../../../gleam.mjs";
import * as $effect from "../../../lustre/effect.mjs";
import * as $transport from "../../../lustre/runtime/transport.mjs";
import * as $cache from "../../../lustre/vdom/cache.mjs";
import * as $vnode from "../../../lustre/vdom/vnode.mjs";

export class State extends $CustomType {
  constructor(self, selector, base_selector, model, update, view, config, vdom, cache, providers, subscribers, callbacks) {
    super();
    this.self = self;
    this.selector = selector;
    this.base_selector = base_selector;
    this.model = model;
    this.update = update;
    this.view = view;
    this.config = config;
    this.vdom = vdom;
    this.cache = cache;
    this.providers = providers;
    this.subscribers = subscribers;
    this.callbacks = callbacks;
  }
}
export const State$State = (self, selector, base_selector, model, update, view, config, vdom, cache, providers, subscribers, callbacks) =>
  new State(self,
  selector,
  base_selector,
  model,
  update,
  view,
  config,
  vdom,
  cache,
  providers,
  subscribers,
  callbacks);
export const State$isState = (value) => value instanceof State;
export const State$State$self = (value) => value.self;
export const State$State$0 = (value) => value.self;
export const State$State$selector = (value) => value.selector;
export const State$State$1 = (value) => value.selector;
export const State$State$base_selector = (value) => value.base_selector;
export const State$State$2 = (value) => value.base_selector;
export const State$State$model = (value) => value.model;
export const State$State$3 = (value) => value.model;
export const State$State$update = (value) => value.update;
export const State$State$4 = (value) => value.update;
export const State$State$view = (value) => value.view;
export const State$State$5 = (value) => value.view;
export const State$State$config = (value) => value.config;
export const State$State$6 = (value) => value.config;
export const State$State$vdom = (value) => value.vdom;
export const State$State$7 = (value) => value.vdom;
export const State$State$cache = (value) => value.cache;
export const State$State$8 = (value) => value.cache;
export const State$State$providers = (value) => value.providers;
export const State$State$9 = (value) => value.providers;
export const State$State$subscribers = (value) => value.subscribers;
export const State$State$10 = (value) => value.subscribers;
export const State$State$callbacks = (value) => value.callbacks;
export const State$State$11 = (value) => value.callbacks;

export class Config extends $CustomType {
  constructor(open_shadow_root, adopt_styles, attributes, properties, contexts, on_connect, on_disconnect) {
    super();
    this.open_shadow_root = open_shadow_root;
    this.adopt_styles = adopt_styles;
    this.attributes = attributes;
    this.properties = properties;
    this.contexts = contexts;
    this.on_connect = on_connect;
    this.on_disconnect = on_disconnect;
  }
}
export const Config$Config = (open_shadow_root, adopt_styles, attributes, properties, contexts, on_connect, on_disconnect) =>
  new Config(open_shadow_root,
  adopt_styles,
  attributes,
  properties,
  contexts,
  on_connect,
  on_disconnect);
export const Config$isConfig = (value) => value instanceof Config;
export const Config$Config$open_shadow_root = (value) => value.open_shadow_root;
export const Config$Config$0 = (value) => value.open_shadow_root;
export const Config$Config$adopt_styles = (value) => value.adopt_styles;
export const Config$Config$1 = (value) => value.adopt_styles;
export const Config$Config$attributes = (value) => value.attributes;
export const Config$Config$2 = (value) => value.attributes;
export const Config$Config$properties = (value) => value.properties;
export const Config$Config$3 = (value) => value.properties;
export const Config$Config$contexts = (value) => value.contexts;
export const Config$Config$4 = (value) => value.contexts;
export const Config$Config$on_connect = (value) => value.on_connect;
export const Config$Config$5 = (value) => value.on_connect;
export const Config$Config$on_disconnect = (value) => value.on_disconnect;
export const Config$Config$6 = (value) => value.on_disconnect;

export class ClientDispatchedMessage extends $CustomType {
  constructor(message) {
    super();
    this.message = message;
  }
}
export const Message$ClientDispatchedMessage = (message) =>
  new ClientDispatchedMessage(message);
export const Message$isClientDispatchedMessage = (value) =>
  value instanceof ClientDispatchedMessage;
export const Message$ClientDispatchedMessage$message = (value) => value.message;
export const Message$ClientDispatchedMessage$0 = (value) => value.message;

export class ClientRegisteredSubject extends $CustomType {
  constructor(client) {
    super();
    this.client = client;
  }
}
export const Message$ClientRegisteredSubject = (client) =>
  new ClientRegisteredSubject(client);
export const Message$isClientRegisteredSubject = (value) =>
  value instanceof ClientRegisteredSubject;
export const Message$ClientRegisteredSubject$client = (value) => value.client;
export const Message$ClientRegisteredSubject$0 = (value) => value.client;

export class ClientDeregisteredSubject extends $CustomType {
  constructor(client) {
    super();
    this.client = client;
  }
}
export const Message$ClientDeregisteredSubject = (client) =>
  new ClientDeregisteredSubject(client);
export const Message$isClientDeregisteredSubject = (value) =>
  value instanceof ClientDeregisteredSubject;
export const Message$ClientDeregisteredSubject$client = (value) => value.client;
export const Message$ClientDeregisteredSubject$0 = (value) => value.client;

export class ClientRegisteredCallback extends $CustomType {
  constructor(callback) {
    super();
    this.callback = callback;
  }
}
export const Message$ClientRegisteredCallback = (callback) =>
  new ClientRegisteredCallback(callback);
export const Message$isClientRegisteredCallback = (value) =>
  value instanceof ClientRegisteredCallback;
export const Message$ClientRegisteredCallback$callback = (value) =>
  value.callback;
export const Message$ClientRegisteredCallback$0 = (value) => value.callback;

export class ClientDeregisteredCallback extends $CustomType {
  constructor(callback) {
    super();
    this.callback = callback;
  }
}
export const Message$ClientDeregisteredCallback = (callback) =>
  new ClientDeregisteredCallback(callback);
export const Message$isClientDeregisteredCallback = (value) =>
  value instanceof ClientDeregisteredCallback;
export const Message$ClientDeregisteredCallback$callback = (value) =>
  value.callback;
export const Message$ClientDeregisteredCallback$0 = (value) => value.callback;

export class EffectAddedSelector extends $CustomType {
  constructor(selector) {
    super();
    this.selector = selector;
  }
}
export const Message$EffectAddedSelector = (selector) =>
  new EffectAddedSelector(selector);
export const Message$isEffectAddedSelector = (value) =>
  value instanceof EffectAddedSelector;
export const Message$EffectAddedSelector$selector = (value) => value.selector;
export const Message$EffectAddedSelector$0 = (value) => value.selector;

export class EffectDispatchedMessage extends $CustomType {
  constructor(message) {
    super();
    this.message = message;
  }
}
export const Message$EffectDispatchedMessage = (message) =>
  new EffectDispatchedMessage(message);
export const Message$isEffectDispatchedMessage = (value) =>
  value instanceof EffectDispatchedMessage;
export const Message$EffectDispatchedMessage$message = (value) => value.message;
export const Message$EffectDispatchedMessage$0 = (value) => value.message;

export class EffectEmitEvent extends $CustomType {
  constructor(name, data) {
    super();
    this.name = name;
    this.data = data;
  }
}
export const Message$EffectEmitEvent = (name, data) =>
  new EffectEmitEvent(name, data);
export const Message$isEffectEmitEvent = (value) =>
  value instanceof EffectEmitEvent;
export const Message$EffectEmitEvent$name = (value) => value.name;
export const Message$EffectEmitEvent$0 = (value) => value.name;
export const Message$EffectEmitEvent$data = (value) => value.data;
export const Message$EffectEmitEvent$1 = (value) => value.data;

export class EffectProvidedValue extends $CustomType {
  constructor(key, value) {
    super();
    this.key = key;
    this.value = value;
  }
}
export const Message$EffectProvidedValue = (key, value) =>
  new EffectProvidedValue(key, value);
export const Message$isEffectProvidedValue = (value) =>
  value instanceof EffectProvidedValue;
export const Message$EffectProvidedValue$key = (value) => value.key;
export const Message$EffectProvidedValue$0 = (value) => value.key;
export const Message$EffectProvidedValue$value = (value) => value.value;
export const Message$EffectProvidedValue$1 = (value) => value.value;

export class EffectRequestedContextSubscription extends $CustomType {
  constructor(key, decoder) {
    super();
    this.key = key;
    this.decoder = decoder;
  }
}
export const Message$EffectRequestedContextSubscription = (key, decoder) =>
  new EffectRequestedContextSubscription(key, decoder);
export const Message$isEffectRequestedContextSubscription = (value) =>
  value instanceof EffectRequestedContextSubscription;
export const Message$EffectRequestedContextSubscription$key = (value) =>
  value.key;
export const Message$EffectRequestedContextSubscription$0 = (value) =>
  value.key;
export const Message$EffectRequestedContextSubscription$decoder = (value) =>
  value.decoder;
export const Message$EffectRequestedContextSubscription$1 = (value) =>
  value.decoder;

export class EffectRemovedContextSubscription extends $CustomType {
  constructor(key) {
    super();
    this.key = key;
  }
}
export const Message$EffectRemovedContextSubscription = (key) =>
  new EffectRemovedContextSubscription(key);
export const Message$isEffectRemovedContextSubscription = (value) =>
  value instanceof EffectRemovedContextSubscription;
export const Message$EffectRemovedContextSubscription$key = (value) =>
  value.key;
export const Message$EffectRemovedContextSubscription$0 = (value) => value.key;

export class MonitorReportedDown extends $CustomType {
  constructor(monitor) {
    super();
    this.monitor = monitor;
  }
}
export const Message$MonitorReportedDown = (monitor) =>
  new MonitorReportedDown(monitor);
export const Message$isMonitorReportedDown = (value) =>
  value instanceof MonitorReportedDown;
export const Message$MonitorReportedDown$monitor = (value) => value.monitor;
export const Message$MonitorReportedDown$0 = (value) => value.monitor;

export class SystemRequestedShutdown extends $CustomType {}
export const Message$SystemRequestedShutdown = () =>
  new SystemRequestedShutdown();
export const Message$isSystemRequestedShutdown = (value) =>
  value instanceof SystemRequestedShutdown;

export function start(_, _1, _2, _3, _4, _5) {
  return new Error(new $actor.InitFailed("Not Erlang"));
}
