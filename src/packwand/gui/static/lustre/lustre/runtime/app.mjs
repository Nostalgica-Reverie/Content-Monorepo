import * as $process from "../../../gleam_erlang/gleam/erlang/process.mjs";
import * as $dict from "../../../gleam_stdlib/gleam/dict.mjs";
import * as $decode from "../../../gleam_stdlib/gleam/dynamic/decode.mjs";
import * as $list from "../../../gleam_stdlib/gleam/list.mjs";
import * as $option from "../../../gleam_stdlib/gleam/option.mjs";
import { CustomType as $CustomType } from "../../gleam.mjs";
import * as $effect from "../../lustre/effect.mjs";
import * as $constants from "../../lustre/internals/constants.mjs";
import * as $runtime from "../../lustre/runtime/server/runtime.mjs";
import * as $vnode from "../../lustre/vdom/vnode.mjs";

export class App extends $CustomType {
  constructor(name, init, update, view, config) {
    super();
    this.name = name;
    this.init = init;
    this.update = update;
    this.view = view;
    this.config = config;
  }
}
export const App$App = (name, init, update, view, config) =>
  new App(name, init, update, view, config);
export const App$isApp = (value) => value instanceof App;
export const App$App$name = (value) => value.name;
export const App$App$0 = (value) => value.name;
export const App$App$init = (value) => value.init;
export const App$App$1 = (value) => value.init;
export const App$App$update = (value) => value.update;
export const App$App$2 = (value) => value.update;
export const App$App$view = (value) => value.view;
export const App$App$3 = (value) => value.view;
export const App$App$config = (value) => value.config;
export const App$App$4 = (value) => value.config;

export class Config extends $CustomType {
  constructor(open_shadow_root, adopt_styles, delegates_focus, attributes, properties, contexts, is_form_associated, on_form_autofill, on_form_reset, on_form_restore, on_form_disabled, on_connect, on_adopt, on_disconnect) {
    super();
    this.open_shadow_root = open_shadow_root;
    this.adopt_styles = adopt_styles;
    this.delegates_focus = delegates_focus;
    this.attributes = attributes;
    this.properties = properties;
    this.contexts = contexts;
    this.is_form_associated = is_form_associated;
    this.on_form_autofill = on_form_autofill;
    this.on_form_reset = on_form_reset;
    this.on_form_restore = on_form_restore;
    this.on_form_disabled = on_form_disabled;
    this.on_connect = on_connect;
    this.on_adopt = on_adopt;
    this.on_disconnect = on_disconnect;
  }
}
export const Config$Config = (open_shadow_root, adopt_styles, delegates_focus, attributes, properties, contexts, is_form_associated, on_form_autofill, on_form_reset, on_form_restore, on_form_disabled, on_connect, on_adopt, on_disconnect) =>
  new Config(open_shadow_root,
  adopt_styles,
  delegates_focus,
  attributes,
  properties,
  contexts,
  is_form_associated,
  on_form_autofill,
  on_form_reset,
  on_form_restore,
  on_form_disabled,
  on_connect,
  on_adopt,
  on_disconnect);
export const Config$isConfig = (value) => value instanceof Config;
export const Config$Config$open_shadow_root = (value) => value.open_shadow_root;
export const Config$Config$0 = (value) => value.open_shadow_root;
export const Config$Config$adopt_styles = (value) => value.adopt_styles;
export const Config$Config$1 = (value) => value.adopt_styles;
export const Config$Config$delegates_focus = (value) => value.delegates_focus;
export const Config$Config$2 = (value) => value.delegates_focus;
export const Config$Config$attributes = (value) => value.attributes;
export const Config$Config$3 = (value) => value.attributes;
export const Config$Config$properties = (value) => value.properties;
export const Config$Config$4 = (value) => value.properties;
export const Config$Config$contexts = (value) => value.contexts;
export const Config$Config$5 = (value) => value.contexts;
export const Config$Config$is_form_associated = (value) =>
  value.is_form_associated;
export const Config$Config$6 = (value) => value.is_form_associated;
export const Config$Config$on_form_autofill = (value) => value.on_form_autofill;
export const Config$Config$7 = (value) => value.on_form_autofill;
export const Config$Config$on_form_reset = (value) => value.on_form_reset;
export const Config$Config$8 = (value) => value.on_form_reset;
export const Config$Config$on_form_restore = (value) => value.on_form_restore;
export const Config$Config$9 = (value) => value.on_form_restore;
export const Config$Config$on_form_disabled = (value) => value.on_form_disabled;
export const Config$Config$10 = (value) => value.on_form_disabled;
export const Config$Config$on_connect = (value) => value.on_connect;
export const Config$Config$11 = (value) => value.on_connect;
export const Config$Config$on_adopt = (value) => value.on_adopt;
export const Config$Config$12 = (value) => value.on_adopt;
export const Config$Config$on_disconnect = (value) => value.on_disconnect;
export const Config$Config$13 = (value) => value.on_disconnect;

export class Option extends $CustomType {
  constructor(apply) {
    super();
    this.apply = apply;
  }
}
export const Option$Option = (apply) => new Option(apply);
export const Option$isOption = (value) => value instanceof Option;
export const Option$Option$apply = (value) => value.apply;
export const Option$Option$0 = (value) => value.apply;

export const default_config = /* @__PURE__ */ new Config(
  true,
  true,
  false,
  $constants.empty_list,
  $constants.empty_list,
  $constants.empty_list,
  false,
  /* @__PURE__ */ new $option.None(),
  /* @__PURE__ */ new $option.None(),
  /* @__PURE__ */ new $option.None(),
  /* @__PURE__ */ new $option.None(),
  /* @__PURE__ */ new $option.None(),
  /* @__PURE__ */ new $option.None(),
  /* @__PURE__ */ new $option.None(),
);

export function configure(options) {
  return $list.fold(
    options,
    default_config,
    (config, option) => { return option.apply(config); },
  );
}

export function configure_server_component(config) {
  return new $runtime.Config(
    config.open_shadow_root,
    config.adopt_styles,
    $dict.from_list($list.reverse(config.attributes)),
    $dict.from_list($list.reverse(config.properties)),
    $dict.from_list($list.reverse(config.contexts)),
    config.on_connect,
    config.on_disconnect,
  );
}
