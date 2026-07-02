import { SchemaGenerator } from "../../schemaDSL.ts";
import { JSONSchema7 } from "npm:@types/json-schema";

export class Path implements SchemaGenerator {
	public readonly schema: JSONSchema7 = {
		type: "string",
		description:
			"A relative path using forward slashes. Must not reference a file outside the pack root, and should not include characters or filenames restricted on common operating systems.\n\nImplementations must support special characters including those that are percent-encoded in URLs, such as spaces and square brackets. Implementations must guard against path traversal attacks and manually validate paths.",
		// Ban characters not allowed in NTFS filenames, and empty path components
		pattern: `^([^/\\x00-\\x1F\\x7F"*/:<>?\\\\|]+\\/)*[^/\\x00-\\x1F\\x7F"*/:<>?\\\\|]*$`,
	};
}
