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
}
