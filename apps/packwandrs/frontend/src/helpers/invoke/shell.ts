import { call } from './core'

/** One line of pw4shell output, tagged with the output dock's tone. */
export interface ShellLine {
  text: string
  tone: 'info' | 'error' | 'success'
}

export interface ShellResult {
  lines: ShellLine[]
  /** False when the verb was not recognised, or the line was malformed. */
  handled: boolean
}

/** Run one pw4shell line in the active project or pack folder. */
export const shellExec = (line: string, cwd?: string) =>
  call<ShellResult>('shell_exec', { line, cwd: cwd ?? null })

/**
 * Tokenise a line without running it, using the kernel's own quoting rules.
 *
 * Prefer this over splitting on whitespace here: quoting, escapes and comments
 * are defined once, in C, and a console that disagrees with its own backend
 * about what `"a b"` means is worse than one with no completion at all.
 */
export const shellParse = (line: string) => call<string[]>('shell_parse', { line })
