package link.infra.packwiz.installer.target.path

import link.infra.packwiz.installer.request.RequestException
import link.infra.packwiz.installer.target.ClientHolder
import okhttp3.HttpUrl
import okhttp3.Request
import okio.BufferedSource
import okio.IOException

class HttpUrlPath(
	private val url: HttpUrl,
	path: String? = null,
	private val requestHeaders: Map<String, String> = emptyMap()
): PackwizPath<HttpUrlPath>(path) {
	private fun build() = if (path == null) { url } else { url.newBuilder().addPathSegments(path).build() }
	internal fun request(): Request {
		val builder = Request.Builder()
			.url(build())
			.header("Accept", "application/octet-stream")
			.header("User-Agent", "packwiz-installer")
			.get()
		for ((name, value) in requestHeaders) {
			builder.header(name, value)
		}
		return builder.build()
	}

	@Throws(RequestException::class)
	override fun source(clientHolder: ClientHolder): BufferedSource {
		val req = request()
		try {
			val res = clientHolder.okHttpClient.newCall(req).execute()
			// Can't use .use since it would close the response body before returning it to the caller
			try {
				if (!res.isSuccessful) {
					if ((res.code == 401 || res.code == 403) && req.header("X-API-Key") != null) {
						throw RequestException.Response.HTTP.APIKeyRejected(req, res)
					}
					throw RequestException.Response.HTTP.ErrorCode(req, res)
				}

				val body = res.body ?: throw RequestException.Internal.HTTP.NoResponseBody()
				return body.source()
			} catch (e: Exception) {
				// If an exception is thrown, close the response and rethrow
				res.close()
				throw e
			}
		} catch (e: IOException) {
			throw RequestException.Internal.HTTP.RequestFailed(e)
		} catch (e: IllegalStateException) {
			throw RequestException.Internal.HTTP.IllegalState(e)
		}
	}

	override fun construct(path: String): HttpUrlPath = HttpUrlPath(url, path, requestHeaders)

	override val folder: Boolean
		get() = pathFolder ?: (url.pathSegments.last() == "")
	override val filename: String
		get() = pathFilename ?: url.pathSegments.last()

	override fun equals(other: Any?): Boolean {
		if (this === other) return true
		if (javaClass != other?.javaClass) return false
		if (!super.equals(other)) return false

		other as HttpUrlPath

		if (url != other.url) return false
		if (requestHeaders != other.requestHeaders) return false

		return true
	}

	override fun hashCode(): Int {
		var result = super.hashCode()
		result = 31 * result + url.hashCode()
		result = 31 * result + requestHeaders.hashCode()
		return result
	}

	override fun toString() = build().toString()
}
