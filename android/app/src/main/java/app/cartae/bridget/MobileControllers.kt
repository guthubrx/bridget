package app.cartae.bridget

import org.json.JSONArray
import org.json.JSONObject

/**
 * Contrôleurs de domaine extraits de `MobileViewModel.kt` (spec 306 / porte D15) :
 * recherche dans l'historique, filtrage de notifications et construction des actions opt-in.
 */
object LedgerSearchController {
    fun filterLocal(
        entries: List<MobileLedgerSearchEntry>,
        query: String,
        projectFilter: String? = null,
        limit: Int = 50,
    ): List<MobileLedgerSearchEntry> {
        val needle = query.trim().lowercase()
        val project = projectFilter?.trim()?.takeIf { it.isNotEmpty() }
        return entries.asSequence()
            .filter { entry ->
                (project == null || entry.project.equals(project, ignoreCase = true)) &&
                    (needle.isEmpty() ||
                        entry.body.lowercase().contains(needle) ||
                        entry.sender.lowercase().contains(needle) ||
                        entry.project.lowercase().contains(needle))
            }
            .take(limit.coerceIn(1, 200))
            .toList()
    }

    fun parseSearchResponse(response: JSONObject): List<MobileLedgerSearchEntry> {
        val items = response.optJSONArray("items") ?: JSONArray()
        return buildList {
            for (i in 0 until items.length()) {
                val obj = items.optJSONObject(i) ?: continue
                add(MobileLedgerSearchEntry.fromJson(obj))
            }
        }
    }
}

object NotificationFilterController {
    fun shouldNotify(
        preferences: NotificationFilterPreferences,
        project: String?,
        urgencyWire: String?,
    ): Boolean = preferences.matches(project, NotificationUrgency.parse(urgencyWire))

    fun toggleProject(
        current: NotificationFilterPreferences,
        project: String,
    ): NotificationFilterPreferences {
        val clean = project.trim()
        if (clean.isEmpty()) return current
        val next = if (clean in current.projects) current.projects - clean else current.projects + clean
        return current.copy(projects = next)
    }
}

object OptInAgentActionController {
    fun buildRelaunchBody(commandId: String, reason: String? = null): JSONObject =
        JSONObject().apply {
            put("command_id", commandId)
            if (!reason.isNullOrBlank()) put("reason", reason.trim())
        }

    fun parseAgentDirectory(response: JSONObject): List<MobileAgentSummary> {
        val items = response.optJSONArray("items") ?: JSONArray()
        return buildList {
            for (i in 0 until items.length()) {
                val obj = items.optJSONObject(i) ?: continue
                add(MobileAgentSummary.fromJson(obj))
            }
        }
    }
}
