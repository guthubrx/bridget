package app.cartae.bridget

import org.junit.Assert.assertEquals
import org.junit.Test

class ConversationTextTest {
    private val id = "828d4bca-c571-4108-9075-98a1fddd7dc0"
    @Test fun ownHumanMessageShowsItsExactStoredBody() {
        val raw = "💬 Message Bridget de human (id $id, reply=no) :\n\nCoucou\n\nConsigne de transport"
        assertEquals("Coucou", conversationBody(raw, true, mapOf(id to "Coucou")))
        assertEquals(raw, conversationBody(raw, true, emptyMap()))
        assertEquals(raw, conversationBody(raw, false, mapOf(id to "Coucou")))
    }
    @Test fun timestampsUseThePhoneTimezoneInsteadOfUtc() {
        assertEquals("19:49", conversationTime("2026-10-24T17:49:00Z", java.time.ZoneId.of("Europe/Paris")))
        assertEquals("", conversationTime("unknown"))
    }
    @Test fun ordinaryUserTextAndUnknownEnvelopesStayUnchanged() {
        val raw = "Ce texte cite id $id et doit rester entier"
        assertEquals(raw, conversationBody(raw, true, mapOf(id to "Autre texte")))
    }
}
