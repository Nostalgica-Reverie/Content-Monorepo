import * as $process from "../gleam_erlang/gleam/erlang/process.mjs";
import * as $actor from "../gleam_otp/gleam/otp/actor.mjs";
import * as $factory_supervisor from "../gleam_otp/gleam/otp/factory_supervisor.mjs";
import * as $supervision from "../gleam_otp/gleam/otp/supervision.mjs";
import * as $bool from "../gleam_stdlib/gleam/bool.mjs";
import { identity as hide_subject } from "../gleam_stdlib/gleam/function.mjs";
import * as $option from "../gleam_stdlib/gleam/option.mjs";
import { Error, CustomType as $CustomType } from "./gleam.mjs";
import * as $component from "./lustre/component.mjs";
import * as $effect from "./lustre/effect.mjs";
import * as $element from "./lustre/element.mjs";
import * as $app from "./lustre/runtime/app.mjs";
import { App } from "./lustre/runtime/app.mjs";
import { make_component as register } from "./lustre/runtime/client/component.ffi.mjs";
import { is_browser, send, is_registered } from "./lustre/runtime/client/runtime.ffi.mjs";
import { start as do_start } from "./lustre/runtime/client/spa.ffi.mjs";
import { start as start_server_component } from "./lustre/runtime/server/runtime.ffi.mjs";
import * as $runtime from "./lustre/runtime/server/runtime.mjs";

export { is_browser, is_registered, register, send, start_server_component };

export class ActorError extends $CustomType {
  constructor(reason) {
    super();
    this.reason = reason;
  }
}
export const Error$ActorError = (reason) => new ActorError(reason);
export const Error$isActorError = (value) => value instanceof ActorError;
export const Error$ActorError$reason = (value) => value.reason;
export const Error$ActorError$0 = (value) => value.reason;

export class BadComponentName extends $CustomType {
  constructor(name) {
    super();
    this.name = name;
  }
}
export const Error$BadComponentName = (name) => new BadComponentName(name);
export const Error$isBadComponentName = (value) =>
  value instanceof BadComponentName;
export const Error$BadComponentName$name = (value) => value.name;
export const Error$BadComponentName$0 = (value) => value.name;

export class ComponentAlreadyRegistered extends $CustomType {
  constructor(name) {
    super();
    this.name = name;
  }
}
export const Error$ComponentAlreadyRegistered = (name) =>
  new ComponentAlreadyRegistered(name);
export const Error$isComponentAlreadyRegistered = (value) =>
  value instanceof ComponentAlreadyRegistered;
export const Error$ComponentAlreadyRegistered$name = (value) => value.name;
export const Error$ComponentAlreadyRegistered$0 = (value) => value.name;

export class ElementNotFound extends $CustomType {
  constructor(selector) {
    super();
    this.selector = selector;
  }
}
export const Error$ElementNotFound = (selector) =>
  new ElementNotFound(selector);
export const Error$isElementNotFound = (value) =>
  value instanceof ElementNotFound;
export const Error$ElementNotFound$selector = (value) => value.selector;
export const Error$ElementNotFound$0 = (value) => value.selector;

export class NotABrowser extends $CustomType {}
export const Error$NotABrowser = () => new NotABrowser();
export const Error$isNotABrowser = (value) => value instanceof NotABrowser;

/**
 * A complete Lustre application that follows the Model-View-Update architecture
 * and can handle side effects like HTTP requests or querying the DOM. Most real
 * Lustre applications will use this constructor.
 *
 * To learn more about effects and their purpose, take a look at the
 * [`effect`](./lustre/effect.html) module or the
 * [HTTP requests example](https://github.com/lustre-labs/lustre/tree/main/examples/05-http-requests).
 */
export function application(init, update, view) {
  return new App(new $option.None(), init, update, view, $app.default_config);
}

/**
 * The simplest type of Lustre application. The `element` application is
 * primarily used for demonstration purposes. It renders a static Lustre `Element`
 * on the page and does not have any state or update logic.
 */
export function element(view) {
  return application(
    (_) => { return [undefined, $effect.none()]; },
    (_, _1) => { return [undefined, $effect.none()]; },
    (_) => { return view; },
  );
}

/**
 * A `simple` application has the basic Model-View-Update building blocks present
 * in all Lustre applications, but it cannot handle effects. This is a great way
 * to learn the basics of Lustre and its architecture.
 *
 * Once you're comfortable with the Model-View-Update loop and want to start
 * building more complex applications that can communicate with the outside world,
 * you'll want to use the [`application`](#application) constructor instead.
 */
export function simple(init, update, view) {
  let init$1 = (arguments$) => { return [init(arguments$), $effect.none()]; };
  let update$1 = (model, message) => {
    return [update(model, message), $effect.none()];
  };
  return application(init$1, update$1, view);
}

/**
 * A `component` is a type of Lustre application designed to be embedded within
 * another application and has its own encapsulated update loop. This constructor
 * is almost identical to the [`application`](#application) constructor, but it
 * also allows you to specify a dictionary of attribute names and decoders.
 *
 * When a component is rendered in a parent application, it can receive data from
 * the parent application through HTML attributes and properties just like any
 * other HTML element. This dictionary of decoders allows you to specify how to
 * decode those attributes into messages your component's update loop can handle.
 *
 * > **Note**: Lustre components take a bit more set up than components in JavaScript
 * > frameworks like React. They should be used for more complex UI widgets
 * > like a combobox with complex keyboard interactions rather than simple things
 * > like buttons or text inputs. Where possible try to think about how to build
 * > your UI with simple view functions (functions that return [Elements](./lustre/element.html#Element))
 * > and only reach for components when you really need to encapsulate that update
 * > loop.
 */
export function component(init, update, view, options) {
  return new App(
    new $option.None(),
    init,
    update,
    view,
    $app.configure(options),
  );
}

/**
 * Assign a [`Name`](https://hexdocs.pm/gleam_erlang/gleam/erlang/process.html#Name)
 * to a Lustre application. This is useful for [_supervised_](#supervised) server
 * components as it allows other processes to find and communicate with the
 * runtime even if it is restarted.
 *
 * > **Note**: names must **never** be created dynamically as too many names
 * > will exhaust the atom table and cause the VM to crash. Names should be
 * > created at the start of your program and passed down where needed.
 *
 * > **Note**: a named application should **never** be used to create a
 * > [factory supervisor](#factory) as only one process can be registered under
 * > a given name.
 */
export function named(app, name) {
  return new App(
    new $option.Some(name),
    app.init,
    app.update,
    app.view,
    app.config,
  );
}

/**
 * Start a constructed application as a client-side single-page application (SPA).
 * This is the most typical way to start a Lustre application and will *only* work
 * in the browser
 *
 * The second argument is a [CSS selector](https://developer.mozilla.org/en-US/docs/Web/API/Document/querySelector)
 * used to locate the DOM element where the application will be mounted on to.
 * The most common selectors are `"#app"` to target an element with an id of `app`
 * or `[data-lustre-app]` to target an element with a `data-lustre-app` attribute.
 *
 * The third argument is the starting data for the application. This is passed
 * to the application's `init` function.
 */
export function start(app, selector, arguments$) {
  return $bool.guard(
    !is_browser(),
    new Error(new NotABrowser()),
    () => { return do_start(app, selector, arguments$); },
  );
}

/**
 * Create a server component child specification suitable for supervision in a
 * [static supervisor](https://hexdocs.pm/gleam_otp/gleam/otp/static_supervisor.html).
 * This is the preferred way of starting Lustre server components on the Erlang
 * target.
 */
export function supervised(app, arguments$) {
  return $supervision.worker(
    () => {
      return $runtime.start(
        app.name,
        app.init,
        app.update,
        app.view,
        $app.configure_server_component(app.config),
        arguments$,
      );
    },
  );
}

/**
 * Create a [factory supervisor](https://hexdocs.pm/gleam_otp/gleam/otp/factory_supervisor.html)
 * capable of starting many instances of a Lustre server component dynamically.
 * Along with [`supervised`](#supervised), this is one of the ways to ensure
 * proper supervision and fault-tolerance for Lustre server components on the
 * Erlang target.
 */
export function factory(app) {
  return $factory_supervisor.worker_child(
    (arguments$) => {
      return $runtime.start(
        app.name,
        app.init,
        app.update,
        app.view,
        $app.configure_server_component(app.config),
        arguments$,
      );
    },
  );
}

/**
 * Build a message for a running application's `update` function.
 *
 * This message can be delivered to the runtime using [`send`](#send), allowing
 * communication with a Lustre app without having to use an effect.
 */
export function dispatch(message) {
  return new $runtime.EffectDispatchedMessage(message);
}

/**
 * Instruct a running application to shut down. For client SPAs this will stop
 * the runtime and unmount the app from the DOM. For server components, this will
 * stop the runtime and prevent any further patches from being sent to connected
 * clients.
 */
export function shutdown() {
  return new $runtime.SystemRequestedShutdown();
}
