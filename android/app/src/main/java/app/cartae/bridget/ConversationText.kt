package app.cartae.bridget

/** L’original reste accessible dans les événements techniques ; aucun texte inconnu n’est retiré. */
internal fun conversationBody(raw: String, isUser: Boolean, localMessages: Map<String, String>): String {
    if (!isUser || !raw.startsWith("💬")) return raw
    val id = Regex("\\bid[ =:]*(" + "[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}" + ")")
        .find(raw.take(256))?.groupValues?.get(1) ?: return raw
    return localMessages[id] ?: raw
}

internal fun conversationTime(value: String, zone: java.time.ZoneId = java.time.ZoneId.systemDefault()): String =
    runCatching { java.time.Instant.parse(value).atZone(zone).format(java.time.format.DateTimeFormatter.ofPattern("HH:mm")) }.getOrDefault("")
