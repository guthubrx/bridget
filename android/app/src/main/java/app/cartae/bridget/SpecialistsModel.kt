package app.cartae.bridget

import java.util.UUID

// Constructeurs purs (Map/List Kotlin, sans org.json) des commandes spécialistes 222.
// Ils ne fabriquent ni droit, ni compte, ni identité d'acteur : le canal natif décide.
internal val SPECIALIST_READ_ACTIONS = setOf("profiles", "profile_versions", "list", "search", "get", "memory_recall", "memory_versions", "export", "request_status")
internal val SPECIALIST_PAGED_ACTIONS = setOf("profiles", "profile_versions", "list", "search", "memory_recall")
internal val SPECIALIST_MEMORY_KINDS = listOf("fact", "decision", "hypothesis", "reported_result")
// États ou codes qui ne sont PAS un résultat final : la commande reste conservée.
internal val SPECIALIST_OPEN_STATES = setOf("source_reading", "history_capture_busy", "starting", "pending", "in_flight", "queued")
internal val SPECIALIST_UNKNOWN_CODES = setOf("native_submission_unknown", "specialist_execution_unknown")
private val DEFINITION_KEYS = setOf("profile_id", "name", "kind", "instructions", "roles", "procedures", "requirements", "preferences", "members", "provenance")
internal const val SPECIALIST_IMPORT_LIMIT = 32 * 1024
internal const val SPECIALIST_DISPLAY_LIMIT = 2000

fun specialistIsRead(action: String) = action in SPECIALIST_READ_ACTIONS

fun specialistOutcomeOpen(code: String?, status: String?, delivery: String?): Boolean =
    code in SPECIALIST_UNKNOWN_CODES || delivery == "unconfirmed" ||
        status !in setOf("ok", "refused", "known", "submitted", "result_available")


fun specialistClip(text: String, max: Int = SPECIALIST_DISPLAY_LIMIT): String =
    if (text.length <= max) text else text.substring(0, max) + "…"

fun specialistWords(text: String) = text.split(',').map(String::trim).filter(String::isNotEmpty)

fun specialistScope(projects: String): Map<String, Any?> =
    mapOf("projects" to specialistWords(projects).filter { it != "*" }, "transversal" to (projects.trim() == "*"))

fun specialistRequestKey(input: String, fresh: () -> String = { UUID.randomUUID().toString() }): String? {
    val key = input.trim()
    if (key.isEmpty()) return fresh()
    return key.takeIf { it.length <= 256 && key.none(Char::isISOControl) }
}

fun specialistUuid(text: String): String? = runCatching { UUID.fromString(text.trim()).toString() }.getOrNull()
    ?.takeIf { it.equals(text.trim(), ignoreCase = true) }

private fun base(action: String, requestId: String?): MutableMap<String, Any?> =
    mutableMapOf<String, Any?>("version" to 1, "action" to action).also { if (requestId != null) it["request_id"] = requestId }

fun specialistRequestRole(requestId: String, roleId: String, projectId: String, task: String, specialistAgentId: String?): Map<String, Any?>? {
    if (roleId.isBlank() || projectId.isBlank() || task.isBlank()) return null
    val agent = specialistAgentId?.takeIf { it.isNotBlank() }?.let { specialistUuid(it) ?: return null }
    return base("request_role", requestId).also {
        if (agent != null) it["specialist_agent_id"] = agent
        it["role_id"] = roleId.trim(); it["project_id"] = projectId.trim(); it["task"] = task.trim()
    }
}

fun specialistRequestStatus(requestId: String): Map<String, Any?> = base("request_status", requestId)

fun specialistMemoryWrite(requestId: String, agentId: String, projectId: String, shared: Boolean, kind: String, text: String): Map<String, Any?>? {
    if (kind !in SPECIALIST_MEMORY_KINDS || text.isBlank() || projectId.isBlank() || agentId.isBlank()) return null
    return base("memory_write", requestId).also {
        it["specialist_agent_id"] = agentId; it["project_id"] = projectId.trim(); it["shared"] = shared
        it["kind"] = kind; it["text"] = text.trim()
        it["source"] = mapOf("kind" to "statement", "request_id" to requestId) // déclaration humaine
    }
}

fun specialistMemoryCorrect(requestId: String, memoryId: String, expectedRevision: Long, kind: String, text: String): Map<String, Any?>? {
    if (kind !in SPECIALIST_MEMORY_KINDS || text.isBlank() || memoryId.isBlank()) return null
    return base("memory_correct", requestId).also {
        it["memory_id"] = memoryId; it["expected_revision"] = expectedRevision; it["text"] = text.trim(); it["kind"] = kind
    }
}

fun specialistExport(profileId: String, revision: Long): Map<String, Any?> =
    base("export", null).also { it["profile_id"] = profileId; it["revision"] = revision }

/** Définition seule : clés fermées, taille bornée, aucun champ d'autorité ou de compte. */
fun specialistImportDefinitionValid(definition: Map<String, Any?>, textLength: Int): Boolean =
    textLength in 1..SPECIALIST_IMPORT_LIMIT && definition.keys.all { it in DEFINITION_KEYS } &&
        definition["profile_id"] is String && definition["name"] is String && definition["instructions"] is String &&
        (definition["kind"] as? String) in setOf("profile", "team")

fun specialistImport(requestId: String, definition: Map<String, Any?>, textLength: Int): Map<String, Any?>? =
    if (!specialistImportDefinitionValid(definition, textLength)) null
    else base("import", requestId).also { it["definition"] = definition }

fun specialistConvert(requestId: String, agentId: String, profileId: String, profileRevision: Long, expectedRevision: Long): Map<String, Any?>? {
    val agent = specialistUuid(agentId) ?: return null
    if (profileId.isBlank() || profileRevision < 1) return null
    return base("convert", requestId).also {
        it["specialist_agent_id"] = agent; it["profile_id"] = profileId; it["profile_revision"] = profileRevision
        it["expected_revision"] = expectedRevision
    }
}

/** Une ligne par membre : `clé | rôle | livrable | dépendances,séparées`. Null si incohérent. */
fun specialistTeamMembers(lines: String): List<Map<String, Any?>>? {
    val rows = lines.lines().map(String::trim).filter(String::isNotEmpty).map { line ->
        val parts = line.split('|').map(String::trim)
        if (parts.size !in 3..4 || parts.take(3).any(String::isEmpty)) return null
        mapOf("key" to parts[0], "role_id" to parts[1], "deliverable" to parts[2], "depends_on" to specialistWords(parts.getOrElse(3) { "" }))
    }
    if (rows.isEmpty() || rows.size > 32) return null
    val keys = rows.map { it["key"] as String }
    if (keys.toSet().size != keys.size) return null
    @Suppress("UNCHECKED_CAST")
    val deps = rows.associate { (it["key"] as String) to (it["depends_on"] as List<String>) }
    if (deps.any { (k, d) -> d.any { it == k || it !in deps } }) return null
    // refuse les cycles
    val done = mutableSetOf<String>()
    var left = deps.keys.toMutableSet()
    while (left.isNotEmpty()) {
        val ready = left.filter { k -> deps.getValue(k).all { it in done } }
        if (ready.isEmpty()) return null
        done += ready; left = (left - ready.toSet()).toMutableSet()
    }
    return rows
}

fun specialistTeamProposal(requestId: String, profileId: String, name: String, instructions: String, members: List<Map<String, Any?>>): Map<String, Any?>? {
    if (name.isBlank() || members.isEmpty()) return null
    return base("propose_profile", requestId).also {
        it["definition"] = mapOf("profile_id" to profileId, "name" to name.trim(), "kind" to "team", "instructions" to instructions,
            "roles" to emptyList<String>(), "members" to members)
    }
}

fun specialistRequestTeam(requestId: String, profileId: String, revision: Long, projectId: String, task: String): Map<String, Any?>? {
    if (profileId.isBlank() || revision < 1 || projectId.isBlank() || task.isBlank()) return null
    return base("request_team", requestId).also {
        it["profile_id"] = profileId; it["revision"] = revision; it["project_id"] = projectId.trim(); it["task"] = task.trim()
    }
}

/** Même commande avec une page suivante ; un curseur vide ne change rien. */
fun specialistWithCursor(command: Map<String, Any?>, cursor: String?, limit: Int = 25): Map<String, Any?> =
    if ((command["action"] as? String) in SPECIALIST_PAGED_ACTIONS && !cursor.isNullOrBlank())
        command + mapOf("cursor" to cursor, "limit" to limit) else command

/** Libellé de portée affichable à partir des codes reçus (jamais « tout disponible »). */
fun specialistScopeLabel(projects: List<String>, transversal: Boolean): String =
    if (transversal) "*" else projects.joinToString(", ")
