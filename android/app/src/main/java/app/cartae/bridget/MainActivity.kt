package app.cartae.bridget

import android.Manifest
import android.content.Intent
import android.net.ConnectivityManager
import android.net.Network
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.compose.foundation.gestures.scrollBy
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.clickable
import androidx.compose.foundation.Image
import androidx.compose.ui.res.painterResource
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.interaction.collectIsDraggedAsState
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.Alignment
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.first
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import com.journeyapps.barcodescanner.ScanContract
import com.journeyapps.barcodescanner.ScanOptions
import org.json.JSONObject

class MainActivity : ComponentActivity() {
    private val model: MobileViewModel by viewModels()
    private var scanned by mutableStateOf<String?>(null)
    private var networkCallback: ConnectivityManager.NetworkCallback? = null
    private val notificationPermission = registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
        if (granted) model.changeNotifications(true) else model.notificationPermissionDenied()
    }
    private fun enableNotifications() {
        if (!MobileNotifications.configured()) { model.changeNotifications(true); return }
        if (MobileNotifications.permitted(this)) model.changeNotifications(true)
        else notificationPermission.launch(Manifest.permission.POST_NOTIFICATIONS)
    }
    private val scanner = registerForActivityResult(ScanContract()) { result ->
        result.contents?.let { contents ->
            if (contents.length <= 4096 && runCatching { BridgetClient.validatedPairing(contents) }.isSuccess)
                scanned = contents else model.invalidQr()
        }
    }
    private val filePicker = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        uri?.let { runCatching { contentResolver.takePersistableUriPermission(it, Intent.FLAG_GRANT_READ_URI_PERMISSION) } }
        model.importFile(uri)
    }
    private val fileExporter = registerForActivityResult(ActivityResultContracts.CreateDocument("application/octet-stream")) { uri ->
        model.exportFile(uri)
    }
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            val colors = if (isSystemInDarkTheme()) darkColorScheme(primary = Color(0xFF9BBEFF))
                else lightColorScheme(primary = Color(0xFF155BE8), background = Color(0xFFF4F6FB))
            MaterialTheme(colorScheme = colors) {
                MobileApp(model, scanned,
                    onScan = { scanner.launch(ScanOptions().setDesiredBarcodeFormats(ScanOptions.QR_CODE)
                        .setPrompt(getString(R.string.scan_prompt)).setBeepEnabled(false).setBarcodeImageEnabled(false)) },
                    onConfirm = { scanned?.let(model::associate); scanned = null }, onDismiss = { scanned = null },
                    onPick = { if (model.beginFileSelection()) filePicker.launch(arrayOf("application/pdf", "text/plain", "text/html", "text/markdown", "image/png", "image/jpeg")) },
                    onExport = { metadata -> model.beginExport(metadata)?.let(fileExporter::launch) },
                    onNotifications = { if (model.notificationsEnabled) model.changeNotifications(false) else enableNotifications() })
            }
        }
    }
    override fun onStart() {
        super.onStart()
        model.setVisible(true)
        MobileUpdates.foreground(applicationContext, true)
        MobileNotifications.refresh(applicationContext)
        val manager = getSystemService(ConnectivityManager::class.java)
        val callback = object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) { runOnUiThread { model.networkAvailable() } }
        }
        runCatching { manager.registerDefaultNetworkCallback(callback); networkCallback = callback }
    }
    override fun onStop() {
        networkCallback?.let { callback -> runCatching { getSystemService(ConnectivityManager::class.java).unregisterNetworkCallback(callback) } }
        networkCallback = null
        model.setVisible(false)
        MobileUpdates.foreground(applicationContext, false)
        super.onStop()
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun MobileApp(vm: MobileViewModel, scanned: String?, onScan: () -> Unit, onConfirm: () -> Unit,
    onDismiss: () -> Unit, onPick: () -> Unit, onExport: (JSONObject) -> Unit, onNotifications: () -> Unit) {
    var tab by rememberSaveable { mutableIntStateOf(0) }
    var revoke by remember { mutableStateOf(false) }
    var forget by remember { mutableStateOf(false) }
    BackHandler(enabled = vm.selected != null && tab == 0 && !vm.busy) { vm.select(null) }
    val conversationOpen = vm.associated && tab == 0 && vm.selected != null
    Scaffold(
        topBar = { if (!conversationOpen) TopAppBar(title = { Text("Bridget", style = MaterialTheme.typography.headlineSmall) }) },
        bottomBar = {
            if (vm.associated && !conversationOpen) NavigationBar {
                listOf(R.string.agents, R.string.questions, R.string.outbox, R.string.settings).forEachIndexed { index, label ->
                    NavigationBarItem(selected = tab == index, onClick = { tab = index },
                        icon = { Text(listOf("◉", "?", "↗", "⚙")[index]) }, label = { Text(stringResource(label)) })
                }
            }
        }
    ) { padding ->
        Column(Modifier.fillMaxSize().padding(padding).imePadding().padding(horizontal = 16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp)) {
            if (vm.busy) LinearProgressIndicator(Modifier.fillMaxWidth())
            vm.transferProgress?.let { progress -> LinearProgressIndicator(progress = { progress }, modifier = Modifier.fillMaxWidth()) }
            vm.error?.let { code ->
                Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.errorContainer)) {
                    Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text(stringResource(errorText(code)), style = MaterialTheme.typography.titleSmall)
                        Text(stringResource(recoveryText(code)), style = MaterialTheme.typography.bodySmall)
                        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                            Text(code, style = MaterialTheme.typography.labelSmall)
                            TextButton(onClick = vm::clearError) { Text(stringResource(R.string.dismiss)) }
                        }
                    }
                }
            }
            vm.notice?.let { notice ->
                Text(stringResource(when (notice) {
                    "history_window_limit" -> R.string.history_window_limit
                    "file_saved" -> R.string.file_saved
                    "instruction_draft" -> R.string.instruction_draft
                    else -> R.string.saved_locally
                }), style = MaterialTheme.typography.bodySmall)
            }
            Box(Modifier.weight(1f)) {
                when {
                    !vm.associated -> Welcome(vm, onScan) { forget = true }
                    tab == 0 && vm.selected == null -> AgentDirectory(vm)
                    tab == 0 -> Conversation(vm, onPick, onExport)
                    tab == 1 -> Questions(vm)
                    tab == 2 -> Outbox(vm)
                    else -> Settings(vm, onRevoke = { revoke = true }, onForget = { forget = true }, onNotifications = onNotifications)
                }
            }
        }
    }
    if (scanned != null) AlertDialog(onDismissRequest = onDismiss,
        title = { Text(stringResource(R.string.confirm_pairing)) },
        text = { Text(runCatching { JSONObject(scanned).getString("endpoint") }.getOrDefault("")) },
        confirmButton = { TextButton(onClick = onConfirm) { Text(stringResource(R.string.associate)) } },
        dismissButton = { TextButton(onClick = onDismiss) { Text(stringResource(R.string.cancel)) } })
    if (revoke) Confirm(stringResource(R.string.revoke), stringResource(R.string.revoke_detail),
        onDismiss = { revoke = false }) { revoke = false; vm.revoke() }
    if (forget) Confirm(stringResource(R.string.forget_authorization), stringResource(R.string.forget_authorization_detail),
        onDismiss = { forget = false }) { forget = false; vm.forgetAuthorization() }
}

@Composable
private fun Welcome(vm: MobileViewModel, onScan: () -> Unit, onForget: () -> Unit) {
    Column(Modifier.fillMaxWidth().verticalScroll(rememberScrollState()).padding(top = 24.dp),
        verticalArrangement = Arrangement.spacedBy(18.dp)) {
        Text(stringResource(R.string.welcome), style = MaterialTheme.typography.headlineLarge)
        Text(stringResource(R.string.welcome_detail), style = MaterialTheme.typography.bodyLarge)
        Button(onClick = onScan, enabled = !vm.busy && !vm.pairingPending, modifier = Modifier.fillMaxWidth()) {
            Text(stringResource(R.string.scan))
        }
        if (vm.pairingPending) {
            Text(stringResource(R.string.pending_pairing))
            OutlinedButton(onClick = { vm.associate() }, enabled = !vm.busy) { Text(stringResource(R.string.reconcile)) }
            TextButton(onClick = onForget, enabled = !vm.busy) { Text(stringResource(R.string.forget_authorization)) }
        }
    }
}

@Composable
private fun AgentDirectory(vm: MobileViewModel) {
    var machineMenu by remember { mutableStateOf(false) }
    var projectMenu by remember { mutableStateOf(false) }
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Box {
                OutlinedButton(onClick = { machineMenu = true }, enabled = !vm.busy) {
                    Text(vm.machine.ifEmpty { stringResource(R.string.federation) })
                }
                DropdownMenu(expanded = machineMenu, onDismissRequest = { machineMenu = false }) {
                    DropdownMenuItem(text = { Text(stringResource(R.string.federation)) },
                        onClick = { machineMenu = false; vm.filter("") })
                    vm.machines.forEach { machine -> DropdownMenuItem(
                        text = { Text(machine.getString("machine") + " · " + stateText(machine.optString("availability"))) },
                        onClick = { machineMenu = false; vm.filter(machine.getString("machine")) }) }
                }
            }
            TextButton(onClick = { vm.refresh() }, enabled = !vm.busy) { Text(stringResource(R.string.refresh)) }
        }
        Box {
            OutlinedButton(onClick = { projectMenu = true }, enabled = !vm.busy, modifier = Modifier.fillMaxWidth()) {
                Text(vm.project?.optString("root")?.let(::projectTitle) ?: stringResource(R.string.all_projects), maxLines = 1, overflow=TextOverflow.Ellipsis)
            }
            DropdownMenu(expanded = projectMenu, onDismissRequest = { projectMenu = false }, modifier = Modifier.heightIn(max = 360.dp)) {
                DropdownMenuItem(text = { Text(stringResource(R.string.all_projects)) },
                    onClick = { projectMenu = false; vm.filter(vm.machine) })
                vm.projects.forEach { project -> DropdownMenuItem(text = { ProjectLabel(project,vm.projects) },
                    onClick = { projectMenu = false; vm.filter(vm.machine, project) }) }
            }
        }
        FilterChip(selected = vm.favoritesOnly, onClick = vm::toggleFavorites, label = { Text(stringResource(R.string.favorites)) })
        val agents = vm.visibleAgents()
        LazyColumn {
            if (agents.isEmpty()) item { Text(stringResource(R.string.no_agents)) }
            items(agents, key = { it.getString("agent_id") }) { agent ->
                val project=agent.optJSONObject("project")
                val chosen=vm.favorites.any { it.getString("agent_id")==agent.getString("agent_id") }
                val favoriteLabel=stringResource(if(chosen) R.string.unfavorite else R.string.favorite)
                ListItem(
                    modifier=Modifier.fillMaxWidth().clickable { vm.select(agent) },
                    headlineContent={ Text(agent.getString("display_name"),maxLines=1,overflow=TextOverflow.Ellipsis,style=MaterialTheme.typography.titleSmall) },
                    supportingContent={
                        val name=project?.getString("root")?.let(::projectTitle).orEmpty()
                        val qualifier=project?.let { value -> projectQualifier(ProjectIdentity(value.optString("host"),value.getString("root")),vm.projects.map { ProjectIdentity(it.optString("host"),it.getString("root")) }) }.orEmpty()
                        Text(listOf(name,qualifier,agent.getString("machine")).filter { it.isNotBlank() }.joinToString(" · "),maxLines=1,overflow=TextOverflow.Ellipsis,style=MaterialTheme.typography.bodySmall)
                    },
                    leadingContent={ ProjectAvatar(project) },
                    trailingContent={ IconButton(onClick={ vm.favorite(agent) },modifier=Modifier.semantics { contentDescription=favoriteLabel }) {
                        Text(if(chosen) "★" else "☆",color=if(chosen) Color(0xFFFFBB45) else MaterialTheme.colorScheme.onSurfaceVariant,style=MaterialTheme.typography.titleLarge)
                    } },
                    colors=ListItemDefaults.colors(containerColor=Color.Transparent),
                )

            }
            if (!vm.favoritesOnly && vm.nextCursor != null) item {
                TextButton(onClick = { vm.refresh(true) }, enabled = !vm.busy) { Text(stringResource(R.string.more)) }
            }
        }
    }
}

@Composable
private fun ProjectLabel(project: JSONObject, peers: List<JSONObject>) {
    val identity=ProjectIdentity(project.optString("host"),project.getString("root"))
    val qualifier=projectQualifier(identity,peers.map { ProjectIdentity(it.optString("host"),it.getString("root")) })
    Row(verticalAlignment=Alignment.CenterVertically,horizontalArrangement=Arrangement.spacedBy(8.dp)) {
        ProjectAvatar(project)
        Column {
            Text(projectTitle(identity.root),maxLines=1,overflow=TextOverflow.Ellipsis)
            if(qualifier.isNotEmpty()) Text(qualifier,style=MaterialTheme.typography.labelSmall,maxLines=1,overflow=TextOverflow.Ellipsis)
        }
    }
}

@Composable
private fun ProjectAvatar(project: JSONObject?) {
    val root=project?.optString("root").orEmpty()
    val palette=listOf(0xFF2358B7,0xFF7351B5,0xFF007C78,0xFFB14D27,0xFF9D3974,0xFF466C2A)
    val index=((project?.optString("host").orEmpty()+"\u0000"+root).hashCode().ushr(1))%palette.size
    val isBridget=projectTitle(root) in setOf("64.bridget","bridget")
    Surface(modifier=Modifier.size(38.dp),shape=RoundedCornerShape(12.dp),color=if(isBridget) Color.Transparent else Color(palette[index])) {
        Box(contentAlignment=Alignment.Center) {
            if(isBridget)
                Image(painterResource(R.drawable.bridget_logo),contentDescription=null,modifier=Modifier.size(34.dp))
            else Text(projectMonogram(root).ifBlank { "•" },color=Color.White,style=MaterialTheme.typography.labelMedium)
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun Conversation(vm: MobileViewModel, onPick: () -> Unit, onExport: (JSONObject) -> Unit) {
    val agent = vm.selected!!
    val id = agent.getString("agent_id")
    var section by rememberSaveable(id) { mutableIntStateOf(0) }
    var menu by remember { mutableStateOf(false) }
    var details by remember { mutableStateOf(false) }
    var technical by rememberSaveable(id) { mutableStateOf(false) }
    val list = rememberLazyListState()
    val scope = rememberCoroutineScope()
    val dragging by list.interactionSource.collectIsDraggedAsState()
    var followBottom by remember(id) { mutableStateOf(true) }
    var viewportHeight by remember { mutableIntStateOf(0) }
    val messages = remember(vm.events, technical) {
        val groups=mutableListOf<MutableList<JSONObject>>()
        val latest=latestActivitySequences(vm.events.mapNotNull { event ->
            activityKey(event)?.let { it to event.optLong("seq") }
        })
        for(event in vm.events) {
            if(!technical && activityKey(event)?.let { latest[it]!=event.optLong("seq") }==true) continue
            if(!technical && !hasMessageText(event)) continue
            if(!technical && isToolActivity(event) && groups.lastOrNull()?.firstOrNull()?.let(::isToolActivity)==true)
                groups.last().add(event)
            else groups.add(mutableListOf(event))
        }
        groups
    }
    val localMessages = remember(vm.outbox, id) { vm.conversationEntries(id).mapNotNull { entry ->
        val envelope = entry.optJSONObject("envelope")
        if (envelope?.optString("type") == "message" && envelope.optString("agent_id") == id)
            envelope.getString("command_id") to entry.optString("queue_body", envelope.optString("body")) else null
    }.toMap() }
    LaunchedEffect(list, dragging) {
        snapshotFlow { Triple(list.isScrollInProgress, list.firstVisibleItemIndex, list.firstVisibleItemScrollOffset) }
            .collect { (scrolling, _, _) -> if (scrolling && dragging) followBottom = !list.canScrollForward }
    }
    LaunchedEffect(id, messages.lastOrNull()?.lastOrNull()?.optLong("seq"), viewportHeight) {
        if (followBottom && messages.isNotEmpty()) list.scrollToMessageEnd(messages.lastIndex+1)
    }
    DisposableEffect(section, id) {
        vm.setConversationVisible(section == 0)
        onDispose { vm.setConversationVisible(false) }
    }
    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().heightIn(min = 56.dp), verticalAlignment = Alignment.CenterVertically) {
            val backLabel = stringResource(R.string.back)
            IconButton(onClick = { if (section == 0) vm.select(null) else section = 0 }, modifier = Modifier.semantics { contentDescription = backLabel }) {
                Text("←", fontSize = 25.sp)
            }
            Column(Modifier.weight(1f)) {
                Text(agent.getString("display_name"), style = MaterialTheme.typography.titleMedium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(agent.getString("machine"), style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            Box {
                val menuLabel = stringResource(R.string.conversation_menu)
                IconButton(onClick = { menu = true }, modifier = Modifier.semantics { contentDescription = menuLabel }) { Text("⋮", fontSize = 26.sp) }
                DropdownMenu(expanded = menu, onDismissRequest = { menu = false }) {
                    listOf(R.string.conversation, R.string.files, R.string.tasks).forEachIndexed { index, label ->
                        DropdownMenuItem(text = { Text(stringResource(label)) }, onClick = { menu = false; section = index })
                    }
                    DropdownMenuItem(text = { Text(stringResource(R.string.refresh)) }, onClick = { menu = false; vm.readEvents() })
                    DropdownMenuItem(text = { Text(stringResource(R.string.history_restart)) }, onClick = { menu = false; vm.restartHistory() })
                    DropdownMenuItem(text = { Text(stringResource(R.string.technical_events)) }, onClick = { menu = false; technical = !technical })
                    DropdownMenuItem(text = { Text(stringResource(R.string.conversation_details)) }, onClick = { menu = false; details = true })
                }
            }
        }
        when (section) {
            0 -> {
                Row(Modifier.fillMaxWidth().padding(horizontal = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                    val live = vm.followState == "live"
                    if (live && agent.optString("state") in setOf("busy", "working", "running"))
                        CircularProgressIndicator(Modifier.size(12.dp), strokeWidth = 2.dp)
                    else Text("●", color = if (live) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outline, fontSize = 10.sp)
                    Spacer(Modifier.width(7.dp))
                    Text(if (live) stateText(agent.optString("state")) else stringResource(if (vm.followState == "connecting") R.string.sync_connecting else R.string.sync_paused),
                        style = MaterialTheme.typography.labelSmall, modifier = Modifier.weight(1f))
                    if (!live && vm.followState != "connecting") TextButton(onClick = vm::resumeFollow) { Text(stringResource(R.string.reconnect), style = MaterialTheme.typography.labelSmall) }
                    if (vm.eventGap) TextButton(onClick = { details = true }) { Text("ⓘ", style = MaterialTheme.typography.labelMedium) }
                }
                Box(Modifier.weight(1f).fillMaxWidth().onSizeChanged { viewportHeight=it.height }) {
                    LazyColumn(state = list, modifier = Modifier.fillMaxSize(), contentPadding = PaddingValues(vertical = 14.dp),
                        verticalArrangement = Arrangement.spacedBy(18.dp)) {
                        item(key="older:$id") {
                            if(vm.olderBefore!=null) TextButton(onClick={followBottom=false;vm.readOlder()},enabled=!vm.busy) {
                                Text(stringResource(R.string.older_messages))
                            }
                        }
                        items(messages, key = { it.first().getLong("seq") }) { group ->
                            val event=group.first()
                            val user = event.optString("event") == "turn_start"
                            Column(Modifier.fillMaxWidth(), horizontalAlignment = if (user) Alignment.End else Alignment.Start) {
                                if (!technical && isToolActivity(event)) ToolGroup(group)
                                else if (user) Surface(shape = RoundedCornerShape(20.dp), color = MaterialTheme.colorScheme.surfaceContainerHigh,
                                    modifier = Modifier.widthIn(max = 340.dp)) {
                                    SelectionContainer { Text(if (technical) eventText(event) else conversationBody(eventText(event), true, localMessages), Modifier.padding(14.dp), style = MaterialTheme.typography.bodyLarge) }
                                } else RichMarkdown(eventText(event))
                                Text(conversationTime(event.optString("ts")), Modifier.padding(top = 4.dp, start = 4.dp, end = 4.dp),
                                    style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                    }
                    if (list.canScrollForward) FilledTonalButton(onClick = {
                        followBottom = true; scope.launch { if (messages.isNotEmpty()) list.scrollToMessageEnd(messages.lastIndex+1) }
                    }, modifier = Modifier.align(Alignment.BottomCenter).padding(bottom = 8.dp), shape = RoundedCornerShape(30.dp)) {
                        Text("↓  " + stringResource(R.string.latest_messages))
                    }
                }
                MessageQueue(vm, id)
                Surface(shape = RoundedCornerShape(26.dp), color = MaterialTheme.colorScheme.surfaceContainerHigh,
                    modifier = Modifier.fillMaxWidth().padding(bottom = 8.dp)) {
                    Row(Modifier.padding(horizontal = 4.dp, vertical = 2.dp), verticalAlignment = Alignment.Bottom) {
                        val attachLabel = stringResource(R.string.attach)
                        IconButton(onClick = onPick, enabled = !vm.busy, modifier = Modifier.semantics { contentDescription = attachLabel }) { Text("＋", fontSize = 26.sp) }
                        TextField(value = vm.draft, onValueChange = vm::editDraft, enabled = !vm.busy, modifier = Modifier.weight(1f),
                            placeholder = { Text(stringResource(R.string.message_placeholder)) }, maxLines = 5,
                            colors = TextFieldDefaults.colors(focusedContainerColor = Color.Transparent, unfocusedContainerColor = Color.Transparent,
                                disabledContainerColor = Color.Transparent, focusedIndicatorColor = Color.Transparent, unfocusedIndicatorColor = Color.Transparent))
                        val sendLabel = stringResource(R.string.send)
                        FilledIconButton(onClick = { followBottom = true; vm.send() }, enabled = !vm.busy && vm.draft.isNotBlank(),
                            modifier = Modifier.semantics { contentDescription = sendLabel }) { Text("↑", fontSize = 23.sp) }
                    }
                }
            }
            1 -> FilePanel(vm, onPick, onExport)
            else -> TaskPanel(vm) { section = 0 }
        }
    }
    if (details) AlertDialog(onDismissRequest = { details = false }, title = { Text(stringResource(R.string.conversation_details)) },
        text = { Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(stringResource(R.string.explicit_recipient))
            agent.optJSONObject("project")?.let { project -> SelectionContainer { Text(project.optString("host")+"\n"+project.optString("root"),style=MaterialTheme.typography.bodySmall) } }
            if(agent.optBoolean("cached_observation")) Text(stringResource(R.string.cached_observation),style=MaterialTheme.typography.bodySmall)
            Text(stringResource(R.string.immediate_delivery_detail))
            Text(stringResource(if (vm.eventGap) R.string.history_gap else R.string.follow_connected))
            Text(vm.followState, style = MaterialTheme.typography.labelSmall)
        } }, confirmButton = { TextButton(onClick = { details = false }) { Text(stringResource(R.string.dismiss)) } })
}

private suspend fun LazyListState.scrollToMessageEnd(index: Int) {
    scrollToItem(index)
    withFrameNanos { }
    val last=layoutInfo.visibleItemsInfo.lastOrNull { it.index==index } ?: return
    val overflow=last.offset+last.size-layoutInfo.viewportEndOffset+layoutInfo.afterContentPadding
    if(overflow>0) scrollBy(overflow.toFloat())
}

private fun activityKey(event: JSONObject): String? {
    if(!isToolActivity(event)) return null
    val payload=event.optJSONObject("payload") ?: return null
    val id=payload.optString("tool_id").takeIf { it.isNotBlank() && it!="null" } ?: return null
    return event.optString("session_id")+":"+id
}

private fun isToolActivity(event: JSONObject): Boolean = event.optJSONObject("payload")?.optString("kind") in setOf("tool","command","file","file_write","write")

@Composable
private fun ToolGroup(events: List<JSONObject>) {
    var expanded by remember(events.first().optLong("seq")) { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth()) {
        TextButton(onClick={expanded=!expanded},contentPadding=PaddingValues(horizontal=8.dp,vertical=2.dp)) {
            Text((if(expanded) "▾  " else "▸  ")+stringResource(R.string.activity_group),
                style=MaterialTheme.typography.labelSmall,color=MaterialTheme.colorScheme.onSurfaceVariant)
        }
        if(expanded) Column(Modifier.padding(start=12.dp)) {
            events.forEach { ToolActivity(it) }
        }
    }
}

@Composable
private fun ToolActivity(event: JSONObject) {
    var expanded by remember(event.optLong("seq")) { mutableStateOf(false) }
    val p=event.optJSONObject("payload") ?: return
    val category=when(p.optString("tool_type",p.optString("kind"))) {
        "command_execution","command" -> R.string.activity_command
        "web_search" -> R.string.activity_web
        "file_search" -> R.string.activity_search
        "file","file_write","write" -> R.string.activity_file
        else -> R.string.activity_tool
    }
    val status=when(p.optString("status")) {
        "completed" -> R.string.activity_done
        "failed" -> R.string.activity_failed
        "cancelled" -> R.string.activity_cancelled
        "pending","running" -> R.string.activity_running
        else -> R.string.activity_seen
    }
    val input=listOf("input","command","cmd","arguments").firstNotNullOfOrNull { key -> p.opt(key)?.takeIf { it!=JSONObject.NULL }?.toString()?.takeIf { it.isNotBlank() } }
    val output=listOf("output","result","stdout").firstNotNullOfOrNull { key -> p.opt(key)?.takeIf { it!=JSONObject.NULL }?.toString()?.takeIf { it.isNotBlank() } }
    val technicalNames=setOf("command_execution","dynamic_tool","mcp_tool_call","web_search","file_search","image_view","tool")
    val name=p.optString("tool").takeIf { it.isNotBlank() && it!="null" && it !in technicalNames }
        ?: input?.lineSequence()?.firstOrNull()?.take(90)
    Column(Modifier.fillMaxWidth()) {
        TextButton(onClick={expanded=!expanded}, contentPadding=PaddingValues(horizontal=8.dp,vertical=2.dp)) {
            Text((if(expanded) "▾  " else "▸  ")+(name ?: stringResource(category))+" · "+stringResource(status),
                style=MaterialTheme.typography.labelSmall,color=MaterialTheme.colorScheme.onSurfaceVariant,maxLines=2,overflow=TextOverflow.Ellipsis)
        }
        if(expanded) Column(Modifier.padding(horizontal=8.dp),verticalArrangement=Arrangement.spacedBy(6.dp)) {
            if(input!=null) {
                Text(stringResource(R.string.tool_input),style=MaterialTheme.typography.labelSmall,color=MaterialTheme.colorScheme.onSurfaceVariant)
                HighlightedCode(input,if(category==R.string.activity_command) "shell" else if(input.trimStart().firstOrNull() in setOf('{','[')) "json" else null)
            }
            if(output!=null) {
                Text(stringResource(R.string.tool_output),style=MaterialTheme.typography.labelSmall,color=MaterialTheme.colorScheme.onSurfaceVariant)
                HighlightedCode(output,if(output.trimStart().firstOrNull() in setOf('{','[')) "json" else null)
            }
            if(p.has("exit_code") && !p.isNull("exit_code")) Text(stringResource(R.string.tool_exit,p.optInt("exit_code")),style=MaterialTheme.typography.labelSmall)
            if(p.optBoolean("details_truncated")) Text(stringResource(R.string.tool_details_partial),style=MaterialTheme.typography.labelSmall)
            if((input==null && output==null) || p.optBoolean("details_unavailable") || p.optBoolean("output_omitted"))
                Text(stringResource(R.string.tool_details_unavailable),style=MaterialTheme.typography.labelSmall,color=MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

private fun hasMessageText(event: JSONObject): Boolean {
    val payload = event.optJSONObject("payload") ?: return false
    if (isToolActivity(event)) return true
    return listOf("body", "text", "content", "message").any { key ->
        val value = payload.opt(key)
        value is String && value.isNotBlank() || value is JSONObject && value.optString("text").isNotBlank()
    }
}

@Composable
private fun FilePanel(vm: MobileViewModel, onPick: () -> Unit, onExport: (JSONObject) -> Unit) {
    LaunchedEffect(vm.selected?.optString("agent_id")) {
        snapshotFlow { vm.busy }.first { !it }
        vm.files()
    }
    vm.previewContent?.let { text ->
        if (isHtmlPreview(vm.previewTitle, vm.previewMediaType)) {
            HtmlPreview(text, vm::closePreview)
        } else {
            androidx.compose.ui.window.Dialog(
                onDismissRequest = vm::closePreview,
                properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false),
            ) {
                Surface(Modifier.fillMaxSize()) {
                    Column(Modifier.fillMaxSize().statusBarsPadding().navigationBarsPadding()
                        .padding(16.dp).verticalScroll(rememberScrollState())) {
                        TextButton(onClick = vm::closePreview) { Text(stringResource(R.string.dismiss)) }
                        if (isMarkdownPreview(vm.previewTitle, vm.previewMediaType)) RichMarkdown(text)
                        else HighlightedCode(text, vm.previewTitle.substringAfterLast('.'))
                    }
                }
            }
        }
    }
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Text(stringResource(R.string.file_limits), style = MaterialTheme.typography.bodySmall)
        Row {
            Button(onClick = onPick, enabled = !vm.busy) { Text(stringResource(if (vm.uploadPending) R.string.resume_file_selection else R.string.attach)) }
            TextButton(onClick = { vm.files() }, enabled = !vm.busy) { Text(stringResource(R.string.refresh)) }
        }
        LazyColumn(verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(vm.attachments, key = { it.getString("version_ref") }) { file ->
                Card(Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Text(file.getString("title"), style = MaterialTheme.typography.titleMedium)
                        if (file.has("code")) Text(stringResource(R.string.file_unavailable))
                        else {
                            Text(file.getString("media_type") + " · " + file.getLong("byte_length") + " " + stringResource(R.string.bytes))
                            OutlinedButton(onClick = { onExport(file) }, enabled = !vm.busy) { Text(stringResource(R.string.save_file)) }
                            if(isTextPreview(file.getString("title"),file.getString("media_type"))) TextButton(onClick={vm.previewFile(file)},enabled=!vm.busy) { Text(stringResource(R.string.preview_file)) }
                        }
                    }
                }
            }
            if (vm.attachmentCursor != null) item { TextButton(onClick = { vm.files(true) }, enabled = !vm.busy) { Text(stringResource(R.string.more)) } }
        }
    }
}

@Composable
private fun TaskPanel(vm: MobileViewModel, onInstruction: () -> Unit) {
    var cancel by remember { mutableStateOf<JSONObject?>(null) }
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        TextButton(onClick = { vm.readTasks() }, enabled = !vm.busy) { Text(stringResource(R.string.refresh_tasks)) }
        LazyColumn(verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(vm.tasks, key = { it.getString("task_id") }) { task ->
                Card(onClick = { vm.showTask(task.getString("task_id")) }, modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text(task.getString("title"), style = MaterialTheme.typography.titleMedium)
                        Text(stateText(task.getString("status")))
                    }
                }
            }
            if (vm.taskCursor != null) item { TextButton(onClick = { vm.readTasks(true) }, enabled = !vm.busy) { Text(stringResource(R.string.more)) } }
            vm.taskDetail?.let { task -> item {
                Card(Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                        Text(task.getString("title"), style = MaterialTheme.typography.titleMedium)
                        Text(task.getString("task_id"), style = MaterialTheme.typography.labelSmall)
                        Text(stateText(task.getString("status")))
                        task.optString("error").takeIf { it.isNotBlank() && it != "null" }?.let { Text(it, color = MaterialTheme.colorScheme.error) }
                        SelectionContainer { Text(vm.taskResult.ifEmpty { stringResource(R.string.no_result) }) }
                        if (vm.resultOffset != null) TextButton(onClick = { vm.showTask(task.getString("task_id"), true) }, enabled = !vm.busy) { Text(stringResource(R.string.more_result)) }
                        val actions = task.getJSONObject("actions")
                        if (actions.optBoolean("request_pause")) OutlinedButton(onClick = { vm.taskInstruction(task, true); onInstruction() }, enabled = !vm.busy) { Text(stringResource(R.string.request_pause)) }
                        if (actions.optBoolean("request_continue")) OutlinedButton(onClick = { vm.taskInstruction(task, false); onInstruction() }, enabled = !vm.busy) { Text(stringResource(R.string.request_continue)) }
                        if (actions.optBoolean("cancel_native")) OutlinedButton(onClick = { cancel = task }, enabled = !vm.busy) { Text(stringResource(R.string.cancel_task)) }
                        Text(stringResource(R.string.task_action_semantics), style = MaterialTheme.typography.bodySmall)
                    }
                }
            } }
        }
    }
    cancel?.let { task -> Confirm(stringResource(R.string.cancel_task), stringResource(R.string.cancel_task_detail),
        onDismiss = { cancel = null }) { cancel = null; vm.cancelTask(task) } }
}

@Composable
private fun Questions(vm: MobileViewModel) {
    var answer by remember { mutableStateOf<Pair<JSONObject, String>?>(null) }
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Text(stringResource(R.string.questions), style = MaterialTheme.typography.headlineSmall)
        Text(stringResource(R.string.question_semantics), style = MaterialTheme.typography.bodySmall)
        TextButton(onClick = { vm.readQuestions() }, enabled = !vm.busy) { Text(stringResource(R.string.refresh)) }
        LazyColumn(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            items(vm.questions, key = { it.getString("item_id") }) { item ->
                Card(Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                        Text(item.getString("summary"), style = MaterialTheme.typography.titleMedium)
                        item.optJSONObject("facts")?.let { facts ->
                            Text(stringResource(R.string.request_facts), style = MaterialTheme.typography.labelMedium)
                            SelectionContainer { Text(facts.toString(2), fontFamily = FontFamily.Monospace, style = MaterialTheme.typography.bodySmall) }
                        }
                        item.optString("quoted_untrusted").takeIf { it.isNotBlank() && it != "null" }?.let { text ->
                            Text(stringResource(R.string.untrusted_text), style = MaterialTheme.typography.labelMedium)
                            SelectionContainer { Text(text) }
                        }
                        val choices = item.getJSONArray("options")
                        if (item.optBoolean("can_respond")) for (i in 0 until choices.length()) {
                            val choice = choices.getString(i)
                            OutlinedButton(onClick = { answer = item to choice }, enabled = !vm.busy) { Text(choiceText(choice)) }
                        }
                    }
                }
            }
            if (vm.questionCursor != null) item { TextButton(onClick = { vm.readQuestions(true) }, enabled = !vm.busy) { Text(stringResource(R.string.more)) } }
        }
    }
    answer?.let { (item, choice) -> Confirm(stringResource(R.string.confirm_choice),
        item.getString("summary") + "\n\n" + choiceText(choice), onDismiss = { answer = null }) {
        answer = null; vm.answerQuestion(item, choice)
    } }
}

@Composable
private fun Outbox(vm: MobileViewModel) {
    var replay by remember { mutableStateOf<JSONObject?>(null) }
    var discard by remember { mutableStateOf<JSONObject?>(null) }
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Text(stringResource(R.string.outbox), style = MaterialTheme.typography.headlineSmall)
        Text(stringResource(R.string.outbox_detail), style = MaterialTheme.typography.bodySmall)
        LazyColumn(verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(vm.outbox, key = { it.getJSONObject("envelope").getString("command_id") }) { entry ->
                val envelope = entry.getJSONObject("envelope")
                val type = envelope.getString("type")
                val state = entry.getString("state")
                Card(Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Text(stringResource(when (type) {
                            "file" -> R.string.files
                            "task_cancel" -> R.string.cancel_task
                            "question_respond" -> R.string.questions
                            else -> R.string.instruction
                        }), style = MaterialTheme.typography.labelMedium)
                        SelectionContainer { Text(envelope.optString("body", envelope.optString("file_name",
                            envelope.optString("task_id", envelope.optString("choice")))), maxLines = 5) }
                        val observation = entry.optJSONObject("delivery_observation")?.optString("event")
                        Text(when (observation) {
                            "turn_end" -> stringResource(R.string.response_finished)
                            "turn_start" -> stringResource(R.string.delivered_to_agent)
                            else -> commandState(state)
                        }, style = MaterialTheme.typography.titleSmall)
                        Text(entry.getString("endpoint"), style = MaterialTheme.typography.labelSmall)
                        val receipt = entry.optJSONObject("receipt")?.optJSONObject("receipt")
                        receipt?.optJSONObject("application")?.let { application ->
                            Text(stringResource(when {
                                application.opt("confirmed") == true -> R.string.application_confirmed
                                application.optString("state") == "not_applied" -> R.string.application_not_applied
                                application.optBoolean("producer_acknowledged") -> R.string.producer_acknowledged
                                else -> R.string.application_unconfirmed
                            }), style = MaterialTheme.typography.bodySmall)
                        }
                        Text(envelope.getString("command_id"), style = MaterialTheme.typography.labelSmall)
                        if (state == "local") TextButton(onClick = { vm.sendStored(entry) }, enabled = !vm.busy) { Text(stringResource(R.string.send)) }
                        else TextButton(onClick = { vm.reconcile(entry) }, enabled = !vm.busy) { Text(stringResource(R.string.reconcile)) }
                        if (state == "outcome_unknown") TextButton(onClick = { replay = entry }, enabled = !vm.busy) { Text(stringResource(R.string.strict_replay)) }
                        if (type == "file" && state == "accepted") TextButton(onClick = { vm.attachReference(entry) }, enabled = !vm.busy) { Text(stringResource(R.string.insert_file_reference)) }
                        if (state in setOf("local", "accepted", "rejected", "idempotency_expired", "orphaned")) TextButton(onClick = { discard = entry }, enabled = !vm.busy) { Text(stringResource(R.string.remove_local)) }
                    }
                }
            }
        }
    }
    replay?.let { entry -> Confirm(stringResource(R.string.strict_replay), stringResource(R.string.strict_replay_detail),
        onDismiss = { replay = null }) { replay = null; vm.sendStored(entry, strictReplay = true) } }
    discard?.let { entry -> Confirm(stringResource(R.string.remove_local), stringResource(R.string.remove_local_detail),
        onDismiss = { discard = null }) { discard = null; vm.discard(entry) } }
}

@Composable
private fun Settings(vm: MobileViewModel, onRevoke: () -> Unit, onForget: () -> Unit, onNotifications: () -> Unit) {
    LaunchedEffect(vm.associated) {
        while (true) { vm.refreshUpdateStatus(); kotlinx.coroutines.delay(30_000) }
    }
    var revokeDevice by remember { mutableStateOf<JSONObject?>(null) }
    val quietLabel = stringResource(R.string.quiet_hours)
    var quiet by remember(vm.quietSettings) { mutableStateOf(vm.quietSettings.optBoolean("enabled")) }
    var quietStart by remember(vm.quietSettings) { mutableStateOf(vm.quietSettings.optString("start", "22:00")) }
    var quietEnd by remember(vm.quietSettings) { mutableStateOf(vm.quietSettings.optString("end", "08:00")) }
    LazyColumn(verticalArrangement = Arrangement.spacedBy(12.dp)) {
        item {
            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(stringResource(R.string.notifications_title), style = MaterialTheme.typography.titleMedium)
                    Text(stringResource(R.string.notifications_explanation), style = MaterialTheme.typography.bodySmall)
                    Text(stringResource(if (vm.notificationsPending) R.string.notifications_pending else if (vm.notificationsEnabled) R.string.notifications_enabled else R.string.notifications_disabled))
                    OutlinedButton(onClick = onNotifications, enabled = !vm.busy) {
                        Text(stringResource(if (vm.notificationsEnabled) R.string.notifications_disable else R.string.notifications_enable))
                    }
                    Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
                        Switch(checked = quiet, onCheckedChange = { quiet = it }, enabled = !vm.busy,
                            modifier = Modifier.semantics { contentDescription = quietLabel })
                        Text(stringResource(R.string.quiet_hours))
                    }
                    if (quiet) {
                        OutlinedTextField(value = quietStart, onValueChange = { if (it.length <= 5) quietStart = it },
                            label = { Text(stringResource(R.string.quiet_start)) }, singleLine = true)
                        OutlinedTextField(value = quietEnd, onValueChange = { if (it.length <= 5) quietEnd = it },
                            label = { Text(stringResource(R.string.quiet_end)) }, singleLine = true)
                        Text(stringResource(R.string.quiet_explanation), style = MaterialTheme.typography.bodySmall)
                    }
                    TextButton(onClick = { vm.setQuiet(quiet, quietStart, quietEnd) }, enabled = !vm.busy) {
                        Text(stringResource(R.string.quiet_save))
                    }
                }
            }
        }
        item {
            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(stringResource(R.string.app_version, BuildConfig.VERSION_NAME), style = MaterialTheme.typography.titleMedium)
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Switch(checked = vm.updateState.optBoolean("automatic", true),
                            onCheckedChange = vm::changeAutomaticUpdates, enabled = !vm.busy,
                            modifier = Modifier.semantics { contentDescription = "auto-update-android" })
                        Text(stringResource(R.string.automatic_updates), Modifier.weight(1f))
                    }
                    Text(stringResource(R.string.automatic_updates_detail), style = MaterialTheme.typography.bodySmall)
                    Text(stringResource(R.string.android_updates_title), style = MaterialTheme.typography.titleSmall)
                    Text(updateStage(vm.updateState.optString("stage")))
                    Text(stringResource(R.string.host_updates_title), style = MaterialTheme.typography.titleSmall)
                    Text(stringResource(if (vm.hostUpdatesPending) R.string.host_updates_pending
                        else if (!vm.hostUpdates.has("automatic")) R.string.host_updates_unknown
                        else if (vm.hostUpdates.optBoolean("automatic")) R.string.host_updates_on else R.string.host_updates_off))
                    vm.hostUpdates.optJSONObject("hosts")?.let { hosts ->
                        hosts.keys().asSequence().sorted().forEach { host ->
                            val item = hosts.getJSONObject(host)
                            Text(host + " : " + item.optString("installed") + " · " + hostUpdateStage(item.optString("stage")), style = MaterialTheme.typography.bodySmall)
                            Text(stringResource(R.string.update_last_check, dateText(item.optLong("observed_at"))), style = MaterialTheme.typography.bodySmall)
                        }
                    }
                    if (vm.updateState.has("checked_at")) Text(stringResource(R.string.update_last_check,
                        dateText(vm.updateState.optLong("checked_at") / 1000)), style = MaterialTheme.typography.bodySmall)
                    if (vm.updateState.has("error")) Text(stringResource(errorText(vm.updateState.optString("error"))),
                        color = MaterialTheme.colorScheme.error)
                    TextButton(onClick = vm::checkForUpdates, enabled = !vm.busy &&
                        vm.updateState.optString("stage") !in setOf("checking", "downloading", "installing")) {
                        Text(stringResource(R.string.check_updates))
                    }
                    val hosts = vm.hostUpdates.optJSONObject("hosts")
                    if (hosts != null && hosts.keys().asSequence().any { host ->
                        val item = hosts.getJSONObject(host)
                        item.optString("available").let { it.isNotEmpty() && it != "null" && it != item.optString("installed") }
                    }) OutlinedButton(onClick = vm::installHostUpdates, enabled = !vm.busy && !vm.hostUpdatesPending) {
                        Text(stringResource(R.string.host_update_install_action))
                    }
                    vm.release?.let { release ->
                        Text(stringResource(R.string.update_available, release.version))
                        OutlinedButton(onClick = vm::installUpdate, enabled = !vm.busy &&
                            vm.updateState.optString("stage") !in setOf("checking", "downloading", "installing")) {
                            Text(stringResource(R.string.install_update))
                        }
                    }
                }
            }
        }
        item { Text(stringResource(R.string.settings), style = MaterialTheme.typography.headlineSmall) }
        item { Text(vm.endpoint) }
        item { OutlinedButton(onClick = vm::renew, enabled = !vm.busy) { Text(stringResource(R.string.renew)) } }
        item { OutlinedButton(onClick = onRevoke, enabled = !vm.busy) { Text(stringResource(R.string.revoke)) } }
        item { TextButton(onClick = onForget, enabled = !vm.busy) { Text(stringResource(R.string.forget_authorization)) } }
        item { TextButton(onClick = { vm.readDevices() }, enabled = !vm.busy) { Text(stringResource(R.string.list_devices)) } }
        items(vm.devices, key = { it.getString("device_id") }) { device ->
            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text(device.getString("label"), style = MaterialTheme.typography.titleMedium)
                    Text(device.getString("device_id"), style = MaterialTheme.typography.labelSmall)
                    Text(stringResource(R.string.device_created, dateText(device.getLong("created_at"))), style = MaterialTheme.typography.bodySmall)
                    Text(stringResource(R.string.device_expiry, dateText(device.getLong("expires_at"))), style = MaterialTheme.typography.bodySmall)
                    Text(stringResource(if (device.isNull("revoked_at")) R.string.device_active else R.string.device_revoked))
                    if (device.isNull("revoked_at")) TextButton(onClick = { revokeDevice = device }, enabled = !vm.busy) { Text(stringResource(R.string.revoke_device)) }
                }
            }
        }
        if (vm.deviceCursor != null) item { TextButton(onClick = { vm.readDevices(true) }, enabled = !vm.busy) { Text(stringResource(R.string.more)) } }
        item { Text(stringResource(R.string.new_agents_later), style = MaterialTheme.typography.bodySmall) }
    }
    revokeDevice?.let { device -> Confirm(stringResource(R.string.revoke_device), device.getString("label"),
        onDismiss = { revokeDevice = null }) { revokeDevice = null; vm.revokeDevice(device.getString("device_id")) } }
}

@Composable
private fun Confirm(title: String, message: String, onDismiss: () -> Unit, onConfirm: () -> Unit) {
    AlertDialog(onDismissRequest = onDismiss, title = { Text(title) },
        text = { Text(message, Modifier.verticalScroll(rememberScrollState())) },
        confirmButton = { TextButton(onClick = onConfirm) { Text(stringResource(R.string.confirm)) } },
        dismissButton = { TextButton(onClick = onDismiss) { Text(stringResource(R.string.cancel)) } })
}
private fun eventText(event: JSONObject): String {
    val payload = event.optJSONObject("payload") ?: return event.optString("event")
    for (name in listOf("body", "text", "content", "message")) {
        val value = payload.opt(name)
        if (value is String && value.isNotBlank()) return value
        if (value is JSONObject && value.optString("text").isNotBlank()) return value.getString("text")
    }
    payload.optString("path").takeIf { it.isNotBlank() }?.let { return it }
    return event.optString("event") + payload.optString("stop_reason").let { if (it.isBlank()) "" else " · $it" }
}
@Composable private fun choiceText(value: String): String = when (value) {
    "accept" -> stringResource(R.string.allow_once)
    "decline" -> stringResource(R.string.decline)
    else -> value
}
@Composable private fun commandState(value: String): String = stringResource(when (value) {
    "local" -> R.string.saved_locally
    "accepted" -> R.string.received_by_bridget
    "rejected" -> R.string.command_rejected
    "idempotency_expired" -> R.string.receipt_expired
    "envelope_mismatch" -> R.string.command_conflict
    "orphaned" -> R.string.destination_unavailable
    else -> R.string.command_unconfirmed
})
@Composable private fun stateText(value: String): String = when (value) {
    "connected", "active", "ready", "idle" -> stringResource(R.string.state_available)
    "stale", "disconnected", "offline" -> stringResource(R.string.state_unavailable)
    "working", "running", "busy" -> stringResource(R.string.state_working)
    "observed_agent" -> stringResource(R.string.state_observed)
    "unknown" -> stringResource(R.string.state_unknown)
    "waiting_for_children" -> stringResource(R.string.state_waiting_children)
    "result_available" -> stringResource(R.string.state_result)
    "cancelled" -> stringResource(R.string.state_cancelled)
    "cancelling" -> stringResource(R.string.state_cancelling)
    "failed" -> stringResource(R.string.state_failed)
    "queued", "mission_pending", "starting" -> stringResource(R.string.state_pending)
    else -> value
}
private fun recoveryText(code: String): Int = when (code) {
    "recipient_unavailable" -> R.string.recipient_recovery
    "pairing_unavailable", "pairing_missing" -> R.string.pairing_recovery
    "pairing_pending" -> R.string.pending_pairing
    "command_origin_changed" -> R.string.command_origin_recovery
    "authorization_origin_changed" -> R.string.authorization_origin_recovery
    "history_scope_unavailable", "forbidden_scope" -> R.string.scope_recovery
    "invalid_file", "payload_too_large" -> R.string.file_limits
    "file_export_incomplete" -> R.string.file_export_recovery
    "resync_required" -> R.string.resync_recovery
    else -> R.string.error_recovery
}
private fun errorText(code: String): Int = when (code) {
    "update_hosts_unavailable" -> R.string.update_hosts_unavailable
    "update_in_progress" -> R.string.update_server_in_progress
    "not_held" -> R.string.queue_already_sent
    "queue_changed" -> R.string.queue_changed
    "no_active_turn", "steer_unavailable" -> R.string.queue_interrupt_unavailable
    "recipient_unavailable" -> R.string.recipient_unavailable
    "pairing_unavailable", "pairing_missing" -> R.string.pairing_unavailable
    "pairing_pending" -> R.string.pending_pairing
    "update_check_unavailable" -> R.string.update_check_unavailable
    "update_integrity_failed" -> R.string.update_integrity_failed
    "update_install_failed" -> R.string.update_install_failed
    "update_schedule_unavailable", "update_storage_unavailable" -> R.string.update_storage_unavailable
    "update_disabled" -> R.string.update_disabled
    "android_release_unpublished" -> R.string.android_release_unpublished
    "invalid_quiet_period" -> R.string.quiet_invalid
    "notifications_not_configured" -> R.string.notifications_unconfigured
    "notification_permission_denied" -> R.string.notifications_permission_denied
    "notification_registration_unconfirmed" -> R.string.notifications_unconfirmed
    "authorization_required", "device_revoked", "credential_expired" -> R.string.auth_error
    "forbidden_scope", "history_scope_unavailable" -> R.string.scope_error
    "capability_unavailable" -> R.string.capability_error
    "busy" -> R.string.busy_error
    "invalid_qr" -> R.string.qr_error
    "command_origin_changed" -> R.string.command_origin_error
    "authorization_origin_changed" -> R.string.authorization_origin_error
    "device_storage_unavailable", "device_storage_locked", "device_storage_invalid", "device_storage_limit" -> R.string.storage_error
    "preview_unavailable" -> R.string.preview_unavailable
    "invalid_file", "payload_too_large", "file_export_incomplete" -> R.string.file_unavailable
    "choice_not_offered" -> R.string.choice_not_offered
    "question_already_resolved" -> R.string.question_already_resolved
    "publication_capacity_reached" -> R.string.file_quota
    "file_selection_pending" -> R.string.resume_file_selection
    "resource_limit" -> R.string.resource_limit
    "invalid_request" -> R.string.request_invalid
    "envelope_mismatch" -> R.string.command_conflict
    "idempotency_expired", "invalid_issued_at" -> R.string.receipt_expired
    "endpoint_unavailable" -> R.string.master_unavailable
    "journal_unavailable" -> R.string.history_unavailable
    "not_found" -> R.string.destination_unavailable
    "resync_required" -> R.string.history_gap
    else -> R.string.network_error
}

private fun dateText(seconds: Long): String = java.text.DateFormat.getDateTimeInstance().format(java.util.Date(seconds * 1000))

@Composable private fun updateStage(value: String): String = stringResource(when (value) {
    "checking" -> R.string.update_checking
    "available" -> R.string.update_available_stage
    "downloading" -> R.string.update_downloading
    "ready" -> R.string.update_ready_title
    "waiting_idle" -> R.string.update_waiting_idle
    "permission_required" -> R.string.update_permission_detail
    "confirmation_required" -> R.string.update_confirmation_detail
    "installing" -> R.string.update_installing
    "installed", "up_to_date" -> R.string.app_up_to_date
    "error" -> R.string.update_error_stage
    else -> R.string.update_not_checked
})

@Composable private fun hostUpdateStage(stage: String): String = stringResource(when(stage) {
    "active" -> R.string.host_update_active
    "error" -> R.string.update_error_stage
    "up_to_date" -> R.string.app_up_to_date
    "disabled" -> R.string.host_updates_off
    "rolled_back" -> R.string.host_update_rollback
    "waiting_idle" -> R.string.host_update_waiting
    "ready" -> R.string.update_ready_title
    "available" -> R.string.update_available_stage
    "downloading" -> R.string.update_downloading
    "installing" -> R.string.host_update_installing
    else -> R.string.host_updates_unknown
})
