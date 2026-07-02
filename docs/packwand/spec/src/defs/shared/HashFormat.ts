import { SchemaGenerator } from "../../schemaDSL.ts";
import { JSONSchema7 } from "npm:@types/json-schema";

export class HashFormat implements SchemaGenerator {
	public readonly schema: JSONSchema7 = {
		type: "string",
		description:
			"The hash algorithm used to determine if a file is valid. All functions listed must be supported by tools implementing the packwiz pack format.",
		enum: ["sha256", "sha512", "sha1", "md5", "murmur2"],

		// TODO(format): Mark insecure hash functions as deprecated, when extra hashes field is added
		// TODO(format): File size as insecure "hash"
		// TODO(v2): Change murmur2 to something indicating that this is the CF variant
		"x-taplo": {
			docs: {
				enumValues: [
					"The SHA2-256 hashing standard. Used by default for metadata files.",
					"The SHA2-512 hashing standard.",
					"The SHA1 hashing standard.",
					"The MD5 hashing standard.",
					"The CurseForge variant of MurmurHash2: MurmurHash2 32-bit hashing standard (seed 1) with some characters removed before applying the hash (decimal bytes 9, 10, 13, 32), stored as an unsigned integer.\n\nIn Java, use Integer.parseUnsignedInt to parse this value.",
				],
			},
		},
	};
}
