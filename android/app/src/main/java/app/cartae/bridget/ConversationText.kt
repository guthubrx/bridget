package app.cartae.bridget

private val nativeAttachmentLine=Regex("""^\[Fichier Bridget : (.+); artifact_ref=artifact:[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}; version_ref=artifact-version:[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}; sha256=[0-9a-f]{64}]$""")

/** L’original reste accessible dans les événements techniques ; aucun format inconnu n’est retiré. */
internal fun conversationBody(raw: String, isUser: Boolean, localMessages: Map<String, String>): String {
    if(!isUser) return raw
    val body=if(raw.startsWith("💬")) humanInstructionId(raw)?.let {localMessages[it]} ?: raw else raw
    return body.lineSequence().joinToString("\n") {line -> nativeAttachmentLine.matchEntire(line)?.let {"📎 "+it.groupValues[1]} ?: line}
}

internal fun humanInstructionId(raw: String): String? {
    if (!raw.startsWith("💬")) return null
    return Regex("\\bid[ =:]*(" + "[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}" + ")")
        .find(raw.take(256))?.groupValues?.get(1)
}

internal fun conversationTime(value: String, zone: java.time.ZoneId = java.time.ZoneId.systemDefault()): String =
    runCatching { java.time.Instant.parse(value).atZone(zone).format(java.time.format.DateTimeFormatter.ofPattern("HH:mm")) }.getOrDefault("")

internal fun messageDeliveryPending(state: String, receiptKind: String?, deliveryId: String?): Boolean =
    state=="accepted" || (state=="outcome_unknown" && receiptKind=="outcome_unknown"
        && !deliveryId.isNullOrBlank() && deliveryId!="null")

/** O(n): page overlaps and repeated observations retain only the latest state of an identified tool. */
internal fun latestActivitySequences(entries: List<Pair<String, Long>>): Map<String, Long> {
    val latest=mutableMapOf<String,Long>()
    for((id,seq) in entries) latest[id]=maxOf(latest[id] ?: 0L,seq)
    return latest
}

internal fun messageSearchMatches(query: String, texts: List<String>): List<Int> =
    if(query.isBlank()) emptyList() else texts.mapIndexedNotNull { index,text ->
        index.takeIf { text.contains(query.trim(),ignoreCase=true) }
    }

internal fun agentAttention(question: Boolean, unread: Boolean, working: Boolean, fresh: Boolean): String = when {
    !fresh -> "stale"
    question -> "question"
    unread -> "unread"
    working -> "working"
    else -> "idle"
}

/** Une transition observée invite à consulter ; elle ne prouve pas une réussite. */
internal fun observedActivityEnded(previous: String?, current: String, previousGeneration: Int?, generation: Int?, freshPair: Boolean): Boolean =
    freshPair && previousGeneration!=null && previousGeneration==generation &&
        previous in setOf("busy","working","running") && current=="connected"

internal data class CompactionSignal(val session: String, val turn: String, val item: String, val phase: String)

/** Les fins sont mémorisées : doublons et ordre de projection ne rouvrent pas un tour terminé. */
internal fun activeContextCompaction(signals: List<CompactionSignal>, live: Boolean): CompactionSignal? {
    if(!live) return null
    val closedTurns=mutableSetOf<Pair<String,String>>()
    val closedItems=mutableSetOf<Triple<String,String,String>>()
    val currentTurns=mutableMapOf<String,String>()
    var currentSession: String?=null
    var active: CompactionSignal?=null
    for(signal in signals) {
        val turn=signal.session to signal.turn
        val item=Triple(signal.session,signal.turn,signal.item)
        when(signal.phase) {
            "turn_start" -> {
                currentSession=signal.session;currentTurns[signal.session]=signal.turn
                active?.let {current -> if(current.session!=signal.session || current.turn!=signal.turn) {
                    closedTurns.add(current.session to current.turn);active=null
                }}
            }
            "turn_end" -> { closedTurns.add(turn); if(active?.let {it.session==signal.session && it.turn==signal.turn}==true) active=null }
            "completed","failed","cancelled" -> { closedItems.add(item); if(active?.let { Triple(it.session,it.turn,it.item)==item }==true) active=null }
            "running" -> if(turn !in closedTurns && item !in closedItems &&
                (currentSession==null || currentSession==signal.session) &&
                (currentTurns[signal.session]==null || currentTurns[signal.session]==signal.turn)) active=signal
        }
    }
    return active
}

internal fun dictationDraft(origin: String?, requestedOrigin: String?, draft: String, recognized: String?): String? {
    if(origin==null || origin!=requestedOrigin || recognized.isNullOrBlank()) return null
    val combined=listOf(draft,recognized).filter {it.isNotBlank()}.joinToString(" ")
    return combined.takeIf {it.toByteArray(Charsets.UTF_8).size<=10_000}
}

internal fun attachmentReference(name: String, artifact: String, version: String, digest: String): String =
    "[Fichier Bridget : $name; artifact_ref=$artifact; version_ref=$version; sha256=$digest]"
