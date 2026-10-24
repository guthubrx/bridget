package app.cartae.bridget

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class UpgradeStoreTest {
    private val context = InstrumentationRegistry.getInstrumentation().targetContext
    private val database = "upgrade-mobile177.db"
    private val alias = "bridget.mobile177.upgrade-proof"
    @Test fun writeBeforeUpgrade() {
        LocalStore(context, database, alias).use { store ->
            store.put("proof", JSONObject().put("value", "preserve177"))
            assertEquals("preserve177", store.get("proof")!!.getString("value"))
        }
    }
}
