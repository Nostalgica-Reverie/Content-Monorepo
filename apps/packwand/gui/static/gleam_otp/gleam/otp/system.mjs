import * as $atom from "../../../gleam_erlang/gleam/erlang/atom.mjs";
import * as $process from "../../../gleam_erlang/gleam/erlang/process.mjs";
import * as $dynamic from "../../../gleam_stdlib/gleam/dynamic.mjs";
import { CustomType as $CustomType } from "../../gleam.mjs";

/**
 * Currently handling message as normal.
 */
export class Running extends $CustomType {}
export const Mode$Running = () => new Running();
export const Mode$isRunning = (value) => value instanceof Running;

/**
 * Termporarily not handling messages, other than system messages.
 */
export class Suspended extends $CustomType {}
export const Mode$Suspended = () => new Suspended();
export const Mode$isSuspended = (value) => value instanceof Suspended;

export class NoDebug extends $CustomType {}
export const DebugOption$NoDebug = () => new NoDebug();
export const DebugOption$isNoDebug = (value) => value instanceof NoDebug;

export class StatusInfo extends $CustomType {
  constructor(module, parent, mode, debug_state, state) {
    super();
    this.module = module;
    this.parent = parent;
    this.mode = mode;
    this.debug_state = debug_state;
    this.state = state;
  }
}
export const StatusInfo$StatusInfo = (module, parent, mode, debug_state, state) =>
  new StatusInfo(module, parent, mode, debug_state, state);
export const StatusInfo$isStatusInfo = (value) => value instanceof StatusInfo;
export const StatusInfo$StatusInfo$module = (value) => value.module;
export const StatusInfo$StatusInfo$0 = (value) => value.module;
export const StatusInfo$StatusInfo$parent = (value) => value.parent;
export const StatusInfo$StatusInfo$1 = (value) => value.parent;
export const StatusInfo$StatusInfo$mode = (value) => value.mode;
export const StatusInfo$StatusInfo$2 = (value) => value.mode;
export const StatusInfo$StatusInfo$debug_state = (value) => value.debug_state;
export const StatusInfo$StatusInfo$3 = (value) => value.debug_state;
export const StatusInfo$StatusInfo$state = (value) => value.state;
export const StatusInfo$StatusInfo$4 = (value) => value.state;

export class Resume extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const SystemMessage$Resume = ($0) => new Resume($0);
export const SystemMessage$isResume = (value) => value instanceof Resume;
export const SystemMessage$Resume$0 = (value) => value[0];

export class Suspend extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const SystemMessage$Suspend = ($0) => new Suspend($0);
export const SystemMessage$isSuspend = (value) => value instanceof Suspend;
export const SystemMessage$Suspend$0 = (value) => value[0];

export class GetState extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const SystemMessage$GetState = ($0) => new GetState($0);
export const SystemMessage$isGetState = (value) => value instanceof GetState;
export const SystemMessage$GetState$0 = (value) => value[0];

export class GetStatus extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const SystemMessage$GetStatus = ($0) => new GetStatus($0);
export const SystemMessage$isGetStatus = (value) => value instanceof GetStatus;
export const SystemMessage$GetStatus$0 = (value) => value[0];
