/**
 * The mcdoc compat layer.
 *
 * A thin adapter over Spyglass's schema packages, not a fork of any upstream.
 * `@spyglassmc/mcdoc` supplies the type model; this layer resolves it into
 * something renderable and draws it. Upstream's generators are schema-driven,
 * so a renderer over the same schemas is all that is needed to have them here,
 * in Packwand's own design language rather than embedded in an iframe.
 */

export { default as McdocField } from './McdocField.vue'
export { createMemorySchemaSource, emptySchemaSource, type SchemaSource } from './schema'
export {
	childPath,
	isRecord,
	rootPath,
	simplifyType,
	structFields,
	structKeys,
	type ValuePath,
} from './simplify'
export {
	attributeString,
	defaultValue,
	findAttribute,
	idRegistry,
	selectUnionMember,
	typeLabel,
} from './value'
export { fixtureSchemaSource, generatorDefinitions, type GeneratorDefinition } from './fixtures'
