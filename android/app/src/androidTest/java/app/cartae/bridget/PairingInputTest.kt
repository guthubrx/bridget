package app.cartae.bridget
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.util.UUID

@RunWith(AndroidJUnit4::class)
class PairingInputTest {
    private fun qr() = JSONObject().put("version", 1).put("endpoint", "https://bridget.example.net")
        .put("pairing_id", UUID.randomUUID().toString()).put("credential", "a".repeat(64))
    @Test fun malformedQrCannotBecomeAStorageFailureOrContactAnotherEndpoint() {
        assertEquals("https://bridget.example.net", BridgetClient.validatedPairing(qr().toString()).getString("endpoint"))
        listOf(qr().put("version", "1"), qr().put("credential", 123), qr().put("scope", "all"),
            qr().put("endpoint", "https://user:secret@bridget.example.net"),
            qr().put("endpoint", "http://bridget.example.net"), qr().put("pairing_id", "invalid"),
            JSONObject().put("endpoint", "https://bridget.example.net")
        ).forEach { value ->
            val result = runCatching { BridgetClient.validatedPairing(value.toString()) }
            assertTrue(result.exceptionOrNull() is MobileFailure)
            assertEquals("invalid_qr", (result.exceptionOrNull() as MobileFailure).code)
        }
    }
}
