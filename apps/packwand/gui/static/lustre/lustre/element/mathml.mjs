import { toList } from "../../gleam.mjs";
import * as $attribute from "../../lustre/attribute.mjs";
import * as $element from "../../lustre/element.mjs";
import { namespaced } from "../../lustre/element.mjs";

/**
 * The MathML namespace URI: `"http://www.w3.org/1998/Math/MathML"`. You might use this
 * with [`element.namespaced`](../element.html#namespaced) to create elements
 * in the MathML namespace not provided here.
 */
export const namespace = "http://www.w3.org/1998/Math/MathML";

/**
 *
 */
export function merror(attrs, children) {
  return namespaced(namespace, "merror", attrs, children);
}

/**
 *
 */
export function mphantom(attrs, children) {
  return namespaced(namespace, "mphantom", attrs, children);
}

/**
 *
 */
export function mprescripts(attrs, children) {
  return namespaced(namespace, "mprescripts", attrs, children);
}

/**
 *
 */
export function mrow(attrs, children) {
  return namespaced(namespace, "mrow", attrs, children);
}

/**
 *
 */
export function mstyle(attrs, children) {
  return namespaced(namespace, "mstyle", attrs, children);
}

/**
 *
 */
export function semantics(attrs, children) {
  return namespaced(namespace, "semantics", attrs, children);
}

/**
 *
 */
export function mmultiscripts(attrs, children) {
  return namespaced(namespace, "mmultiscripts", attrs, children);
}

/**
 *
 */
export function mover(attrs, children) {
  return namespaced(namespace, "mover", attrs, children);
}

/**
 *
 */
export function msub(attrs, children) {
  return namespaced(namespace, "msub", attrs, children);
}

/**
 *
 */
export function msubsup(attrs, children) {
  return namespaced(namespace, "msubsup", attrs, children);
}

/**
 *
 */
export function msup(attrs, children) {
  return namespaced(namespace, "msup", attrs, children);
}

/**
 *
 */
export function munder(attrs, children) {
  return namespaced(namespace, "munder", attrs, children);
}

/**
 *
 */
export function munderover(attrs, children) {
  return namespaced(namespace, "munderover", attrs, children);
}

/**
 *
 */
export function mroot(attrs, children) {
  return namespaced(namespace, "mroot", attrs, children);
}

/**
 *
 */
export function msqrt(attrs, children) {
  return namespaced(namespace, "msqrt", attrs, children);
}

/**
 *
 */
export function annotation(attrs, children) {
  return namespaced(namespace, "annotation", attrs, children);
}

/**
 *
 */
export function annotation_xml(attrs, children) {
  return namespaced(namespace, "annotation-xml", attrs, children);
}

/**
 *
 */
export function mfrac(attrs, children) {
  return namespaced(namespace, "mfrac", attrs, children);
}

/**
 *
 */
export function mn(attrs, text) {
  return namespaced(namespace, "mn", attrs, toList([$element.text(text)]));
}

/**
 *
 */
export function mo(attrs, text) {
  return namespaced(namespace, "mo", attrs, toList([$element.text(text)]));
}

/**
 *
 */
export function mi(attrs, text) {
  return namespaced(namespace, "mi", attrs, toList([$element.text(text)]));
}

/**
 *
 */
export function mpadded(attrs, children) {
  return namespaced(namespace, "mpadded", attrs, children);
}

/**
 *
 */
export function ms(attrs, text) {
  return namespaced(namespace, "ms", attrs, toList([$element.text(text)]));
}

/**
 *
 */
export function mspace(attrs) {
  return namespaced(namespace, "mspace", attrs, toList([]));
}

/**
 *
 */
export function mtable(attrs, children) {
  return namespaced(namespace, "mtable", attrs, children);
}

/**
 *
 */
export function mtd(attrs, children) {
  return namespaced(namespace, "mtd", attrs, children);
}

/**
 *
 */
export function mtext(attrs, text) {
  return namespaced(namespace, "mtext", attrs, toList([$element.text(text)]));
}

/**
 *
 */
export function mtr(attrs, children) {
  return namespaced(namespace, "mtr", attrs, children);
}
