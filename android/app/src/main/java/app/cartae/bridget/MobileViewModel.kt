package app.cartae.bridget

import android.app.Application
import android.net.Uri
import android.provider.OpenableColumns
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject

class MobileViewModel(app: Application) : AndroidViewModel(app) {
    private val store = LocalStore(app)
    private val client = BridgetClient(store)
    private var followJob: Job? = null
    private var receiptJob: Job? = null
    private var tailPending = true
    var olderBefore by mutableStateOf<Long?>(null); private set
    private var visible = false
    private var conversationVisible = false
    var release by mutableStateOf<MobileRelease?>(null); private set
    var updateChecked by mutableStateOf(false); private set
    var hostUpdates by mutableStateOf(JSONObject()); private set
    var hostUpdatesPending by mutableStateOf(false); private set
    var updateState by mutableStateOf(JSONObject()); private set
    var notificationsPending by mutableStateOf(false); private set
    var notificationsEnabled by mutableStateOf(false); private set
    var quietSettings by mutableStateOf(JSONObject()); private set
    var associated by mutableStateOf(false); private set
    var pairingPending by mutableStateOf(false); private set
    var busy by mutableStateOf(false); private set
    var error by mutableStateOf<String?>(null); private set
    var notice by mutableStateOf<String?>(null); private set
    var endpoint by mutableStateOf(""); private set
    var machine by mutableStateOf(""); private set
    var project by mutableStateOf<JSONObject?>(null); private set
    var machines by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var projects by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var agents by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var nextCursor by mutableStateOf<String?>(null); private set
    var favorites by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var favoritesOnly by mutableStateOf(false); private set
    var selected by mutableStateOf<JSONObject?>(null); private set
    var draft by mutableStateOf(""); private set
    var outbox by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var fresh by mutableStateOf(false); private set
    var events by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var eventGap by mutableStateOf(false); private set
    var eventNextSeq by mutableStateOf(0L); private set
    var followState by mutableStateOf("stopped"); private set
    var attachments by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var attachmentCursor by mutableStateOf<String?>(null); private set
    var transferProgress by mutableStateOf<Float?>(null); private set
    var uploadPending by mutableStateOf(false); private set
    var exportPending by mutableStateOf(false); private set
    var previewContent by mutableStateOf<String?>(null); private set
    var previewTitle by mutableStateOf(""); private set
    var previewMediaType by mutableStateOf(""); private set
    var tasks by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var taskCursor by mutableStateOf<String?>(null); private set
    var taskDetail by mutableStateOf<JSONObject?>(null); private set
    var taskResult by mutableStateOf(""); private set
    var resultOffset by mutableStateOf<Int?>(null); private set
    var questions by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var questionCursor by mutableStateOf<String?>(null); private set
    var devices by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var deviceCursor by mutableStateOf<String?>(null); private set

    init {
        viewModelScope.launch {
            MobileUpdates.changes.collect {
                updateState = MobileUpdates.state(getApplication())
                hostUpdates = store.get("host-update-status") ?: JSONObject()
                hostUpdatesPending = store.get("host-update-choice") != null
                release = updateState.optJSONObject("available")?.let(MobileRelease::stored)
                updateChecked = updateState.optString("stage") == "up_to_date"
            }
        }
        act {
            val session = store.get("session")
            associated = session != null
            endpoint = session?.getString("endpoint") ?: ""
            if (associated) {
                loadPreferences()
                runCatching { MobileUpdates.syncHosts(getApplication()) }
            }
        }
    }
    private fun act(work: suspend () -> Unit) {
        if (busy) return
        busy = true
        MobileUpdates.operations.incrementAndGet()
        viewModelScope.launch {
            error = null
            notice = null
            try { work() }
            catch (e: CancellationException) { throw e }
            catch (e: MobileFailure) { error = e.code; fresh = false }
            catch (e: Exception) {
                error = e.message?.takeIf { it in setOf("invalid_file", "payload_too_large", "device_storage_limit",
                    "device_storage_locked", "device_storage_invalid") } ?: "device_storage_unavailable"
                fresh = false
            }
            finally {
                pairingPending = runCatching { store.get("pairing") != null }.getOrDefault(false)
                outbox = runCatching { store.list("command:") }.getOrDefault(outbox)
                uploadPending = runCatching { store.get("upload-picker") != null }.getOrDefault(false)
                exportPending = runCatching { store.get("file-export") != null }.getOrDefault(false)
                notificationsEnabled = MobileNotifications.enabled(store)
                notificationsPending = MobileNotifications.pending(store)
                quietSettings = MobileNotifications.quietSettings(store)
                busy = false
                MobileUpdates.operations.decrementAndGet()
            }
        }
    }
    private fun preferences() = store.get(store.preferencesKey()) ?: JSONObject()
    private fun updatePreferences(change: (JSONObject) -> Unit) {
        val value = preferences()
        change(value)
        store.put(store.preferencesKey(), value)
    }
    private fun loadPreferences() {
        val value = preferences()
        machine = value.optString("machine")
        project = value.optJSONObject("project")
        favoritesOnly = value.optBoolean("favorites_only")
        favorites = value.optJSONArray("favorites")?.let(::objects).orEmpty()
        selected = value.optJSONObject("selected")
        selected?.let { restoreConversation(it.getString("agent_id")) }
    }
    fun checkForUpdates() {
        MobileUpdates.launch(getApplication(), manual = true)
        act { if (associated) { MobileUpdates.syncHosts(getApplication()); hostUpdates = client.requestUpdates(false) } }
    }
    fun changeAutomaticUpdates(enabled: Boolean) = act {
        MobileUpdates.setAutomatic(getApplication(), enabled)
        hostUpdatesPending = store.get("host-update-choice") != null
        if (associated) { hostUpdates = MobileUpdates.syncHosts(getApplication()); hostUpdatesPending = false }
    }
    fun installHostUpdates() = act {
        val hosts = MobileUpdates.syncHosts(getApplication()).getJSONObject("hosts")
        val target = hosts.keys().asSequence().mapNotNull { hosts.getJSONObject(it).optString("available").takeIf { v -> v.isNotEmpty() && v != "null" } }
            .maxByOrNull(MobileRelease::versionCode) ?: return@act
        hostUpdates = client.requestUpdates(true, target)
    }
    suspend fun refreshUpdateStatus() {
        if (associated && !busy) {
            try { hostUpdates = MobileUpdates.syncHosts(getApplication()) }
            catch (e: CancellationException) { throw e }
            catch (_: Exception) { /* Le dernier état reste daté ; aucun succès inventé. */ }
        }
    }
    fun installUpdate() { MobileUpdates.confirm(getApplication()) }
    fun changeNotifications(enabled: Boolean) = act {
        MobileNotifications.setEnabled(getApplication(), store, enabled)
        notificationsEnabled = MobileNotifications.enabled(store)
    }
    fun setQuiet(enabled: Boolean, start: String, end: String) = act {
        MobileNotifications.setQuiet(store, enabled, start, end)
    }
    fun notificationPermissionDenied() { error = "notification_permission_denied" }
    fun invalidQr() { error = "invalid_qr" }
    fun clearError() { error = null; notice = null }
    fun associate(qr: String? = null) = act {
        client.pair(qr)
        associated = true
        endpoint = store.get("session")!!.getString("endpoint")
        loadPreferences()
    }
    fun setVisible(value: Boolean) {
        visible = value
        notificationsEnabled = MobileNotifications.enabled(store)
        notificationsPending = MobileNotifications.pending(store)
        if (!value) { followJob?.cancel(); followJob = null; followState = "stopped" }
        else startFollow()
    }
    fun setConversationVisible(value: Boolean) {
        conversationVisible = value
        if (!value) { followJob?.cancel(); followJob = null; followState = "stopped" }
        else startFollow()
    }
    fun networkAvailable() {
        if (!visible || !associated) return
        if (selected != null) startFollow() else if (!fresh && !busy) refresh()
    }
    fun refresh(next: Boolean = false) = act {
        fresh = false
        if (!next) {
            machines = allPages("/v1/federation")
            val filter = if (machine.isNotEmpty()) "&machine=" + BridgetClient.encode(machine) else ""
            projects = allPages("/v1/projects?limit=50$filter")
        }
        val filter = buildString {
            if (machine.isNotEmpty()) append("&machine=" + BridgetClient.encode(machine))
            project?.let {
                append("&project_host=" + BridgetClient.encode(it.getString("host")))
                append("&project_root=" + BridgetClient.encode(it.getString("root")))
            }
        }
        val afterSuffix = if (next && nextCursor != null) "&after=" + BridgetClient.encode(nextCursor!!) else ""
        val page = client.read("/v1/agents?limit=50$filter$afterSuffix")
        val items = objects(page.getJSONArray("items"))
        val combined = if (next) (agents + items).distinctBy { it.getString("agent_id") } else items
        if (combined.size > 1000) throw MobileFailure("resource_limit")
        agents = combined
        nextCursor = cursor(page)
        fresh = true
    }
    private suspend fun allPages(path: String): List<JSONObject> {
        val items = mutableListOf<JSONObject>()
        val visited = mutableSetOf<String>()
        var after: String? = null
        do {
            val suffix = after?.let { (if (path.contains('?')) "&" else "?") + "after=" + BridgetClient.encode(it) } ?: ""
            val page = client.read(path + suffix)
            items += objects(page.getJSONArray("items"))
            after = cursor(page)
            if (items.size > 256 || (after != null && !visited.add(after))) throw MobileFailure("resource_limit")
        } while (after != null)
        return items
    }
    fun filter(value: String, selectedProject: JSONObject? = null) {
        if (busy) return
        try {
            updatePreferences { it.put("machine", value).put("project", selectedProject ?: JSONObject.NULL) }
            machine = value; project = selectedProject
            agents = emptyList(); nextCursor = null; fresh = false
            refresh()
        } catch (_: Exception) { error = "device_storage_unavailable" }
    }
    fun favorite(agent: JSONObject) {
        if (busy) return
        try {
            val id = agent.getString("agent_id")
            val changed = if (favorites.any { it.getString("agent_id") == id }) favorites.filterNot { it.getString("agent_id") == id }
                else favorites + JSONObject(agent.toString())
            if (changed.size > 50) throw MobileFailure("resource_limit")
            updatePreferences { it.put("favorites", JSONArray(changed)) }
            favorites = changed
        } catch (e: MobileFailure) { error = e.code }
        catch (_: Exception) { error = "device_storage_limit" }
    }
    fun toggleFavorites() {
        try {
            updatePreferences { it.put("favorites_only", !favoritesOnly) }
            favoritesOnly = !favoritesOnly
        } catch (_: Exception) { error = "device_storage_unavailable" }
    }
    fun visibleAgents(): List<JSONObject> {
        if (!favoritesOnly) return agents
        return favorites.filter { favorite ->
            (machine.isEmpty() || favorite.optString("machine") == machine)
                && (project == null || favorite.optJSONObject("project")?.let {
                    it.optString("host") == project!!.optString("host") && it.optString("root") == project!!.optString("root")
                } == true)
        }.map { favorite ->
            agents.find { it.getString("agent_id") == favorite.getString("agent_id") }
                ?: JSONObject(favorite.toString()).put("cached_observation", true)
        }
    }
    fun select(agent: JSONObject?) {
        if (busy) return
        followJob?.cancel(); followJob = null
        try {
            updatePreferences { it.put("selected", agent ?: JSONObject.NULL) }
            previewContent=null; previewTitle=""; previewMediaType=""
            selected = agent
            tailPending = true; olderBefore = null
            attachments = emptyList(); attachmentCursor = null; tasks = emptyList(); taskCursor = null
            taskDetail = null; taskResult = ""; resultOffset = null
            events = emptyList(); eventGap = false; eventNextSeq = 0L; draft = ""
            agent?.let { restoreConversation(it.getString("agent_id")) }
            startFollow()
        } catch (_: Exception) { error = "device_storage_unavailable" }
    }
    private fun restoreConversation(id: String) {
        draft = store.get(store.draftKey(id))?.optString("body").orEmpty()
        store.get(store.historyKey(id))?.let { saved ->
            events = saved.optJSONArray("events")?.let(::objects).orEmpty()
            eventNextSeq = saved.optLong("next_seq")
            eventGap = saved.optBoolean("gap")
        }
    }
    fun editDraft(value: String) {
        if (value.toByteArray(Charsets.UTF_8).size > 10_000) return
        val id = selected?.getString("agent_id") ?: return
        try {
            store.put(store.draftKey(id), JSONObject().put("body", value))
            draft = value
        } catch (_: Exception) { error = "device_storage_unavailable" }
    }
    fun send(guideNow: Boolean = false) = act {
        val id = selected?.getString("agent_id") ?: return@act
        val entry = store.enqueue(id, draft, guideNow)
        store.remove(store.draftKey(id)); draft = ""
        try { client.send(entry) } finally { refreshPendingReceipt(id) }
        applyEvents(id, client.read("/v1/agents/" + BridgetClient.encode(id) + "/events?after_seq=" + eventNextSeq))
        resumeFollow()
    }
    fun reconcile(entry: JSONObject) = act { client.reconcile(entry) }
    fun sendStored(entry: JSONObject, strictReplay: Boolean = false) = act {
        check(entry.getString("state") == "local" || strictReplay)
        client.send(entry)
    }
    fun discard(entry: JSONObject) = act { store.discardCommand(entry.getJSONObject("envelope").getString("command_id")) }
    fun readEvents() = act {
        val id = selected?.getString("agent_id") ?: return@act
        val page = client.read("/v1/agents/" + BridgetClient.encode(id) + "/events?after_seq=" + eventNextSeq)
        applyEvents(id, page)
        resumeFollow()
    }
    fun resumeFollow() {
        followJob?.cancel(); followJob = null
        startFollow()
    }
    fun restartHistory() {
        followJob?.cancel(); followJob = null
        events = emptyList(); eventGap = false; eventNextSeq = 0
        tailPending = true; olderBefore = null
        startFollow()
    }
    fun readOlder() = act {
        val id=selected?.getString("agent_id") ?: return@act
        val before=olderBefore ?: return@act
        val start=(before-51).coerceAtLeast(0)
        val page=client.read("/v1/agents/$id/events?after_seq=$start&limit=50")
        if (selected?.optString("agent_id") != id) return@act
        val old=objects(page.getJSONArray("items")).filter { it.getLong("seq") < before }
        val merged=(events+old).distinctBy { it.getLong("seq") }
        if (merged.size>500 || merged.sumOf { it.toString().toByteArray(Charsets.UTF_8).size }>256*1024) {
            olderBefore=null; notice="history_window_limit"; return@act
        }
        applyEvents(id,JSONObject(page.toString()).put("items",JSONArray(old)).put("next_seq",eventNextSeq))
        olderBefore=if(start==0L) null else start+1
    }
    private fun applyEvents(id: String, page: JSONObject) {
        if (selected?.optString("agent_id") != id) return
        page.optString("agent_state").takeIf { it.isNotBlank() }?.let { state ->
            selected = JSONObject(selected!!.toString()).put("state", state)
        }
        val next = page.getLong("next_seq")
        if (next < eventNextSeq) throw MobileFailure("resync_required")
        val incoming = objects(page.getJSONArray("items"))
        if (incoming.any { it.optLong("seq") <= 0 || it.optLong("seq") > next }) throw MobileFailure("resync_required")
        var merged = (events + incoming).distinctBy { it.getLong("seq") }.sortedBy { it.getLong("seq") }
        var bytes = merged.sumOf { it.toString().toByteArray(Charsets.UTF_8).size }
        while (merged.size > 500 || bytes > 256 * 1024) {
            bytes -= merged.first().toString().toByteArray(Charsets.UTF_8).size
            merged = merged.drop(1); eventGap = true
        }
        for (event in incoming) store.observeMessage(id, event)
        outbox = store.list("command:")
        events = merged; eventNextSeq = next
        eventGap = eventGap || !page.isNull("gap")
        val retained = mutableListOf<JSONObject>()
        bytes = 0
        for (event in events.asReversed().take(100)) {
            val count = event.toString().toByteArray(Charsets.UTF_8).size
            if (bytes + count > 48 * 1024) break
            bytes += count; retained.add(0, event)
        }
        store.put(store.historyKey(id), JSONObject().put("events", JSONArray(retained)).put("next_seq", next)
            .put("gap", eventGap || retained.size != events.size))
    }
    private fun refreshPendingReceipt(id: String) {
        receiptJob?.cancel()
        receiptJob=viewModelScope.launch {
            repeat(5) { attempt ->
                if(attempt>0) delay(2000L shl (attempt-1))
                if(!visible || selected?.optString("agent_id")!=id) return@launch
                val entry=runCatching { store.list("command:") }.getOrElse { return@launch }.filter {
                    val e=it.optJSONObject("envelope")
                    e?.optString("type")=="message" && e.optString("agent_id")==id
                        && it.optString("state")=="outcome_unknown"
                        && runCatching { BridgetClient.requireCommandOrigin(it,store.get("session")!!);true }.getOrDefault(false)
                }.maxByOrNull { it.getJSONObject("envelope").optLong("issued_at") } ?: return@launch
                try { client.reconcile(entry); outbox=store.list("command:") }
                catch(e: CancellationException) { throw e }
                catch(_: MobileFailure) { /* Saved command and journal observation stay intact. */ }
                catch(_: Exception) { return@launch }
            }
        }
    }
    private fun startFollow() {
        if (!visible || !conversationVisible || !associated || followJob?.isActive == true) return
        val id = selected?.optString("agent_id") ?: return
        val origin = runCatching { store.draftKey(id) }.getOrNull() ?: return
        refreshPendingReceipt(id)
        followState = "connecting"
        followJob = viewModelScope.launch {
            var after: String? = null
            var failures = 0
            while (isActive && visible && conversationVisible && selected?.optString("agent_id") == id) {
                try {
                    if(failures>0) followState = "connecting"
                    if (tailPending) {
                        val recent=client.read("/v1/agents/$id/events?tail=true&limit=50")
                        if (store.draftKey(id)!=origin || selected?.optString("agent_id")!=id) break
                        events=emptyList(); eventNextSeq=0; eventGap=false
                        applyEvents(id,recent)
                        olderBefore=recent.optLong("older_before_seq").takeIf { it>1 }
                        tailPending=false; after=null; followState="live"
                    }
                    val suffix = after?.let { "&after=" + BridgetClient.encode(it) } ?: "&after_seq=" + eventNextSeq
                    val page = client.read("/v1/events?agent_id=$id&limit=50$suffix")
                    if (store.draftKey(id) != origin || selected?.optString("agent_id") != id) break
                    val before = eventNextSeq
                    applyEvents(id, page)
                    after = page.optString("next_cursor").takeIf { it.isNotEmpty() && it != "null" }
                    failures = 0
                    followState = "live"
                    if (page.getJSONArray("items").length() == 0 && page.isNull("gap")
                        && !page.optBoolean("idle") && eventNextSeq == before) { followState = "stopped"; break }
                    delay(250)
                } catch (e: CancellationException) { throw e }
                catch (e: MobileFailure) {
                    followState = e.code
                    if (e.code !in setOf("outcome_unknown", "endpoint_unavailable", "journal_unavailable", "busy", "not_found") || ++failures >= 7) break
                    delay((1000L shl (failures - 1)).coerceAtMost(15_000))
                } catch (_: Exception) { followState = "device_storage_unavailable"; break }
            }
        }
    }
    fun files(next: Boolean = false) = act {
        val id = selected?.getString("agent_id") ?: return@act
        val after = if (next) attachmentCursor?.let { "&after=" + BridgetClient.encode(it) }.orEmpty() else ""
        val page = client.read("/v1/agents/$id/artifacts?limit=50$after")
        attachments = bounded(if (next) attachments + objects(page.getJSONArray("items")) else objects(page.getJSONArray("items")))
        attachmentCursor = cursor(page)
    }
    fun beginFileSelection(): Boolean {
        if (busy) return false
        return try {
            val agentId = selected?.getString("agent_id") ?: return false
            val pending = store.get("upload-picker")
            if (pending != null) {
                BridgetClient.requireCommandOrigin(pending, store.get("session") ?: throw MobileFailure("authorization_required"))
                if (pending.getJSONObject("envelope").getString("agent_id") != agentId) throw MobileFailure("file_selection_pending")
            } else {
                val session = store.get("session") ?: throw MobileFailure("authorization_required")
                store.put("upload-picker", JSONObject().put("endpoint", session.getString("endpoint"))
                    .put("envelope", JSONObject().put("device_id", session.getJSONObject("device").getString("device_id"))
                        .put("command_id", java.util.UUID.randomUUID().toString()).put("agent_id", agentId)))
            }
            uploadPending = true
            true
        } catch (e: MobileFailure) { error = e.code; false }
        catch (_: Exception) { error = "device_storage_unavailable"; false }
    }
    fun importFile(uri: Uri?) {
        if (uri == null) {
            runCatching { store.remove("upload-picker") }
            uploadPending = false
            return
        }
        act {
            val pending = store.get("upload-picker") ?: throw MobileFailure("file_selection_pending")
            BridgetClient.requireCommandOrigin(pending, store.get("session") ?: throw MobileFailure("authorization_required"))
            pending.put("uri", uri.toString()); store.put("upload-picker", pending)
            val command = pending.getJSONObject("envelope")
            val agentId = command.getString("agent_id")
            val id = command.getString("command_id")
            val resolver = getApplication<Application>().contentResolver
            val entry = store.get("command:$id") ?: withContext(Dispatchers.IO) {
                val name = resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE), null, null, null)?.use {
                    check(it.moveToFirst()) { "invalid_file" }
                    val size = it.getColumnIndex(OpenableColumns.SIZE)
                    if (size >= 0 && !it.isNull(size) && it.getLong(size) > LocalStore.MAX_FILE_BYTES) throw MobileFailure("payload_too_large")
                    it.getString(it.getColumnIndexOrThrow(OpenableColumns.DISPLAY_NAME))
                } ?: throw MobileFailure("invalid_file")
                val mime = resolver.getType(uri) ?: throw MobileFailure("invalid_file")
                resolver.openInputStream(uri)?.use { store.enqueueFile(agentId, name, mime, it, id) }
                    ?: throw MobileFailure("invalid_file")
            }
            // Le checkpoint d'interface est clos avant tout réseau ; l'enveloppe est déjà durable.
            store.remove("upload-picker")
            if (entry.getString("state") == "local") client.send(entry) else client.reconcile(entry)
            attachReference(store.get("command:$id")!!)
        }
    }
    fun attachReference(entry: JSONObject) {
        try { BridgetClient.requireCommandOrigin(entry, store.get("session") ?: throw MobileFailure("authorization_required")) }
        catch (e: MobileFailure) { error = e.code; return }
        val envelope = entry.getJSONObject("envelope")
        val receipt = entry.optJSONObject("receipt")?.optJSONObject("receipt") ?: return
        if (envelope.optString("type") != "file" || selected?.optString("agent_id") != envelope.optString("agent_id")) return
        val text = ("\n[Fichier Bridget : " + envelope.getString("file_name") + "; artifact_ref="
            + receipt.getString("artifact_ref") + "; version_ref=" + receipt.getString("version_ref")
            + "; sha256=" + envelope.getString("expected_digest") + "]")
        editDraft(draft + text)
    }
    fun closePreview() { previewContent=null; previewTitle=""; previewMediaType="" }
    fun previewFile(metadata: JSONObject) = act {
        val id=selected?.getString("agent_id") ?: return@act
        val title=metadata.getString("title")
        if(!isTextPreview(title,metadata.getString("media_type")) || metadata.getLong("byte_length") !in 1..262_144) throw MobileFailure("preview_unavailable")
        val bytes=client.download(id,metadata)
        if(selected?.optString("agent_id")!=id) throw MobileFailure("command_origin_changed")
        val decoder=Charsets.UTF_8.newDecoder().onMalformedInput(java.nio.charset.CodingErrorAction.REPORT)
        val text=withContext(Dispatchers.Default) { decoder.decode(java.nio.ByteBuffer.wrap(bytes)).toString() }
        previewTitle=title;previewMediaType=metadata.getString("media_type");previewContent=text
    }
    fun beginExport(metadata: JSONObject): String? {
        if (busy) return null
        return try {
            val agentId = selected?.getString("agent_id") ?: return null
            val session = store.get("session") ?: throw MobileFailure("authorization_required")
            store.put("file-export", JSONObject().put("endpoint", session.getString("endpoint"))
                .put("envelope", JSONObject().put("device_id", session.getJSONObject("device").getString("device_id")))
                .put("agent_id", agentId).put("metadata", metadata))
            exportPending = true
            metadata.getString("title").substringAfterLast('/').substringAfterLast('\\').take(240).ifBlank { "bridget-file" }
        } catch (_: Exception) { error = "device_storage_unavailable"; null }
    }
    fun exportFile(uri: Uri?) {
        if (uri == null) {
            runCatching { store.remove("file-export") }; exportPending = false; return
        }
        act {
            val pending = store.get("file-export") ?: throw MobileFailure("file_export_incomplete")
            BridgetClient.requireCommandOrigin(pending, store.get("session") ?: throw MobileFailure("authorization_required"))
            transferProgress = 0f
            try {
                val bytes = client.download(pending.getString("agent_id"), pending.getJSONObject("metadata")) { transferProgress = it }
                withContext(Dispatchers.IO) {
                    getApplication<Application>().contentResolver.openOutputStream(uri, "wt")?.use { it.write(bytes) }
                        ?: throw MobileFailure("file_export_incomplete")
                }
                store.remove("file-export"); notice = "file_saved"
            } finally { transferProgress = null }
        }
    }
    fun readTasks(next: Boolean = false) = act {
        val id = selected?.getString("agent_id") ?: return@act
        val after = if (next) taskCursor?.let { "&after=" + BridgetClient.encode(it) }.orEmpty() else ""
        val page = client.read("/v1/tasks?agent_id=$id&limit=50$after")
        tasks = bounded(if (next) tasks + objects(page.getJSONArray("tasks")) else objects(page.getJSONArray("tasks")))
        taskCursor = cursor(page)
    }
    fun showTask(id: String, more: Boolean = false) = act {
        val offset = if (more) resultOffset ?: return@act else 0
        val page = client.read("/v1/tasks/$id?offset=$offset")
        taskDetail = page.getJSONObject("task")
        taskResult = (if (more) taskResult else "") + if (page.isNull("result")) "" else page.getString("result")
        if (taskResult.toByteArray(Charsets.UTF_8).size > 256 * 1024) throw MobileFailure("resource_limit")
        resultOffset = if (page.isNull("result_next_offset")) null else page.getInt("result_next_offset")
    }
    fun cancelTask(task: JSONObject) = act {
        val id = task.getString("task_id")
        val prior = store.list("command:").firstOrNull { entry ->
            val envelope = entry.getJSONObject("envelope")
            envelope.optString("type") == "task_cancel" && envelope.optString("task_id") == id
                && entry.optString("state") in setOf("local", "outcome_unknown")
                && runCatching { BridgetClient.requireCommandOrigin(entry, store.get("session")!!); true }.getOrDefault(false)
        }
        val entry = prior ?: store.enqueueAction("task_cancel", id)
        if (entry.getString("state") == "local") client.send(entry) else client.reconcile(entry)
    }
    fun taskInstruction(task: JSONObject, pause: Boolean) {
        val parent = task.getString("parent_agent_id")
        val known = agents.find { it.getString("agent_id") == parent }
        if (known == null) { error = "not_found"; return }
        select(known)
        editDraft(getApplication<Application>().getString(
            if (pause) R.string.pause_instruction else R.string.continue_instruction, task.getString("task_id")))
        notice = "instruction_draft"
    }
    fun readQuestions(next: Boolean = false) = act {
        val after = if (next) questionCursor?.let { "&after=" + BridgetClient.encode(it) }.orEmpty() else ""
        val page = client.read("/v1/questions?limit=50$after")
        questions = bounded(if (next) questions + objects(page.getJSONArray("items")) else objects(page.getJSONArray("items")))
        questionCursor = cursor(page)
    }
    fun answerQuestion(item: JSONObject, choice: String) = act {
        val id = item.getString("item_id")
        val prior = store.list("command:").firstOrNull { entry ->
            val envelope = entry.getJSONObject("envelope")
            envelope.optString("type") == "question_respond" && envelope.optString("item_id") == id
                && runCatching { BridgetClient.requireCommandOrigin(entry, store.get("session")!!); true }.getOrDefault(false)
        }
        if (prior != null && prior.getJSONObject("envelope").getString("choice") != choice) throw MobileFailure("envelope_mismatch")
        val entry = prior ?: store.enqueueAction("question_respond", id, choice)
        if (entry.getString("state") == "local") client.send(entry) else client.reconcile(entry)
        val page = client.read("/v1/questions?limit=50")
        questions = objects(page.getJSONArray("items")); questionCursor = cursor(page)
    }
    fun readDevices(next: Boolean = false) = act {
        val after = if (next) deviceCursor?.let { "?after=" + BridgetClient.encode(it) }.orEmpty() else ""
        val page = client.read("/v1/devices$after")
        devices = bounded(if (next) devices + objects(page.getJSONArray("items")) else objects(page.getJSONArray("items")))
        deviceCursor = cursor(page)
    }
    fun forgetAuthorization() = act {
        followJob?.cancel(); followJob = null
        store.forgetAuthorization()
        associated = false; endpoint = ""; agents = emptyList(); selected = null
        questions = emptyList(); devices = emptyList(); events = emptyList(); draft = ""
        favorites = emptyList(); projects = emptyList(); machines = emptyList(); fresh = false
    }
    fun revokeDevice(id: String) = act {
        val own = store.get("session")!!.getJSONObject("device").getString("device_id") == id
        client.revokeDevice(id)
        if (own) {
            followJob?.cancel(); followJob = null
            associated = false; endpoint = ""; selected = null; agents = emptyList(); events = emptyList(); fresh = false
        } else {
            val page = client.read("/v1/devices")
            devices = objects(page.getJSONArray("items")); deviceCursor = cursor(page)
        }
    }
    fun renew() = act { client.renew() }
    fun revoke() = act {
        followJob?.cancel(); followJob = null
        client.revokeSelf(); associated = false; endpoint = ""; agents = emptyList(); selected = null
        questions = emptyList(); devices = emptyList(); events = emptyList(); draft = ""; fresh = false
    }
    override fun onCleared() { followJob?.cancel(); receiptJob?.cancel(); store.close() }
    private fun objects(array: JSONArray): List<JSONObject> = (0 until array.length()).map { array.getJSONObject(it) }
    private fun bounded(items: List<JSONObject>): List<JSONObject> {
        if (items.size > 1000) throw MobileFailure("resource_limit")
        return items
    }
    private fun cursor(page: JSONObject): String? = if (page.isNull("next_cursor")) null else page.getString("next_cursor")
}
