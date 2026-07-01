import * as $atom from "../../../gleam_erlang/gleam/erlang/atom.mjs";
import * as $process from "../../../gleam_erlang/gleam/erlang/process.mjs";
import * as $dynamic from "../../../gleam_stdlib/gleam/dynamic.mjs";
import * as $list from "../../../gleam_stdlib/gleam/list.mjs";
import { Ok, toList, prepend as listPrepend, CustomType as $CustomType } from "../../gleam.mjs";
import * as $actor from "../../gleam/otp/actor.mjs";
import * as $supervision from "../../gleam/otp/supervision.mjs";

class Supervisor extends $CustomType {
  constructor(pid) {
    super();
    this.pid = pid;
  }
}

/**
 * If one child process terminates and is to be restarted, only that child
 * process is affected. This is the default restart strategy.
 */
export class OneForOne extends $CustomType {}
export const Strategy$OneForOne = () => new OneForOne();
export const Strategy$isOneForOne = (value) => value instanceof OneForOne;

/**
 * If one child process terminates and is to be restarted, all other child
 * processes are terminated and then all child processes are restarted.
 */
export class OneForAll extends $CustomType {}
export const Strategy$OneForAll = () => new OneForAll();
export const Strategy$isOneForAll = (value) => value instanceof OneForAll;

/**
 * If one child process terminates and is to be restarted, the 'rest' of the
 * child processes (that is, the child processes after the terminated child
 * process in the start order) are terminated. Then the terminated child
 * process and all child processes after it are restarted.
 */
export class RestForOne extends $CustomType {}
export const Strategy$RestForOne = () => new RestForOne();
export const Strategy$isRestForOne = (value) => value instanceof RestForOne;

/**
 * Automic shutdown is disabled. This is the default setting.
 *
 * With auto_shutdown set to never, child specs with the significant flag
 * set to true are considered invalid and will be rejected.
 */
export class Never extends $CustomType {}
export const AutoShutdown$Never = () => new Never();
export const AutoShutdown$isNever = (value) => value instanceof Never;

/**
 * The supervisor will shut itself down when any significant child
 * terminates, that is, when a transient significant child terminates
 * normally or when a temporary significant child terminates normally or
 * abnormally.
 */
export class AnySignificant extends $CustomType {}
export const AutoShutdown$AnySignificant = () => new AnySignificant();
export const AutoShutdown$isAnySignificant = (value) =>
  value instanceof AnySignificant;

/**
 * The supervisor will shut itself down when all significant children have
 * terminated, that is, when the last active significant child terminates.
 * The same rules as for any_significant apply.
 */
export class AllSignificant extends $CustomType {}
export const AutoShutdown$AllSignificant = () => new AllSignificant();
export const AutoShutdown$isAllSignificant = (value) =>
  value instanceof AllSignificant;

class Builder extends $CustomType {
  constructor(strategy, intensity, period, auto_shutdown, children) {
    super();
    this.strategy = strategy;
    this.intensity = intensity;
    this.period = period;
    this.auto_shutdown = auto_shutdown;
    this.children = children;
  }
}

class Strategy extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}

class Intensity extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}

class Period extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}

class AutoShutdown extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}

class Id extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}

class Start extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}

class Restart extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}

class Significant extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}

class Type extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}

class Shutdown extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}

/**
 * Create a new supervisor builder, ready for further configuration.
 */
export function new$(strategy) {
  return new Builder(strategy, 2, 5, new Never(), toList([]));
}

/**
 * To prevent a supervisor from getting into an infinite loop of child
 * process terminations and restarts, a maximum restart tolerance is
 * defined using two integer values specified with keys intensity and
 * period in the above map. Assuming the values MaxR for intensity and MaxT
 * for period, then, if more than MaxR restarts occur within MaxT seconds,
 * the supervisor terminates all child processes and then itself. The
 * termination reason for the supervisor itself in that case will be
 * shutdown. 
 *
 * Intensity defaults to 2 and period defaults to 5.
 */
export function restart_tolerance(builder, intensity, period) {
  return new Builder(
    builder.strategy,
    intensity,
    period,
    builder.auto_shutdown,
    builder.children,
  );
}

/**
 * A supervisor can be configured to automatically shut itself down with
 * exit reason shutdown when significant children terminate.
 */
export function auto_shutdown(builder, value) {
  return new Builder(
    builder.strategy,
    builder.intensity,
    builder.period,
    value,
    builder.children,
  );
}

/**
 * Add a child to the supervisor.
 */
export function add(builder, child) {
  return new Builder(
    builder.strategy,
    builder.intensity,
    builder.period,
    builder.auto_shutdown,
    listPrepend(
      $supervision.map_data(child, (_) => { return undefined; }),
      builder.children,
    ),
  );
}

export function init(start_data) {
  return new Ok(start_data);
}

export function start_child_callback(start) {
  let $ = start();
  if ($ instanceof Ok) {
    let started = $[0];
    return new Ok(started.pid);
  } else {
    return $;
  }
}
