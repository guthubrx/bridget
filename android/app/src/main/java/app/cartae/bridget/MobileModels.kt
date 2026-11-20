package app.cartae.bridget

import org.json.JSONArray
import org.json.JSONObject

/**
 * Modèles Kotlin typés (spec 306 / porte D15) remplaçant les accès `org.json.JSONObject` bruts
 * dans `MobileViewModel` et les écrans Compose.
 */
enum class NotificationUrgency(val wire: String, val rank: Int) {
    ALL("all", 0),
    NORMAL("normal", 1),
    HIGH("high", 2);

    companion object {
        fun parse(raw: String?): NotificationUrgency =
            entries.firstOrNull { it.wire.equals(raw?.trim(), ignoreCase = true) } ?: ALL
    }
}

data class NotificationFilterPreferences(
    val projects: Set<String> = emptySet(),
    val minUrgency: NotificationUrgency = NotificationUrgency.ALL,
) {
    fun matches(project: String?, urgency: NotificationUrgency): Boolean {
        if (urgency.rank < minUrgency.rank) return false
        if (projects.isEmpty()) return true
        return project != null && projects.any { it.equals(project, ignoreCase = true) }
    }

    fun toJson(): JSONObject = JSONObject().apply {
        put("projects", JSONArray(projects.sorted()))
        put("min_urgency", minUrgency.wire)
    }

    companion object {
        fun fromJson(json: JSONObject?): NotificationFilterPreferences {
            if (json == null) return NotificationFilterPreferences()
            val array = json.optJSONArray("projects")
            val set = buildSet {
                if (array != null) {
                    for (i in 0 until array.length()) {
                        val item = array.optString(i).trim()
                        if (item.isNotEmpty()) add(item)
                    }
                }
            }
            return NotificationFilterPreferences(
                projects = set,
                minUrgency = NotificationUrgency.parse(json.optString("min_urgency")),
            )
        }
    }
}

data class MobileAgentSummary(
    val id: String,
    val name: String,
    val state: String,
    val host: String,
    val project: String,
    val provider: String,
    val canRelaunch: Boolean,
) {
    companion object {
        fun fromJson(json: JSONObject): MobileAgentSummary {
            val state = json.optString("state", "unknown")
            return MobileAgentSummary(
                id = json.optString("agent_id", json.optString("id")),
                name = json.optString("display_name", json.optString("name")),
                state = state,
                host = json.optString("host"),
                project = json.optString("project"),
                provider = json.optString("provider", json.optString("agent_type")),
                canRelaunch = state == "stopped" || state == "unreachable",
            )
        }
    }
}

data class MobileLedgerSearchEntry(
    val id: String,
    val agentId: String,
    val project: String,
    val sender: String,
    val body: String,
    val timestampSecs: Long,
) {
    companion object {
        fun fromJson(json: JSONObject): MobileLedgerSearchEntry = MobileLedgerSearchEntry(
            id = json.optString("id"),
            agentId = json.optString("agent_id"),
            project = json.optString("project"),
            sender = json.optString("from"),
            body = json.optString("body"),
            timestampSecs = json.optLong("ts", 0L),
        )
    }
}

data class OptInSpecialistLaunchRequest(
    val specialistId: String,
    val project: String,
    val brief: String,
    val commandId: String,
) {
    fun toJson(): JSONObject = JSONObject()
        .put("specialist_id", specialistId)
        .put("project", project)
        .put("brief", brief)
        .put("command_id", commandId)
}
