package app.cartae.bridget

/** Le texte libre reste distinct des choix jusqu’au callback du fournisseur. */
data class InputOption(val label: String, val description: String = "", val value: String = label)
data class InputQuestion(val id: String, val header: String, val question: String, val options: List<InputOption>,
    val multiple: Boolean = false, val custom: Boolean = true, val initial: String = "", val required: Boolean = true)
data class InputAnswer(val selected: List<String> = emptyList(), val text: String = "")
fun inputAnswerValid(question: InputQuestion, answer: InputAnswer): Boolean {
    if (answer.text.toByteArray(Charsets.UTF_8).size > 10_000 || (!question.custom && answer.text.isNotEmpty())) return false
    if (question.required && answer.selected.isEmpty() && answer.text.isBlank()) return false
    if (!question.multiple && answer.selected.size > 1) return false
    return answer.selected.distinct().size == answer.selected.size && answer.selected.all { choice -> question.options.any { it.value == choice } }
}
fun inputSelect(question: InputQuestion, answer: InputAnswer, value: String): InputAnswer =
    answer.copy(selected = if (!question.multiple) listOf(value) else if (value in answer.selected) answer.selected - value else answer.selected + value)
fun inputDraftScope(binding: String, request: String, revision: String): String = LocalDigest.sha256("$binding\u0000$request\u0000$revision")
private object LocalDigest {
    fun sha256(value: String): String = java.security.MessageDigest.getInstance("SHA-256").digest(value.toByteArray(Charsets.UTF_8)).joinToString("") { "%02x".format(it.toInt() and 255) }
}
