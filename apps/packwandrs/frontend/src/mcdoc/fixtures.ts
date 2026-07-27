/**
 * An interim schema source.
 *
 * The real source is `SpyglassMC/vanilla-mcdoc`, which describes every vanilla
 * registry for every supported version. Wiring that in means vendoring pinned
 * snapshots per Minecraft version and running Spyglass's mcdoc parser over
 * them — tracked separately, because it is a data-supply problem rather than a
 * rendering one.
 *
 * Until then these hand-written definitions cover the shapes that actually
 * exercise the renderer: a dispatcher keyed on a sibling field, a spread that
 * mixes fixed and dispatched keys, unions of struct and scalar, id attributes,
 * enums, and recursive nesting. They are a fixture, not a substitute — nothing
 * here should be treated as authoritative for Minecraft's real schema.
 */

import type { McdocType } from '@spyglassmc/mcdoc'

import { createMemorySchemaSource, type SchemaSource } from './schema'

const literal = (value: string): McdocType => ({ kind: 'literal', value: { kind: 'string', value } })

const id = (registry: string): McdocType => ({
  kind: 'string',
  attributes: [{ name: 'id', value: literal(registry) }],
})

const intRange: McdocType = {
  kind: 'union',
  members: [
    { kind: 'int' },
    {
      kind: 'struct',
      fields: [
        { kind: 'pair', key: 'min', type: { kind: 'int' } },
        { kind: 'pair', key: 'max', type: { kind: 'int' } },
      ],
    },
  ],
}

/** `{"function": "...", ...}` — the shape every loot function shares. */
const lootFunction: McdocType = {
  kind: 'struct',
  fields: [
    { kind: 'pair', key: 'function', type: id('minecraft:loot_function'), desc: 'Which function to apply.' },
    {
      kind: 'spread',
      type: {
        kind: 'dispatcher',
        registry: 'minecraft:loot_function',
        parallelIndices: [{ kind: 'dynamic', accessor: ['function'] }],
      },
    },
  ],
}

const lootCondition: McdocType = {
  kind: 'struct',
  fields: [
    { kind: 'pair', key: 'condition', type: id('minecraft:loot_condition') },
    {
      kind: 'spread',
      type: {
        kind: 'dispatcher',
        registry: 'minecraft:loot_condition',
        parallelIndices: [{ kind: 'dynamic', accessor: ['condition'] }],
      },
    },
  ],
}

const lootEntry: McdocType = {
  kind: 'struct',
  fields: [
    { kind: 'pair', key: 'type', type: id('minecraft:loot_pool_entry_type') },
    { kind: 'pair', key: 'weight', type: { kind: 'int' }, optional: true },
    { kind: 'pair', key: 'quality', type: { kind: 'int' }, optional: true },
    {
      kind: 'pair',
      key: 'conditions',
      type: { kind: 'list', item: lootCondition },
      optional: true,
    },
    {
      kind: 'pair',
      key: 'functions',
      type: { kind: 'list', item: lootFunction },
      optional: true,
    },
    {
      kind: 'spread',
      type: {
        kind: 'dispatcher',
        registry: 'minecraft:loot_pool_entry_type',
        parallelIndices: [{ kind: 'dynamic', accessor: ['type'] }],
      },
    },
  ],
}

const lootPool: McdocType = {
  kind: 'struct',
  fields: [
    { kind: 'pair', key: 'rolls', type: intRange, desc: 'How many times to draw from this pool.' },
    { kind: 'pair', key: 'bonus_rolls', type: intRange, optional: true },
    { kind: 'pair', key: 'entries', type: { kind: 'list', item: lootEntry } },
    { kind: 'pair', key: 'conditions', type: { kind: 'list', item: lootCondition }, optional: true },
    { kind: 'pair', key: 'functions', type: { kind: 'list', item: lootFunction }, optional: true },
  ],
}

const lootTable: McdocType = {
  kind: 'struct',
  fields: [
    {
      kind: 'pair',
      key: 'type',
      type: {
        kind: 'enum',
        enumKind: 'string',
        values: [
          { identifier: 'chest', value: 'minecraft:chest' },
          { identifier: 'block', value: 'minecraft:block' },
          { identifier: 'entity', value: 'minecraft:entity' },
          { identifier: 'fishing', value: 'minecraft:fishing' },
          { identifier: 'gift', value: 'minecraft:gift' },
          { identifier: 'generic', value: 'minecraft:generic' },
        ],
      },
      optional: true,
      desc: 'Context the table is rolled in.',
    },
    { kind: 'pair', key: 'pools', type: { kind: 'list', item: lootPool }, optional: true },
    { kind: 'pair', key: 'functions', type: { kind: 'list', item: lootFunction }, optional: true },
    { kind: 'pair', key: 'random_sequence', type: id('minecraft:random_sequence'), optional: true },
  ],
}

const predicate: McdocType = { kind: 'union', members: [lootCondition, { kind: 'list', item: lootCondition }] }

/** A generator the shell can offer, paired with the type that drives its form. */
export interface GeneratorDefinition {
  id: string
  title: string
  /** Where the result belongs inside a datapack, for the save hint. */
  folder: string
  type: McdocType
}

export const generatorDefinitions: readonly GeneratorDefinition[] = [
  { id: 'loot_table', title: 'Loot table', folder: 'data/<namespace>/loot_table', type: lootTable },
  { id: 'predicate', title: 'Predicate', folder: 'data/<namespace>/predicate', type: predicate },
]

export const fixtureSchemaSource: SchemaSource = createMemorySchemaSource({
  dispatchers: {
    'minecraft:loot_pool_entry_type': {
      'minecraft:item': {
        kind: 'struct',
        fields: [{ kind: 'pair', key: 'name', type: id('minecraft:item') }],
      },
      'minecraft:loot_table': {
        kind: 'struct',
        fields: [{ kind: 'pair', key: 'value', type: id('minecraft:loot_table') }],
      },
      'minecraft:tag': {
        kind: 'struct',
        fields: [
          { kind: 'pair', key: 'name', type: id('minecraft:item_tag') },
          { kind: 'pair', key: 'expand', type: { kind: 'boolean' }, optional: true },
        ],
      },
      'minecraft:empty': { kind: 'struct', fields: [] },
      // `alternatives` nests entries inside entries, which is what proves the
      // renderer survives a recursive schema.
      'minecraft:alternatives': {
        kind: 'struct',
        fields: [{ kind: 'pair', key: 'children', type: { kind: 'list', item: lootEntry } }],
      },
    },
    'minecraft:loot_function': {
      'minecraft:set_count': {
        kind: 'struct',
        fields: [
          { kind: 'pair', key: 'count', type: intRange },
          { kind: 'pair', key: 'add', type: { kind: 'boolean' }, optional: true },
        ],
      },
      'minecraft:looting_enchant': {
        kind: 'struct',
        fields: [
          { kind: 'pair', key: 'count', type: intRange },
          { kind: 'pair', key: 'limit', type: { kind: 'int' }, optional: true },
        ],
      },
      'minecraft:furnace_smelt': { kind: 'struct', fields: [] },
    },
    'minecraft:loot_condition': {
      'minecraft:random_chance': {
        kind: 'struct',
        fields: [{ kind: 'pair', key: 'chance', type: { kind: 'float', valueRange: { kind: 0, min: 0, max: 1 } as never } }],
      },
      'minecraft:killed_by_player': { kind: 'struct', fields: [] },
      'minecraft:inverted': {
        kind: 'struct',
        fields: [{ kind: 'pair', key: 'term', type: lootCondition }],
      },
    },
  },
  registries: {
    'minecraft:loot_function': ['minecraft:set_count', 'minecraft:looting_enchant', 'minecraft:furnace_smelt'],
    'minecraft:loot_condition': ['minecraft:random_chance', 'minecraft:killed_by_player', 'minecraft:inverted'],
    'minecraft:loot_pool_entry_type': [
      'minecraft:item',
      'minecraft:tag',
      'minecraft:loot_table',
      'minecraft:alternatives',
      'minecraft:empty',
    ],
    'minecraft:item': ['minecraft:diamond', 'minecraft:iron_ingot', 'minecraft:emerald', 'minecraft:stick'],
  },
})
