package app.cartae.bridget

import android.Manifest
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat
import com.google.firebase.FirebaseApp
import com.google.firebase.FirebaseOptions
import com.google.firebase.messaging.FirebaseMessaging
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withTimeout
import org.json.JSONObject
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

/** Sonnette générique : aucune commande ni contenu métier dans la poussée. */
object MobileNotifications {
    private const val CHANNEL = "bridget_updates"
    private val syncing = AtomicBoolean(false)
    private val executor = Executors.newSingleThreadExecutor { work -> Thread(work, "bridget-push-registration").apply { isDaemon = true } }
    fun configured() = BuildConfig.FIREBASE_APP_ID.isNotBlank() && BuildConfig.FIREBASE_API_KEY.isNotBlank() &&
        BuildConfig.FIREBASE_SENDER_ID.isNotBlank() && BuildConfig.FIREBASE_PROJECT_ID.isNotBlank()
    fun permitted(context: Context) = Build.VERSION.SDK_INT < 33 ||
        ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED
    private fun initialize(context: Context): FirebaseMessaging {
        if (!configured()) throw MobileFailure("notifications_not_configured")
        synchronized(this) {
            if (FirebaseApp.getApps(context).isEmpty()) FirebaseApp.initializeApp(context,
                FirebaseOptions.Builder().setApplicationId(BuildConfig.FIREBASE_APP_ID).setApiKey(BuildConfig.FIREBASE_API_KEY)
                    .setGcmSenderId(BuildConfig.FIREBASE_SENDER_ID).setProjectId(BuildConfig.FIREBASE_PROJECT_ID).build())
        }
        return FirebaseMessaging.getInstance()
    }
    fun binding(session: JSONObject): String = LocalStore.digest((session.getString("endpoint") + "\u0000" +
        session.getJSONObject("device").getString("device_id")).toByteArray(Charsets.UTF_8))
    private fun preference(store: LocalStore) = "push:" + binding(store.get("session") ?: throw MobileFailure("authorization_required"))
    fun enabled(store: LocalStore) = runCatching { store.get(preference(store))?.optBoolean("enabled") == true }.getOrDefault(false)
    fun pending(store: LocalStore) = runCatching { store.get(preference(store))?.optBoolean("pending") == true }.getOrDefault(false)
    fun accepts(store: LocalStore, data: Map<String, String>): Boolean {
        if (data.keys != setOf("version", "binding") || data["version"] != "1") return false
        val session = store.get("session") ?: return false
        if (session.getJSONObject("device").optLong("expires_at") <= System.currentTimeMillis() / 1000) return false
        return enabled(store) && data["binding"] == binding(session)
    }
    fun quietSettings(store: LocalStore): JSONObject = runCatching {
        store.get("quiet:" + binding(store.get("session") ?: throw MobileFailure("authorization_required")))
    }.getOrNull() ?: JSONObject().put("enabled", false).put("start", "22:00").put("end", "08:00")
    fun setQuiet(store: LocalStore, enabled: Boolean, start: String, end: String) {
        try {
            require(start.length == 5 && end.length == 5)
            val from = java.time.LocalTime.parse(start)
            val until = java.time.LocalTime.parse(end)
            require(from != until)
        } catch (_: Exception) { throw MobileFailure("invalid_quiet_period") }
        val session = store.get("session") ?: throw MobileFailure("authorization_required")
        store.put("quiet:" + binding(session), JSONObject().put("enabled", enabled).put("start", start).put("end", end))
    }
    fun quietNow(store: LocalStore, minute: Int = java.time.LocalTime.now().let { it.hour * 60 + it.minute }): Boolean {
        val settings = quietSettings(store)
        if (!settings.optBoolean("enabled")) return false
        fun minutes(key: String) = java.time.LocalTime.parse(settings.getString(key)).let { it.hour * 60 + it.minute }
        val start = minutes("start"); val end = minutes("end")
        return if (start < end) minute in start until end else minute >= start || minute < end
    }
    suspend fun setEnabled(context: Context, store: LocalStore, enabled: Boolean) {
        val key = preference(store)
        if (enabled && !permitted(context)) throw MobileFailure("notification_permission_denied")
        if (enabled && !configured()) throw MobileFailure("notifications_not_configured")
        // Le choix local précède le réseau : un refus Android ne peut plus afficher de poussée.
        store.put(key, JSONObject().put("enabled", enabled).put("pending", true))
        if (enabled) {
            val messaging = initialize(context)
            messaging.isAutoInitEnabled = true
            val token: String = suspendCancellableCoroutine { continuation ->
                messaging.token.addOnCompleteListener { task ->
                    if (continuation.isActive) {
                        if (task.isSuccessful) continuation.resume(task.result)
                        else continuation.resumeWithException(MobileFailure("notification_registration_unconfirmed"))
                    }
                }
            }
            store.put("firebase-token", JSONObject().put("token", token))
        } else {
            if (FirebaseApp.getApps(context).isNotEmpty()) FirebaseMessaging.getInstance().isAutoInitEnabled = false
            NotificationManagerCompat.from(context).cancel(177)
        }
        synchronize(store)
    }
    private suspend fun synchronize(store: LocalStore) {
        val key = preference(store)
        val settings = store.get(key) ?: return
        val enabled = settings.optBoolean("enabled")
        val token = if (enabled) store.get("firebase-token")?.optString("token")?.takeIf { it.isNotBlank() }
            ?: throw MobileFailure("notification_registration_unconfirmed") else null
        BridgetClient(store).configureNotifications(enabled, token)
        if (key == preference(store) && store.get(key)?.optBoolean("enabled") == enabled)
            store.put(key, JSONObject().put("enabled", enabled).put("pending", false))
    }
    fun refresh(context: Context) {
        if (!configured() || !syncing.compareAndSet(false, true)) return
        executor.execute {
            try {
                LocalStore(context.applicationContext).use { store ->
                    if (store.get("session") == null) return@use
                    val value = store.get(preference(store)) ?: return@use
                    if (enabled(store) && permitted(context)) initialize(context).isAutoInitEnabled = true
                    if (value.optBoolean("pending")) runBlocking { withTimeout(20_000) { synchronize(store) } }
                }
            } catch (_: Exception) { /* Une absence de reçu reste en attente, sans nouvelle autorité. */ }
            finally { syncing.set(false) }
        }
    }
    fun tokenChanged(context: Context, token: String) {
        if (token.length !in 20..4096 || !token.all { it.isLetterOrDigit() || it in "_-:." }) return
        runCatching {
            LocalStore(context.applicationContext).use { store ->
                store.put("firebase-token", JSONObject().put("token", token))
                if (store.get("session") != null) {
                    val key = preference(store)
                    store.get(key)?.let { store.put(key, it.put("pending", true)) }
                }
            }
        }
        refresh(context)
    }
    fun show(context: Context, data: Map<String, String>) {
        if (!permitted(context)) return
        val allowed = runCatching { LocalStore(context.applicationContext).use { accepts(it, data) && !quietNow(it) } }.getOrDefault(false)
        if (!allowed) return
        val manager = context.getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(NotificationChannel(CHANNEL, context.getString(R.string.notifications_title), NotificationManager.IMPORTANCE_DEFAULT))
        val intent = Intent(context, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP)
        val open = PendingIntent.getActivity(context, 177, intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        val notification = NotificationCompat.Builder(context, CHANNEL).setSmallIcon(R.drawable.ic_notification)
            .setContentTitle(context.getString(R.string.app_name)).setContentText(context.getString(R.string.notification_generic))
            .setVisibility(NotificationCompat.VISIBILITY_PRIVATE).setAutoCancel(true).setContentIntent(open)
            .setOnlyAlertOnce(true).build()
        try { NotificationManagerCompat.from(context).notify(177, notification) }
        catch (_: SecurityException) { return }
        catch (_: Exception) { return }
    }
}

class BridgetMessagingService : FirebaseMessagingService() {
    override fun onNewToken(token: String) { MobileNotifications.tokenChanged(applicationContext, token) }
    override fun onMessageReceived(message: RemoteMessage) { MobileNotifications.show(applicationContext, message.data) }
    override fun onDeletedMessages() { /* L’ouverture relit les faits natifs ; une poussée n’est jamais une preuve. */ }
}
