package app.cartae.bridget

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.UUID

@RunWith(AndroidJUnit4::class)
class Updates179InstrumentationTest {
    private fun metadata(version: String = "0.6.0") = JSONObject()
        .put("draft", false).put("prerelease", false).put("tag_name", "v$version")
        .put("assets", JSONArray().put(JSONObject().put("name", "bridget-android.apk")
            .put("size", 12).put("digest", "sha256:" + "a".repeat(64))
            .put("browser_download_url", "https://github.com/guthubrx/bridget/releases/download/v$version/bridget-android.apk")))

    @Test fun olderReleaseWithoutApkIsNotAnUpdate() {
        val value = metadata("0.4.1").put("assets", JSONArray())
        assertNull(MobileRelease.parse(value, "0.5.0"))
        assertNull(MobileRelease.parse(metadata("0.5.0"), "0.5.0"))
        assertEquals("0.10.0", MobileRelease.parse(metadata("0.10.0"), "0.9.99")!!.version)
    }
    @Test fun newReleaseNeedsExactOfficialAssetAndDigest() {
        for (field in listOf("digest", "browser_download_url", "size")) {
            val value = metadata()
            value.getJSONArray("assets").getJSONObject(0).remove(field)
            assertTrue(runCatching { MobileRelease.parse(value, "0.5.0") }.isFailure)
        }
        val duplicate = metadata()
        duplicate.getJSONArray("assets").put(duplicate.getJSONArray("assets").getJSONObject(0))
        assertTrue(runCatching { MobileRelease.parse(duplicate, "0.5.0") }.isFailure)
        for (field in listOf("draft", "prerelease")) {
            assertTrue(runCatching { MobileRelease.parse(metadata().put(field, true), "0.5.0") }.isFailure)
        }
    }
    @Test fun validDigestDoesNotMakeAnInvalidApkInstallable() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        val file = File(context.cacheDir, "updates179-" + UUID.randomUUID() + ".apk")
        val content = "This is not an APK".toByteArray()
        file.writeBytes(content)
        try {
            val digest = java.security.MessageDigest.getInstance("SHA-256").digest(content)
                .joinToString("") { "%02x".format(it.toInt() and 255) }
            val release = MobileRelease("0.6.0", "https://github.com/guthubrx/bridget/releases/download/v0.6.0/bridget-android.apk", digest, content.size.toLong())
            assertEquals("update_integrity_failed", (runCatching { MobileUpdates.verifyApk(context, file, release) }.exceptionOrNull() as MobileFailure).code)
        } finally { file.delete() }
    }
}
