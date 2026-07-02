import { SchemaGenerator } from "../../schemaDSL.ts";
import { JSONSchema7 } from "npm:@types/json-schema";

export class PackwizURL implements SchemaGenerator {
	public readonly schema: JSONSchema7 = {
		type: "string",
		description:
			"An absolute URI compliant with RFC 3986. Implementations may need to be more lenient in accepting reserved characters in paths due to historical implementation bugs. Only the HTTP/HTTPS protocols must be supported, other protocols should not be used.",
		format: "uri",
	};
}
