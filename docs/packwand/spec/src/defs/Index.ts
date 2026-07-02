import { property, schema, SchemaGenerator } from "../schemaDSL.ts";
import { Hash } from "./shared/Hash.ts";
import { HashFormat } from "./shared/HashFormat.ts";
import { Path } from "./shared/Path.ts";

@schema({
	// TODO(gen): $id?
	$schema: "http://json-schema.org/draft-07/schema",
	title: "index.toml",
	description: `The index file of the modpack, storing references to every file to be downloaded in the pack.`,
	examples: [
		await Deno.readTextFile("./example-pack/index.toml"),
	],
	// TODO(gen): Taplo extensions: links?
})
export class Index {
	@property.arrayRef(
		"The files listed in this index. If it is not defined, defaults to an empty list.",
		true,
	)
	files = new IndexFile();

	@property.ref("The default hash format for every file in the index.")
	@property.required
	@property.default("sha256")
	"hash-format" = new HashFormat();
}
// deno-lint-ignore no-empty-interface
export interface Index extends SchemaGenerator {}

@schema({
	description: "A single file in the index, to be downloaded by the modpack installer.",
})
class IndexFile {
	@property.ref(
		`The path to the file to be downloaded, relative to this index file.`,
	)
	@property.required
	file = new Path();

	@property.ref("The hash of the specified file, as a string.")
	@property.required
	hash = new Hash();

	@property.ref(
		"The hash format for the hash of the specified file. Defaults to the hash format specified in the index - ideally remove this value if it is equal to the hash format for the index to save space.",
	)
	"hash-format" = new HashFormat();

	@property.boolean(
		"True when this entry points to a .toml metadata file, which references a file outside the pack.",
	)
	@property.default(false)
	metafile: undefined;

	@property.boolean(
		"When this is set to true, the file is not overwritten if it already exists, to preserve changes made by a user.",
	)
	@property.default(false)
	preserve: undefined;

	@property.string(
		"The name with which this file should be downloaded, instead of the filename specified in the path. Not compatible with metafile, and may not be very well supported.",
	)
	alias: undefined;
}
// deno-lint-ignore no-empty-interface
interface IndexFile extends SchemaGenerator {}

export default Index;
