import { SchemaGenerator } from "../../schemaDSL.ts";
import { JSONSchema7 } from "npm:@types/json-schema";

export class Hash implements SchemaGenerator {
	public readonly schema: JSONSchema7 = {
		type: "string",
		description:
			"Binary hashes should be stored as hexadecimal, and case should be ignored during parsing. Numeric hashes (e.g. Murmur2) should still be stored as a string, to ensure the value is preserved correctly.",
		examples: ["08f98e28057a0fbdb288c947b7871def4b18b176"],
	};
}
