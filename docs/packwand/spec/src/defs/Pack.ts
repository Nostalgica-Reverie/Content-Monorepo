import { property, schema, SchemaGenerator } from "../schemaDSL.ts";
import { Hash } from "./shared/Hash.ts";
import { HashFormat } from "./shared/HashFormat.ts";
import { Path } from "./shared/Path.ts";

@schema({
	// TODO(gen): $id?
	$schema: "http://json-schema.org/draft-07/schema",
	title: "pack.toml",
	description: `The main modpack file for a packwiz modpack.
	This is the first file loaded, to allow the modpack downloader to download all the files in the modpack.`,
	examples: [
		await Deno.readTextFile("./example-pack/pack.toml"),
	],
	// TODO(gen): Taplo extensions: links?
})
export class Pack {
	// TODO(gen): Regular expression to match valid values
	// TODO(gen): Generate different versions of schemas for different uses?
	@property.string(
		`A version string identifying the pack format and version of it. Currently, this pack format uses version 1.1.0.
If it is not defined, default to \"packwiz:1.0.0\" for backwards-compatibility with packs created before this field was added.

If it is defined:
- All consumers should fail to load the modpack if it does not begin with \"packwiz:\"
- All consumers should fail to load the modpack if the latter section is not valid semver as defined in https://semver.org/spec/v2.0.0.html
- All consumers should fail to load the modpack if the major version is greater than the version they support
- Consumers can suggest updating themselves if the minor version is greater than the version they implement
- Pack tools should suggest and support migration when they support a version newer than this field`,
	)
	@property.required
	@property.default("packwiz:1.1.0")
	"pack-format": undefined;

	@property.string(
		"The name of the modpack. This can be displayed in user interfaces to identify the pack, and it does not need to be unique between packs.",
	)
	@property.required
	name: undefined;
	@property.string(
		"The author(s) of the modpack. This is output when exporting to the CurseForge pack format, and can be displayed in user interfaces.",
	)
	author: undefined;
	@property.string(
		"The version of the modpack. This is output when exporting to the CurseForge pack format, but is not currently used elsewhere by the tools or installer. It must not be used for determining if the modpack is outdated.",
	)
	version: undefined;
	@property.string(
		"A short description of the modpack. This is output when exporting to the Modrinth pack format, but is not currently used elsewhere by the tools or installer.",
	)
	description: undefined;

	@property.ref(`Information about the index file in this modpack.`)
	@property.required
	index = new IndexRef();

	@property.ref(
		`The versions of components used by this modpack - usually Minecraft and the mod loader this pack uses. The existence of a component implies that it should be installed. These values can also be used by tools to determine which versions of mods should be installed.`,
	)
	@property.required
	versions = new ComponentVersions();

	// TODO(doc): export, options
}
// deno-lint-ignore no-empty-interface
export interface Pack extends SchemaGenerator {}

@schema()
class IndexRef {
	@property.ref(`The path to the file that contains the index.`)
	@property.required
	file = new Path();

	@property.ref(`The hash of the index file, as a string.`)
	@property.required
	hash = new Hash();

	@property.ref("The hash format for the hash of the index file.")
	@property.required
	"hash-format" = new HashFormat();
}
// deno-lint-ignore no-empty-interface
interface IndexRef extends SchemaGenerator {}

@schema({
	additionalProperties: {
		type: "string",
	},
})
class ComponentVersions {
	@property.string(
		"The version of Minecraft used by this modpack. This should be in the format used by the version.json files.",
	)
	@property.required
	@property.examples(["1.17.1", "16w02a"])
	minecraft: undefined;

	@property.string("The version of Fabric loader used by this modpack.")
	@property.examples(["0.12.1"])
	fabric: undefined;
	@property.string(
		"The version of Forge used by this modpack. This version must not include the Minecraft version as a prefix.",
	)
	@property.examples(["14.23.5.2838"])
	forge: undefined;
	@property.string("The version of Liteloader used by this modpack.")
	@property.examples(["1.12.2-SNAPSHOT"])
	liteloader: undefined;

	@property.string("The version of Quilt loader used by this modpack.")
	@property.examples(["0.12.1"])
	quilt: undefined;
	// TODO(format): others?
}
// deno-lint-ignore no-empty-interface
interface ComponentVersions extends SchemaGenerator {}

export default Pack;
