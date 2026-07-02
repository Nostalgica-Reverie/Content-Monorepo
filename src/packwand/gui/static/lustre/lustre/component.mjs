import * as $dynamic from "../../gleam_stdlib/gleam/dynamic.mjs";
import * as $decode from "../../gleam_stdlib/gleam/dynamic/decode.mjs";
import * as $list from "../../gleam_stdlib/gleam/list.mjs";
import * as $option from "../../gleam_stdlib/gleam/option.mjs";
import { Some } from "../../gleam_stdlib/gleam/option.mjs";
import * as $string from "../../gleam_stdlib/gleam/string.mjs";
import { Ok, toList, Empty as $Empty, prepend as listPrepend } from "../gleam.mjs";
import * as $attribute from "../lustre/attribute.mjs";
import { attribute } from "../lustre/attribute.mjs";
import * as $effect from "../lustre/effect.mjs";
import * as $element from "../lustre/element.mjs";
import * as $html from "../lustre/element/html.mjs";
import * as $app from "../lustre/runtime/app.mjs";
import { Config, Option } from "../lustre/runtime/app.mjs";
import * as $vattr from "../lustre/vdom/vattr.mjs";
import { Attribute, Event, Property } from "../lustre/vdom/vattr.mjs";
import {
  set_form_value as do_set_form_value,
  clear_form_value as do_clear_form_value,
  set_pseudo_state as do_set_pseudo_state,
  remove_pseudo_state as do_remove_pseudo_state,
} from "./runtime/client/component.ffi.mjs";

/**
 * Register a decoder to run whenever the named attribute changes. Attributes
 * can be set in Lustre using the [`attribute`](./attribute.html#attribute)
 * function, set directly on the component's HTML tag, or in JavaScript using
 * the [`setAttribute`](https://developer.mozilla.org/en-US/docs/Web/API/Element/setAttribute)
 * method.
 *
 * Attributes are always strings, but your decoder is responsible for decoding
 * the string into a message that your component can understand.
 */
export function on_attribute_change(name, decoder) {
  return new Option(
    (config) => {
      let attributes = listPrepend([name, decoder], config.attributes);
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        config.delegates_focus,
        attributes,
        config.properties,
        config.contexts,
        config.is_form_associated,
        config.on_form_autofill,
        config.on_form_reset,
        config.on_form_restore,
        config.on_form_disabled,
        config.on_connect,
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * Register decoder to run whenever the given property is set on the component.
 * Properties can be set in Lustre using the [`property`](./attribute.html#property)
 * function or in JavaScript by setting a property directly on the component
 * object.
 *
 * Properties can be any JavaScript object. For server components, properties
 * will be any _JSON-serialisable_ value.
 */
export function on_property_change(name, decoder) {
  return new Option(
    (config) => {
      let properties = listPrepend([name, decoder], config.properties);
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        config.delegates_focus,
        config.attributes,
        properties,
        config.contexts,
        config.is_form_associated,
        config.on_form_autofill,
        config.on_form_reset,
        config.on_form_restore,
        config.on_form_disabled,
        config.on_connect,
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * Register a decoder to run whenever a parent component or application
 * [provides](./effect.html#provide) a new context value for the given `key`.
 * Contexts are a powerful feature that allow parents to inject data into
 * child components without knowledge of the DOM structurre, making them great
 * for advanced use-cases like design systems and flexible component hierarchies.
 *
 * Contexts can be any JavaScript object. For server components, contexts will
 * be any _JSON-serialisable_ value.
 */
export function on_context_change(key, decoder) {
  return new Option(
    (config) => {
      let contexts = listPrepend([key, decoder], config.contexts);
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        config.delegates_focus,
        config.attributes,
        config.properties,
        contexts,
        config.is_form_associated,
        config.on_form_autofill,
        config.on_form_reset,
        config.on_form_restore,
        config.on_form_disabled,
        config.on_connect,
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * Mark a component as "form-associated". This lets your component participate
 * in form submission and respond to additional form-specific events such as
 * the form being reset or the browser autofilling this component's value.
 *
 * > **Note**: form-associated components are not supported in server components
 * > for both technical and ideological reasons. If you'd like a component that
 * > participates in form submission, you should use a client component!
 */
export function form_associated() {
  return new Option(
    (config) => {
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        config.delegates_focus,
        config.attributes,
        config.properties,
        config.contexts,
        true,
        config.on_form_autofill,
        config.on_form_reset,
        config.on_form_restore,
        config.on_form_disabled,
        config.on_connect,
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * Register a callback that runs when the browser autofills this
 * [form-associated](#form_associated) component's `"value"` attribute. The
 * callback should convert the autofilled value into a message that you handle
 * in your `update` function.
 *
 * > **Note**: server components cannot participate in form submission and configuring
 * > this option will do nothing.
 */
export function on_form_autofill(handler) {
  return new Option(
    (config) => {
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        config.delegates_focus,
        config.attributes,
        config.properties,
        config.contexts,
        true,
        new Some(handler),
        config.on_form_reset,
        config.on_form_restore,
        config.on_form_disabled,
        config.on_connect,
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * Set a message to be dispatched whenever a form containing this
 * [form-associated](#form_associated) component is reset.
 *
 * > **Note**: server components cannot participate in form submission and configuring
 * > this option will do nothing.
 */
export function on_form_reset(message) {
  return new Option(
    (config) => {
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        config.delegates_focus,
        config.attributes,
        config.properties,
        config.contexts,
        true,
        config.on_form_autofill,
        new Some(message),
        config.on_form_restore,
        config.on_form_disabled,
        config.on_connect,
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * Set a callback that runs when the browser restores this
 * [form-associated](#form_associated) component's `"value"` attribute. This is
 * often triggered when the user navigates back or forward in their history.
 *
 * > **Note**: server components cannot participate in form submission and configuring
 * > this option will do nothing.
 */
export function on_form_restore(handler) {
  return new Option(
    (config) => {
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        config.delegates_focus,
        config.attributes,
        config.properties,
        config.contexts,
        true,
        config.on_form_autofill,
        config.on_form_reset,
        new Some(handler),
        config.on_form_disabled,
        config.on_connect,
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * Set a message to be dispatched whenever a form or fieldset containing this
 * [form-associated](#form_associated) component changes its disabled state.
 *
 * > **Note**: this event is not fired when the `"disabled"` attribute or
 * > property of the custom element itself changes. You must register the
 * > appropriate event handler separately.
 *
 * > **Note**: server components cannot participate in form submission and configuring
 * > this option will do nothing.
 */
export function on_form_disabled(handler) {
  return new Option(
    (config) => {
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        config.delegates_focus,
        config.attributes,
        config.properties,
        config.contexts,
        true,
        config.on_form_autofill,
        config.on_form_reset,
        config.on_form_restore,
        new Some(handler),
        config.on_connect,
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * Configure whether a component's [Shadow Root](https://developer.mozilla.org/en-US/docs/Web/API/ShadowRoot)
 * is open or closed. A closed shadow root means the elements rendered inside
 * the component are not accessible from JavaScript outside the component.
 *
 * By default a component's shadow root is **open**. You may want to configure
 * this option manually if you intend to build a component for use outside of
 * Lustre.
 */
export function open_shadow_root(open) {
  return new Option(
    (config) => {
      return new Config(
        open,
        config.adopt_styles,
        config.delegates_focus,
        config.attributes,
        config.properties,
        config.contexts,
        config.is_form_associated,
        config.on_form_autofill,
        config.on_form_reset,
        config.on_form_restore,
        config.on_form_disabled,
        config.on_connect,
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * Configure whether a component should attempt to adopt stylesheets from
 * its parent document. Components in Lustre use the shadow DOM to unlock native
 * web component features like slots, but this means elements rendered inside a
 * component are isolated from the document's styles.
 *
 * To get around this, Lustre can attempt to adopt all stylesheets from the
 * parent document when the component is first created; meaning in many cases
 * you can use the same CSS to style your components as you do the rest of your
 * application.
 *
 * By default, this option is **enabled**. You may want to disable this option
 * if you are building a component for use outside of Lustre and do not want
 * document styles to interfere with your component's styling
 */
export function adopt_styles(adopt) {
  return new Option(
    (config) => {
      return new Config(
        config.open_shadow_root,
        adopt,
        config.delegates_focus,
        config.attributes,
        config.properties,
        config.contexts,
        config.is_form_associated,
        config.on_form_autofill,
        config.on_form_reset,
        config.on_form_restore,
        config.on_form_disabled,
        config.on_connect,
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * Indicates whether or not this component should delegate focus to its children.
 * When set to `True`, a number of focus-related features are enabled:
 *
 * - Clicking on any non-interactive part of the component will automatically
 *   focus the first focusable child element.
 *
 * - The component can receive focus through the `.focus()` method or the
 *   `autofocus` attribute, and it will automatically focus the first
 *   focusable child element.
 *
 * - The component receives the `:focus` CSS pseudo-class when any of its
 *   focusable children have focus.
 *
 * By default this option is **disabled**. You may want to enable this option
 * when creating complex interactive widgets.
 */
export function delegates_focus(delegates) {
  return new Option(
    (config) => {
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        delegates,
        config.attributes,
        config.properties,
        config.contexts,
        config.is_form_associated,
        config.on_form_autofill,
        config.on_form_reset,
        config.on_form_restore,
        config.on_form_disabled,
        config.on_connect,
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * Set a message to be sent when a client component is connected to a document
 * or a server component registers a new connection.
 *
 * ## Client components
 *
 * The provided message will be dispatched when the component is connected to a
 * new document. This corresponds to the custom element `connectedCallback` and
 * is a good signal to perform effects that interact with the DOM or many browser
 * APIs.
 *
 * ## Server components
 *
 * The provided message will be dispatched when a new connection is registered
 * by either [`server_component.register_subject`](./server_component.html#register_subject)
 * or [`server_component.register_callback`](./server_component.html#register_callback).
 * Importantly, repeated calls to either of these functions will **not** trigger
 * the message multiple times.
 */
export function on_connect(message) {
  return new Option(
    (config) => {
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        config.delegates_focus,
        config.attributes,
        config.properties,
        config.contexts,
        config.is_form_associated,
        config.on_form_autofill,
        config.on_form_reset,
        config.on_form_restore,
        config.on_form_disabled,
        new Some(message),
        config.on_adopt,
        config.on_disconnect,
      );
    },
  );
}

/**
 * The message provided to this option will be dispatched whenever a client component
 * is adopted into a new document.
 *
 * > **Note**: this option is only useful for components that will be built and
 * > distributed outside of a typical Lustre application.
 */
export function on_adopt(message) {
  return new Option(
    (config) => {
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        config.delegates_focus,
        config.attributes,
        config.properties,
        config.contexts,
        config.is_form_associated,
        config.on_form_autofill,
        config.on_form_reset,
        config.on_form_restore,
        config.on_form_disabled,
        config.on_connect,
        new Some(message),
        config.on_disconnect,
      );
    },
  );
}

/**
 * Set a message to be sent when a client component is disconnected from a document
 * or a server component deregisters a connection.
 *
 * ## Client components
 *
 * The provided message will be dispatched when the component is disconnected from
 * a document, for example when the element is no longer rendered by your app's
 * `view` function. This corresponds to the custom element `disconnectedCallback`
 * and should be used to clean up any effects.
 *
 * ## Server components
 *
 * The provided message will be dispatched when a connection is deregistered by
 * either [`server_component.deregister_subject`](./server_component.html#deregister_subject)
 * or [`server_component.deregister_callback`](./server_component.html#deregister_callback).
 */
export function on_disconnect(message) {
  return new Option(
    (config) => {
      return new Config(
        config.open_shadow_root,
        config.adopt_styles,
        config.delegates_focus,
        config.attributes,
        config.properties,
        config.contexts,
        config.is_form_associated,
        config.on_form_autofill,
        config.on_form_reset,
        config.on_form_restore,
        config.on_form_disabled,
        config.on_connect,
        config.on_adopt,
        new Some(message),
      );
    },
  );
}

/**
 * Create a default slot for a component. Any elements rendered as children of
 * the component will be placed inside the default slot unless explicitly
 * redirected using the [`slot`](#slot) attribute.
 *
 * If no children are placed into the slot, the `fallback` elements will be
 * rendered instead.
 *
 * To learn more about Shadow DOM and slots, see this excellent guide:
 *
 *   - https://javascript.info/slots-composition
 */
export function default_slot(attributes, fallback) {
  return $html.slot(attributes, fallback);
}

/**
 * Create a named slot for a component. Any elements rendered as children of
 * the component with a [`slot`](#slot) attribute matching the `name` will be
 * rendered inside this slot.
 *
 * If no children are placed into the slot, the `fallback` elements will be
 * rendered instead.
 *
 * To learn more about Shadow DOM and slots, see this excellent guide:
 *
 *   - https://javascript.info/slots-composition
 */
export function named_slot(name, attributes, fallback) {
  return $html.slot(listPrepend(attribute("name", name), attributes), fallback);
}

/**
 * Lustre's component system is built on top the Custom Elements API and the
 * Shadow DOM API. A component's `view` function is rendered inside a shadow
 * root, which means the component's HTML is isolated from the rest of the
 * document.
 *
 * This can make it difficult to style components from CSS outside the component.
 * To help with this, the `part` attribute lets you expose parts of your component
 * by name to be styled by external CSS.
 *
 * For example, if the `view` function for a component called `"my-component`"
 * looks like this:
 *
 * ```gleam
 * import gleam/int
 * import lustre/component
 * import lustre/element/html
 *
 * fn view(model) {
 *   html.div([], [
 *     html.button([], [html.text("-")]),
 *     html.p([component.part("count")], [html.text(int.to_string(model.count))]),
 *     html.button([], [html.text("+")]),
 *   ])
 * }
 * ```
 *
 * Then the following CSS in the **parent** document can be used to style the
 * `<p>` element:
 *
 * ```css
 * my-component::part(count) {
 *   color: red;
 * }
 * ```
 *
 * To learn more about the CSS Shadow Parts specification, see:
 *
 *   - https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/part
 *
 *   - https://developer.mozilla.org/en-US/docs/Web/CSS/::part
 */
export function part(name) {
  return attribute("part", name);
}

function do_parts(loop$names, loop$part) {
  while (true) {
    let names = loop$names;
    let part = loop$part;
    if (names instanceof $Empty) {
      return part;
    } else {
      let $ = names.head[1];
      if ($) {
        let rest = names.tail;
        let name = names.head[0];
        return ((part + name) + " ") + do_parts(rest, part);
      } else {
        let rest = names.tail;
        loop$names = rest;
        loop$part = part;
      }
    }
  }
}

/**
 * A convenience function that makes it possible to toggle different parts on or
 * off in a single call. This is useful for example when you have a menu item
 * that may be active and you want to conditionally assign the `"active"` part:
 *
 * ```gleam
 * import lustre/component
 * import lustre/element/html
 *
 * fn view(item) {
 *   html.li(
 *     [
 *       component.parts([
 *         #("item", True)
 *         #("active", item.is_active)
 *       ]),
 *     ],
 *     [html.text(item.label)],
 *   ])
 * }
 * ```
 */
export function parts(names) {
  return part(do_parts(names, ""));
}

/**
 * While the [`part`](#part) attribute can be used to expose parts of a component
 * to its parent, these parts will not automatically become available to the
 * _document_ when components are nested inside each other.
 *
 * The `exportparts` attribute lets you forward the parts of a nested component
 * to the parent component so they can be styled from the parent document.
 *
 * Consider we have two components, `"my-component"` and `"my-nested-component"`
 * with the following `view` functions:
 *
 * ```gleam
 * import gleam/int
 * import lustre/attribute.{property}
 * import lustre/component
 * import lustre/element.{element}
 * import lustre/element/html
 *
 * fn my_component_view(model) {
 *   html.div([], [
 *     html.button([], [html.text("-")]),
 *     element(
 *       "my-nested-component",
 *       [
 *         property("count", model.count),
 *         component.exportparts(["count"]),
 *       ],
 *       []
 *     )
 *     html.button([], [html.text("+")]),
 *   ])
 * }
 *
 * fn my_nested_component_view(model) {
 *   html.p([component.part("count")], [html.text(int.to_string(model.count))])
 * }
 * ```
 *
 * The `<my-nested-component />` component has a part called `"count"` which the
 * `<my-component />` then forwards to the parent document using the `"exportparts"`
 * attribute. Now the following CSS can be used to style the `<p>` element nested
 * deep inside the `<my-component />`:
 *
 * ```css
 * my-component::part(count) {
 *   color: red;
 * }
 * ```
 *
 * Notice how the styles are applied to the `<my-component />` element, not the
 * `<my-nested-component />` element!
 *
 * To learn more about the CSS Shadow Parts specification, see:
 *
 *   - https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/exportparts
 *
 *   - https://developer.mozilla.org/en-US/docs/Web/CSS/::part
 */
export function exportparts(names) {
  return attribute("exportparts", $string.join(names, ", "));
}

/**
 * Associate an element with a [named slot](#named_slot) in a component. Multiple
 * elements can be associated with the same slot name.
 *
 * To learn more about Shadow DOM and slots, see:
 *
 *   - https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/slot
 *
 *   - https://javascript.info/slots-composition
 */
export function slot(name) {
  return attribute("slot", name);
}

/**
 * Set the value of a [form-associated component](#form_associated). If the
 * component is rendered inside a `<form>` element, the value will be
 * automatically included in the form submission and available in the form's
 * `FormData` object.
 */
export function set_form_value(value) {
  return $effect.before_paint(
    (_, root) => { return do_set_form_value(root, value); },
  );
}

/**
 * Clear a form value previously set with [`set_form_value`](#set_form_value).
 * When the form is submitted, this component's value will not be included in
 * the form data.
 */
export function clear_form_value() {
  return $effect.before_paint(
    (_, root) => { return do_clear_form_value(root); },
  );
}

/**
 * Set a custom state on the component. This state is not reflected in the DOM
 * but can be selected in CSS using the `:state` pseudo-class. For example,
 * calling `set_pseudo_state("checked")` on a component called `"my-checkbox"`
 * means the following CSS will apply:
 *
 * ```css
 * my-checkbox:state(checked) {
 *   border: solid;
 * }
 * ```
 *
 * If you are styling a component by rendering a `<style>` element _inside_ the
 * component, the previous CSS would be rewritten as:
 *
 * ```css
 * :host(:state(checked)) {
 *   border: solid;
 * }
 * ```
 */
export function set_pseudo_state(value) {
  return $effect.before_paint(
    (_, root) => { return do_set_pseudo_state(root, value); },
  );
}

/**
 * Remove a custom state set by [`set_pseudo_state`](#set_pseudo_state).
 */
export function remove_pseudo_state(value) {
  return $effect.before_paint(
    (_, root) => { return do_remove_pseudo_state(root, value); },
  );
}

/**
 * Prerender a component with a declarative shadow DOM. This is different to
 * just rendering the component's tag because it also renders the component's
 * internal `view`. Calling this when server-rendering a component allows components
 * to benefit from hydration by providing an initial HTML structure similar to
 * hydratation for client applications.
 *
 * If the component responds to attribute changes, the attributes passed here
 * will be applied before the component is rendered.
 *
 * To support both prerendering and client-side rendering, component authors
 * can use [`lustre.is_browser`](../lustre.html#is_browser) to detect the
 * environment and prerender the component where appropriate:
 *
 * ```gleam
 * import lustre.{type App}
 * import lustre/attribute.{type Attribute}
 * import lustre/component
 * import lustre/element.{type Element, element}
 *
 * pub fn element(
 *   attributes: List(Attribute(message)),
 *   children: List(Element(message))
 * ) -> Element(message) {
 *   case lustre.is_browser() {
 *     True -> element(tag, attributes, children)
 *     False -> component.prerender(component(), tag, attributes, children)
 *   }
 * }
 *
 * const tag = "my-component"
 *
 * fn component() -> App(Nil, Model, Message) {
 *   lustre.component(init:, update:, view:, options:)
 * }
 * ```
 */
export function prerender(component, tag, attributes, children) {
  let $ = $list.fold(
    attributes,
    component.init(undefined),
    (state, attribute) => {
      if (attribute instanceof Attribute) {
        let name = attribute.name;
        let value = attribute.value;
        let $1 = $list.key_find(component.config.attributes, name);
        if ($1 instanceof Ok) {
          let handler = $1[0];
          let $2 = handler(value);
          if ($2 instanceof Ok) {
            let message = $2[0];
            return component.update(state[0], message);
          } else {
            return state;
          }
        } else {
          return state;
        }
      } else if (attribute instanceof Property) {
        return state;
      } else {
        return state;
      }
    },
  );
  let model = $[0];
  let shadowrootmode = $attribute.shadowrootmode(
    (() => {
      let $1 = component.config.open_shadow_root;
      if ($1) {
        return "open";
      } else {
        return "closed";
      }
    })(),
  );
  let shadowrootdelegatesfocus = $attribute.shadowrootdelegatesfocus(
    component.config.delegates_focus,
  );
  return $element.element(
    tag,
    attributes,
    listPrepend(
      $html.template(
        toList([shadowrootmode, shadowrootdelegatesfocus]),
        toList([component.view(model)]),
      ),
      children,
    ),
  );
}
