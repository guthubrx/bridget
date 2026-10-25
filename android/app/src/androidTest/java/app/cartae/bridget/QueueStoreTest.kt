package app.cartae.bridget

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.security.KeyStore

@RunWith(AndroidJUnit4::class)
class QueueStoreTest {
    @Test fun queuedTextRemainsEditableOnlyBeforeSendAndNativeObservationMatchesTheBridgetEnvelope() {
        val context=InstrumentationRegistry.getInstrumentation().targetContext
        val alias="test.mobile180.queue"
        val name="test-mobile180-queue.db"
        val agent="89000000-0000-4000-8000-000000000180"
        context.deleteDatabase(name)
        KeyStore.getInstance("AndroidKeyStore").apply {load(null); deleteEntry(alias)}
        val store=LocalStore(context,name,alias)
        try {
            store.put("session",JSONObject().put("endpoint","https://bridge.example.com").put("device_key",LocalStore.newKey())
                .put("device",JSONObject().put("device_id","89000000-0000-4000-8000-000000000181")))
            val entry=store.enqueue(agent,"First")
            val id=entry.getJSONObject("envelope").getString("command_id")
            store.editUnsentMessage(id,"Edited before send")
            assertEquals("Edited before send",store.get("command:$id")!!.getJSONObject("envelope").getString("body"))
            store.setCommandState(id,"outcome_unknown")
            assertTrue(runCatching {store.editUnsentMessage(id,"Must not change")}.isFailure)
            val native="89000000-0000-4000-8000-000000000182"
            store.observeMessage(agent,JSONObject().put("event","turn_start").put("seq",7).put("message_id",native)
                .put("payload",JSONObject().put("body","💬 Instruction de l’utilisateur (id=$id)\nEdited before send")))
            assertEquals("turn_start",store.get("command:$id")!!.getJSONObject("delivery_observation").getString("event"))
            store.observeMessage(agent,JSONObject().put("event","turn_end").put("seq",8).put("message_id",native))
            assertEquals("turn_end",store.get("command:$id")!!.getJSONObject("delivery_observation").getString("event"))
            store.observeMessage(agent,JSONObject().put("event","turn_start").put("seq",7).put("message_id",native))
            assertEquals(8,store.get("command:$id")!!.getJSONObject("delivery_observation").getInt("seq"))
        } finally {
            store.close(); context.deleteDatabase(name)
            KeyStore.getInstance("AndroidKeyStore").apply {load(null);deleteEntry(alias)}
        }
    }
    @Test fun preparedFilesStayLocalUntilSendAndCompositionCommitsAtomically() = kotlinx.coroutines.runBlocking {
        val context=InstrumentationRegistry.getInstrumentation().targetContext
        val alias="test.mobile183.prepared";val name="test-mobile183-prepared.db"
        val agent="89000000-0000-4000-8000-000000000183"
        val files=java.io.File(context.filesDir,"mobile-files-"+LocalStore.digest(alias.toByteArray()).take(16))
        files.listFiles().orEmpty().forEach {it.delete()};files.delete()
        context.deleteDatabase(name)
        KeyStore.getInstance("AndroidKeyStore").apply {load(null);deleteEntry(alias)}
        val store=LocalStore(context,name,alias)
        try {
            store.put("session",JSONObject().put("endpoint","https://bridge.example.com").put("device_key",LocalStore.newKey())
                .put("device",JSONObject().put("device_id","89000000-0000-4000-8000-000000000184")))
            val file=store.enqueueFile(agent,"prepared.html","text/html","local bytes".byteInputStream(),staged=true)
            val id=file.getJSONObject("envelope").getString("command_id")
            assertTrue(store.get("command:$id")!!.getBoolean("staged"))
            val failure=runCatching {BridgetClient(store).send(file)}.exceptionOrNull()
            assertTrue(failure is MobileFailure && failure.code=="attachment_not_sent")
            assertEquals("local",store.get("command:$id")!!.getString("state"))
            assertArrayEquals("local bytes".toByteArray(),store.fileBytes(file))
            store.removePreparedFile(id);assertNull(store.get("command:$id"))
            val next=store.enqueueFile(agent,"prepared.html","text/html","local bytes".byteInputStream(),staged=true)
            val key=next.getJSONObject("envelope").getString("command_id")
            val released=store.releasePreparedFile(key)
            assertFalse(released.getBoolean("staged"))
            assertEquals(next.getJSONObject("envelope").toString(),released.getJSONObject("envelope").toString())
            store.setCommandState(key,"accepted",JSONObject().put("receipt",JSONObject().put("artifact_ref","artifact:test").put("version_ref","version:test")))
            store.put(store.draftKey(agent),JSONObject().put("body","Brouillon"))
            assertTrue(runCatching {store.enqueueComposed(agent,"",listOf(key))}.isFailure)
            assertTrue(store.get("command:$key")!!.getBoolean("compose_attachment"))
            assertEquals("Brouillon",store.get(store.draftKey(agent))!!.getString("body"))
            val message=store.enqueueComposed(agent,"Message with its exact reference",listOf(key))
            assertEquals("message",message.getJSONObject("envelope").getString("type"))
            assertFalse(store.get("command:$key")!!.getBoolean("compose_attachment"))
            assertNull(store.get(store.draftKey(agent)))
            assertEquals("accepted",store.get("command:$key")!!.getString("state"))
        } finally {
            store.list("command:").forEach {store.discardCommand(it.getJSONObject("envelope").getString("command_id"))}
            store.close();context.deleteDatabase(name)
            files.listFiles().orEmpty().forEach {it.delete()};files.delete()
            KeyStore.getInstance("AndroidKeyStore").apply {load(null);deleteEntry(alias)}
        }
    }
}
