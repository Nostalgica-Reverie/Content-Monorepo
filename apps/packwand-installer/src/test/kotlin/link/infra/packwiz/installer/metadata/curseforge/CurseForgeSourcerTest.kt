package link.infra.packwiz.installer.metadata.curseforge

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse

class CurseForgeSourcerTest {
	@Test
	fun downloadPathAuthenticatesCdnRequest() {
		val apiKey = "test-api-key"
		val request = createCurseForgeDownloadPath(
			"https://edge.forgecdn.net/files/1234/56/ExampleMod.jar",
			apiKey
		).request()

		assertFalse(request.header("X-API-Key").isNullOrBlank())
		assertEquals(apiKey, request.header("X-API-Key"))
		assertEquals("application/octet-stream", request.header("Accept"))
		assertEquals("packwiz-installer", request.header("User-Agent"))
	}
}
