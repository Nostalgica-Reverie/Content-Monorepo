export interface Logger {
	info: (msg: string) => void
	warn: (msg: string) => void
	error: (msg: string | Error) => void
}

let implementation: Logger | null = null

export function setLogger(logger: Logger) {
	implementation = logger
}

// These are what the rest of the bot calls
export function info(msg: string) {
	implementation?.info(msg)
}

export function warn(msg: string) {
	implementation?.warn(msg)
}

export function error(msg: string | Error) {
	implementation?.error(msg)
}
