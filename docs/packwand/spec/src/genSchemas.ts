import Pack from "./defs/Pack.ts";
import Index from "./defs/Index.ts";
import Mod from "./defs/Mod.ts";

const packSchema = new Pack().schema;
const indexSchema = new Index().schema;
const modSchema = new Mod().schema;

await Deno.writeTextFile(
	"schemas/pack.json",
	JSON.stringify(packSchema, null, "\t"),
);
await Deno.writeTextFile(
	"schemas/index.json",
	JSON.stringify(indexSchema, null, "\t"),
);
await Deno.writeTextFile(
	"schemas/mod.json",
	JSON.stringify(modSchema, null, "\t"),
);
