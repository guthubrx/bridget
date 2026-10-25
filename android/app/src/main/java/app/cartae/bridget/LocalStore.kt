package app.cartae.bridget

import android.util.AtomicFile
import java.io.File
import java.io.InputStream
import java.io.ByteArrayOutputStream
import java.security.MessageDigest
import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteOpenHelper
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import org.json.JSONObject
import java.security.KeyStore
import java.security.SecureRandom
import java.util.UUID
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** Clé dans Android Keystore ; enveloppes chiffrées, sauvegarde Android désactivée. */
class LocalStore(context: Context, databaseName: String = "bridget-private.db", private val alias: String = "bridget.mobile.v1") : SQLiteOpenHelper(context, databaseName, null, 1) {
    private val attachmentDirectory = File(context.filesDir, "mobile-files-" + digest(alias.toByteArray()).take(16))

    override fun onCreate(db: SQLiteDatabase) {
        db.execSQL("CREATE TABLE entries (id TEXT PRIMARY KEY NOT NULL, value BLOB NOT NULL)")
    }
    override fun onUpgrade(db: SQLiteDatabase, old: Int, new: Int) { error("storage_version_unsupported") }
    private fun key(): SecretKey = synchronized(keyLock) {
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (store.getKey(alias, null) as? SecretKey)?.let { return@synchronized it }
        readableDatabase.rawQuery("SELECT count(*) FROM entries", null).use {
            check(it.moveToFirst() && it.getInt(0) == 0) { "device_storage_locked" }
        }
        check(attachmentDirectory.listFiles().orEmpty().isEmpty()) { "device_storage_locked" }
        return@synchronized KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore").run {
            init(KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM).setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setRandomizedEncryptionRequired(true).build())
            generateKey()
        }
    }
    @Synchronized fun get(id: String): JSONObject? {
        val bytes = readableDatabase.query("entries", arrayOf("value"), "id=?", arrayOf(id), null, null, null).use {
            if (!it.moveToFirst()) return null
            it.getBlob(0)
        }
        require(bytes.size in 29..131_101) { "device_storage_invalid" }
        val cipher = Cipher.getInstance("AES/GCM/NoPadding").apply {
            init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(128, bytes.copyOfRange(0, 12)))
            updateAAD(id.toByteArray(Charsets.UTF_8))
        }
        return JSONObject(String(cipher.doFinal(bytes.copyOfRange(12, bytes.size)), Charsets.UTF_8))
    }
    @Synchronized fun put(id: String, value: JSONObject) {
        require(id.length in 1..128)
        val plain = value.toString().toByteArray(Charsets.UTF_8)
        require(plain.size <= 131_072) { "device_storage_limit" }
        val cipher = Cipher.getInstance("AES/GCM/NoPadding").apply {
            init(Cipher.ENCRYPT_MODE, key()); updateAAD(id.toByteArray(Charsets.UTF_8))
        }
        val encrypted = cipher.iv + cipher.doFinal(plain)
        readableDatabase.rawQuery("SELECT count(*) FROM entries WHERE id<>?", arrayOf(id)).use {
            check(it.moveToFirst() && it.getInt(0) < 512) { "device_storage_limit" }
        }
        writableDatabase.insertWithOnConflict("entries", null, ContentValues().apply {
            put("id", id); put("value", encrypted)
        }, SQLiteDatabase.CONFLICT_REPLACE).also { check(it >= 0) { "device_storage_unavailable" } }
    }
    @Synchronized fun remove(id: String) { writableDatabase.delete("entries", "id=?", arrayOf(id)) }
    @Synchronized fun list(prefix: String): List<JSONObject> {
        require(prefix in setOf("command:", "draft:"))
        val ids = readableDatabase.query("entries", arrayOf("id"), "id LIKE ?", arrayOf("$prefix%"), null, null, "id", "256").use {
            buildList { while (it.moveToNext()) add(it.getString(0)) }
        }
        return ids.mapNotNull { get(it) }
    }
    @Synchronized fun draftKey(agentId: String): String {
        val session = get("session") ?: throw MobileFailure("authorization_required")
        val endpoint = session.getString("endpoint")
        EndpointPolicy.requireEndpoint(endpoint)
        val device = session.getJSONObject("device").getString("device_id")
        val scope = java.security.MessageDigest.getInstance("SHA-256")
            .digest((endpoint + "\u0000" + device).toByteArray(Charsets.UTF_8))
            .joinToString("") { "%02x".format(it.toInt() and 255) }
        require(UUID.fromString(agentId).toString() == agentId)
        return "draft:" + scope + ":" + agentId
    }
    @Synchronized fun preferencesKey(): String {
        val session = get("session") ?: throw MobileFailure("authorization_required")
        val origin = session.getString("endpoint") + "\u0000" + session.getJSONObject("device").getString("device_id")
        return "preferences:" + digest(origin.toByteArray(Charsets.UTF_8))
    }
    @Synchronized fun historyKey(agentId: String): String = draftKey(agentId).replaceFirst("draft:", "history:")
    @Synchronized fun enqueue(agentId: String, text: String, guideNow: Boolean = false): JSONObject {
        require(text.isNotBlank() && text.toByteArray(Charsets.UTF_8).size <= 10_000)
        check(list("command:").size < 256) { "device_storage_limit" }
        val session = get("session") ?: throw MobileFailure("authorization_required")
        val endpoint = session.getString("endpoint")
        EndpointPolicy.requireEndpoint(endpoint)
        val device = session.getJSONObject("device").getString("device_id")
        val envelope = JSONObject().put("device_id", device).put("command_id", UUID.randomUUID().toString())
            .put("issued_at", System.currentTimeMillis() / 1000).put("type", "message")
            .put("agent_id", agentId).put("body", text)
        if(guideNow) envelope.put("guide_now",true)
        val entry = JSONObject().put("endpoint", endpoint).put("envelope", envelope).put("state", "local")
        put("command:" + envelope.getString("command_id"), entry)
        return entry
    }
    @Synchronized fun enqueueAction(type: String, objectId: String, choice: String? = null): JSONObject {
        require(type in setOf("task_cancel", "question_respond"))
        require(UUID.fromString(objectId).toString() == objectId)
        require((type == "question_respond") == (choice != null))
        require(choice == null || (choice.isNotBlank() && choice.toByteArray(Charsets.UTF_8).size <= 1024))
        check(list("command:").size < 256) { "device_storage_limit" }
        val session = get("session") ?: throw MobileFailure("authorization_required")
        val endpoint = session.getString("endpoint")
        EndpointPolicy.requireEndpoint(endpoint)
        val envelope = JSONObject().put("type", type).put("device_id", session.getJSONObject("device").getString("device_id"))
            .put("command_id", UUID.randomUUID().toString())
        if (type == "task_cancel") envelope.put("task_id", objectId)
        else envelope.put("item_id", objectId).put("choice", choice)
        val entry = JSONObject().put("endpoint", endpoint).put("envelope", envelope).put("state", "local")
            .put("created_at", System.currentTimeMillis())
        put("command:" + envelope.getString("command_id"), entry)
        return entry
    }
    @Synchronized fun enqueueFile(agentId: String, name: String, mediaType: String, input: InputStream, commandId: String = UUID.randomUUID().toString()): JSONObject {
        require(name.isNotBlank() && name.toByteArray(Charsets.UTF_8).size <= 240
            && name != "." && name != ".." && name.none { it.isISOControl() || it == '/' || it == '\\' }) { "invalid_file" }
        require(mediaType in setOf("application/pdf", "text/plain", "text/html", "text/markdown", "image/png", "image/jpeg")) { "invalid_file" }
        require(UUID.fromString(agentId).toString() == agentId)
        check(list("command:").size < 256) { "device_storage_limit" }
        val session = get("session") ?: throw MobileFailure("authorization_required")
        val output = ByteArrayOutputStream()
        val buffer = ByteArray(8192)
        while (true) {
            val count = input.read(buffer)
            if (count < 0) break
            check(output.size() + count <= MAX_FILE_BYTES) { "payload_too_large" }
            output.write(buffer, 0, count)
        }
        val bytes = output.toByteArray()
        require(bytes.isNotEmpty()) { "invalid_file" }
        val id = commandId
        require(UUID.fromString(id).toString() == id)
        check(get("command:$id") == null) { "envelope_mismatch" }
        val envelope = JSONObject().put("type", "file").put("device_id", session.getJSONObject("device").getString("device_id"))
            .put("command_id", id).put("agent_id", agentId).put("file_name", name).put("media_type", mediaType)
            .put("expected_digest", digest(bytes)).put("byte_length", bytes.size)
        val entry = JSONObject().put("endpoint", session.getString("endpoint")).put("envelope", envelope)
            .put("created_at", System.currentTimeMillis()).put("state", "local")
        check(attachmentDirectory.exists() || attachmentDirectory.mkdirs()) { "device_storage_unavailable" }
        val used = attachmentDirectory.listFiles().orEmpty().sumOf { it.length() }
        check(used + bytes.size + 28 <= 32L * 1024 * 1024) { "device_storage_limit" }
        val cipher = Cipher.getInstance("AES/GCM/NoPadding").apply {
            init(Cipher.ENCRYPT_MODE, key()); updateAAD(fileAad(entry))
        }
        val atomic = AtomicFile(attachmentFile(id))
        val stream = atomic.startWrite()
        try {
            stream.write(cipher.iv)
            stream.write(cipher.doFinal(bytes))
            atomic.finishWrite(stream)
        } catch (error: Exception) {
            atomic.failWrite(stream)
            throw error
        }
        try { put("command:$id", entry) }
        catch (error: Exception) { atomic.delete(); throw error }
        return entry
    }
    @Synchronized fun fileBytes(entry: JSONObject): ByteArray {
        val session = get("session") ?: throw MobileFailure("authorization_required")
        BridgetClient.requireCommandOrigin(entry, session)
        val envelope = entry.getJSONObject("envelope")
        require(envelope.getString("type") == "file")
        val file = attachmentFile(envelope.getString("command_id"))
        check(file.length() in 29..(MAX_FILE_BYTES + 28L)) { "device_storage_invalid" }
        val encrypted = AtomicFile(file).readFully()
        val cipher = Cipher.getInstance("AES/GCM/NoPadding").apply {
            init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(128, encrypted.copyOfRange(0,12)))
            updateAAD(fileAad(entry))
        }
        val bytes = cipher.doFinal(encrypted.copyOfRange(12, encrypted.size))
        check(bytes.size == envelope.getInt("byte_length") && digest(bytes) == envelope.getString("expected_digest")) { "device_storage_invalid" }
        return bytes
    }
    private fun attachmentFile(id: String): File {
        require(UUID.fromString(id).toString() == id)
        return File(attachmentDirectory, "$id.enc")
    }
    private fun fileAad(entry: JSONObject): ByteArray {
        val envelope = entry.getJSONObject("envelope")
        return (entry.getString("endpoint") + "\u0000" + envelope.getString("device_id")
            + "\u0000" + envelope.getString("command_id")).toByteArray(Charsets.UTF_8)
    }
    @Synchronized fun discardCommand(id: String) {
        val entry = get("command:$id") ?: return
        require(entry.getString("state") in setOf("local", "accepted", "rejected", "idempotency_expired", "orphaned")) { "outcome_unknown" }
        if (entry.getJSONObject("envelope").optString("type") == "file") AtomicFile(attachmentFile(id)).delete()
        remove("command:$id")
    }
    @Synchronized fun setCommandState(id: String, state: String, receipt: JSONObject? = null) {
        require(state in setOf("local", "outcome_unknown", "accepted", "rejected", "idempotency_expired", "envelope_mismatch", "invalid_issued_at", "orphaned"))
        val entry = get("command:$id") ?: error("command_missing")
        entry.put("state", state)
        receipt?.let { entry.put("receipt", it) }
        put("command:$id", entry)
    }
    @Synchronized fun forgetAuthorization() {
        // L'accès seul est oublié ; aucune commande, pièce jointe ni preuve n'est effacée.
        writableDatabase.beginTransaction()
        try {
            for (id in listOf("session", "pairing", "renewal")) remove(id)
            writableDatabase.setTransactionSuccessful()
        } finally { writableDatabase.endTransaction() }
    }
    @Synchronized fun observeMessage(agentId: String, event: JSONObject) {
        val kind = event.optString("event")
        if (kind !in setOf("turn_start", "turn_end")) return
        val id = event.optString("message_id")
        if (runCatching { UUID.fromString(id).toString() == id }.getOrDefault(false) != true) return
        val entry = get("command:$id") ?: return
        val envelope = entry.getJSONObject("envelope")
        if (envelope.optString("type") != "message" || envelope.optString("agent_id") != agentId) return
        val current = get("session") ?: return
        if (runCatching { BridgetClient.requireCommandOrigin(entry, current) }.isFailure) return
        val seq = event.optLong("seq")
        if (seq <= 0 || seq <= (entry.optJSONObject("delivery_observation")?.optLong("seq") ?: 0)) return
        entry.put("delivery_observation", JSONObject().put("agent_id", agentId).put("seq", seq).put("event", kind))
        put("command:$id", entry)
    }
    companion object {
        private val keyLock = Any()
        const val MAX_FILE_BYTES = 8 * 1024 * 1024
        fun digest(bytes: ByteArray): String = MessageDigest.getInstance("SHA-256").digest(bytes)
            .joinToString("") { "%02x".format(it.toInt() and 255) }
        fun newKey(): String = ByteArray(32).also { SecureRandom().nextBytes(it) }
            .joinToString("") { "%02x".format(it.toInt() and 255) }
    }
}
