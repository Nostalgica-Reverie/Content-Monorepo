import { property, schema, SchemaGenerator } from "../schemaDSL.ts";
import { Hash } from "./shared/Hash.ts";
import { HashFormat } from "./shared/HashFormat.ts";
import { PackwizURL } from "./shared/PackwizURL.ts";
import { Path } from "./shared/Path.ts";

// TODO(gen): Rename to external?
// TODO(doc): Document file extension

@schema({
	// TODO(gen): $id?
	$schema: "http://json-schema.org/draft-07/schema",
	title: "mod.pw.toml",
	description:
		'A metadata file which references an external file from a URL. This allows for side-only mods, optional mods and stores metadata to allow finding updates from Modrinth and CurseForge. The "mod" terminology is used a lot here, but this should work for any file.',
	examples: [
		await Deno.readTextFile(
			"./example-pack/mods/borderless-mining.pw.toml",
		),
	],
	// TODO(gen): Taplo extensions: links?
})
export class Mod {
	@property.string(
		"The name of the mod, which can be displayed in user interfaces to identify the mod. It does not need to be unique between mods, although this may cause confusion.",
	)
	@property.examples(["Borderless Mining"])
	@property.required
	name: undefined;

	@property.ref(
		"The destination path of the mod file, relative to this file.",
	)
	@property.examples(["borderless-mining-1.1.5+1.19.jar"])
	@property.required
	filename = new Path();

	// TODO(gen): Taplo enum docs?
	@property.enum(
		["both", "client", "server"],
		"The side on which this mod should be installed.\n\nA physical Minecraft side. Server applies to the dedicated server, client applies to the client (and integrated server), and both applies to every installation.",
	)
	@property.default("both")
	side: undefined;

	@property.ref()
	@property.required
	download = new Download();
	@property.ref()
	option = new Option();
	@property.ref()
	update = new Update();
}
// deno-lint-ignore no-empty-interface
export interface Mod extends SchemaGenerator {}

@schema({
	description: "Information about how to download this mod.",
})
class Download {
	@property.ref(`The URL to download the mod from.`)
	@property.required
	url = new PackwizURL();

	// TODO(doc): Download mode

	@property.ref("The hash of the specified file, as a string.")
	@property.required
	hash = new Hash();

	// TODO(format): Store additional hashes (particularly for Modrinth export)

	@property.ref("The hash format for the hash of the specified file.")
	@property.required
	"hash-format" = new HashFormat();
}
// deno-lint-ignore no-empty-interface
interface Download extends SchemaGenerator {}

@schema({
	description:
		"Information about the optional state of this mod. When excluded, this indicates that the mod is not optional.",
})
class Option {
	@property.boolean(
		"Whether or not the mod is optional. This can be set to false if you want to keep the description but make the mod required.",
	)
	@property.default(false)
	@property.required
	optional: undefined;

	@property.string(
		"A description displayed to the user when they select optional mods. This should explain why or why not the user should enable the mod.",
	)
	description: undefined;

	@property.boolean(
		"If true, the mod will be enabled by default. If false, the mod will be disabled by default. If a pack format does not support optional mods but it does support disabling mods, the mod will be disabled if it defaults to being disabled.",
	)
	@property.default(false)
	default: undefined;
}
// deno-lint-ignore no-empty-interface
interface Option extends SchemaGenerator {}

@schema({
	description: `Information about how to update the download details of this mod with tools.

If this object does not exist or there are no defined update sources, the mod will not be automatically updated.

If there are multiple defined update sources, one of them will be chosen. The source that is chosen is not defined, so it is therefore dependent on the implementation of the tool (may not be deterministic, so do not rely on one source being chosen over another).`,
	additionalProperties: {
		type: "object",
		description:
			"Implementations are free to support other update sources, but it would be beneficial to document and standardise these.",
	},
})
class Update {
	@property.ref()
	curseforge = new CurseForgeUpdate();
	@property.ref()
	modrinth = new ModrinthUpdate();
}
// deno-lint-ignore no-empty-interface
interface Update extends SchemaGenerator {}

@schema({
	description: `An update source for updating mods downloaded from CurseForge.`,
})
class CurseForgeUpdate {
	@property.number(
		"An integer representing the unique project ID of this mod. Updating will retrieve the latest file for this project ID that is valid (correct Minecraft version, release channel, modloader, etc.).",
	)
	@property.required
	@property.examples([327154])
	"project-id": undefined;

	@property.number(
		"An integer representing the unique file ID of this mod file. This can be used if more metadata needs to be obtained relating to the mod.",
	)
	@property.required
	@property.examples([3643025])
	"file-id": undefined;
}
// deno-lint-ignore no-empty-interface
interface CurseForgeUpdate extends SchemaGenerator {}

@schema({
	description: `An update source for updating mods downloaded from Modrinth.`,
})
class ModrinthUpdate {
	@property.string(
		"A string representing the unique mod ID of this mod. Updating will retrieve the latest file for this project ID that is valid (correct Minecraft version, release channel, modloader, etc.).",
	)
	@property.required
	@property.examples(["kYq5qkSL"])
	// TODO(v2): change to "project-id"
	"mod-id": undefined;

	@property.string(
		"A string representing the unique version ID of this file. This can be used if more metadata needs to be obtained relating to the mod.",
	)
	@property.required
	@property.examples(["gqoXgtxO"])
	// TODO(v2): change to "version-id"
	"version": undefined;
}
// deno-lint-ignore no-empty-interface
interface ModrinthUpdate extends SchemaGenerator {}

export default Mod;
