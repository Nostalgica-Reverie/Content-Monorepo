/**
 * Repo-wide Prettier configuration.
 *
 * Deliberately short. Indentation, line width, and line endings are not set
 * here — Prettier reads those from .editorconfig, which is also what editors
 * and every non-JavaScript formatter in this repo read. Restating them would
 * create two sources of truth that drift.
 *
 * These two options are the ones EditorConfig cannot express, and they match
 * the style already written throughout apps/bot and apps/packwandrs/frontend.
 *
 * @see https://prettier.io/docs/configuration
 * @type {import("prettier").Config}
 */
const config = {
	semi: false,
	singleQuote: true,
}

export default config
