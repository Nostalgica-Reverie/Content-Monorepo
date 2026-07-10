import { CustomType as $CustomType } from "../../../gleam.mjs";

export class Ok extends $CustomType {
  constructor($0, $1) {
    super();
    this[0] = $0;
    this[1] = $1;
  }
}
export const Result2$Ok = ($0, $1) => new Ok($0, $1);
export const Result2$isOk = (value) => value instanceof Ok;
export const Result2$Ok$0 = (value) => value[0];
export const Result2$Ok$1 = (value) => value[1];

export class Error extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Result2$Error = ($0) => new Error($0);
export const Result2$isError = (value) => value instanceof Error;
export const Result2$Error$0 = (value) => value[0];
