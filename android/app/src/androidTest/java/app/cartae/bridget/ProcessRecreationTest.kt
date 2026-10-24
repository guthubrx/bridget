package app.cartae.bridget

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.security.KeyStore

/** Le contrôleur USB arrête réellement le processus entre les deux étapes, sans données de production. */
@RunWith(AndroidJUnit4::class)
class ProcessRecreationTest {
    private val context = InstrumentationRegistry.getInstrumentation().targetContext
    private val database = "test-mobile177-process-recreation.db"
    private val alias = "test.mobile177.process.recreation"
    private fun cleanup() {
        context.deleteDatabase(database)
        val directory = File(context.filesDir, "mobile-files-" + LocalStore.digest(alias.toByteArray()).take(16))
        directory.listFiles().orEmpty().forEach { it.delete() }
        directory.delete()
        KeyStore.getInstance("AndroidKeyStore").apply { load(null); deleteEntry(alias) }
    }
    @Test fun protectedCommandsAndFilesSurviveAnExternalProcessStop() {
        val stage = InstrumentationRegistry.getArguments().getString("processStage")
        assumeTrue("This test requires the USB controller to stop the process between write and read.", stage in setOf("write", "read"))
        if (stage == "write") {
            cleanup()
            val vault = LocalStore(context, database, alias)
            vault.put("session", JSONObject().put("endpoint", "https://bridge.example.com")
                .put("device_key", LocalStore.newKey()).put("device", JSONObject()
                    .put("device_id", "89000000-0000-4000-8000-000000000101")))
            val agent = "89000000-0000-4000-8000-000000000102"
            val message = vault.enqueue(agent, "Process recreation fixture")
            val file = vault.enqueueFile(agent, "preuve.txt", "text/plain", "Persistent encrypted file".byteInputStream())
            val action = vault.enqueueAction("task_cancel", agent)
            vault.setCommandState(message.getJSONObject("envelope").getString("command_id"), "outcome_unknown")
            vault.setCommandState(file.getJSONObject("envelope").getString("command_id"), "outcome_unknown")
            vault.put("process-fixture", JSONObject().put("message", message).put("file", file).put("action", action))
            vault.put(vault.draftKey(agent), JSONObject().put("body", "Draft before process stop"))
            vault.close()
            return
        }
        try {
            val vault = LocalStore(context, database, alias)
            val fixture = vault.get("process-fixture")!!
            assertEquals(3, vault.list("command:").size)
            for (kind in listOf("message", "file", "action")) {
                val expected = fixture.getJSONObject(kind)
                val id = expected.getJSONObject("envelope").getString("command_id")
                val restored = vault.get("command:$id")!!
                assertEquals(expected.getJSONObject("envelope").toString(), restored.getJSONObject("envelope").toString())
                if (kind != "action") assertEquals("outcome_unknown", restored.getString("state"))
            }
            val staged = fixture.getJSONObject("file")
            assertEquals("Persistent encrypted file", vault.fileBytes(staged).toString(Charsets.UTF_8))
            assertEquals("Draft before process stop", vault.get(vault.draftKey("89000000-0000-4000-8000-000000000102"))!!.getString("body"))
            vault.close()
        } finally { cleanup() }
    }
}
