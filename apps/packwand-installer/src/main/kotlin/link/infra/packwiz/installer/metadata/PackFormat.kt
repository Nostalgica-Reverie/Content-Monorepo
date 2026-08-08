package link.infra.packwiz.installer.metadata

@JvmInline
value class PackFormat(val format: String) {
	companion object {
		val DEFAULT = PackFormat("packwiz:1.0.0")

		// Highest packwiz major version this installer understands.
		private const val MAX_PACKWIZ_MAJOR = 1

		// Highest packwand generation this installer understands. Mirrors
		// CURRENT_PACK_FORMAT in apps/packwandrs/crates/packwand-pack/src/model.rs
		// — bump together.
		//
		// Generation 27 moved metadata and the index from TOML to JSON; this
		// installer reads both, see [JsonDocument].
		private const val MAX_PACKWAND_GENERATION = 27
	}

	sealed class Support {
		object Ok : Support()
		data class Newer(val message: String) : Support()
		data class Unsupported(val message: String) : Support()
	}

	/**
	 * Checks whether this pack-format can be installed. Newer-than-known
	 * versions of a known scheme produce [Support.Newer] (install proceeds
	 * with a warning, matching packwand's own behaviour); unknown schemes or
	 * unparseable versions produce [Support.Unsupported].
	 */
	fun support(): Support = when {
		format.startsWith("packwand:") -> {
			val generation = format.removePrefix("packwand:").toIntOrNull()
			when {
				generation == null ->
					Support.Unsupported("Invalid pack-format \"$format\" (expected packwand:<generation>)")
				generation > MAX_PACKWAND_GENERATION ->
					Support.Newer("Pack uses $format, newer than this installer supports (packwand:$MAX_PACKWAND_GENERATION); update packwiz-installer for full support")
				else -> Support.Ok
			}
		}
		format.startsWith("packwiz:") -> {
			val major = format.removePrefix("packwiz:").substringBefore('.').toIntOrNull()
			when {
				major == null ->
					Support.Unsupported("Invalid pack-format \"$format\" (expected packwiz:<version>)")
				major > MAX_PACKWIZ_MAJOR ->
					Support.Newer("Pack uses $format, newer than this installer supports (packwiz:$MAX_PACKWIZ_MAJOR.x); update packwiz-installer for full support")
				else -> Support.Ok
			}
		}
		else -> Support.Unsupported("Unknown pack-format \"$format\" (expected packwiz:… or packwand:…)")
	}
}
