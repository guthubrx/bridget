package app.cartae.bridget

/** Identifiants seulement : aucune parenté ni activité ne se déduit d'un nom. */
internal fun rememberConversation(recent: List<String>, id: String): List<String> =
    (listOf(id) + recent).filter { it.isNotBlank() }.distinct().take(50)

internal data class DirectoryContact(val id: String, val name: String, val project: String, val machine: String)

/** O(n log n), sans lecture réseau par ligne. Les récents sont propres à l'appareil. */
internal fun orderedContacts(
    contacts: List<DirectoryContact>, recent: List<String>, favorites: Set<String>,
    conversationsOnly: Boolean, query: String,
): List<String> {
    val ranks = recent.withIndex().associate { it.value to it.index }
    val term = query.trim()
    return contacts.distinctBy { it.id }.filter {
        (!conversationsOnly || it.id in ranks || it.id in favorites) &&
            (term.isEmpty() || listOf(it.name, it.project, it.machine).any { text -> text.contains(term, ignoreCase = true) })
    }.sortedWith(compareByDescending<DirectoryContact> { it.id in favorites }
        .thenBy { ranks[it.id] ?: Int.MAX_VALUE }.thenBy { it.name.lowercase() }.thenBy { it.id })
        .map { it.id }
}

internal data class DelegateContact(val taskId: String, val parentId: String, val childId: String, val title: String)
internal fun directDelegates(owner: String, tasks: List<DelegateContact>): List<DelegateContact> =
    tasks.filter { it.parentId == owner }.distinctBy { it.taskId }
