package app.cartae.bridget

import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class MobileModelsAndFiltersTest {
    @Test
    fun notificationFilterRespectsProjectAndUrgency() {
        val prefs = NotificationFilterPreferences(
            projects = setOf("bridget"),
            minUrgency = NotificationUrgency.HIGH,
        )
        assertTrue(NotificationFilterController.shouldNotify(prefs, "bridget", "high"))
        assertFalse(NotificationFilterController.shouldNotify(prefs, "bridget", "normal"))
        assertFalse(NotificationFilterController.shouldNotify(prefs, "autre", "high"))

        val roundtrip = NotificationFilterPreferences.fromJson(prefs.toJson())
        assertEquals(prefs, roundtrip)
    }

    @Test
    fun ledgerSearchFiltersByTermAndProject() {
        val payload = JSONObject().put(
            "items",
            JSONArray()
                .put(JSONObject().put("id", "1").put("agent_id", "a1").put("project", "bridget").put("from", "codex").put("body", "Audit sécurité terminé").put("ts", 100L))
                .put(JSONObject().put("id", "2").put("agent_id", "a2").put("project", "parix").put("from", "claude").put("body", "Autre sujet").put("ts", 200L)),
        )
        val parsed = LedgerSearchController.parseSearchResponse(payload)
        assertEquals(2, parsed.size)
        val matches = LedgerSearchController.filterLocal(parsed, "sécurité", "bridget")
        assertEquals(1, matches.size)
        assertEquals("1", matches.first().id)
    }

    @Test
    fun agentSummaryMarksStoppedAgentRelaunchable() {
        val payload = JSONObject().put(
            "items",
            JSONArray()
                .put(JSONObject().put("agent_id", "u1").put("display_name", "Worker").put("state", "stopped").put("host", "mac").put("project", "bridget"))
                .put(JSONObject().put("agent_id", "u2").put("display_name", "Live").put("state", "connected").put("host", "mac").put("project", "bridget")),
        )
        val agents = OptInAgentActionController.parseAgentDirectory(payload)
        assertTrue(agents[0].canRelaunch)
        assertFalse(agents[1].canRelaunch)
    }
}
