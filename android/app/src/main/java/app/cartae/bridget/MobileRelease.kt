package app.cartae.bridget

import java.io.File
import java.net.URI
import java.net.URL
import javax.net.ssl.HttpsURLConnection
import org.json.JSONObject

/** Publication officielle : aucun credential Bridget n'entre dans ce client. */
data class MobileRelease(val version: String, val url: String, val sha256: String, val size: Long) {
    fun json() = JSONObject().put("version", version).put("url", url).put("sha256", sha256).put("size", size)
    fun download(destination: File, permitted: () -> Boolean) {
        val temporary = File(destination.parentFile, destination.name + ".part")
        try {
            val digest = java.security.MessageDigest.getInstance("SHA-256")
            var count = 0L
            temporary.outputStream().use { output ->
                fetch(url, size, 180_000, true) { bytes, length ->
                    if (!permitted()) throw MobileFailure("update_disabled")
                    count += length
                    digest.update(bytes, 0, length)
                    output.write(bytes, 0, length)
                }
                output.flush()
            }
            val actual = digest.digest().joinToString("") { "%02x".format(it.toInt() and 255) }
            if (count != size || actual != sha256) throw MobileFailure("update_integrity_failed")
            if (!permitted()) throw MobileFailure("update_disabled")
            if (!temporary.renameTo(destination)) throw MobileFailure("update_storage_unavailable")
        } finally { temporary.delete() }
    }
    companion object {
        private const val API = "https://api.github.com/repos/guthubrx/bridget/releases/latest"
        const val MAX_APK_BYTES = 100L * 1024 * 1024
        fun versionCode(value: String): Int {
            require(value.matches(Regex("(0|[1-9][0-9]{0,2})[.](0|[1-9][0-9]{0,2})[.](0|[1-9][0-9]{0,2})")))
            val parts = value.split(".").map(String::toInt)
            return parts[0] * 1_000_000 + parts[1] * 1_000 + parts[2]
        }
        fun stored(value: JSONObject): MobileRelease {
            val version = value.getString("version")
            versionCode(version)
            val url = "https://github.com/guthubrx/bridget/releases/download/v$version/bridget-android.apk"
            require(value.getString("url") == url)
            val sha = value.getString("sha256")
            require(sha.matches(Regex("[0-9a-f]{64}")))
            val size = value.getLong("size")
            require(size in 1..MAX_APK_BYTES)
            return MobileRelease(version, url, sha, size)
        }
        fun parse(json: JSONObject, installed: String): MobileRelease? {
            require(!json.optBoolean("draft", true) && !json.optBoolean("prerelease", true))
            val tag = json.getString("tag_name")
            require(tag.startsWith("v"))
            val version = tag.removePrefix("v")
            // Une ancienne release peut ne pas encore publier Android.
            if (versionCode(version) <= versionCode(installed)) return null
            val assets = json.getJSONArray("assets")
            require(assets.length() <= 64)
            val matches = (0 until assets.length()).map { assets.getJSONObject(it) }
                .filter { it.optString("name") == "bridget-android.apk" }
            if (matches.isEmpty()) throw MobileFailure("android_release_unpublished")
            require(matches.size == 1)
            val asset = matches.single()
            val digest = asset.getString("digest")
            require(digest.startsWith("sha256:"))
            return stored(JSONObject().put("version", version).put("url", asset.getString("browser_download_url"))
                .put("sha256", digest.removePrefix("sha256:")).put("size", asset.getLong("size")))
        }
        fun check(installed: String): MobileRelease? {
            try {
                val output = java.io.ByteArrayOutputStream()
                fetch(API, 262_144, 15_000, false) { bytes, length -> output.write(bytes, 0, length) }
                return parse(JSONObject(output.toString("UTF-8")), installed)
            } catch (failure: MobileFailure) { throw failure }
            catch (_: Exception) { throw MobileFailure("update_check_unavailable") }
        }
        internal fun allowedDownload(url: String): Boolean = runCatching {
            val uri = URI(url)
            uri.scheme == "https" && uri.userInfo == null && uri.port in setOf(-1, 443) && uri.fragment == null &&
                uri.host in setOf("github.com", "release-assets.githubusercontent.com")
        }.getOrDefault(false)
        private fun fetch(initial: String, limit: Long, timeout: Long, redirects: Boolean, consume: (ByteArray, Int) -> Unit) {
            val deadline = System.nanoTime() + timeout * 1_000_000
            var location = initial
            repeat(4) { attempt ->
                if (redirects && !allowedDownload(location)) throw MobileFailure("update_integrity_failed")
                val connection = URL(location).openConnection() as HttpsURLConnection
                try {
                    connection.instanceFollowRedirects = false
                    connection.connectTimeout = 10_000
                    connection.readTimeout = 10_000
                    connection.setRequestProperty("User-Agent", "Bridget-Android")
                    connection.setRequestProperty("Accept", if (redirects) "application/octet-stream" else "application/vnd.github+json")
                    val code = connection.responseCode
                    if (code in setOf(301, 302, 303, 307, 308) && redirects && attempt < 3) {
                        location = URL(URL(location), connection.getHeaderField("Location") ?: "").toString()
                    } else {
                        if (code != 200 || connection.contentLengthLong > limit) throw MobileFailure("update_check_unavailable")
                        var count = 0L
                        connection.inputStream.use { input ->
                            val buffer = ByteArray(32_768)
                            while (true) {
                                val remaining = (deadline - System.nanoTime()) / 1_000_000
                                if (remaining <= 0) throw MobileFailure("update_check_unavailable")
                                connection.readTimeout = remaining.coerceAtMost(10_000).toInt()
                                val n = input.read(buffer)
                                if (n < 0) break
                                count += n
                                if (count > limit) throw MobileFailure("update_integrity_failed")
                                consume(buffer, n)
                            }
                        }
                        return
                    }
                } finally { connection.disconnect() }
            }
            throw MobileFailure("update_check_unavailable")
        }
    }
}
