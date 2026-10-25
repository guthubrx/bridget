package app.cartae.bridget

import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.Intent
import android.speech.RecognizerIntent
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import org.json.JSONArray
import org.json.JSONObject
import java.util.Locale

internal fun inputQuestions(item:JSONObject):List<InputQuestion> {
    val array=item.getJSONArray("questions")
    require(array.length() in 1..16)
    return (0 until array.length()).map { i -> val q=array.getJSONObject(i); val options=q.getJSONArray("options")
        InputQuestion(q.getString("id"),q.getString("header"),q.getString("question"),(0 until options.length()).map {j->
            val o=options.getJSONObject(j);InputOption(o.getString("label"),o.optString("description"),o.optString("value",o.getString("label")))},
            q.optBoolean("multiSelect"),q.optBoolean("allowCustomAnswer",true),q.optString("initialAnswer"),q.optBoolean("required",true))
    }
}
internal fun inputAnswers(value:JSONObject,questions:List<InputQuestion>):Map<String,InputAnswer> = questions.associate { q ->
    val a=value.optJSONObject(q.id); val selected=a?.optJSONArray("selected")
    q.id to InputAnswer(selected?.let {(0 until it.length()).map(it::getString)}.orEmpty(),a?.optString("text") ?: q.initial)
}
internal fun inputAnswerJson(value:Map<String,InputAnswer>)=JSONObject().apply {value.forEach {(id,a)->put(id,JSONObject().put("selected",JSONArray(a.selected)).put("text",a.text))}}

internal fun inputAnswerText(entry:JSONObject):String {
    val answers=entry.getJSONObject("envelope").optJSONObject("answers") ?: return ""
    val questions=entry.optJSONArray("input_questions")
    return answers.keys().asSequence().map {id->
        val a=answers.getJSONObject(id);val selected=a.optJSONArray("selected")
        val question=questions?.let {(0 until it.length()).map(it::getJSONObject).firstOrNull {q->q.optString("id")==id}}
        val options=question?.optJSONArray("options")
        val choices=selected?.let {(0 until it.length()).map {i->val value=it.getString(i)
            options?.let {array->(0 until array.length()).map(array::getJSONObject).firstOrNull {o->o.optString("value",o.optString("label"))==value}?.optString("label")} ?: value
        }}.orEmpty()
        (choices+listOf(a.optString("text"))).filter {it.isNotBlank()}.joinToString(" · ")
    }.joinToString("\n")
}

@Composable
fun UserInputCard(vm:MobileViewModel,item:JSONObject,onFullVisibility:(Boolean)->Unit = {}) {
    val key=vm.userInputDraftKey(item)
    var expanded by rememberSaveable(key) {mutableStateOf(false)}
    var full by rememberSaveable(key) {mutableStateOf(vm.focusedQuestion==item.getString("request_id"))}
    LaunchedEffect(vm.focusedQuestion) {if(vm.focusedQuestion==item.getString("request_id")) full=true}
    val questions=remember(key,item.has("questions")) {runCatching {inputQuestions(item)}.getOrDefault(emptyList())}
    if(questions.isEmpty()) {Card(Modifier.fillMaxWidth()) {Text(inputState(item),Modifier.padding(12.dp))};return}
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(12.dp),verticalArrangement=Arrangement.spacedBy(6.dp)) {
            item.optJSONObject("source_agent")?.let {source->Text(source.optString("display_name")+" · "+source.optString("machine"),style=MaterialTheme.typography.labelMedium)}
            Row(Modifier.fillMaxWidth()) {
                TextButton(onClick={expanded=!expanded},modifier=Modifier.weight(1f)) {Text((if(expanded) "⌃  " else "⌄  ")+stringResource(R.string.user_input_count,questions.size))}
                TextButton(onClick={full=true}) {Text(stringResource(R.string.user_input_expand))}
            }
            Text(questions.first().question,style=MaterialTheme.typography.titleSmall,maxLines=if(expanded) Int.MAX_VALUE else 2)
            if(expanded && !full) Column(Modifier.heightIn(max=320.dp).verticalScroll(rememberScrollState())) {UserInputForm(vm,item,questions,key)}
            else if(!expanded) Text(inputState(item),style=MaterialTheme.typography.bodySmall)
        }
    }
    DisposableEffect(full) {if(full) onFullVisibility(true);onDispose {if(full) onFullVisibility(false)}}
    if(full) Dialog(onDismissRequest={full=false;vm.questionFocusHandled(item.getString("request_id"))},properties=DialogProperties(usePlatformDefaultWidth=false)) {
        Surface(Modifier.fillMaxSize().safeDrawingPadding()) {
            Column(Modifier.padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(12.dp)) {
                TextButton(onClick={full=false;vm.questionFocusHandled(item.getString("request_id"))}) {Text(stringResource(R.string.user_input_close))}
                UserInputForm(vm,item,questions,key)
            }
        }
    }
}
@Composable private fun inputState(item:JSONObject):String = stringResource(when(item.optString("state")) {
    "applied"->R.string.user_input_applied
    "submitted"->R.string.user_input_submitted
    "pending"->if(item.optBoolean("can_respond")) R.string.user_input_pending else R.string.user_input_unavailable
    "unknown"->R.string.user_input_unknown
    else->R.string.user_input_closed
})
@Composable
private fun UserInputForm(vm:MobileViewModel,item:JSONObject,questions:List<InputQuestion>,key:String) {
    val pending=vm.userInputCommandState(item)
    var answers by remember(key,pending) {mutableStateOf(inputAnswers(vm.userInputDraft(item),questions))}
    var index by rememberSaveable(key) {mutableIntStateOf(0)}
    var voiceQuestion by rememberSaveable(key) {mutableStateOf<String?>(null)}
    var voiceError by remember(key) {mutableStateOf(false)}
    val canRespond=item.optBoolean("can_respond") && !vm.busy && pending !in setOf("local","accepted","outcome_unknown")
    fun update(id:String,answer:InputAnswer) {
        val next=answers+(id to answer)
        if(inputAnswerJson(next).toString().toByteArray(Charsets.UTF_8).size>32*1024) return
        answers=next;vm.saveUserInputDraft(item,key,inputAnswerJson(next))
    }
    val voice=rememberLauncherForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        val qid=voiceQuestion;voiceQuestion=null
        if(result.resultCode==Activity.RESULT_OK && qid!=null) {
            val text=result.data?.getStringArrayListExtra(RecognizerIntent.EXTRA_RESULTS)?.firstOrNull()
            val old=answers[qid]
            if(!text.isNullOrBlank() && old!=null) {
                val combined=listOf(old.text,text).filter {it.isNotEmpty()}.joinToString(" ")
                if(combined.toByteArray(Charsets.UTF_8).size<=10_000) update(qid,old.copy(text=combined)) else voiceError=true
            }
        }
    }
    val question=questions[index.coerceIn(0,questions.lastIndex)]
    val answer=answers.getValue(question.id)
    Column(verticalArrangement=Arrangement.spacedBy(10.dp)) {
        Text(stringResource(R.string.user_input_position,index+1,questions.size),style=MaterialTheme.typography.labelMedium)
        Text(question.header,style=MaterialTheme.typography.titleMedium)
        Text(question.question)
        question.options.forEach { option ->
            Surface(shape=MaterialTheme.shapes.medium,color=if(option.value in answer.selected) MaterialTheme.colorScheme.secondaryContainer else MaterialTheme.colorScheme.surfaceContainer) {
                Row(Modifier.fillMaxWidth().clickable(enabled=canRespond) {update(question.id,inputSelect(question,answer,option.value))}.padding(8.dp)) {
                    if(question.multiple) Checkbox(checked=option.value in answer.selected,onCheckedChange={update(question.id,inputSelect(question,answer,option.value))},enabled=canRespond)
                    else RadioButton(selected=option.value in answer.selected,onClick={update(question.id,inputSelect(question,answer,option.value))},enabled=canRespond)
                    Column(Modifier.weight(1f).padding(top=8.dp)) {Text(option.label,style=MaterialTheme.typography.titleSmall);if(option.description.isNotEmpty()) Text(option.description,style=MaterialTheme.typography.bodySmall)}
                }
            }
        }
        if(answer.selected.isNotEmpty() && canRespond) TextButton(onClick={update(question.id,answer.copy(selected=emptyList()))}) {Text(stringResource(R.string.user_input_clear_choices))}
        if(question.custom) {
            OutlinedTextField(value=answer.text,onValueChange={if(it.toByteArray(Charsets.UTF_8).size<=10_000) update(question.id,answer.copy(text=it))},
                label={Text(stringResource(R.string.user_input_free))},modifier=Modifier.fillMaxWidth(),enabled=canRespond,minLines=2,maxLines=6)
            TextButton(onClick={voiceQuestion=question.id;voiceError=false
                try {voice.launch(Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).putExtra(RecognizerIntent.EXTRA_LANGUAGE_MODEL,RecognizerIntent.LANGUAGE_MODEL_FREE_FORM)
                    .putExtra(RecognizerIntent.EXTRA_LANGUAGE,Locale.getDefault().toLanguageTag()).putExtra(RecognizerIntent.EXTRA_PROMPT,question.question))}
                catch(_:ActivityNotFoundException){voiceQuestion=null;voiceError=true}
                catch(_:SecurityException){voiceQuestion=null;voiceError=true}
            },enabled=canRespond) {Text(stringResource(R.string.user_input_dictate))}
            if(voiceError) Text(stringResource(R.string.user_input_voice_unavailable),color=MaterialTheme.colorScheme.error)
        }
        Row(horizontalArrangement=Arrangement.spacedBy(8.dp)) {
            if(index>0) OutlinedButton(onClick={index--}) {Text(stringResource(R.string.user_input_previous))}
            if(index<questions.lastIndex) Button(onClick={index++},enabled=inputAnswerValid(question,answer)) {Text(stringResource(R.string.user_input_next))}
        }
        Text(if(pending=="outcome_unknown") stringResource(R.string.user_input_unknown) else inputState(item),style=MaterialTheme.typography.bodySmall)
        Button(onClick={vm.answerUserInput(item,key,inputAnswerJson(answers))},enabled=!vm.busy && (pending in setOf("local","outcome_unknown") || (canRespond && questions.all {inputAnswerValid(it,answers.getValue(it.id))})),modifier=Modifier.fillMaxWidth()) {
            Text(stringResource(if(pending=="outcome_unknown") R.string.user_input_verify else if(pending=="local") R.string.user_input_send_saved else R.string.user_input_send))
        }
    }
}
