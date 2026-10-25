package app.cartae.bridget
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import kotlinx.coroutines.runBlocking
import java.io.File
import java.security.KeyStore
@RunWith(AndroidJUnit4::class)
class UserInputStoreTest {
    @Test fun draftsAndCommandsStayBoundAndSurviveReopen() {
        val context=InstrumentationRegistry.getInstrumentation().targetContext
        val name="user-input181-store.db";val alias="test.mobile181.input"
        context.deleteDatabase(name);KeyStore.getInstance("AndroidKeyStore").apply {load(null);deleteEntry(alias)}
        val id="89000000-0000-4000-8000-000000000181"
        val item=JSONObject().put("request_id",id).put("revision","a".repeat(64))
        val answers=JSONObject("{\"q\":{\"selected\":[\"A\"],\"text\":\"Une précision\"}}")
        var key="";var command=""
        try {
            LocalStore(context,name,alias).use {store->
                store.put("session",JSONObject().put("endpoint","https://bridge.example.com").put("device_key",LocalStore.newKey()).put("device",JSONObject().put("device_id",id)))
                key=store.userInputDraftKey(item);store.put(key,JSONObject().put("answers",answers).put("updated_at",System.currentTimeMillis()))
                val entry=store.enqueueUserInput(item,answers);command=entry.getJSONObject("envelope").getString("command_id")
                assertEquals(command,store.enqueueUserInput(item,answers,command).getJSONObject("envelope").getString("command_id"))
                store.setCommandState(command,"outcome_unknown")
            }
            LocalStore(context,name,alias).use {store->
                assertEquals("Une précision",store.get(key)!!.getJSONObject("answers").getJSONObject("q").getString("text"))
                assertEquals("outcome_unknown",store.get("command:$command")!!.getString("state"))
                val other=JSONObject(item.toString()).put("revision","b".repeat(64))
                assertNotEquals(key,store.userInputDraftKey(other))
                assertNull(store.get(store.userInputDraftKey(other)))
                assertTrue(runCatching {store.enqueueUserInput(item,JSONObject("{\"q\":{\"selected\":[],\"text\":\"changed\"}}"),command)}.isFailure)
            }
        } finally {context.deleteDatabase(name);KeyStore.getInstance("AndroidKeyStore").apply {load(null);deleteEntry(alias)}}
    }
    @Test fun associateIsolatedQaInstanceWhenExplicitlyProvided()=runBlocking {
        val context=InstrumentationRegistry.getInstrumentation().targetContext
        val source=File(context.filesDir,"qa181-pairing.json")
        assumeTrue("Private QA pairing not provided",source.isFile)
        LocalStore(context).use {store->
            if(store.get("session")==null) BridgetClient(store).pair(if(store.get("pairing")!=null) null else source.readText())
            assertEquals("https://qa181.bridget.cartae.app",store.get("session")!!.getString("endpoint"))
            assertTrue(BridgetClient(store).read("/v1/user-inputs?limit=10").has("items"))
        }
        source.delete()
        Unit
    }
}
