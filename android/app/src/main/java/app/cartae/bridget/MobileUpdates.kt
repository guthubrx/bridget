package app.cartae.bridget

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.job.JobInfo
import android.app.job.JobParameters
import android.app.job.JobScheduler
import android.app.job.JobService
import android.content.BroadcastReceiver
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageInstaller
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import org.json.JSONObject
import java.io.File
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

/** Un seul propriétaire par processus ; aucun message métier n'est rejoué. */
object MobileUpdates {
    const val PERIODIC_JOB = 17901
    private const val READY_JOB = 17902
    private const val NOTIFICATION = 179
    private const val CHANNEL = "bridget_app_updates"
    private const val INTERVAL = 6 * 60 * 60 * 1000L
    private val running = AtomicBoolean(false)
    private val foreground = AtomicInteger(0)
    val operations = AtomicInteger(0)
    val changes = MutableStateFlow(0L)
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    @Volatile private var confirmation: Intent? = null

    @Synchronized fun state(context: Context): JSONObject = LocalStore(context).use {
        it.get("application-updates") ?: JSONObject().put("automatic", true).put("stage", "not_checked")
    }
    @Synchronized private fun save(context: Context, change: (JSONObject) -> Unit) {
        LocalStore(context).use { store ->
            val value = store.get("application-updates") ?: JSONObject().put("automatic", true).put("stage", "not_checked")
            change(value); store.put("application-updates", value)
        }
        changes.value += 1
    }
    fun automatic(context: Context): Boolean = state(context).optBoolean("automatic", true)
    @Synchronized fun setAutomatic(context: Context, enabled: Boolean) {
        save(context) { it.put("automatic", enabled) }
        LocalStore(context).use { store -> store.get("session")?.let { session ->
            store.put("host-update-choice", JSONObject().put("automatic", enabled).put("endpoint", session.getString("endpoint")))
        } }
        val scheduler = context.getSystemService(JobScheduler::class.java)
        if (enabled) { schedule(context); launch(context, manual = false) }
        else {
            scheduler.cancel(PERIODIC_JOB); scheduler.cancel(READY_JOB)
            val id = state(context).optInt("session_id", -1)
            if (id >= 0 && state(context).optBoolean("session_automatic")) {
                if (runCatching { context.packageManager.packageInstaller.abandonSession(id) }.isSuccess) {
                    save(context) { it.remove("session_id"); it.put("stage", "ready") }
                } // Une session déjà engagée garde son identifiant jusqu'au reçu Android.
            }
            context.getSystemService(NotificationManager::class.java).cancel(NOTIFICATION)
            retryHostChoice(context)
        }
    }
    private val hostLock = Mutex()
    suspend fun syncHosts(context: Context): JSONObject = withContext(Dispatchers.IO) { hostLock.withLock {
        LocalStore(context).use { store ->
            val session = store.get("session") ?: throw MobileFailure("authorization_required")
            val pending = synchronized(MobileUpdates) { store.get("host-update-choice") }
            val client = BridgetClient(store)
            val value = try { if (pending != null) {
                if (pending.optString("endpoint") != session.getString("endpoint")) throw MobileFailure("authorization_required")
                val result = client.configureUpdates(pending.getBoolean("automatic"))
                synchronized(MobileUpdates) {
                    if (store.get("host-update-choice")?.toString() == pending.toString()) store.remove("host-update-choice")
                }
                result
            } else client.read("/v1/updates") }
            catch (e: MobileFailure) {
                if (e.code == "not_found") throw MobileFailure("update_hosts_unavailable")
                throw e
            }
            store.put("host-update-status", value)
            if (store.get("host-update-choice") == null && automatic(context) != value.getBoolean("automatic")) {
                save(context) { it.put("automatic", value.getBoolean("automatic")) }; schedule(context)
            }
            changes.value += 1; value
        }
    }
    }
    fun hostChoicePending(context: Context): Boolean = LocalStore(context).use { it.get("host-update-choice") != null }
    fun retryHostChoice(context: Context) {
        if (!hostChoicePending(context)) return
        val job = JobInfo.Builder(READY_JOB, ComponentName(context, UpdateJob::class.java))
            .setRequiredNetworkType(JobInfo.NETWORK_TYPE_ANY).setMinimumLatency(60_000)
            .setBackoffCriteria(60_000, JobInfo.BACKOFF_POLICY_EXPONENTIAL).build()
        context.getSystemService(JobScheduler::class.java).schedule(job)
    }
    fun schedule(context: Context) {
        val scheduler = context.getSystemService(JobScheduler::class.java)
        if (!automatic(context)) { scheduler.cancel(PERIODIC_JOB); scheduler.cancel(READY_JOB); retryHostChoice(context); return }
        if (scheduler.getPendingJob(PERIODIC_JOB) == null) {
            val job = JobInfo.Builder(PERIODIC_JOB, ComponentName(context, UpdateJob::class.java))
                .setRequiredNetworkType(JobInfo.NETWORK_TYPE_ANY).setPersisted(true).setPeriodic(INTERVAL).build()
            if (scheduler.schedule(job) != JobScheduler.RESULT_SUCCESS)
                save(context) { it.put("stage", "error").put("error", "update_schedule_unavailable") }
        }
    }
    fun foreground(context: Context, visible: Boolean) {
        if (visible) {
            foreground.incrementAndGet(); schedule(context); launch(context, manual = false)
        } else {
            foreground.updateAndGet { (it - 1).coerceAtLeast(0) }
            if (automatic(context) && state(context).has("available")) {
                val job = JobInfo.Builder(READY_JOB, ComponentName(context, UpdateJob::class.java))
                    .setRequiredNetworkType(JobInfo.NETWORK_TYPE_ANY).setMinimumLatency(5_000).build()
                context.getSystemService(JobScheduler::class.java).schedule(job)
            }
        }
    }
    fun launch(context: Context, manual: Boolean, install: Boolean = false) {
        scope.launch { run(context.applicationContext, manual, install) }
    }
    suspend fun run(context: Context, manual: Boolean, install: Boolean = false) = withContext(Dispatchers.IO) {
        if (!running.compareAndSet(false, true)) return@withContext
        try {
            if (LocalStore(context).use { it.get("session") != null }) {
                try { syncHosts(context) }
                catch (e: CancellationException) { throw e }
                catch (_: Exception) { /* Le choix local reste en attente et sera retenté. */ }
            }
            var value = state(context)
            if (!manual && !value.optBoolean("automatic", true)) return@withContext
            reconcile(context); value = state(context)
            if (value.has("session_id")) return@withContext
            val due = System.currentTimeMillis() - value.optLong("attempted_at") >= INTERVAL
            if (manual && !install || due || !value.has("attempted_at")) {
                save(context) { it.put("stage", "checking").put("attempted_at", System.currentTimeMillis()); it.remove("error") }
                val release = MobileRelease.check(BuildConfig.VERSION_NAME)
                save(context) {
                    it.put("checked_at", System.currentTimeMillis())
                    if (release != null) it.put("available", release.json()).put("stage", "available")
                    else { it.remove("available"); it.put("stage", "up_to_date") }
                }
            }
            val candidate = state(context).optJSONObject("available")?.let(MobileRelease::stored) ?: return@withContext
            if (MobileRelease.versionCode(candidate.version) <= BuildConfig.VERSION_CODE) {
                save(context) { it.remove("available"); it.put("stage", "up_to_date") }; return@withContext
            }
            if (manual && !install || !manual && !automatic(context)) return@withContext
            if (!manual && state(context).optString("failed_version") == candidate.version) return@withContext
            val directory = File(context.filesDir, "app-update").apply { mkdirs() }
            val apk = File(directory, "candidate.apk")
            if (!apk.isFile || apk.length() != candidate.size || digest(apk) != candidate.sha256) {
                save(context) { it.put("stage", "downloading") }
                candidate.download(apk) { manual || automatic(context) }
            }
            verifyApk(context, apk, candidate); save(context) { it.put("stage", "ready") }
            if (!manual && !automatic(context)) return@withContext
            if (operations.get() > 0 || !manual && foreground.get() > 0) {
                save(context) { it.put("stage", "waiting_idle") }; return@withContext
            }
            if (!context.packageManager.canRequestPackageInstalls()) {
                save(context) { it.put("stage", "permission_required") }
                notify(context, Intent(context, MainActivity::class.java), R.string.update_permission_detail)
                return@withContext
            }
            commit(context, apk, candidate, !manual)
        } catch (e: CancellationException) { throw e }
        catch (e: Exception) {
            val code = (e as? MobileFailure)?.code ?: "update_check_unavailable"
            save(context) { it.put("stage", if (code == "update_disabled") "ready" else "error").put("error", code) }
        } finally { running.set(false) }
    }
    private fun digest(file: File): String {
        val hash = java.security.MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input ->
            val bytes = ByteArray(32_768)
            while (true) { val count = input.read(bytes); if (count < 0) break; hash.update(bytes, 0, count) }
        }
        return hash.digest().joinToString("") { "%02x".format(it.toInt() and 255) }
    }
    internal fun verifyApk(context: Context, apk: File, release: MobileRelease) {
        if (apk.length() != release.size || digest(apk) != release.sha256) throw MobileFailure("update_integrity_failed")
        @Suppress("DEPRECATION")
        val candidate = context.packageManager.getPackageArchiveInfo(apk.absolutePath, PackageManager.GET_SIGNING_CERTIFICATES)
            ?: throw MobileFailure("update_integrity_failed")
        @Suppress("DEPRECATION")
        val current = context.packageManager.getPackageInfo(context.packageName, PackageManager.GET_SIGNING_CERTIFICATES)
        val signers = current.signingInfo?.apkContentsSigners.orEmpty()
        val history = candidate.signingInfo?.signingCertificateHistory.orEmpty()
        val sameSigners = signers.isNotEmpty() && signers.toSet() == candidate.signingInfo?.apkContentsSigners.orEmpty().toSet()
        val validRotation = signers.size == 1 && history.contains(signers.single())
        if (candidate.packageName != context.packageName ||
            candidate.longVersionCode != MobileRelease.versionCode(release.version).toLong() ||
            candidate.longVersionCode <= current.longVersionCode || candidate.versionName != release.version ||
            !(sameSigners || validRotation)) throw MobileFailure("update_integrity_failed")
        // Android vérifie cryptographiquement la signature et sa rotation au commit.
    }
    @Synchronized private fun commit(context: Context, apk: File, release: MobileRelease, automatic: Boolean) {
        if (automatic && !automatic(context)) return
        if (operations.get() > 0 || automatic && foreground.get() > 0) {
            save(context) { it.put("stage", "waiting_idle") }; return
        }
        val installer = context.packageManager.packageInstaller
        val params = PackageInstaller.SessionParams(PackageInstaller.SessionParams.MODE_FULL_INSTALL).apply {
            setAppPackageName(context.packageName); setSize(release.size)
            if (Build.VERSION.SDK_INT >= 31) setRequireUserAction(
                if (automatic) PackageInstaller.SessionParams.USER_ACTION_NOT_REQUIRED else PackageInstaller.SessionParams.USER_ACTION_REQUIRED)
        }
        val id = installer.createSession(params)
        save(context) { it.put("session_id", id).put("session_automatic", automatic).put("stage", "installing") }
        try {
            installer.openSession(id).use { session ->
                session.openWrite("base.apk", 0, release.size).use { output ->
                    apk.inputStream().use { it.copyTo(output) }; session.fsync(output)
                }
                if (automatic && !automatic(context)) {
                    session.abandon(); save(context) { it.remove("session_id"); it.put("stage", "ready") }; return
                }
                if (operations.get() > 0 || automatic && foreground.get() > 0) {
                    session.abandon(); save(context) { it.remove("session_id"); it.put("stage", "waiting_idle") }; return
                }
                val intent = Intent(context, UpdateResultReceiver::class.java).setAction("app.cartae.bridget.UPDATE_RESULT")
                val callback = PendingIntent.getBroadcast(context, id, intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE)
                session.commit(callback.intentSender)
            }
        } catch (e: Exception) {
            runCatching { installer.abandonSession(id) }; save(context) { it.remove("session_id") }; throw e
        }
    }
    fun receive(context: Context, intent: Intent) {
        if (intent.action != "app.cartae.bridget.UPDATE_RESULT") return
        val id = intent.getIntExtra(PackageInstaller.EXTRA_SESSION_ID, -1)
        val current = state(context)
        if (id < 0 || current.optInt("session_id", -2) != id) return
        when (intent.getIntExtra(PackageInstaller.EXTRA_STATUS, PackageInstaller.STATUS_FAILURE)) {
            PackageInstaller.STATUS_PENDING_USER_ACTION -> {
                @Suppress("DEPRECATION")
                val action = intent.getParcelableExtra<Intent>(Intent.EXTRA_INTENT) ?: return
                if (current.optBoolean("session_automatic") && !automatic(context)) {
                    if (runCatching { context.packageManager.packageInstaller.abandonSession(id) }.isSuccess)
                        save(context) { it.remove("session_id"); it.put("stage", "ready") }
                    return
                }
                confirmation = action; save(context) { it.put("stage", "confirmation_required") }
                notify(context, action, R.string.update_confirmation_detail)
                if (foreground.get() > 0) runCatching { context.startActivity(action.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)) }
            }
            PackageInstaller.STATUS_SUCCESS -> {
                save(context) { it.remove("session_id"); it.put("stage", "installed"); it.remove("available"); it.remove("error") }
                File(context.filesDir, "app-update/candidate.apk").delete(); confirmation = null
                context.getSystemService(NotificationManager::class.java).cancel(NOTIFICATION)
            }
            else -> {
                save(context) {
                    it.remove("session_id"); it.put("stage", "error").put("error", "update_install_failed")
                    it.optJSONObject("available")?.let { release -> it.put("failed_version", release.optString("version")) }
                }
                confirmation = null
            }
        }
    }
    fun confirm(context: Context) {
        if (!context.packageManager.canRequestPackageInstalls()) {
            context.startActivity(Intent(android.provider.Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES,
                android.net.Uri.parse("package:" + context.packageName)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)); return
        }
        confirmation?.let { context.startActivity(Intent(it).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)); return }
        // Après perte du processus, une confirmation perdue n'est pas un succès.
        val id = state(context).optInt("session_id", -1)
        if (id >= 0 && state(context).optString("stage") == "confirmation_required") {
            context.packageManager.packageInstaller.abandonSession(id)
            save(context) { it.remove("session_id"); it.put("stage", "ready") }
        }
        launch(context, manual = true, install = true)
    }
    private fun reconcile(context: Context) {
        val value = state(context)
        val release = value.optJSONObject("available")?.let(MobileRelease::stored)
        if (release != null && MobileRelease.versionCode(release.version) <= BuildConfig.VERSION_CODE) {
            save(context) { it.remove("session_id"); it.remove("available"); it.remove("error"); it.put("stage", "up_to_date") }
            File(context.filesDir, "app-update/candidate.apk").delete()
        } else {
            val id = value.optInt("session_id", -1)
            if (id >= 0 && context.packageManager.packageInstaller.getSessionInfo(id) == null)
                save(context) {
                    it.remove("session_id"); it.put("stage", "error").put("error", "update_install_failed")
                    it.optJSONObject("available")?.let { release -> it.put("failed_version", release.optString("version")) }
                }
        }
    }
    private fun notify(context: Context, action: Intent, message: Int) {
        if (!MobileNotifications.permitted(context)) return
        val manager = context.getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(NotificationChannel(CHANNEL, context.getString(R.string.check_updates), NotificationManager.IMPORTANCE_DEFAULT))
        val target = PendingIntent.getActivity(context, NOTIFICATION, action.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        manager.notify(NOTIFICATION, NotificationCompat.Builder(context, CHANNEL)
            .setSmallIcon(R.drawable.ic_notification).setContentTitle(context.getString(R.string.update_ready_title))
            .setContentText(context.getString(message)).setContentIntent(target).setAutoCancel(true).build())
    }
}
class UpdateJob : JobService() {
    private var work: Job? = null
    override fun onStartJob(params: JobParameters): Boolean {
        work = CoroutineScope(SupervisorJob() + Dispatchers.IO).launch {
            try { MobileUpdates.run(applicationContext, manual = false) }
            finally { jobFinished(params, MobileUpdates.hostChoicePending(applicationContext)) }
        }
        return true
    }
    override fun onStopJob(params: JobParameters): Boolean { work?.cancel(); return true }
}
class UpdateResultReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) { MobileUpdates.receive(context, intent) }
}

class UpdateBootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == Intent.ACTION_MY_PACKAGE_REPLACED) runCatching { MobileUpdates.schedule(context) }
    }
}
