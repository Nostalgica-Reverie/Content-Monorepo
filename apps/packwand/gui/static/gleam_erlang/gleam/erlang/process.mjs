import * as $dynamic from "../../../gleam_stdlib/gleam/dynamic.mjs";
import * as $decode from "../../../gleam_stdlib/gleam/dynamic/decode.mjs";
import * as $string from "../../../gleam_stdlib/gleam/string.mjs";
import { Ok, Error, CustomType as $CustomType } from "../../gleam.mjs";
import * as $atom from "../../gleam/erlang/atom.mjs";
import * as $port from "../../gleam/erlang/port.mjs";
import * as $reference from "../../gleam/erlang/reference.mjs";

class Subject extends $CustomType {
  constructor(owner, tag) {
    super();
    this.owner = owner;
    this.tag = tag;
  }
}

class NamedSubject extends $CustomType {
  constructor(name) {
    super();
    this.name = name;
  }
}

export class ExitMessage extends $CustomType {
  constructor(pid, reason) {
    super();
    this.pid = pid;
    this.reason = reason;
  }
}
export const ExitMessage$ExitMessage = (pid, reason) =>
  new ExitMessage(pid, reason);
export const ExitMessage$isExitMessage = (value) =>
  value instanceof ExitMessage;
export const ExitMessage$ExitMessage$pid = (value) => value.pid;
export const ExitMessage$ExitMessage$0 = (value) => value.pid;
export const ExitMessage$ExitMessage$reason = (value) => value.reason;
export const ExitMessage$ExitMessage$1 = (value) => value.reason;

export class Normal extends $CustomType {}
export const ExitReason$Normal = () => new Normal();
export const ExitReason$isNormal = (value) => value instanceof Normal;

export class Killed extends $CustomType {}
export const ExitReason$Killed = () => new Killed();
export const ExitReason$isKilled = (value) => value instanceof Killed;

export class Abnormal extends $CustomType {
  constructor(reason) {
    super();
    this.reason = reason;
  }
}
export const ExitReason$Abnormal = (reason) => new Abnormal(reason);
export const ExitReason$isAbnormal = (value) => value instanceof Abnormal;
export const ExitReason$Abnormal$reason = (value) => value.reason;
export const ExitReason$Abnormal$0 = (value) => value.reason;

class Anything extends $CustomType {}

class Process extends $CustomType {}

export class ProcessDown extends $CustomType {
  constructor(monitor, pid, reason) {
    super();
    this.monitor = monitor;
    this.pid = pid;
    this.reason = reason;
  }
}
export const Down$ProcessDown = (monitor, pid, reason) =>
  new ProcessDown(monitor, pid, reason);
export const Down$isProcessDown = (value) => value instanceof ProcessDown;
export const Down$ProcessDown$monitor = (value) => value.monitor;
export const Down$ProcessDown$0 = (value) => value.monitor;
export const Down$ProcessDown$pid = (value) => value.pid;
export const Down$ProcessDown$1 = (value) => value.pid;
export const Down$ProcessDown$reason = (value) => value.reason;
export const Down$ProcessDown$2 = (value) => value.reason;

export class PortDown extends $CustomType {
  constructor(monitor, port, reason) {
    super();
    this.monitor = monitor;
    this.port = port;
    this.reason = reason;
  }
}
export const Down$PortDown = (monitor, port, reason) =>
  new PortDown(monitor, port, reason);
export const Down$isPortDown = (value) => value instanceof PortDown;
export const Down$PortDown$monitor = (value) => value.monitor;
export const Down$PortDown$0 = (value) => value.monitor;
export const Down$PortDown$port = (value) => value.port;
export const Down$PortDown$1 = (value) => value.port;
export const Down$PortDown$reason = (value) => value.reason;
export const Down$PortDown$2 = (value) => value.reason;

export const Down$monitor = (value) => value.monitor;
export const Down$reason = (value) => value.reason;

/**
 * The timer could not be found. It likely has already triggered.
 */
export class TimerNotFound extends $CustomType {}
export const Cancelled$TimerNotFound = () => new TimerNotFound();
export const Cancelled$isTimerNotFound = (value) =>
  value instanceof TimerNotFound;

/**
 * The timer was found and cancelled before it triggered.
 *
 * The amount of remaining time before the timer was due to be triggered is
 * returned in milliseconds.
 */
export class Cancelled extends $CustomType {
  constructor(time_remaining) {
    super();
    this.time_remaining = time_remaining;
  }
}
export const Cancelled$Cancelled = (time_remaining) =>
  new Cancelled(time_remaining);
export const Cancelled$isCancelled = (value) => value instanceof Cancelled;
export const Cancelled$Cancelled$time_remaining = (value) =>
  value.time_remaining;
export const Cancelled$Cancelled$0 = (value) => value.time_remaining;

class Kill extends $CustomType {}

/**
 * Create a subject for the given process with the give tag. This is unsafe!
 * There's nothing here that verifies that the message the subject receives is
 * expected and that the tag is not already in use.
 *
 * You should almost certainly not use this function.
 * 
 * @ignore
 */
export function unsafely_create_subject(owner, tag) {
  return new Subject(owner, tag);
}

/**
 * Create a subject for a name, which can be used to send and receive messages.
 *
 * All subjects created for the same name behave identically and can be used
 * interchangably.
 */
export function named_subject(name) {
  return new NamedSubject(name);
}

/**
 * Get the name of a subject, returning an error if it doesn't have one.
 */
export function subject_name(subject) {
  if (subject instanceof Subject) {
    return new Error(undefined);
  } else {
    let name = subject.name;
    return new Ok(name);
  }
}
