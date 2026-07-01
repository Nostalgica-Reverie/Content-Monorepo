import { CustomType as $CustomType } from "../../gleam.mjs";
import * as $atom from "../../gleam/erlang/atom.mjs";

/**
 * Was unable to connect to the node.
 */
export class FailedToConnect extends $CustomType {}
export const ConnectError$FailedToConnect = () => new FailedToConnect();
export const ConnectError$isFailedToConnect = (value) =>
  value instanceof FailedToConnect;

/**
 * The local node is not alive, so it is not possible to connect to the other
 * node.
 */
export class LocalNodeIsNotAlive extends $CustomType {}
export const ConnectError$LocalNodeIsNotAlive = () => new LocalNodeIsNotAlive();
export const ConnectError$isLocalNodeIsNotAlive = (value) =>
  value instanceof LocalNodeIsNotAlive;
