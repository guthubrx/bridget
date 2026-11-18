package app.cartae.bridget

import org.junit.Assert.*
import org.junit.Test

class AgentDirectoryModelTest {
    private val contacts = listOf(
        DirectoryContact("worker", "Agent 123", "Parix", "sys"),
        DirectoryContact("motor", "Moteur", "Parix", "mac"),
        DirectoryContact("chief", "Chef", "Parix", "mac"),
        DirectoryContact("review", "Relecteur", "SNCF", "ks"),
    )
    @Test fun favoritesThenRecentConversationsWithoutUnsolicitedWorkers() {
        assertEquals(listOf("chief", "review", "motor"),
            orderedContacts(contacts, listOf("review", "motor"), setOf("chief"), true, ""))
        assertEquals(4, orderedContacts(contacts, emptyList(), emptySet(), false, "").size)
        assertTrue(orderedContacts(contacts, emptyList(), emptySet(), true, "").isEmpty())
    }
    @Test fun searchPreservesScopeAndFindsProjectMachineAndName() {
        assertEquals(listOf("review"), orderedContacts(contacts, emptyList(), emptySet(), false, " sncf "))
        assertEquals(listOf("worker"), orderedContacts(contacts, emptyList(), emptySet(), false, "SYS"))
        assertTrue(orderedContacts(contacts, listOf("motor"), emptySet(), true, "Agent").isEmpty())
        assertEquals(listOf("motor"), orderedContacts(contacts, listOf("motor"), emptySet(), true, "MOTEUR"))
    }
    @Test fun reopeningMovesToFrontWithoutDuplicatesAndBoundsStorage() {
        assertEquals(listOf("a", "b"), rememberConversation(listOf("b", "a"), "a"))
        val recent = rememberConversation((0..60).map { "$it" }, "new")
        assertEquals(50, recent.size)
        assertEquals("new", recent.first())
    }
    @Test fun lineageIsExplicitAndDoesNotConflateGrandchildrenOrDuplicateTasks() {
        val a = DelegateContact("a", "chief", "motor", "Code")
        val b = DelegateContact("b", "motor", "worker", "Review")
        val unknown = DelegateContact("c", "", "mystery", "Unknown")
        assertEquals(listOf(a), directDelegates("chief", listOf(a, b, unknown, a)))
        assertTrue(directDelegates("missing", listOf(a, b)).isEmpty())
    }
}
