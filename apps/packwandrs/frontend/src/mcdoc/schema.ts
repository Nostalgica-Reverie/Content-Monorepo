/**
 * Where mcdoc type definitions come from.
 *
 * mcdoc is Spyglass's schema language for Minecraft's data structures, and
 * `SpyglassMC/vanilla-mcdoc` publishes the vanilla definitions written in it.
 * Because the generators upstream are driven by those schemas rather than
 * hand-written per registry, a renderer over mcdoc gets every generator at
 * once — that is the whole reason this layer exists instead of a fork.
 *
 * The renderer never talks to Spyglass directly. Anything it cannot decide
 * from a type alone — `reference` paths, `dispatcher` arms, registry ids —
 * comes through this interface, which keeps the form layer a pure function of
 * (type, value) and lets the schema source be a fixture in tests today and a
 * vendored vanilla-mcdoc snapshot in the app later, without the renderer
 * changing.
 */

import type { McdocType } from '@spyglassmc/mcdoc'

export interface SchemaSource {
  /**
   * Resolves a `reference` type's path, such as
   * `::java::data::loot::LootPool`.
   */
  reference: (path: string) => McdocType | undefined
  /**
   * Resolves one arm of a dispatcher — the mechanism behind Minecraft's
   * `type`-tagged JSON, where `minecraft:loot_function[set_count]` selects
   * the shape that goes with a given `function` value.
   */
  dispatch: (registry: string, index: string) => McdocType | undefined
  /**
   * Known ids for a registry, used to populate pickers. Optional: a source
   * that cannot enumerate a registry still renders, just with a free-text
   * field instead of a list.
   */
  registry?: (registry: string) => readonly string[]
}

/** A schema source backed by plain objects. Used by tests and by fixtures. */
export function createMemorySchemaSource(tables: {
  references?: Record<string, McdocType>
  dispatchers?: Record<string, Record<string, McdocType>>
  registries?: Record<string, readonly string[]>
}): SchemaSource {
  const { references = {}, dispatchers = {}, registries = {} } = tables
  return {
    reference: (path) => references[path],
    dispatch: (registry, index) => dispatchers[registry]?.[index],
    registry: (registry) => registries[registry] ?? [],
  }
}

/** A source that resolves nothing, for rendering self-contained types. */
export const emptySchemaSource: SchemaSource = {
  reference: () => undefined,
  dispatch: () => undefined,
  registry: () => [],
}
