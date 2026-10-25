package app.cartae.bridget
import org.junit.Assert.*
import org.junit.Test
class UserInputTest {
    private val question=InputQuestion("q","Choix","Lequel ?",listOf(InputOption("A","Premier"),InputOption("B","Second")))
    @Test fun singleMultipleFreeAndComplementStayDistinct() {
        assertFalse(inputAnswerValid(question,InputAnswer()))
        assertTrue(inputAnswerValid(question,InputAnswer(text="Ma réponse")))
        val withNote=inputSelect(question,InputAnswer(text="avec précision"),"A")
        assertEquals("avec précision",withNote.text)
        assertEquals(listOf("B"),inputSelect(question,withNote,"B").selected)
        assertFalse(inputAnswerValid(question,InputAnswer(listOf("A","B"))))
        val multi=question.copy(multiple=true)
        val both=inputSelect(multi,inputSelect(multi,withNote,"B"),"A")
        assertEquals(listOf("B"),both.selected)
        assertTrue(inputAnswerValid(multi,InputAnswer(listOf("A","B"),"Note")))
    }
    @Test fun boundsOfferedValuesAndRequiredAreChecked() {
        assertFalse(inputAnswerValid(question,InputAnswer(listOf("foreign"))))
        assertFalse(inputAnswerValid(question.copy(multiple=true),InputAnswer(listOf("A","A"))))
        assertFalse(inputAnswerValid(question.copy(custom=false),InputAnswer(listOf("A"),"Note")))
        assertFalse(inputAnswerValid(question,InputAnswer(text="é".repeat(5001))))
        assertTrue(inputAnswerValid(question.copy(required=false),InputAnswer()))
    }
    @Test fun draftNeverCrossesBindingRequestOrRevision() {
        val original=inputDraftScope("phoneA","request1","rev1")
        assertEquals(original,inputDraftScope("phoneA","request1","rev1"))
        assertNotEquals(original,inputDraftScope("phoneB","request1","rev1"))
        assertNotEquals(original,inputDraftScope("phoneA","request2","rev1"))
        assertNotEquals(original,inputDraftScope("phoneA","request1","rev2"))
        assertEquals(64,original.length)
    }
}
