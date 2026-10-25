package app.cartae.bridget

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.json.JSONObject

internal fun queuedMessageVisible(state: String, observation: String?, queueState: String, pendingAction: Boolean = false): Boolean =
    pendingAction || observation !in setOf("turn_start", "turn_end") && queueState !in setOf("cancelled", "released")
        && state in setOf("local", "outcome_unknown", "accepted")

/** O(n+h): index par commande, maximum 256 enveloppes locales et 32 retenues. */
@Composable
internal fun MessageQueue(vm: MobileViewModel, agent: String) {
    var expanded by rememberSaveable(agent) { mutableStateOf(true) }
    var editing by remember(agent) { mutableStateOf<JSONObject?>(null) }
    var text by remember { mutableStateOf("") }
    val held = remember(vm.messageQueue) { vm.messageQueue.associateBy { it.getString("command_id") } }
    val positions = remember(vm.messageQueue) { vm.messageQueue.mapIndexed { index,item -> item.getString("command_id") to index }.toMap() }
    val entries = remember(vm.outbox, held, agent) {
        vm.conversationEntries(agent).mapNotNull { entry ->
            val envelope = entry.getJSONObject("envelope")
            if (envelope.optString("type") != "message" || envelope.optString("agent_id") != agent
                || !queuedMessageVisible(entry.getString("state"), entry.optJSONObject("delivery_observation")?.optString("event"),entry.optString("queue_state"),entry.has("queue_change"))) null
            else JSONObject().put("command_id", envelope.getString("command_id"))
                .put("body", held[envelope.getString("command_id")]?.getString("body") ?: entry.optString("queue_body",envelope.getString("body")))
                .put("revision", held[envelope.getString("command_id")]?.getString("revision") ?: entry.optString("queue_revision"))
                .put("local",entry.getString("state")=="local")
                .put("held",held.containsKey(envelope.getString("command_id")) && entry.optString("queue_state") !in setOf("promoted","cancelled"))
                .put("pending",entry.has("queue_change"))
                .put("promoted",entry.optString("queue_state")=="promoted")
                .put("issued_at",entry.optLong("created_at",envelope.optLong("issued_at")*1000))
                .put("position",positions[envelope.getString("command_id")] ?: Int.MAX_VALUE)
        }.sortedWith(compareBy<JSONObject> { it.getInt("position") }.thenBy { it.getLong("issued_at") })
    }
    if (entries.isEmpty()) return
    Surface(shape=RoundedCornerShape(topStart=20.dp,topEnd=20.dp),color=MaterialTheme.colorScheme.surfaceContainer,
        modifier=Modifier.fillMaxWidth()) {
        Column {
            TextButton(onClick={expanded=!expanded},modifier=Modifier.fillMaxWidth()) {
                Text(stringResource(R.string.message_queue,entries.size),modifier=Modifier.weight(1f))
                Text(if(expanded) "⌃" else "⌄",fontSize=22.sp)
            }
            if(expanded) LazyColumn(Modifier.heightIn(max=220.dp)) {
                items(entries,key={it.getString("command_id")}) { item ->
                    val enabled = !vm.busy && !item.getBoolean("pending")
                        && (item.getBoolean("local") || (item.getBoolean("held") && vm.queueAvailable))
                    Column(Modifier.fillMaxWidth().padding(start=14.dp,end=4.dp,bottom=8.dp)) {
                        Row(verticalAlignment=Alignment.CenterVertically) {
                            Text(item.getString("body"),modifier=Modifier.weight(1f),maxLines=3,overflow=TextOverflow.Ellipsis,
                                style=MaterialTheme.typography.bodyMedium)
                            QueueIcon(R.string.queue_edit,"✎",enabled) { editing=item; text=item.getString("body") }
                            QueueIcon(R.string.queue_cancel,"×",enabled) { vm.changeQueuedMessage(item,"cancel") }
                            QueueIcon(R.string.queue_promote,"↗",enabled) { vm.changeQueuedMessage(item,"promote") }
                        }
                        Row(verticalAlignment=Alignment.CenterVertically) {
                            Text(stringResource(when {
                                item.getBoolean("pending") -> R.string.queue_action_unconfirmed
                                item.getBoolean("local") -> R.string.queue_local
                                item.getBoolean("promoted") -> R.string.queue_interrupt_requested
                                !vm.queueAvailable -> R.string.queue_check_pending
                                item.getBoolean("held") -> R.string.queue_waiting
                                else -> R.string.queue_transmitted
                            }),style=MaterialTheme.typography.labelSmall,color=MaterialTheme.colorScheme.onSurfaceVariant,
                                modifier=Modifier.weight(1f))
                            if(item.getBoolean("pending")) QueueIcon(R.string.queue_retry,"↻",!vm.busy) {vm.changeQueuedMessage(item,"retry")}
                            else if(!enabled && !vm.busy) QueueIcon(R.string.queue_retry,"↻",true) {vm.changeQueuedMessage(item,"retry")}
                        }
                    }
                }
            }
        }
    }
    editing?.let { item ->
        AlertDialog(onDismissRequest={editing=null},title={Text(stringResource(R.string.queue_edit))},
            text={OutlinedTextField(value=text,onValueChange={if(it.toByteArray(Charsets.UTF_8).size<=10_000) text=it},
                modifier=Modifier.fillMaxWidth(),minLines=3,maxLines=8)},
            confirmButton={TextButton(onClick={editing=null;vm.changeQueuedMessage(item,"edit",text)},enabled=!vm.busy && text.isNotBlank()) {
                Text(stringResource(R.string.queue_save))
            }},dismissButton={TextButton(onClick={editing=null}) {Text(stringResource(R.string.dismiss))}})
    }
}

@Composable
private fun QueueIcon(label:Int,symbol:String,enabled:Boolean,onClick:()->Unit) {
    val description=stringResource(label)
    IconButton(onClick=onClick,enabled=enabled,modifier=Modifier.size(40.dp).semantics {contentDescription=description}) {
        Text(symbol,fontSize=22.sp)
    }
}
