package link.infra.packwiz.installer.metadata

import cc.ekblad.toml.model.TomlValue
import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonNull
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import okio.Source
import okio.buffer
import java.io.InputStream

/**
 * Reads packwand generation-27 documents, which are JSON rather than TOML.
 *
 * The installer's decoders — [PackwizPath], [HashFormat], [Side],
 * [DownloadMode], [UpdateData] — are all written against 4koma's `TomlValue`
 * tree and encode real knowledge about this format. Rather than duplicate any
 * of that against Gson's type model, this parses JSON into a `TomlValue` and
 * hands it to the *existing* mappers unchanged. Every decoder keeps working,
 * and there is only one place where the two formats meet.
 *
 * This is a holding measure so `packwand test` and launching an instance keep
 * working against migrated packs. The installer is slated for a Rust rewrite;
 * when that lands, this file and the TOML path both go away.
 */
object JsonDocument {
	/**
	 * Whether these bytes are a JSON document.
	 *
	 * Decided by content, not by filename. The filename would seem more
	 * direct, but a pack path reaches the decoders through several layers of
	 * `PackwizPath` resolution and does not reliably still carry the
	 * extension — trusting it silently sent generation-27 packs down the TOML
	 * parser. The two formats are unambiguous at the first byte: a JSON
	 * document here is always an object, and no TOML document starts with `{`.
	 */
	@JvmStatic
	fun isJson(bytes: ByteArray): Boolean {
		// A UTF-8 BOM is three bytes, not one; skipping only the first leaves
		// the next two looking like content.
		var index = if (bytes.size >= 3 &&
			bytes[0] == 0xEF.toByte() &&
			bytes[1] == 0xBB.toByte() &&
			bytes[2] == 0xBF.toByte()
		) 3 else 0
		while (index < bytes.size && bytes[index].toInt().toChar().isWhitespace()) {
			index++
		}
		return index < bytes.size && bytes[index].toInt().toChar() == '{'
	}

	/** Parses JSON bytes into 4koma's value tree. */
	@JvmStatic
	fun parse(bytes: ByteArray): TomlValue =
		convert(JsonParser.parseString(bytes.toString(Charsets.UTF_8)))

	/** Parses JSON from a stream into 4koma's value tree. */
	@JvmStatic
	fun parse(input: InputStream): TomlValue =
		convert(JsonParser.parseReader(input.reader()))

	/** Parses JSON from an okio source into 4koma's value tree. */
	@JvmStatic
	fun parse(source: Source): TomlValue =
		parse(source.buffer().inputStream())

	private fun convert(element: JsonElement): TomlValue = when {
		element.isJsonObject -> TomlValue.Map(
			// TOML has no null, and the decoders treat an absent key as
			// "use the default". Dropping nulls rather than mapping them to
			// something else keeps those defaults working.
			(element as JsonObject).entrySet()
				.filterNot { it.value is JsonNull }
				.associate { (key, value) -> key to convert(value) }
		)
		element.isJsonArray -> TomlValue.List(
			(element as JsonArray).filterNot { it is JsonNull }.map { convert(it) }
		)
		element.isJsonPrimitive -> convertPrimitive(element)
		// A bare null at the top level or inside a value position: TOML cannot
		// represent it, and no field in this format is nullable-with-meaning.
		else -> TomlValue.Map(emptyMap())
	}

	private fun convertPrimitive(element: JsonElement): TomlValue {
		val primitive = element.asJsonPrimitive
		return when {
			primitive.isBoolean -> TomlValue.Bool(primitive.asBoolean)
			primitive.isString -> TomlValue.String(primitive.asString)
			else -> {
				// JSON has one number type; TOML separates integers from
				// floats and the decoders care which they get. CurseForge
				// project and file ids arrive here, and must stay integers.
				val text = primitive.asString
				if (text.contains('.') || text.contains('e') || text.contains('E')) {
					TomlValue.Double(primitive.asDouble)
				} else {
					TomlValue.Integer(primitive.asLong)
				}
			}
		}
	}
}
