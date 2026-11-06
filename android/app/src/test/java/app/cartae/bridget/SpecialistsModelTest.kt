package app.cartae.bridget
import org.junit.Assert.*
import org.junit.Test
class SpecialistsModelTest {
    private val agent="123e4567-e89b-42d3-a456-426614174000"
    @Test fun requestRoleKeepsOptionalSpecialistAndRefusesIncompleteInput() {
        val plain=specialistRequestRole("k1","reviewer","p","review",null)!!
        assertFalse(plain.containsKey("specialist_agent_id"))
        assertEquals(1,plain["version"]); assertEquals("request_role",plain["action"]); assertEquals("k1",plain["request_id"])
        assertEquals(agent,specialistRequestRole("k1","reviewer","p","review",agent)!!["specialist_agent_id"])
        assertNull(specialistRequestRole("k1","reviewer","p","review","not-a-uuid"))
        assertNull(specialistRequestRole("k1","","p","review",null)); assertNull(specialistRequestRole("k1","r","p"," ",null))
    }
    @Test fun stableKeyIsKeptOrGeneratedAndBounded() {
        assertEquals("abc",specialistRequestKey(" abc "))
        assertEquals("gen",specialistRequestKey("") { "gen" })
        assertNull(specialistRequestKey("x".repeat(257))); assertNull(specialistRequestKey("a\u0007b"))
    }
    @Test fun memoryDeclarationUsesStatementSourceOfItsOwnRequest() {
        val c=specialistMemoryWrite("m1",agent,"p",true,"decision","  choix  ")!!
        assertEquals(mapOf("kind" to "statement","request_id" to "m1"),c["source"]); assertEquals("choix",c["text"]); assertEquals(true,c["shared"])
        assertNull(specialistMemoryWrite("m1",agent,"p",false,"opinion","t")); assertNull(specialistMemoryWrite("m1",agent,"",false,"fact","t"))
        val fix=specialistMemoryCorrect("m2","mem",3,"fact","nouveau")!!
        assertEquals(3L,fix["expected_revision"]); assertEquals("memory_correct",fix["action"])
    }
    @Test fun readsCarryNoRequestIdAndCursorOnlyOnPagedReads() {
        assertFalse(specialistExport("p",2).containsKey("request_id"))
        assertTrue(specialistIsRead("request_status")); assertFalse(specialistIsRead("import"))
        val list=mapOf<String,Any?>("version" to 1,"action" to "list")
        assertEquals("c",specialistWithCursor(list,"c")["cursor"]); assertEquals(list,specialistWithCursor(list,null))
        val mem=specialistMemoryWrite("m",agent,"p",false,"fact","t")!!
        assertEquals(mem,specialistWithCursor(mem,"c"))
    }
    @Test fun importAcceptsOnlyClosedDefinitionOfBoundedSize() {
        val ok=mapOf<String,Any?>("profile_id" to "p","name" to "n","kind" to "profile","instructions" to "i","roles" to listOf("r"))
        assertNotNull(specialistImport("i1",ok,100))
        assertNull(specialistImport("i1",ok+("is_human" to true),100)); assertNull(specialistImport("i1",ok+("credential" to "x"),100))
        assertNull(specialistImport("i1",ok,SPECIALIST_IMPORT_LIMIT+1)); assertNull(specialistImport("i1",ok+("kind" to "admin"),100))
        assertNull(specialistImport("i1",ok-"name",100))
    }
    @Test fun convertNeedsExactUuidAndVersion() {
        assertNotNull(specialistConvert("c",agent,"p",1,0)); assertNull(specialistConvert("c","abc","p",1,0)); assertNull(specialistConvert("c",agent,"p",0,0))
    }
    @Test fun teamMembersValidateKeysDependenciesAndCycles() {
        val ok=specialistTeamMembers("a | r1 | plan\nb | r2 | code | a\nc | r3 | test | a, b")!!
        assertEquals(listOf("a","b"),ok[2]["depends_on"]); assertEquals(emptyList<String>(),ok[0]["depends_on"])
        assertNull(specialistTeamMembers("a | r | d\na | r | d")); assertNull(specialistTeamMembers("a | r | d | z"))
        assertNull(specialistTeamMembers("a | r | d | a")); assertNull(specialistTeamMembers("a | r | d | b\nb | r | d | a"))
        assertNull(specialistTeamMembers("a | r")); assertNull(specialistTeamMembers(""))
        val p=specialistTeamProposal("t","pid","Équipe","i",ok)!!
        assertEquals("team",(p["definition"] as Map<*,*>)["kind"]); assertNull(specialistTeamProposal("t","pid"," ","i",ok))
        assertNotNull(specialistRequestTeam("r","pid",2,"p","tâche")); assertNull(specialistRequestTeam("r","pid",0,"p","tâche"))
    }
    @Test fun openStatesAreNeverFinalResults() {
        listOf("source_reading","history_capture_busy","starting").forEach {
            assertTrue(specialistOutcomeOpen(it,null,null)); assertTrue(specialistOutcomeOpen(null,it,null))
        }
        assertTrue(specialistOutcomeOpen("native_submission_unknown",null,null)); assertTrue(specialistOutcomeOpen("specialist_execution_unknown",null,null))
        assertTrue(specialistOutcomeOpen(null,null,"unconfirmed"))
        assertFalse(specialistOutcomeOpen("role_not_found","refused",null)); assertFalse(specialistOutcomeOpen(null,"result_available",null))
    }
    @Test fun scopeAndClipDoNotHideOrInflate() {
        assertEquals(mapOf("projects" to listOf("a","b"),"transversal" to false),specialistScope("a, b"))
        assertEquals(mapOf("projects" to emptyList<String>(),"transversal" to true),specialistScope("*"))
        assertEquals(SPECIALIST_DISPLAY_LIMIT+1,specialistClip("x".repeat(5000)).length); assertEquals("ok",specialistClip("ok"))
    }
}
