package app.cartae.bridget

import org.junit.Assert.*
import org.junit.Test

class ConversationTextTest {
    @Test fun activityUpdatesAndOlderPagesKeepTheLatestToolState() {
        assertEquals(mapOf("session:tool" to 12L,"other:tool" to 7L),
            latestActivitySequences(listOf("session:tool" to 10L,"session:tool" to 12L,"other:tool" to 7L,"session:tool" to 9L)))
    }

    @Test fun terminalReceiptIsNeverPresentedAsPending() {
        assertTrue(messageDeliveryPending("outcome_unknown","outcome_unknown","delivery-id"))
        assertTrue(messageDeliveryPending("accepted","accepted",null))
        assertFalse(messageDeliveryPending("orphaned","orphaned","delivery-id"))
        assertFalse(messageDeliveryPending("rejected","outcome_unknown","stale-id"))
        assertFalse(messageDeliveryPending("local","outcome_unknown","stale-id"))
        assertFalse(messageDeliveryPending("outcome_unknown","outcome_unknown",null))
    }

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
    @org.junit.Test fun wrappedCommandIdIsIndependentOfTheNativeJournalMessageId() {
        org.junit.Assert.assertEquals("12345678-1234-4123-8123-123456789012",humanInstructionId("💬 Instruction de l’utilisateur (id=12345678-1234-4123-8123-123456789012)\nText"))
        org.junit.Assert.assertNull(humanInstructionId("A normal answer with id=12345678-1234-4123-8123-123456789012"))
    }
    @org.junit.Test fun queueKeepsUncertainMessagesAndHidesOnlyObservedOrCancelledOnes() {
        org.junit.Assert.assertTrue(queuedMessageVisible("local", null, ""))
        org.junit.Assert.assertTrue(queuedMessageVisible("outcome_unknown", null, "held"))
        org.junit.Assert.assertTrue(queuedMessageVisible("accepted", null, "promoted"))
        org.junit.Assert.assertFalse(queuedMessageVisible("outcome_unknown", "turn_start", "held"))
        org.junit.Assert.assertFalse(queuedMessageVisible("accepted", "turn_end", "released"))
        org.junit.Assert.assertFalse(queuedMessageVisible("outcome_unknown", null, "cancelled"))
        org.junit.Assert.assertFalse(queuedMessageVisible("rejected", null, ""))
        org.junit.Assert.assertTrue(queuedMessageVisible("outcome_unknown", "turn_start", "cancelled",true))
    }

    @Test fun searchUsesLoadedTextAndKeepsEveryResultIndex() {
        assertEquals(listOf(0,2),messageSearchMatches("été",listOf("ÉTÉ ici","hiver","fin été")))
        assertTrue(messageSearchMatches(" ",listOf("texte")).isEmpty())
        assertTrue(messageSearchMatches("absent",listOf("texte")).isEmpty())
    }
    @Test fun attentionPrioritizesRealQuestionsAndNeverCallsPresenceCompleted() {
        assertEquals("question",agentAttention(true,true,true,true))
        assertEquals("unread",agentAttention(false,true,true,true))
        assertEquals("working",agentAttention(false,false,true,true))
        assertEquals("idle",agentAttention(false,false,false,true))
        assertEquals("stale",agentAttention(true,true,true,false))
        assertFalse(observedActivityEnded(null,"connected",0,0,true))
        assertFalse(observedActivityEnded("busy","connected",0,1,true))
        assertFalse(observedActivityEnded("busy","disconnected",0,0,true))
        assertFalse(observedActivityEnded("busy","connected",0,0,false))
        assertTrue(observedActivityEnded("busy","connected",0,0,true))
    }
    @Test fun compactionClosesOnlyForItsOwnItemOrTurnAndNeverWhenOffline() {
        val start=CompactionSignal("s","t","i","running")
        assertEquals(start,activeContextCompaction(listOf(start),true))
        assertNull(activeContextCompaction(listOf(start),false))
        assertEquals(start,activeContextCompaction(listOf(start,start.copy(turn="other",phase="completed")),true))
        assertEquals(start,activeContextCompaction(listOf(start,start.copy(item="other",phase="completed")),true))
        assertNull(activeContextCompaction(listOf(start,start.copy(phase="completed")),true))
        assertNull(activeContextCompaction(listOf(start,start.copy(phase="turn_end")),true))
        assertNull(activeContextCompaction(listOf(start,start.copy(turn="new",phase="turn_start")),true))
        assertNull(activeContextCompaction(listOf(start,start.copy(phase="completed"),start),true))
        assertNull(activeContextCompaction(listOf(start.copy(turn="new",phase="turn_start"),start),true))
        assertNull(activeContextCompaction(listOf(start,start.copy(session="new",turn="new",phase="turn_start"),start),true))
    }
    @Test fun dictationIsEditableBoundToItsOriginAndNeverAnAutomaticSend() {
        assertEquals("Brouillon dicté",dictationDraft("association:agent","association:agent","Brouillon","dicté"))
        assertNull(dictationDraft("association:other","association:agent","Brouillon","dicté"))
        assertNull(dictationDraft("new:agent","old:agent","Brouillon","dicté"))
        assertNull(dictationDraft("same","same","Brouillon",null))
        assertNull(dictationDraft("same","same","x".repeat(9999),"é"))
        assertNull(activeContextCompaction(listOf(CompactionSignal("s","t","","turn_end"),CompactionSignal("s","t","i","running")),true))
    }
    @Test fun nativeAttachmentReferencesShowTheirNameWithoutChangingUnknownText() {
        val id="00000000-0000-0000-0000-000000000000"
        val ref=attachmentReference("recette.png","artifact:$id","artifact-version:$id","a".repeat(64))
        assertEquals("Voici.\n📎 recette.png",conversationBody("Voici.\n$ref",true,emptyMap()))
        assertEquals(ref,conversationBody(ref,false,emptyMap()))
        assertEquals("[Fichier Bridget : texte libre]",conversationBody("[Fichier Bridget : texte libre]",true,emptyMap()))
    }
}
