package app.cartae.bridget

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException
import org.json.JSONObject
import java.io.ByteArrayOutputStream
import java.net.URL
import java.net.URLEncoder
import java.util.UUID
import javax.net.ssl.HttpsURLConnection

class MobileFailure(val code: String) : Exception(code)

/** TLS système et origine fixe ; les diagnostics ne contiennent jamais les corps ni les clés. */
class BridgetClient(private val store: LocalStore) {
    private suspend fun request(endpoint: String, path: String, method: String = "GET", key: String? = null, payload: JSONObject? = null): JSONObject =
        suspendCancellableCoroutine { continuation ->
            try {
                EndpointPolicy.requireEndpoint(endpoint)
                require(path.startsWith("/v1/") && !path.contains('#') && !path.contains('\n'))
                val connection = URL(endpoint + path).openConnection() as HttpsURLConnection
                continuation.invokeOnCancellation { connection.disconnect() }
                Dispatchers.IO.dispatch(continuation.context, Runnable {
                    if (!continuation.isActive) { connection.disconnect(); return@Runnable }
                    try {
                        val value = performRequest(connection, path, method, key, payload)
                        if (continuation.isActive) continuation.resume(value)
                    } catch (error: Exception) {
                        if (continuation.isActive) continuation.resumeWithException(error)
                    }
                })
            } catch (error: Exception) {
                if (continuation.isActive) continuation.resumeWithException(
                    if (error is MobileFailure) error else MobileFailure("outcome_unknown"))
            }
        }
    private fun performRequest(connection: HttpsURLConnection, path: String, method: String, key: String?, payload: JSONObject?): JSONObject {
        return try {
            connection.instanceFollowRedirects = false
            connection.connectTimeout = 10_000
            connection.readTimeout = if (path.startsWith("/v1/events?")) 30_000 else 20_000
            connection.requestMethod = method
            connection.setRequestProperty("Accept", "application/json")
            key?.let { connection.setRequestProperty("Authorization", "Bearer $it") }
            payload?.let {
                val bytes = it.toString().toByteArray(Charsets.UTF_8)
                require(bytes.size <= if (path == "/v1/artifacts") ((LocalStore.MAX_FILE_BYTES + 2) / 3 * 4 + 16_384) else 65_536)
                connection.doOutput = true
                connection.setRequestProperty("Content-Type", "application/json")
                connection.setFixedLengthStreamingMode(bytes.size)
                connection.outputStream.use { output -> output.write(bytes) }
            }
            val status = connection.responseCode
            if (status in 300..399) throw MobileFailure("endpoint_redirect_refused")
            val stream = if (status in 200..299) connection.inputStream else connection.errorStream
            val bytes = stream?.use { input ->
                val output = ByteArrayOutputStream()
                val buffer = ByteArray(8192)
                while (true) {
                    val count = input.read(buffer)
                    if (count < 0) break
                    if (output.size() + count > 1_048_576) throw MobileFailure("response_too_large")
                    output.write(buffer, 0, count)
                }
                output.toByteArray()
            } ?: throw MobileFailure("outcome_unknown")
            val response = JSONObject(String(bytes, Charsets.UTF_8))
            if (response.optInt("version") != 1) throw MobileFailure("unsupported_version")
            if (status !in 200..299 || response.optString("status") != "ok") {
                val code = response.optString("code")
                throw MobileFailure(if (code.matches(Regex("[a-z_]{1,64}"))) code else "outcome_unknown")
            }
            response.getJSONObject("result")
        } catch (e: MobileFailure) { throw e }
        catch (_: Exception) { throw MobileFailure("outcome_unknown") }
        finally { connection.disconnect() }
    }
    suspend fun pair(qr: String? = null): JSONObject {
        val pending = if (qr == null) store.get("pairing") ?: throw MobileFailure("pairing_missing") else {
            if (store.get("session") != null || store.get("pairing") != null) throw MobileFailure("pairing_pending")
            val value = validatedPairing(qr)
            value.put("attempt_id", UUID.randomUUID().toString()).put("device_key", LocalStore.newKey())
            store.put("pairing", value)
            value
        }
        val body = JSONObject().put("credential", pending.getString("credential"))
            .put("attempt_id", pending.getString("attempt_id")).put("device_key", pending.getString("device_key"))
            .put("label", "Android")
        val device = request(pending.getString("endpoint"), "/v1/pair", "POST", payload = body)
        store.put("session", JSONObject().put("endpoint", pending.getString("endpoint"))
            .put("device_key", pending.getString("device_key")).put("device", device))
        store.remove("pairing")
        return device
    }
    suspend fun renew(): JSONObject {
        val session = store.get("session") ?: throw MobileFailure("authorization_required")
        val pending = store.get("renewal") ?: JSONObject().put("old_key", session.getString("device_key"))
            .put("endpoint", session.getString("endpoint"))
            .put("device_id", session.getJSONObject("device").getString("device_id"))
            .put("new_key", LocalStore.newKey()).put("request_id", UUID.randomUUID().toString())
            .also { store.put("renewal", it) }
        if (pending.optString("endpoint") != session.getString("endpoint")
            || pending.optString("device_id") != session.getJSONObject("device").getString("device_id")) {
            throw MobileFailure("authorization_origin_changed")
        }
        val device = request(session.getString("endpoint"), "/v1/session/renew", "POST", pending.getString("old_key"),
            JSONObject().put("request_id", pending.getString("request_id")).put("device_key", pending.getString("new_key")))
        session.put("device_key", pending.getString("new_key")).put("device", device)
        store.put("session", session)
        store.remove("renewal")
        return device
    }
    suspend fun read(path: String): JSONObject {
        val session = store.get("session") ?: throw MobileFailure("authorization_required")
        val response = request(session.getString("endpoint"), path, key = session.getString("device_key"))
        requireSessionOrigin(session, store.get("session"))
        return response
    }
    suspend fun requestUpdates(install: Boolean, version: String? = null): JSONObject {
        val session = store.get("session") ?: throw MobileFailure("authorization_required")
        val path = if (install) "/v1/updates/install" else "/v1/updates/check"
        val response = request(session.getString("endpoint"), path, "POST", session.getString("device_key"), if (install) JSONObject().put("version", version) else JSONObject())
        requireSessionOrigin(session, store.get("session")); return response
    }
    suspend fun configureUpdates(automatic: Boolean): JSONObject {
        val session = store.get("session") ?: throw MobileFailure("authorization_required")
        val response = request(session.getString("endpoint"), "/v1/updates", "PUT", session.getString("device_key"), JSONObject().put("automatic", automatic))
        requireSessionOrigin(session, store.get("session")); return response
    }
    suspend fun configureNotifications(enabled: Boolean, token: String?): JSONObject {
        val session = store.get("session") ?: throw MobileFailure("authorization_required")
        val body = JSONObject().put("enabled", enabled).put("token", token ?: JSONObject.NULL)
        val response = request(session.getString("endpoint"), "/v1/notifications", "PUT", session.getString("device_key"), body)
        requireSessionOrigin(session, store.get("session"))
        return response
    }
    suspend fun revokeDevice(id: String) {
        require(UUID.fromString(id).toString() == id)
        val session = store.get("session") ?: throw MobileFailure("authorization_required")
        request(session.getString("endpoint"), "/v1/devices/$id", "DELETE", session.getString("device_key"))
        if (id == session.getJSONObject("device").getString("device_id")) store.remove("session")
    }
    suspend fun revokeSelf() {
        val session = store.get("session") ?: throw MobileFailure("authorization_required")
        val id = session.getJSONObject("device").getString("device_id")
        request(session.getString("endpoint"), "/v1/devices/$id", "DELETE", session.getString("device_key"))
        store.remove("session")
    }
    suspend fun send(entry: JSONObject): JSONObject {
        val session = store.get("session") ?: throw MobileFailure("authorization_required")
        requireCommandOrigin(entry, session)
        val envelope = entry.getJSONObject("envelope")
        val id = envelope.getString("command_id")
        store.setCommandState(id, "outcome_unknown")
        val upload = envelope.optString("type") == "file"
        val payload = if (upload) JSONObject(envelope.toString()).apply {
            remove("type"); remove("byte_length")
            put("bytes", android.util.Base64.encodeToString(store.fileBytes(entry), android.util.Base64.NO_WRAP))
        } else envelope
        val receipt = try {
            request(session.getString("endpoint"), if (upload) "/v1/artifacts" else "/v1/commands", "POST", session.getString("device_key"), payload)
        } catch(e: MobileFailure) {
            if(!upload && envelope.optString("type")=="message" && e.code=="not_found") {
                // This message was refused before reservation, so its original saved envelope is still unsent.
                store.setCommandState(id,"local")
                throw MobileFailure("recipient_unavailable")
            }
            throw e
        }
        requireCommandOrigin(entry, store.get("session") ?: throw MobileFailure("authorization_required"))
        retainReceipt(id, receipt)
        return receipt
    }
    suspend fun reconcile(entry: JSONObject): JSONObject {
        val session = store.get("session") ?: throw MobileFailure("authorization_required")
        requireCommandOrigin(entry, session)
        val id = entry.getJSONObject("envelope").getString("command_id")
        val envelope = entry.getJSONObject("envelope")
        val path = when (envelope.optString("type")) {
            "file" -> "/v1/artifacts/uploads/$id"
            "task_cancel" -> "/v1/tasks/" + envelope.getString("task_id") + "/cancel/$id"
            "question_respond" -> "/v1/questions/" + envelope.getString("item_id") + "/responses/$id?choice=" + encode(envelope.getString("choice"))
            else -> "/v1/commands/$id"
        }
        val receipt = request(session.getString("endpoint"), path, key = session.getString("device_key"))
        requireCommandOrigin(entry, store.get("session") ?: throw MobileFailure("authorization_required"))
        retainReceipt(id, receipt)
        return receipt
    }
    private fun retainReceipt(id: String, receipt: JSONObject) {
        val state = if (receipt.optJSONObject("receipt")?.has("artifact_ref") == true) "accepted"
            else receipt.optJSONObject("issue")?.optString("kind") ?: "outcome_unknown"
        store.setCommandState(id, when (state) {
            "accepted" -> "accepted"
            "rejected" -> "rejected"
            "idempotency_expired", "envelope_mismatch", "invalid_issued_at", "orphaned" -> state
            else -> "outcome_unknown"
        }, receipt)
    }
    suspend fun download(agentId: String, metadata: JSONObject, progress: (Float) -> Unit = {}): ByteArray {
        val session = store.get("session") ?: throw MobileFailure("authorization_required")
        val total = metadata.getLong("byte_length")
        if (total !in 1..LocalStore.MAX_FILE_BYTES.toLong()
            || metadata.getString("media_type") !in setOf("application/pdf", "text/plain", "text/html", "text/markdown", "image/png", "image/jpeg")) throw MobileFailure("invalid_file")
        val digest = metadata.getString("digest")
        if (!digest.matches(Regex("[0-9a-f]{64}"))) throw MobileFailure("invalid_file")
        val output = ByteArrayOutputStream()
        var offset = 0L
        while (offset < total) {
            val current = store.get("session") ?: throw MobileFailure("authorization_required")
            if (current.getString("endpoint") != session.getString("endpoint")
                || current.getJSONObject("device").getString("device_id") != session.getJSONObject("device").getString("device_id")) {
                throw MobileFailure("command_origin_changed")
            }
            val page = request(current.getString("endpoint"), "/v1/artifacts/read", "POST", current.getString("device_key"),
                JSONObject().put("action", "artifact_read").put("agent_id", agentId)
                    .put("artifact_ref", metadata.getString("artifact_ref")).put("version_ref", metadata.getString("version_ref"))
                    .put("offset", offset).put("limit", 16_384))
            val chunk = page.getJSONObject("chunk")
            if (chunk.getLong("offset") != offset || chunk.getLong("total_len") != total || chunk.getString("digest") != digest) throw MobileFailure("invalid_file")
            val array = chunk.getJSONArray("bytes")
            if (array.length() !in 1..16_384 || offset + array.length() > total) throw MobileFailure("invalid_file")
            for (i in 0 until array.length()) {
                val value = array.getInt(i)
                if (value !in 0..255) throw MobileFailure("invalid_file")
                output.write(value)
            }
            offset += array.length()
            progress(offset.toFloat() / total)
            if (offset < total && chunk.optLong("next_offset", -1) != offset) throw MobileFailure("invalid_file")
        }
        requireSessionOrigin(session, store.get("session"))
        return output.toByteArray().also { if (LocalStore.digest(it) != digest) throw MobileFailure("invalid_file") }
    }
    companion object {
        private fun requireSessionOrigin(original: JSONObject, current: JSONObject?) {
            if (current == null || original.getString("endpoint") != current.getString("endpoint")
                || original.getJSONObject("device").getString("device_id") != current.getJSONObject("device").getString("device_id")) {
                throw MobileFailure("command_origin_changed")
            }
        }
        fun requireCommandOrigin(entry: JSONObject, session: JSONObject) {
            val sameEndpoint = entry.optString("endpoint").isNotEmpty()
                && entry.optString("endpoint") == session.optString("endpoint")
            val original = entry.optJSONObject("envelope")?.optString("device_id").orEmpty()
            if (!sameEndpoint || original.isEmpty()
                || original != session.optJSONObject("device")?.optString("device_id")) {
                throw MobileFailure("command_origin_changed")
            }
        }

        fun validatedPairing(qr: String): JSONObject {
            try {
                require(qr.length <= 4096)
                val value = JSONObject(qr)
                require(value.keys().asSequence().toSet() == setOf("version", "endpoint", "pairing_id", "credential"))
                require(value.opt("version") == 1)
                require(value.opt("endpoint") is String && value.opt("credential") is String && value.opt("pairing_id") is String)
                EndpointPolicy.requireEndpoint(value.getString("endpoint"))
                require(value.getString("credential").matches(Regex("[0-9a-f]{64}")))
                val id = UUID.fromString(value.getString("pairing_id"))
                require(id.version() == 4 && id.toString() == value.getString("pairing_id"))
                return value
            } catch (_: Exception) { throw MobileFailure("invalid_qr") }
        }
        fun encode(value: String): String = URLEncoder.encode(value, Charsets.UTF_8.name())
    }
}
