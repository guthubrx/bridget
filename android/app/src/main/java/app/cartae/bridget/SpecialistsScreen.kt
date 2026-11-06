package app.cartae.bridget

import android.content.Intent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import org.json.JSONArray
import org.json.JSONObject
import java.util.UUID

private fun specialistCommand(action:String, fill:JSONObject.()->Unit = {}):JSONObject =
    JSONObject().put("version",1).put("action",action).apply {if(!specialistIsRead(action))put("request_id",UUID.randomUUID().toString());fill()}
private fun specialistItems(value:JSONObject, field:String="items"):List<JSONObject> {
    val rows=value.optJSONArray(field) ?: return emptyList()
    return (0 until rows.length()).mapNotNull {rows.optJSONObject(it)}
}
private fun words(text:String)=specialistWords(text)
private fun scope(project:String)=specialistJson(specialistScope(project))
private fun jsonValue(value:Any?):Any? = when(value) {
    is Map<*,*> -> JSONObject().also {o->value.forEach {(k,v)->o.put(k.toString(),jsonValue(v))}}
    is List<*> -> JSONArray().also {a->value.forEach {a.put(jsonValue(it))}}
    else -> value ?: JSONObject.NULL
}
fun specialistJson(command:Map<String,Any?>):JSONObject = jsonValue(command) as JSONObject
private fun plain(value:Any?):Any? = when(value) {
    is JSONObject -> value.keys().asSequence().associateWith {plain(value.opt(it))}
    is JSONArray -> (0 until value.length()).map {plain(value.opt(it))}
    JSONObject.NULL -> null
    else -> value
}
private fun plainMap(text:String):Map<String,Any?>? = runCatching {plain(JSONObject(text)) as? Map<String,Any?>}.getOrNull()

@Composable
fun SpecialistsScreen(vm:MobileViewModel) {
    var role by remember {mutableStateOf("")}
    var project by remember {mutableStateOf("")}
    var query by remember {mutableStateOf("")}
    var dialog by remember {mutableStateOf<String?>(null)}
    var name by remember {mutableStateOf("")}
    var roles by remember {mutableStateOf("")}
    var instructions by remember {mutableStateOf("")}
    var mandate by remember {mutableStateOf("")}
    var launchType by remember {mutableStateOf("")}
    var cwd by remember {mutableStateOf("")}
    var durable by remember {mutableStateOf(true)}
    var autoStart by remember {mutableStateOf(false)}
    var limited by remember {mutableStateOf(false)}
    var tokens by remember {mutableStateOf("")}
    var duration by remember {mutableStateOf("")}
    var descendants by remember {mutableStateOf("")}
    var activationAgent by remember {mutableStateOf("")}
    var activationExpected by remember {mutableStateOf(0L)}
    var profileId by remember {mutableStateOf("")}
    var profileRevision by remember {mutableStateOf(1L)}
    var reqKey by remember {mutableStateOf("")}
    var reqRole by remember {mutableStateOf("")}
    var reqProject by remember {mutableStateOf("")}
    var reqTask by remember {mutableStateOf("")}
    var reqAgent by remember {mutableStateOf("")}
    var memText by remember {mutableStateOf("")}
    var memKind by remember {mutableStateOf("fact")}
    var memShared by remember {mutableStateOf(false)}
    var memTarget by remember {mutableStateOf<JSONObject?>(null)}
    var importText by remember {mutableStateOf("")}
    var convAgent by remember {mutableStateOf("")}
    var teamLines by remember {mutableStateOf("")}
    val context=LocalContext.current
    val detail=vm.specialistDetail
    val specialist=detail?.optJSONObject("specialist")
    val definition=detail?.optJSONObject("definition")
    val agentId=specialist?.optString("agent_id").orEmpty()
    val receipt=vm.specialistReceipt
    fun reload() {vm.specialistCommand(if(role.isBlank())specialistCommand("list") else specialistCommand("search"){put("role_id",role);if(project.isNotBlank())put("project_id",project)})}
    fun refreshDetail() {if(agentId.isNotBlank())vm.specialistCommand(specialistCommand("get"){put("specialist_agent_id",agentId)})}
    fun more(action:String,fill:JSONObject.()->Unit) {
        val next=vm.specialistCursors[action] ?: return
        vm.specialistCommand(specialistCommand(action){fill();put("cursor",next);put("limit",25)})
    }
    LaunchedEffect(Unit) {reload()}
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(12.dp)) {
        Text(stringResource(R.string.specialists),style=MaterialTheme.typography.headlineSmall)
        Text(stringResource(R.string.specialists_intro),style=MaterialTheme.typography.bodyMedium)
        OutlinedTextField(role,{role=it},label={Text(stringResource(R.string.specialist_role))},modifier=Modifier.fillMaxWidth())
        OutlinedTextField(project,{project=it},label={Text(stringResource(R.string.specialist_projects))},supportingText={
            val ids=vm.specialists.optJSONArray("project_options")
            if(ids!=null)Text(ids.toString())
        },modifier=Modifier.fillMaxWidth())
        Row(horizontalArrangement=Arrangement.spacedBy(8.dp)) {
            Button(onClick={reload()}) {Text(stringResource(R.string.specialist_search))}
            OutlinedButton(onClick={name="";roles=role;instructions="";dialog="profile"}) {Text(stringResource(R.string.specialist_create_profile))}
        }
        Row(horizontalArrangement=Arrangement.spacedBy(8.dp)) {
            OutlinedButton(onClick={reqKey="";reqRole=role;reqProject=project;reqTask="";reqAgent="";dialog="request_role"}) {Text(stringResource(R.string.specialist_request_role))}
            OutlinedButton(onClick={name="";instructions="";teamLines="";dialog="team"}) {Text(stringResource(R.string.specialist_create_team))}
        }
        OutlinedButton(onClick={importText="";dialog="import"}) {Text(stringResource(R.string.specialist_import))}
        vm.specialistPending?.let {
            Card(colors=CardDefaults.cardColors(containerColor=MaterialTheme.colorScheme.tertiaryContainer)) {
                Column(Modifier.padding(12.dp)) {
                    Text(stringResource(R.string.specialist_pending))
                    val pc=it.optJSONObject("command")
                    Text(listOf(pc?.optString("action"),pc?.optString("request_id")).joinToString(" · "),style=MaterialTheme.typography.labelSmall)
                    TextButton(onClick=vm::specialistCheckPending){Text(stringResource(R.string.specialist_check_status))}
                    TextButton(onClick=vm::retrySpecialistCommand){Text(stringResource(R.string.specialist_retry_same))}
                }
            }
        }
        OutlinedButton(onClick={vm.specialistCommand(specialistCommand("profiles"))}) {Text(stringResource(R.string.specialist_saved_profiles))}
        specialistItems(vm.specialistProfiles).forEach {profile->
            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(12.dp)) {
                    Text(profile.optString("name"),style=MaterialTheme.typography.titleMedium)
                    Text(stringResource(R.string.specialist_profile_version,profile.optLong("revision")))
                    TextButton(onClick={profileId=profile.getString("profile_id");profileRevision=profile.getLong("revision");vm.specialistCommand(specialistCommand("profile_versions"){put("profile_id",profileId)})}) {
                        Text(stringResource(R.string.specialist_versions))
                    }
                    TextButton(onClick={profileId=profile.getString("profile_id");profileRevision=profile.getLong("revision");activationAgent=UUID.randomUUID().toString();activationExpected=0;dialog="activate"}) {Text(stringResource(R.string.specialist_activate))}
                    Row {
                        TextButton(onClick={vm.specialistCommand(specialistJson(specialistExport(profile.getString("profile_id"),profile.getLong("revision"))))}) {Text(stringResource(R.string.specialist_export))}
                        TextButton(onClick={profileId=profile.getString("profile_id");profileRevision=profile.getLong("revision");convAgent="";dialog="convert"}) {Text(stringResource(R.string.specialist_convert))}
                    }
                    if(profile.optString("kind")=="team")TextButton(onClick={
                        profileId=profile.getString("profile_id");profileRevision=profile.getLong("revision");reqKey="";reqProject=project;reqTask="";dialog="request_team"
                    }) {Text(stringResource(R.string.specialist_request_team))}
                }
            }
        }
        if(vm.specialistCursors.containsKey("profiles"))TextButton(onClick={more("profiles"){}}) {Text(stringResource(R.string.specialist_more))}
        specialistItems(vm.specialistVersions).forEach {version->
            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(12.dp)) {
                    Text(stringResource(R.string.specialist_profile_version,version.optLong("revision")))
                    Text(version.optJSONObject("definition")?.optString("instructions").orEmpty())
                    TextButton(onClick={vm.specialistCommand(specialistCommand("approve_revision"){put("profile_id",version.getJSONObject("definition").getString("profile_id"));put("revision",version.getLong("revision"))})}) {Text(stringResource(R.string.specialist_approve))}
                }
            }
        }
        vm.specialistExport?.let {export->
            val text=(export.optJSONObject("definition") ?: export).toString(2)
            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(12.dp),verticalArrangement=Arrangement.spacedBy(4.dp)) {
                    Text(stringResource(R.string.specialist_export_note),style=MaterialTheme.typography.labelMedium)
                    Text(specialistClip(text),style=MaterialTheme.typography.bodySmall)
                    if(text.length>SPECIALIST_IMPORT_LIMIT)Text(stringResource(R.string.specialist_export_too_large))
                    else TextButton(onClick={context.startActivity(Intent.createChooser(Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT,text),null))}) {
                        Text(stringResource(R.string.specialist_share_text))
                    }
                }
            }
        }
        if(specialistItems(vm.specialists).isEmpty())Text(stringResource(R.string.specialist_empty))
        specialistItems(vm.specialists).forEach {item->
            val person=item.optJSONObject("specialist") ?: return@forEach
            val id=person.optString("agent_id")
            Card(onClick={vm.specialistCommand(specialistCommand("get"){put("specialist_agent_id",id)})},modifier=Modifier.fillMaxWidth()) {
                Column(Modifier.padding(12.dp),verticalArrangement=Arrangement.spacedBy(4.dp)) {
                    val titles=specialistItems(item,"roles").map{it.optString("label")}
                    Text(titles.joinToString().ifBlank {id},style=MaterialTheme.typography.titleMedium)
                    Text(stringResource(if(person.optBoolean("durable"))R.string.specialist_durable else R.string.specialist_retained))
                    Text(stringResource(if(person.optBoolean("enabled"))R.string.specialist_enabled else R.string.specialist_disabled))
                    val presence=item.optJSONObject("presence")
                    Text(presence?.optString("state")?.ifBlank{null} ?: stringResource(R.string.specialist_not_observed))
                }
            }
        }
        if(vm.specialistCursors.containsKey("profile_versions"))TextButton(onClick={more("profile_versions"){put("profile_id",profileId)}}) {Text(stringResource(R.string.specialist_more))}
        if(vm.specialistCursors.containsKey("list"))TextButton(onClick={more("list"){}}) {Text(stringResource(R.string.specialist_more))}
        if(vm.specialistCursors.containsKey("search"))TextButton(onClick={more("search"){put("role_id",role);if(project.isNotBlank())put("project_id",project)}}) {Text(stringResource(R.string.specialist_more))}
        if(specialist!=null) {
            HorizontalDivider()
            Text(definition?.optJSONObject("definition")?.optString("name") ?: agentId,style=MaterialTheme.typography.titleLarge)
            Text(agentId,style=MaterialTheme.typography.labelSmall)
            Text(stringResource(R.string.specialist_profile_version,specialist.optLong("profile_revision")))
            definition?.optJSONObject("definition")?.optString("instructions")?.let {Text(it)}
            Text(specialist.optString("mandate"))
            Row {
                TextButton(onClick={
                    profileId=specialist.optString("profile_id");profileRevision=specialist.optLong("profile_revision",1)
                    activationAgent=agentId;activationExpected=specialist.getLong("revision");dialog="activate"
                },enabled=!specialist.isNull("profile_id")){Text(stringResource(R.string.specialist_activate_existing))}
                TextButton(onClick={vm.specialistCommand(specialistCommand("capture_history"){
                    put("specialist_agent_id",agentId);if(project.isNotBlank())put("project_id",project)
                })}){Text(stringResource(R.string.specialist_capture_history))}
            }
            detail.optJSONObject("memory_capture")?.let {source->
                Text(stringResource(R.string.specialist_capture_state,source.optString("status")),style=MaterialTheme.typography.labelMedium)
                if(source.has("code")&&!source.isNull("code"))Text(source.optString("code"),style=MaterialTheme.typography.labelSmall)
            }
            val budget=specialist.optJSONObject("budget")
            Text(if(budget==null||listOf("max_duration_secs","max_facturable_tokens","max_descendants").all{budget.isNull(it)})
                stringResource(R.string.specialist_budget_unlimited) else stringResource(R.string.specialist_budget_limited))
            Row(horizontalArrangement=Arrangement.spacedBy(8.dp)) {
                OutlinedButton(onClick={refreshDetail()}) {Text(stringResource(R.string.specialist_refresh))}
                OutlinedButton(onClick={
                    profileId=specialist.optString("profile_id");profileRevision=specialist.optLong("profile_revision",1)
                    name=definition?.optJSONObject("definition")?.optString("name").orEmpty()
                    instructions=definition?.optJSONObject("definition")?.optString("instructions").orEmpty()
                    roles=definition?.optJSONObject("definition")?.optJSONArray("roles")?.let{a->(0 until a.length()).joinToString(","){a.optString(it)}}.orEmpty()
                    dialog="revision"
                }){Text(stringResource(R.string.specialist_revise))}
            }
            Row(horizontalArrangement=Arrangement.spacedBy(8.dp)) {
                OutlinedButton(onClick={
                    mandate=specialist.optString("mandate");limited=false;dialog="mandate"
                }){Text(stringResource(R.string.specialist_edit_mandate))}
                OutlinedButton(onClick={dialog="disable"}){Text(stringResource(R.string.specialist_disable))}
            }
            OutlinedButton(onClick={reqKey="";reqRole=role;reqProject=project;reqTask="";reqAgent=agentId;dialog="request_role"}) {Text(stringResource(R.string.specialist_request_this))}
            OutlinedTextField(query,{query=it},label={Text(stringResource(R.string.specialist_memory_query))},modifier=Modifier.fillMaxWidth())
            Button(onClick={vm.specialistCommand(specialistCommand("memory_recall"){put("specialist_agent_id",agentId);put("project_id",project);put("query",query)})},
                enabled=project.isNotBlank()){Text(stringResource(R.string.specialist_recall))}
            OutlinedButton(onClick={memTarget=null;memText="";memKind="fact";memShared=false;dialog="memory"},enabled=project.isNotBlank()) {Text(stringResource(R.string.specialist_declare_memory))}
            if(project.isBlank())Text(stringResource(R.string.specialist_project_required),style=MaterialTheme.typography.labelSmall)
            specialistItems(vm.specialistMemory).forEach {memory->
                Card(Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(12.dp),verticalArrangement=Arrangement.spacedBy(4.dp)) {
                        Text(memory.optString("kind")+" · "+memory.optString("project_id"),style=MaterialTheme.typography.labelMedium)
                        Text(memory.optString("text"))
                        Text(specialistClip(memory.optJSONObject("source")?.toString().orEmpty(),300),style=MaterialTheme.typography.labelSmall)
                        TextButton(onClick={memTarget=memory;memText=memory.optString("text");memKind=memory.optString("kind").ifBlank{"fact"};dialog="correct"}) {
                            Text(stringResource(R.string.specialist_correct))
                        }
                        TextButton(onClick={vm.specialistCommand(specialistCommand("memory_forget"){put("memory_id",memory.getString("memory_id"));put("expected_revision",memory.getLong("revision"))})}) {
                            Text(stringResource(R.string.specialist_forget))
                        }
                    }
                }
            }
        }
        if(vm.specialistCursors.containsKey("memory_recall"))TextButton(onClick={more("memory_recall"){put("specialist_agent_id",agentId);put("project_id",project);put("query",query)}}) {Text(stringResource(R.string.specialist_more))}
        receipt?.let {value->
            Card(colors=CardDefaults.cardColors(containerColor=if(value.optString("status")=="refused")MaterialTheme.colorScheme.errorContainer else MaterialTheme.colorScheme.secondaryContainer)) {
                Column(Modifier.padding(12.dp),verticalArrangement=Arrangement.spacedBy(6.dp)) {
                    Text(specialistClip(value.optString("message").ifBlank {value.optString("status")},600))
                    listOf("action","scope","status","code","original_code","request_id").filter {value.has(it)&&!value.isNull(it)}.takeIf {it.isNotEmpty()}?.let {keys->
                        Text(keys.joinToString(" · ") {it+"="+specialistClip(value.opt(it).toString(),120)},style=MaterialTheme.typography.labelSmall)
                    }
                    if(specialistOutcomeOpen(value.optString("code").ifBlank{null},value.optString("status").ifBlank{null},value.optString("delivery_observation").ifBlank{null}))
                        Text(stringResource(R.string.specialist_not_final),style=MaterialTheme.typography.labelMedium)
                    if(value.has("next_action"))Text(value.optString("next_action"))
                    if(value.has("human_action"))Text(value.optString("human_action"))
                    value.optJSONObject("data")?.let {data->
                        if(data.has("profile_id")&&data.has("revision")) {
                            Text(stringResource(R.string.specialist_profile_version,data.optLong("revision")))
                            Row {
                                TextButton(onClick={vm.specialistCommand(specialistCommand("approve_revision"){put("profile_id",data.getString("profile_id"));put("revision",data.getLong("revision"))})}) {
                                    Text(stringResource(R.string.specialist_approve))
                                }
                                TextButton(onClick={profileId=data.getString("profile_id");profileRevision=data.getLong("revision");activationAgent=UUID.randomUUID().toString();activationExpected=0;dialog="activate"}) {
                                    Text(stringResource(R.string.specialist_activate))
                                }
                            }
                        }
                    }
                }
            }
        }
        Spacer(Modifier.height(24.dp))
    }
    fun build(mode:String,id:String):Map<String,Any?>? = when(mode) {
        "request_role"->specialistRequestRole(id,reqRole,reqProject,reqTask,reqAgent)
        "request_team"->specialistRequestTeam(id,profileId,profileRevision,reqProject,reqTask)
        "memory"->specialistMemoryWrite(id,agentId,project,memShared,memKind,memText)
        "correct"->memTarget?.let {specialistMemoryCorrect(id,it.optString("memory_id"),it.optLong("revision"),memKind,memText)}
        "import"->plainMap(importText)?.let {specialistImport(id,it,importText.length)}
        "convert"->specialistConvert(id,convAgent,profileId,profileRevision,0)
        "team"->specialistTeamMembers(teamLines)?.let {specialistTeamProposal(id,UUID.randomUUID().toString(),name,instructions,it)}
        else->null
    }
    val newModes=setOf("request_role","request_team","memory","correct","import","convert","team")
    dialog?.let {mode->
        val keyOk=mode !in setOf("request_role","request_team") || specialistRequestKey(reqKey)!=null
        val newOk=mode !in newModes || (keyOk && build(mode,"check")!=null)
        AlertDialog(onDismissRequest={dialog=null},title={Text(stringResource(when(mode) {
            "disable"->R.string.specialist_disable
            "mandate"->R.string.specialist_edit_mandate
            "activate"->R.string.specialist_activate
            "revision"->R.string.specialist_revise
            "request_role"->R.string.specialist_request_role
            "request_team"->R.string.specialist_request_team
            "memory","correct"->R.string.specialist_declare_memory
            "import"->R.string.specialist_import
            "convert"->R.string.specialist_convert
            "team"->R.string.specialist_create_team
            else->R.string.specialist_create_profile
        }))},text={
            Column(Modifier.verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)) {
                if(mode=="disable")Text(stringResource(R.string.specialist_disable_detail))
                if(mode=="request_role"||mode=="request_team") {
                    Text(stringResource(R.string.specialist_request_explanation))
                    if(mode=="request_role") {
                        OutlinedTextField(reqRole,{reqRole=it},label={Text(stringResource(R.string.specialist_role))})
                        OutlinedTextField(reqAgent,{reqAgent=it},label={Text(stringResource(R.string.specialist_reuse_agent))},supportingText={Text(stringResource(R.string.specialist_reuse_hint))})
                    }
                    OutlinedTextField(reqProject,{reqProject=it},label={Text(stringResource(R.string.specialist_project))})
                    OutlinedTextField(reqTask,{reqTask=it},label={Text(stringResource(R.string.specialist_task))},minLines=3)
                    OutlinedTextField(reqKey,{reqKey=it},label={Text(stringResource(R.string.specialist_stable_key))},supportingText={Text(stringResource(R.string.specialist_stable_key_hint))})
                }
                if(mode=="memory"||mode=="correct") {
                    Text(stringResource(R.string.specialist_memory_statement))
                    SPECIALIST_MEMORY_KINDS.forEach {k->Row {RadioButton(memKind==k,{memKind=k});Text(k)}}
                    if(mode=="memory")Row {Checkbox(memShared,{memShared=it});Text(stringResource(R.string.specialist_memory_shared))}
                    OutlinedTextField(memText,{memText=it},label={Text(stringResource(R.string.specialist_memory_text))},minLines=3)
                }
                if(mode=="import") {
                    Text(stringResource(R.string.specialist_import_note))
                    OutlinedTextField(importText,{importText=it.take(SPECIALIST_IMPORT_LIMIT+1)},label={Text(stringResource(R.string.specialist_import_text))},minLines=6)
                    if(importText.isNotBlank()&&!newOk)Text(stringResource(R.string.specialist_import_invalid))
                }
                if(mode=="convert") {
                    Text(stringResource(R.string.specialist_convert_note))
                    OutlinedTextField(convAgent,{convAgent=it},label={Text(stringResource(R.string.specialist_conversation_id))})
                    TextButton(onClick={convAgent=UUID.randomUUID().toString()}) {Text(stringResource(R.string.specialist_new_identity))}
                    if(convAgent.isNotBlank()&&specialistUuid(convAgent)==null)Text(stringResource(R.string.specialist_uuid_invalid))
                }
                if(mode=="team") {
                    OutlinedTextField(name,{name=it},label={Text(stringResource(R.string.specialist_name))})
                    OutlinedTextField(instructions,{instructions=it},label={Text(stringResource(R.string.specialist_instructions))},minLines=2)
                    Text(stringResource(R.string.specialist_team_note))
                    OutlinedTextField(teamLines,{teamLines=it},label={Text(stringResource(R.string.specialist_team_members))},minLines=4)
                    if(teamLines.isNotBlank()&&specialistTeamMembers(teamLines)==null)Text(stringResource(R.string.specialist_team_invalid))
                }
                if(mode in setOf("profile","revision")) {
                    OutlinedTextField(name,{name=it},label={Text(stringResource(R.string.specialist_name))})
                    OutlinedTextField(roles,{roles=it},label={Text(stringResource(R.string.specialist_roles))})
                    OutlinedTextField(instructions,{instructions=it},label={Text(stringResource(R.string.specialist_instructions))},minLines=4)
                }
                if(mode in setOf("activate","mandate")) {
                    Text(stringResource(R.string.specialist_mandate_explanation))
                    OutlinedTextField(mandate,{mandate=it},label={Text(stringResource(R.string.specialist_mandate))},minLines=3)
                    OutlinedTextField(project,{project=it},label={Text(stringResource(R.string.specialist_projects))})
                    if(mode=="activate")Row {Checkbox(durable,{durable=it});Text(stringResource(R.string.specialist_durable))}
                    Row {Checkbox(autoStart,{autoStart=it});Text(stringResource(R.string.specialist_auto_start))}
                    if(autoStart) {
                        OutlinedTextField(launchType,{launchType=it},label={Text(stringResource(R.string.specialist_launch_type))})
                        OutlinedTextField(cwd,{cwd=it},label={Text(stringResource(R.string.specialist_work_directory))})
                    }
                    Row {Checkbox(limited,{limited=it});Text(stringResource(R.string.specialist_limit_budget))}
                    if(limited) {
                        OutlinedTextField(tokens,{tokens=it.filter(Char::isDigit)},label={Text(stringResource(R.string.specialist_budget_tokens))})
                        OutlinedTextField(duration,{duration=it.filter(Char::isDigit)},label={Text(stringResource(R.string.specialist_budget_duration))})
                        OutlinedTextField(descendants,{descendants=it.filter(Char::isDigit)},label={Text(stringResource(R.string.specialist_budget_descendants))})
                    } else Text(stringResource(R.string.specialist_budget_unlimited))
                }
            }
        },confirmButton={TextButton(enabled=newOk,onClick={
            if(mode in newModes) {
                val id=if(mode in setOf("request_role","request_team"))specialistRequestKey(reqKey)!! else UUID.randomUUID().toString()
                build(mode,id)?.let {vm.specialistCommand(specialistJson(it))}
                dialog=null
            } else {
            val action=when(mode){"profile"->"propose_profile";"revision"->"propose_revision";"activate"->"activate";"mandate"->"configure_mandate";else->"disable"}
            val command=specialistCommand(action) {
                when(mode) {
                    "profile","revision"->{
                        val body=JSONObject().put("profile_id",if(mode=="profile")UUID.randomUUID().toString() else profileId)
                            .put("name",name).put("kind","profile").put("instructions",instructions).put("roles",JSONArray(words(roles)))
                        put("definition",body)
                        if(mode=="revision")put("expected_revision",profileRevision)
                    }
                    "disable"->{put("specialist_agent_id",agentId);put("expected_revision",specialist!!.getLong("revision"))}
                    else->{
                        put("specialist_agent_id",if(mode=="activate")activationAgent else agentId)
                        put("expected_revision",if(mode=="activate")activationExpected else specialist!!.getLong("revision"))
                        put("scope",scope(project));put("auto_start",autoStart)
                        if(mandate.isNotBlank())put("mandate",mandate) else if(mode=="mandate")put("mandate","")
                        if(mode=="activate"){put("profile_id",profileId);put("profile_revision",profileRevision);put("durable",durable)}
                        if(limited)put("budget",JSONObject().apply {
                            tokens.toLongOrNull()?.let{put("max_facturable_tokens",it)}
                            duration.toLongOrNull()?.let{put("max_duration_secs",it)}
                            descendants.toLongOrNull()?.let{put("max_descendants",it)}
                        })
                        if(autoStart)put("launch_policy",JSONObject().put("agent_type",launchType).put("cwd",cwd).put("posture","development"))
                    }
                }
            }
            vm.specialistCommand(command);dialog=null
            }
        }){Text(stringResource(R.string.specialist_confirm))}},dismissButton={TextButton(onClick={dialog=null}){Text(stringResource(R.string.cancel))}})
    }
}
