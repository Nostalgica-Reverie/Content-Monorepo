import * as $string_tree from "../../gleam_stdlib/gleam/string_tree.mjs";
import { toList, Empty as $Empty, CustomType as $CustomType } from "../gleam.mjs";
import * as $attribute from "../lustre/attribute.mjs";
import * as $mutable_map from "../lustre/internals/mutable_map.mjs";
import * as $ref from "../lustre/internals/ref.mjs";
import * as $vnode from "../lustre/vdom/vnode.mjs";
import { Element, Fragment, Map, Memo, UnsafeInnerHtml } from "../lustre/vdom/vnode.mjs";

class Html extends $CustomType {}

class HeadOnly extends $CustomType {}

class BodyOnly extends $CustomType {}

class HeadAndBody extends $CustomType {}

class Other extends $CustomType {}

/**
 * A general function for constructing any kind of element. In most cases you
 * will want to use the [`lustre/element/html`](./element/html.html) instead but this
 * function is particularly handy when constructing custom elements, either
 * from your own Lustre components or from external JavaScript libraries.
 *
 * > **Note**: Because Lustre is primarily used to create HTML, this function
 * > special-cases the following tags which render as
 * > [void elements](https://developer.mozilla.org/en-US/docs/Glossary/Void_element):
 * >
 * >   - area
 * >   - base
 * >   - br
 * >   - col
 * >   - embed
 * >   - hr
 * >   - img
 * >   - input
 * >   - link
 * >   - meta
 * >   - param
 * >   - source
 * >   - track
 * >   - wbr
 * >
 * > This will only affect the output of `to_string` and `to_string_builder`!
 * > If you need to render any of these tags with children, *or* you want to
 * > render some other tag as self-closing or void, use [`advanced`](#advanced)
 * > to construct the element instead.
 */
export function element(tag, attributes, children) {
  return $vnode.element(
    "",
    "",
    tag,
    attributes,
    children,
    $mutable_map.new$(),
    false,
    $vnode.is_void_html_element(tag, ""),
  );
}

/**
 * A function for constructing elements in a specific XML namespace. This can
 * be used to construct SVG or MathML elements, for example.
 */
export function namespaced(namespace, tag, attributes, children) {
  return $vnode.element(
    "",
    namespace,
    tag,
    attributes,
    children,
    $mutable_map.new$(),
    false,
    $vnode.is_void_html_element(tag, namespace),
  );
}

/**
 * A function for constructing elements with more control over how the element
 * is rendered when converted to a string. This is necessary because some HTML,
 * SVG, and MathML elements are self-closing or void elements, and Lustre needs
 * to know how to render them correctly!
 */
export function advanced(
  namespace,
  tag,
  attributes,
  children,
  self_closing,
  void$
) {
  return $vnode.element(
    "",
    namespace,
    tag,
    attributes,
    children,
    $mutable_map.new$(),
    self_closing,
    void$,
  );
}

/**
 * A function for turning a Gleam string into a text node. Gleam doesn't have
 * union types like some other languages you may be familiar with, like TypeScript.
 * Instead, we need a way to take a `String` and turn it into an `Element` somehow:
 * this function is exactly that!
 */
export function text(content) {
  return $vnode.text("", content);
}

/**
 * A function for rendering nothing. This is mostly useful for conditional
 * rendering, where you might want to render something only if a certain
 * condition is met.
 */
export function none() {
  return $vnode.text("", "");
}

/**
 * A function for constructing a wrapper element with no tag name. This is
 * useful for wrapping a list of elements together without adding an extra
 * `<div>` or other container element, or returning multiple elements in places
 * where only one `Element` is expected.
 */
export function fragment(children) {
  return $vnode.fragment("", children, $mutable_map.new$());
}

/**
 * A function for constructing a wrapper element with custom raw HTML as its
 * content. Lustre will render the provided HTML verbatim, and will not touch
 * its children except when replacing the entire inner html on changes.
 *
 * For HTML elements you can use an empty string for the namespace.
 *
 * > **Note:** The provided HTML will not be escaped automatically and may expose
 * > your applications to XSS attacks! Make sure you absolutely trust the HTML you
 * > pass to this function. In particular, never use this to display un-sanitised
 * > user HTML!
 */
export function unsafe_raw_html(namespace, tag, attributes, inner_html) {
  return $vnode.unsafe_inner_html("", namespace, tag, attributes, inner_html);
}

/**
 * A function for creating "memoised" or "lazy" elements. Lustre will use the
 * dependencies list to skip calling the provided view function if all of the
 * dependencies a _reference equal_ to their previous values.
 *
 * `memo` can be used to optimise performance-critical parts of your application,
 * for example in cases where many instances of the same element are rendered but
 * only one may change at a time, or cases where a part of your view may update
 * very frequently but other parts remain largely static. When Lustre can tell
 * that the dependencies haven't changed, almost all the work typically done to
 * update the DOM can be skipped.
 *
 * In many cases `memo` will not be necessary, so think twice before considering
 * its use! Lustre is designed to handle rerenders and large vdom trees efficiently,
 * so in most cases the naive approach of re-rendering everything will be perfectly
 * fine.
 *
 * > **Note**: reference equality is not the same as Gleam's normal equality.
 * > Two custom types with the same values are not reference equal unless they
 * > are the exact same instance in memory! Because of this, it's important to
 * > avoid list literals or constructing custom types in the dependencies list.
 *
 * > **Note**: memoisation comes with its own trade-offs and can cause performance
 * > regressions in two ways. First, every use of `memo` increases your application's
 * > memory usage slightly, as Lustre needs to keep dependencies around to compare
 * > them on subsequent renders. Second, if dependencies change regularly, the
 * > overhead of comparing dependencies and managing memoisation may be more than
 * > the naive cost of re-rendering the element each time.
 */
export function memo(dependencies, view) {
  return $vnode.memo("", dependencies, view);
}

/**
 * Create a `Ref` dependency value used for [`memo`](#memo) elements.
 *
 * Lustre uses reference equality to compare dependencies. On JavaScript, values
 * are compared using [same-value-zero](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Equality_comparisons_and_sameness#same-value-zero_equality)
 * semantics. This means Lustre will treat `+0` and `-0` as equal, and any errant
 * `NaN` values (which are not typically producible in Gleam code) as equal. On
 * Erlang, there is no difference between reference equality and value equality,
 * so all values are compared using normal equality semantics.
 */
export function ref(value) {
  return $ref.from(value);
}

/**
 * The `Element` type is parameterised by the type of messages it can produce
 * from events. Sometimes you might end up with a fragment of HTML from another
 * library or module that produces a different type of message: this function lets
 * you map the messages produced from one type to another.
 *
 * Think of it like `list.map` or `result.map` but for HTML events!
 */
export function map(element, f) {
  return $vnode.map(element, f);
}

/**
 * Convert a Lustre `Element` to a string. This is _not_ pretty-printed, so
 * there are no newlines or indentation. If you need to pretty-print an element,
 * reach out on the [Gleam Discord](https://discord.gg/Fm8Pwmy) or
 * [open an issue](https://github.com/lustre-labs/lustre/issues/new) with your
 * use case and we'll see what we can do!
 */
export function to_string(element) {
  return $vnode.to_string(element);
}

function get_document_type(loop$el) {
  while (true) {
    let el = loop$el;
    if (el instanceof Fragment) {
      let $ = el.children;
      if ($ instanceof $Empty) {
        return new Other();
      } else {
        let $1 = $.tail;
        if ($1 instanceof $Empty) {
          let child = $.head;
          loop$el = child;
        } else {
          let $2 = $1.tail;
          if ($2 instanceof $Empty) {
            let head = $.head;
            let body = $1.head;
            let $3 = get_document_type(head);
            let $4 = get_document_type(body);
            if ($3 instanceof HeadOnly && $4 instanceof BodyOnly) {
              return new HeadAndBody();
            } else {
              return new Other();
            }
          } else {
            return new Other();
          }
        }
      }
    } else if (el instanceof Element) {
      let $ = el.tag;
      if ($ === "html") {
        return new Html();
      } else if ($ === "head") {
        return new HeadOnly();
      } else if ($ === "body") {
        return new BodyOnly();
      } else {
        return new Other();
      }
    } else if (el instanceof UnsafeInnerHtml) {
      let $ = el.tag;
      if ($ === "html") {
        return new Html();
      } else if ($ === "head") {
        return new HeadOnly();
      } else if ($ === "body") {
        return new BodyOnly();
      } else {
        return new Other();
      }
    } else if (el instanceof Map) {
      let child = el.child;
      loop$el = child;
    } else if (el instanceof Memo) {
      let view = el.view;
      loop$el = view();
    } else {
      return new Other();
    }
  }
}

function wrap_document(el) {
  let $ = get_document_type(el);
  if ($ instanceof Html) {
    return el;
  } else if ($ instanceof HeadOnly) {
    return element("html", toList([]), toList([el]));
  } else if ($ instanceof BodyOnly) {
    return element("html", toList([]), toList([el]));
  } else if ($ instanceof HeadAndBody) {
    return element("html", toList([]), toList([el]));
  } else {
    return element(
      "html",
      toList([]),
      toList([element("body", toList([]), toList([el]))]),
    );
  }
}

/**
 * Converts an element to a string like [`to_string`](#to_string), but prepends
 * a `<!doctype html>` declaration to the string. This is useful for rendering
 * complete HTML documents.
 *
 * If the provided element is not an `html` element, it will be wrapped in both
 * a `html` and `body` element.
 */
export function to_document_string(el) {
  return "<!doctype html>\n" + $vnode.to_string(wrap_document(el));
}

/**
 * Convert a Lustre `Element` to a `StringTree`. This is _not_ pretty-printed,
 * so there are no newlines or indentation. If you need to pretty-print an element,
 * reach out on the [Gleam Discord](https://discord.gg/Fm8Pwmy) or
 * [open an issue](https://github.com/lustre-labs/lustre/issues/new) with your
 * use case and we'll see what we can do!
 */
export function to_string_tree(element) {
  return $vnode.to_string_tree(element, "");
}

/**
 * Converts an element to a `StringTree` like [`to_string_builder`](#to_string_builder),
 * but prepends a `<!doctype html>` declaration. This is useful for rendering
 * complete HTML documents.
 *
 * If the provided element is not an `html` element, it will be wrapped in both
 * a `html` and `body` element.
 */
export function to_document_string_tree(el) {
  let _pipe = $string_tree.from_string("<!doctype html>\n");
  return $string_tree.append_tree(
    _pipe,
    $vnode.to_string_tree(wrap_document(el), ""),
  );
}

/**
 * Converts a Lustre `Element` to a human-readable string by inserting new lines
 * and indentation where appropriate. This is useful for debugging and testing,
 * but for production code you should use [`to_string`](#to_string) or
 * [`to_document_string`](#to_document_string) instead.
 *
 * 💡 This function works great with the snapshot testing library
 *    [birdie](https://hexdocs.pm/birdie)!
 *
 * ## Using `to_string`:
 *
 * ```html
 * <header><h1>Hello, world!</h1></header>
 * ```
 *
 * ## Using `to_readable_string`
 *
 * ```html
 * <header>
 *   <h1>
 *     Hello, world!
 *   </h1>
 * </header>
 * ```
 */
export function to_readable_string(el) {
  return $vnode.to_snapshot(el, false);
}
