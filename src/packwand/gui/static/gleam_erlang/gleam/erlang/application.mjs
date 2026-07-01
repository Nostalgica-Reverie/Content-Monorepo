import { CustomType as $CustomType } from "../../gleam.mjs";
import * as $node from "../../gleam/erlang/node.mjs";

/**
 * A normal application start.
 */
export class Normal extends $CustomType {}
export const StartType$Normal = () => new Normal();
export const StartType$isNormal = (value) => value instanceof Normal;

/**
 * The application is distributed and started at the current node because of
 * a takeover from Node, either because Erlang's `application:takeover/2`
 * function has been called, or because the current node has higher priority
 * than the previous node.
 */
export class Takeover extends $CustomType {
  constructor(previous) {
    super();
    this.previous = previous;
  }
}
export const StartType$Takeover = (previous) => new Takeover(previous);
export const StartType$isTakeover = (value) => value instanceof Takeover;
export const StartType$Takeover$previous = (value) => value.previous;
export const StartType$Takeover$0 = (value) => value.previous;

/**
 * The application is distributed and started at the current node because of
 * a failover from the previous node.
 */
export class Failover extends $CustomType {
  constructor(previous) {
    super();
    this.previous = previous;
  }
}
export const StartType$Failover = (previous) => new Failover(previous);
export const StartType$isFailover = (value) => value instanceof Failover;
export const StartType$Failover$previous = (value) => value.previous;
export const StartType$Failover$0 = (value) => value.previous;
