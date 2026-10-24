package app.cartae.bridget

import java.net.URL
import javax.net.ssl.HttpsURLConnection
import org.json.JSONObject

data class MobileRelease(val version: String, val url: String) {
    companion object {
        private const val LIMIT = 65_536
        private const val API = "https://api.github.com/repos/guthubrx/bridget/releases/latest"
        private fun parts(value: String): List<Int> {
            require(value.matches(Regex("[0-9]{1,3}[.][0-9]{1,3}[.][0-9]{1,3}")))
            return value.split(".").map(String::toInt)
        }
        fun parse(json: JSONObject, installed: String): MobileRelease? {
            require(!json.optBoolean("draft", true) && !json.optBoolean("prerelease", true))
            val tag = json.getString("tag_name")
            require(tag.startsWith("v"))
            val version = tag.removePrefix("v")
            val latest = parts(version)
            val current = parts(installed)
            val assets = json.getJSONArray("assets")
            require(assets.length() <= 64)
            val matches = (0 until assets.length()).map { assets.getJSONObject(it) }
                .filter { it.optString("name") == "bridget-android.apk" }
            if (matches.isEmpty()) throw MobileFailure("android_release_unpublished")
            require(matches.size == 1)
            val asset = matches.single()
            require(asset.getLong("size") in 1..100L * 1024 * 1024)
            val url = "https://github.com/guthubrx/bridget/releases/download/$tag/bridget-android.apk"
            require(asset.getString("browser_download_url") == url)
            val newer = latest.zip(current).firstOrNull { (a, b) -> a != b }?.let { it.first > it.second } ?: false
            return if (newer) MobileRelease(version, url) else null
        }
        fun check(installed: String): MobileRelease? {
            // Public release metadata only. No associated session or credential is accepted.
            val connection = URL(API).openConnection() as HttpsURLConnection
            val deadline = System.nanoTime() + 10_000_000_000L
            try {
                connection.instanceFollowRedirects = false
                connection.connectTimeout = 10_000
                connection.readTimeout = 10_000
                connection.setRequestProperty("Accept", "application/vnd.github+json")
                connection.setRequestProperty("X-GitHub-Api-Version", "2022-11-28")
                connection.setRequestProperty("User-Agent", "Bridget-Android")
                if (connection.responseCode != 200) throw MobileFailure("update_check_unavailable")
                if (connection.contentLengthLong > LIMIT) throw MobileFailure("update_check_unavailable")
                val output = java.io.ByteArrayOutputStream()
                connection.inputStream.use { input ->
                    val buffer = ByteArray(4096)
                    while (true) {
                        val remaining = (deadline - System.nanoTime()) / 1_000_000L
                        if (remaining <= 0) throw MobileFailure("update_check_unavailable")
                        connection.readTimeout = remaining.coerceAtMost(10_000).toInt()
                        val count = input.read(buffer)
                        if (count < 0) break
                        if (output.size() + count > LIMIT) throw MobileFailure("update_check_unavailable")
                        output.write(buffer, 0, count)
                    }
                }
                return parse(JSONObject(output.toString("UTF-8")), installed)
            } catch (failure: MobileFailure) {
                throw failure
            } catch (_: Exception) {
                throw MobileFailure("update_check_unavailable")
            } finally {
                connection.disconnect()
            }
        }
    }
}
