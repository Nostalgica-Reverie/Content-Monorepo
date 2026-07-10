import { Ok, CustomType as $CustomType } from "../../gleam.mjs";
import * as $actor from "../../gleam/otp/actor.mjs";

/**
 * A permanent child process is always restarted.
 */
export class Permanent extends $CustomType {}
export const Restart$Permanent = () => new Permanent();
export const Restart$isPermanent = (value) => value instanceof Permanent;

/**
 * A transient child process is restarted only if it terminates abnormally,
 * that is, with another exit reason than `normal`, `shutdown`, or
 * `{shutdown,Term}`.
 */
export class Transient extends $CustomType {}
export const Restart$Transient = () => new Transient();
export const Restart$isTransient = (value) => value instanceof Transient;

/**
 * A temporary child process is never restarted (even when the supervisor's
 * restart strategy is `RestForOne` or `OneForAll` and a sibling's death
 * causes the temporary process to be terminated).
 */
export class Temporary extends $CustomType {}
export const Restart$Temporary = () => new Temporary();
export const Restart$isTemporary = (value) => value instanceof Temporary;

/**
 * A worker child has to shut-down within a given amount of time.
 */
export class Worker extends $CustomType {
  constructor(shutdown_ms) {
    super();
    this.shutdown_ms = shutdown_ms;
  }
}
export const ChildType$Worker = (shutdown_ms) => new Worker(shutdown_ms);
export const ChildType$isWorker = (value) => value instanceof Worker;
export const ChildType$Worker$shutdown_ms = (value) => value.shutdown_ms;
export const ChildType$Worker$0 = (value) => value.shutdown_ms;

export class Supervisor extends $CustomType {}
export const ChildType$Supervisor = () => new Supervisor();
export const ChildType$isSupervisor = (value) => value instanceof Supervisor;

export class ChildSpecification extends $CustomType {
  constructor(start, restart, significant, child_type) {
    super();
    this.start = start;
    this.restart = restart;
    this.significant = significant;
    this.child_type = child_type;
  }
}
export const ChildSpecification$ChildSpecification = (start, restart, significant, child_type) =>
  new ChildSpecification(start, restart, significant, child_type);
export const ChildSpecification$isChildSpecification = (value) =>
  value instanceof ChildSpecification;
export const ChildSpecification$ChildSpecification$start = (value) =>
  value.start;
export const ChildSpecification$ChildSpecification$0 = (value) => value.start;
export const ChildSpecification$ChildSpecification$restart = (value) =>
  value.restart;
export const ChildSpecification$ChildSpecification$1 = (value) => value.restart;
export const ChildSpecification$ChildSpecification$significant = (value) =>
  value.significant;
export const ChildSpecification$ChildSpecification$2 = (value) =>
  value.significant;
export const ChildSpecification$ChildSpecification$child_type = (value) =>
  value.child_type;
export const ChildSpecification$ChildSpecification$3 = (value) =>
  value.child_type;

/**
 * A regular child process.
 *
 * You should use this unless your process is also a supervisor.
 *
 * The default shutdown timeout is 5000ms. This can be changed with the
 * `timeout` function.
 */
export function worker(start) {
  return new ChildSpecification(start, new Permanent(), false, new Worker(5000));
}

/**
 * A special child that is a supervisor itself.
 *
 * Supervisor children have an unlimited shutdown time, there is no timeout.
 */
export function supervisor(start) {
  return new ChildSpecification(start, new Permanent(), false, new Supervisor());
}

/**
 * This defines if a child is considered significant for automatic
 * self-shutdown of the supervisor.
 *
 * You most likely do not want to consider any children significant.
 *
 * This will be ignored if the supervisor auto shutdown is set to `Never`,
 * which is the default.
 *
 * The default value for significance is `False`.
 */
export function significant(child, significant) {
  return new ChildSpecification(
    child.start,
    child.restart,
    significant,
    child.child_type,
  );
}

/**
 * This defines the amount of milliseconds a child has to shut down before
 * being brutal killed by the supervisor.
 *
 * If not set the default for a child is 5000ms.
 *
 * This will be ignored if the child is a supervisor itself.
 */
export function timeout(child, ms) {
  let $ = child.child_type;
  if ($ instanceof Worker) {
    return new ChildSpecification(
      child.start,
      child.restart,
      child.significant,
      new Worker(ms),
    );
  } else {
    return child;
  }
}

/**
 * When the child is to be restarted. See the `Restart` documentation for
 * more.
 *
 * The default value for restart is `Permanent`.
 */
export function restart(child, restart) {
  return new ChildSpecification(
    child.start,
    restart,
    child.significant,
    child.child_type,
  );
}

/**
 * Transform the data of the started child process.
 */
export function map_data(child, transform) {
  return new ChildSpecification(
    () => {
      let $ = child.start();
      if ($ instanceof Ok) {
        let started = $[0];
        return new Ok(new $actor.Started(started.pid, transform(started.data)));
      } else {
        return $;
      }
    },
    child.restart,
    child.significant,
    child.child_type,
  );
}
