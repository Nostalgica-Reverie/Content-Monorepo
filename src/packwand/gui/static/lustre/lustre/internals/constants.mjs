import { Error, toList, prepend as listPrepend } from "../../gleam.mjs";

export const empty_list = /* @__PURE__ */ toList([]);

export const error_nil = /* @__PURE__ */ new Error(undefined);

export function singleton_list(item) {
  return listPrepend(item, empty_list);
}
