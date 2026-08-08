package link.infra.packwiz.installer.metadata

import cc.ekblad.toml.model.TomlValue
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

class JsonDocumentTest {
	private fun parse(json: String): TomlValue.Map =
		assertIs<TomlValue.Map>(JsonDocument.parse(json.byteInputStream()))

	/**
	 * Dispatch is by content, not filename. An earlier version checked the
	 * path's extension and silently sent every generation-27 pack down the
	 * TOML parser, because the name does not survive `PackwizPath` resolution
	 * intact — so these cases are the actual contract.
	 */
	@Test
	fun recognisesJsonByContent() {
		assertTrue(JsonDocument.isJson("""{"name": "Sodium"}""".toByteArray()))
		assertTrue(JsonDocument.isJson("  \n\t{}".toByteArray()), "leading whitespace is skipped")
		assertTrue(
			JsonDocument.isJson("﻿{}".toByteArray()),
			"a UTF-8 BOM must not hide the opening brace"
		)
		assertFalse(JsonDocument.isJson("""name = "Sodium"""".toByteArray()))
		assertFalse(JsonDocument.isJson("[update.modrinth]\nmod-id = \"x\"".toByteArray()))
		assertFalse(JsonDocument.isJson(ByteArray(0)), "an empty document is not JSON")
	}

	/**
	 * The decoders distinguish integers from floats, and CurseForge project
	 * and file ids arrive as JSON numbers. Mapping them to a float would make
	 * an id render as `7892437.0` and stop matching anything.
	 */
	@Test
	fun wholeNumbersStayIntegers() {
		val document = parse("""{"file-id": 7892437, "project-id": 1311114}""")
		assertEquals(TomlValue.Integer(7892437), document.properties["file-id"])
		assertEquals(TomlValue.Integer(1311114), document.properties["project-id"])
	}

	@Test
	fun fractionalNumbersStayFloats() {
		val document = parse("""{"ratio": 1.5, "exponent": 1e3}""")
		assertIs<TomlValue.Double>(document.properties["ratio"])
		assertIs<TomlValue.Double>(document.properties["exponent"])
	}

	/**
	 * TOML has no null and the decoders treat an absent key as "use the
	 * default", so a null must drop out rather than decode to anything.
	 */
	@Test
	fun nullsAreDroppedSoDefaultsStillApply() {
		val document = parse("""{"name": "Sodium", "url": null}""")
		assertEquals(TomlValue.String("Sodium"), document.properties["name"])
		assertFalse(document.properties.containsKey("url"))
	}

	@Test
	fun nestedObjectsAndArraysConvert() {
		val document = parse(
			"""{"download": {"hash-format": "sha512", "size": 10},
			    "tags": ["a", "b"], "pin": true}"""
		)
		val download = assertIs<TomlValue.Map>(document.properties["download"])
		assertEquals(TomlValue.String("sha512"), download.properties["hash-format"])
		assertEquals(TomlValue.Integer(10), download.properties["size"])
		val tags = assertIs<TomlValue.List>(document.properties["tags"])
		assertEquals(listOf(TomlValue.String("a"), TomlValue.String("b")), tags.elements)
		assertEquals(TomlValue.Bool(true), document.properties["pin"])
	}

	/** A real generation-27 metadata file, as `packwand migrate format` writes it. */
	@Test
	fun decodesRealGeneration27Metadata() {
		val document = parse(
			"""
			{
			  "name": "Animatium",
			  "filename": "animatium-3.2+26.1.1-fabric.jar",
			  "side": "both",
			  "download": {
			    "hash-format": "sha1",
			    "hash": "ef966e0a894fa29c86a310a65f733d70bb497b04",
			    "mode": "metadata:curseforge"
			  },
			  "update": {
			    "curseforge": {
			      "file-id": 7892437,
			      "project-id": 1311114,
			      "release-channel": ""
			    }
			  }
			}
			"""
		)
		assertEquals(TomlValue.String("Animatium"), document.properties["name"])
		val update = assertIs<TomlValue.Map>(document.properties["update"])
		val curseforge = assertIs<TomlValue.Map>(update.properties["curseforge"])
		assertEquals(TomlValue.Integer(7892437), curseforge.properties["file-id"])
	}
}
