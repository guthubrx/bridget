package app.cartae.bridget

import android.content.Context
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.After
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.security.KeyStore
import java.util.UUID

@RunWith(AndroidJUnit4::class)
class ProtectedStoreTest {
    private val context: Context = InstrumentationRegistry.getInstrumentation().targetContext
    private val suffix = UUID.randomUUID().toString()
    private val database = "test-mobile177-$suffix.db"
    private val alias = "test.mobile177.$suffix"
    private fun store() = LocalStore(context, database, alias)
    private fun session(device: String = "89000000-0000-4000-8000-000000000101", key: String = LocalStore.newKey()) =
        JSONObject().put("endpoint", "https://bridge.example.com").put("device_key", key)
            .put("device", JSONObject().put("device_id", device))
    @After fun cleanup() {
        val directory = java.io.File(context.filesDir, "mobile-files-" + LocalStore.digest(alias.toByteArray()).take(16))
        directory.listFiles().orEmpty().forEach { it.delete() }
        directory.delete()
        context.deleteDatabase(database)
        KeyStore.getInstance("AndroidKeyStore").apply { load(null); deleteEntry(alias) }
    }
    @Test fun attachmentSurvivesReopenEncryptedAndBoundToItsCommand() {
        val vault = store()
        vault.put("session", session())
        val bytes = "Pièce jointe privée — fixture".toByteArray(Charsets.UTF_8)
        val entry = vault.enqueueFile("89000000-0000-4000-8000-000000000102",
            "preuve.txt", "text/plain", bytes.inputStream())
        val id = entry.getJSONObject("envelope").getString("command_id")
        val directory = java.io.File(context.filesDir, "mobile-files-" + LocalStore.digest(alias.toByteArray()).take(16))
        val encrypted = java.io.File(directory, "$id.enc").readBytes()
        assertFalse(encrypted.toString(Charsets.UTF_8).contains("Pièce jointe"))
        vault.close()
        val reopened = store()
        val restored = reopened.get("command:$id")!!
        assertArrayEquals(bytes, reopened.fileBytes(restored))
        val altered = JSONObject(restored.toString())
        val other = UUID.randomUUID().toString()
        altered.getJSONObject("envelope").put("command_id", other)
        java.io.File(directory, "$other.enc").writeBytes(encrypted)
        assertTrue(runCatching { reopened.fileBytes(altered) }.isFailure)
        java.io.File(directory, "$other.enc").delete()
        reopened.put("session", session(device = "89000000-0000-4000-8000-000000000199"))
        val error = runCatching { reopened.fileBytes(restored) }.exceptionOrNull()
        assertEquals("command_origin_changed", (error as MobileFailure).code)
        reopened.put("session", session())
        reopened.setCommandState(id, "outcome_unknown")
        assertTrue(runCatching { reopened.discardCommand(id) }.isFailure)
        assertTrue(java.io.File(directory, "$id.enc").exists())
        reopened.setCommandState(id, "accepted")
        reopened.discardCommand(id)
        assertNull(reopened.get("command:$id"))
        assertFalse(java.io.File(directory, "$id.enc").exists())
        reopened.close()
    }
    @Test fun attachmentRejectsPathsAndOversizeWithoutQueueing() {
        val vault = store()
        vault.put("session", session())
        val agent = "89000000-0000-4000-8000-000000000102"
        assertTrue(runCatching { vault.enqueueFile(agent, "../preuve.txt", "text/plain", byteArrayOf(1).inputStream()) }.isFailure)
        assertTrue(runCatching { vault.enqueueFile(agent, "preuve.html", "text/html", byteArrayOf(1).inputStream()) }.isFailure)
        assertTrue(runCatching { vault.enqueueFile(agent, "preuve.txt", "text/plain",
            ByteArray(LocalStore.MAX_FILE_BYTES + 1) { 32 }.inputStream()) }.isFailure)
        assertTrue(vault.list("command:").isEmpty())
        vault.close()
    }

    @Test fun forgettingAuthorizationPreservesCommandsAndDoesNotBorrowNewIdentity() {
        val vault = store()
        vault.put("session", session())
        vault.put("pairing", JSONObject().put("credential", LocalStore.newKey()))
        vault.put("renewal", JSONObject().put("old_key", LocalStore.newKey()))
        val entry = vault.enqueue("89000000-0000-4000-8000-000000000102", "Retain native receipt")
        val id = entry.getJSONObject("envelope").getString("command_id")
        vault.setCommandState(id, "outcome_unknown")
        vault.forgetAuthorization()
        assertNull(vault.get("session")); assertNull(vault.get("pairing")); assertNull(vault.get("renewal"))
        assertEquals("outcome_unknown", vault.get("command:$id")!!.getString("state"))
        vault.put("session", session(device = "89000000-0000-4000-8000-000000000199"))
        assertEquals("command_origin_changed", (runCatching {
            BridgetClient.requireCommandOrigin(entry, vault.get("session")!!)
        }.exceptionOrNull() as MobileFailure).code)
        vault.close()
    }
    @Test fun nativeObservationMustMatchMessageRecipientAndAssociation() {
        val vault = store()
        vault.put("session", session())
        val agent = "89000000-0000-4000-8000-000000000102"
        val entry = vault.enqueue(agent, "Observe the native delivery")
        val id = entry.getJSONObject("envelope").getString("command_id")
        val event = JSONObject().put("seq", 1).put("event", "turn_start").put("message_id", id)
        vault.observeMessage("89000000-0000-4000-8000-000000000199", event)
        assertFalse(vault.get("command:$id")!!.has("delivery_observation"))
        vault.observeMessage(agent, event)
        assertEquals("turn_start", vault.get("command:$id")!!.getJSONObject("delivery_observation").getString("event"))
        vault.observeMessage(agent, JSONObject(event.toString()).put("seq", 2).put("event", "turn_end"))
        assertEquals("turn_end", vault.get("command:$id")!!.getJSONObject("delivery_observation").getString("event"))
        vault.observeMessage(agent, event)
        assertEquals(2, vault.get("command:$id")!!.getJSONObject("delivery_observation").getInt("seq"))
        vault.close()
    }
    @Test fun actionEnvelopeHasOneStableNativeKeyAndNoArtificialTimestamp() {
        val vault = store()
        vault.put("session", session())
        val task = "89000000-0000-4000-8000-000000000102"
        val cancel = vault.enqueueAction("task_cancel", task)
        val answer = vault.enqueueAction("question_respond", task, "decline")
        assertFalse(cancel.getJSONObject("envelope").has("issued_at"))
        assertFalse(answer.getJSONObject("envelope").has("issued_at"))
        assertNotEquals(cancel.getJSONObject("envelope").getString("command_id"), answer.getJSONObject("envelope").getString("command_id"))
        assertEquals(cancel.toString(), vault.get("command:" + cancel.getJSONObject("envelope").getString("command_id"))!!.toString())
        vault.close()
    }

    @Test fun sessionAndOutboxSurviveReopenWithoutPlaintextSecrets() {
        val secret = LocalStore.newKey()
        val first = store()
        first.put("session", session(key = secret))
        val entry = first.enqueue("89000000-0000-4000-8000-000000000102", "private-fixture-text")
        val id = entry.getJSONObject("envelope").getString("command_id")
        first.setCommandState(id, "outcome_unknown")
        first.close()
        val reopened = store()
        assertEquals(secret, reopened.get("session")!!.getString("device_key"))
        val restored = reopened.get("command:$id")!!
        assertEquals(entry.getJSONObject("envelope").toString(), restored.getJSONObject("envelope").toString())
        assertEquals("outcome_unknown", restored.getString("state"))
        reopened.readableDatabase.rawQuery("SELECT value FROM entries", null).use { cursor ->
            while (cursor.moveToNext()) {
                val bytes = cursor.getBlob(0).toString(Charsets.UTF_8)
                assertFalse(bytes.contains(secret)); assertFalse(bytes.contains("private-fixture-text"))
            }
        }
        reopened.close()
    }
    @Test fun reassociationCannotSendOrLookupAnOriginalCommand() = kotlinx.coroutines.runBlocking {
        val vault = store()
        val original = session()
        vault.put("session", original)
        val entry = vault.enqueue("89000000-0000-4000-8000-000000000102", "Keep origin")
        val id = entry.getJSONObject("envelope").getString("command_id")
        val client = BridgetClient(vault)
        for (changed in listOf(
            session(device = "89000000-0000-4000-8000-000000000199"),
            session().put("endpoint", "https://another.example.com")
        )) {
            vault.put("session", changed)
            for (lookup in listOf(false, true)) {
                val error = runCatching { if (lookup) client.reconcile(entry) else client.send(entry) }.exceptionOrNull()
                assertTrue(error is MobileFailure)
                assertEquals("command_origin_changed", (error as MobileFailure).code)
                assertEquals("local", vault.get("command:$id")!!.getString("state"))
            }
        }
        BridgetClient.requireCommandOrigin(entry, session(key = LocalStore.newKey()))
        vault.close()
    }
    @Test fun draftScopeSurvivesRotationButNeverCrossesAssociationOrEndpoint() {
        val vault = store()
        val agent = "89000000-0000-4000-8000-000000000102"
        vault.put("session", session())
        val originalKey = vault.draftKey(agent)
        vault.put(originalKey, JSONObject().put("body", "Original draft"))
        vault.put("session", session(key = LocalStore.newKey()))
        assertEquals("Original draft", vault.get(vault.draftKey(agent))!!.getString("body"))
        vault.put("session", session(device = "89000000-0000-4000-8000-000000000199"))
        assertNull(vault.get(vault.draftKey(agent)))
        vault.put("session", session().put("endpoint", "https://another.example.com"))
        assertNull(vault.get(vault.draftKey(agent)))
        assertEquals("Original draft", vault.get(originalKey)!!.getString("body"))
        vault.close()
    }
    @Test fun pendingRenewalNeverUsesAnotherAssociationOrServer() = kotlinx.coroutines.runBlocking {
        val vault = store()
        val pending = JSONObject().put("endpoint", "https://bridge.example.com")
            .put("device_id", "89000000-0000-4000-8000-000000000101")
            .put("old_key", LocalStore.newKey()).put("new_key", LocalStore.newKey())
            .put("request_id", UUID.randomUUID().toString())
        vault.put("renewal", pending)
        val client = BridgetClient(vault)
        for (changed in listOf(session(device = "89000000-0000-4000-8000-000000000199"),
            session().put("endpoint", "https://another.example.com"))) {
            vault.put("session", changed)
            val error = runCatching { client.renew() }.exceptionOrNull()
            assertEquals("authorization_origin_changed", (error as MobileFailure).code)
            assertEquals(pending.toString(), vault.get("renewal")!!.toString())
            assertEquals(changed.toString(), vault.get("session")!!.toString())
        }
        vault.close()
    }
    @Test fun rowSubstitutionCannotDecryptUnderAnotherIdentity() {
        val vault = store()
        vault.put("session", JSONObject().put("device_key", LocalStore.newKey()))
        vault.writableDatabase.execSQL("UPDATE entries SET id='other' WHERE id='session'")
        assertTrue(runCatching { vault.get("other") }.isFailure)
        vault.close()
    }
    @Test fun lostKeystoreKeyNeverCreatesAReplacementOverExistingCommands() {
        val vault = store()
        vault.put("session", session())
        val entry = vault.enqueue("89000000-0000-4000-8000-000000000102", "Retain this envelope")
        val id = entry.getJSONObject("envelope").getString("command_id")
        val keystore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        keystore.deleteEntry(alias)
        assertTrue(runCatching { vault.get("command:$id") }.isFailure)
        assertFalse(keystore.containsAlias(alias))
        vault.readableDatabase.rawQuery("SELECT count(*) FROM entries", null).use {
            assertTrue(it.moveToFirst()); assertEquals(2, it.getInt(0))
        }
        vault.close()
    }
    @Test fun notificationNeedsConsentExactAssociationAndNoBusinessPayload() {
        val vault = store()
        val association=session().apply { getJSONObject("device").put("expires_at", System.currentTimeMillis()/1000+600) }
        vault.put("session", association)
        val binding=MobileNotifications.binding(association)
        val data=mapOf("version" to "1", "binding" to binding)
        assertFalse(MobileNotifications.accepts(vault,data))
        vault.put("push:$binding",JSONObject().put("enabled",true))
        assertTrue(MobileNotifications.accepts(vault,data))
        assertFalse(MobileNotifications.accepts(vault,data+mapOf("body" to "untrusted")))
        assertFalse(MobileNotifications.accepts(vault,data+mapOf("binding" to "other")))
        vault.put("session", session(device="89000000-0000-4000-8000-000000000199"))
        assertFalse(MobileNotifications.accepts(vault,data))
        vault.put("session",association.apply { getJSONObject("device").put("expires_at",1) })
        assertFalse(MobileNotifications.accepts(vault,data))
        vault.remove("session")
        assertFalse(MobileNotifications.accepts(vault,data))
        vault.close()
    }

    @Test fun quietHoursAreLocalPersistentAndBoundToTheAssociation() {
        val vault = store(); val association = session(); vault.put("session", association)
        assertFalse(MobileNotifications.quietNow(vault, 23 * 60))
        MobileNotifications.setQuiet(vault, true, "22:00", "08:00")
        assertTrue(MobileNotifications.quietNow(vault, 22 * 60))
        assertTrue(MobileNotifications.quietNow(vault, 7 * 60 + 59))
        assertFalse(MobileNotifications.quietNow(vault, 8 * 60))
        assertFalse(MobileNotifications.quietNow(vault, 12 * 60))
        assertTrue(runCatching { MobileNotifications.setQuiet(vault, true, "24:00", "08:00") }.isFailure)
        assertTrue(runCatching { MobileNotifications.setQuiet(vault, true, "08:00", "08:00") }.isFailure)
        vault.close()
        store().use { reopened ->
            assertTrue(MobileNotifications.quietNow(reopened, 23 * 60))
            reopened.put("session", session(device="89000000-0000-4000-8000-000000000199"))
            assertFalse(MobileNotifications.quietNow(reopened, 23 * 60))
            reopened.put("session", association)
            MobileNotifications.setQuiet(reopened, true, "12:00", "13:00")
            assertTrue(MobileNotifications.quietNow(reopened, 12 * 60))
            assertFalse(MobileNotifications.quietNow(reopened, 13 * 60))
            MobileNotifications.setQuiet(reopened, false, "12:00", "13:00")
            assertFalse(MobileNotifications.quietNow(reopened, 12 * 60))
        }
    }

    @Test fun updatesAcceptOnlyStableOfficialApkAndNeverAnArbitraryUrl() {
        fun metadata(version: String = "0.5.0") = JSONObject().put("draft", false).put("prerelease", false)
            .put("tag_name", "v$version").put("assets", org.json.JSONArray().put(JSONObject()
                .put("name", "bridget-android.apk").put("size", 1024)
                .put("browser_download_url", "https://github.com/guthubrx/bridget/releases/download/v$version/bridget-android.apk")))
        assertEquals("0.5.0", MobileRelease.parse(metadata(), "0.4.1")!!.version)
        assertNull(MobileRelease.parse(metadata(), "0.5.0"))
        assertNull(MobileRelease.parse(metadata(), "0.6.0"))
        assertTrue(runCatching { MobileRelease.parse(metadata().put("draft", true), "0.4.1") }.isFailure)
        val foreign = metadata()
        foreign.getJSONArray("assets").getJSONObject(0).put("browser_download_url", "https://example.com/private.apk")
        assertTrue(runCatching { MobileRelease.parse(foreign, "0.4.1") }.isFailure)
        assertTrue(runCatching { MobileRelease.parse(metadata().put("assets", org.json.JSONArray()), "0.4.1") }.isFailure)
        assertTrue(runCatching { MobileRelease.parse(metadata("0.5.0-rc1"), "0.4.1") }.isFailure)
    }

}
