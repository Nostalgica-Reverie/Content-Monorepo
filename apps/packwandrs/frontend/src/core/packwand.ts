import * as gleam from '../../core/build/dev/javascript/packwand_frontend_core/packwand_frontend_core.mjs'

export const core = gleam
export type CoreModel = gleam.Model$
export type CoreMessage = gleam.Message$
export type CoreEffect = gleam.Effect$

export const validateThemeId = gleam.validate_theme_id
export const validateHexColour = gleam.validate_hex_colour
