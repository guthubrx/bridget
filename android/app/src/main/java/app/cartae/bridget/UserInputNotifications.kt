package app.cartae.bridget

import android.app.KeyguardManager
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.RemoteInput
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.json.JSONArray
import org.json.JSONObject
import java.util.UUID
import java.util.concurrent.Executors

/** Les poussées déclenchent une lecture authentifiée. Elles ne portent aucune réponse. */
object UserInputNotifications {
    private const val CHANNEL="bridget_questions"
    private val executor=Executors.newSingleThreadExecutor {r->Thread(r,"bridget-question-notifications").apply {isDaemon=true}}
    private fun notificationId(id:String)=0x18100000 xor id.hashCode()
    fun cancelAll(context:Context) {context.getSystemService(NotificationManager::class.java).activeNotifications.filter {it.notification.channelId==CHANNEL}.forEach {NotificationManagerCompat.from(context).cancel(it.id)}}
    fun stateChanged(context:Context,item:JSONObject) {if(!item.optBoolean("can_respond")) NotificationManagerCompat.from(context).cancel(notificationId(item.getString("request_id")))}
    fun refresh(context:Context,data:Map<String,String>) {
        executor.execute {
            runCatching {LocalStore(context.applicationContext).use {store->
                if(!MobileNotifications.accepts(store,data) || MobileNotifications.quietNow(store) || !MobileNotifications.permitted(context)) return@use
                store.pruneQuestionMetadata()
                val binding=MobileNotifications.binding(store.get("session")!!)
                val active=mutableSetOf<String>()
                runBlocking {withTimeout(15_000) {
                    var after:String?=null
                    // La file native est bornée à2048 ; parcourir ses pages, pas seulement les dix plus anciennes.
                    do {
                        val suffix=after?.let {"&after="+BridgetClient.encode(it)}.orEmpty()
                        val page=BridgetClient(store).read("/v1/user-inputs?limit=50$suffix")
                        if(store.get("session")?.let(MobileNotifications::binding)!=binding) return@withTimeout
                        val items=page.getJSONArray("items")
                        for(i in 0 until items.length()) {
                            val item=items.getJSONObject(i);active.add(item.getString("request_id"));show(context,store,item,binding)
                        }
                        after=page.optString("next_cursor").takeIf {it.isNotBlank() && it!="null"}
                    } while(after!=null)
                }}
                if(store.get("session")?.let(MobileNotifications::binding)!=binding) return@use
                for(record in store.list("q-notified:").filter {it.optString("binding")==binding && it.optString("request_id") !in active}.take(4)) {
                    try {
                        val previous=runBlocking {withTimeout(3_000) {BridgetClient(store).read("/v1/user-inputs/"+record.getString("request_id")).getJSONObject("item")}}
                        stateChanged(context,previous)
                    } catch(error:Exception) {
                        if((error as? MobileFailure)?.code=="not_found") NotificationManagerCompat.from(context).cancel(notificationId(record.getString("request_id")))
                    }
                }
            }}
        }
    }
    private fun action(context:Context,store:LocalStore,item:JSONObject,binding:String,label:String,selected:String?,free:Boolean):NotificationCompat.Action {
        val id=UUID.randomUUID().toString()
        val record=JSONObject().put("binding",binding).put("request_id",item.getString("request_id")).put("revision",item.getString("revision"))
            .put("command_id",id).put("expires",System.currentTimeMillis()+15*60_000).put("selected",selected).put("item",JSONObject(item.toString()))
        store.put("q-action:$id",record)
        val intent=Intent(context,UserInputReplyReceiver::class.java).setAction("app.cartae.bridget.ANSWER_QUESTION").setData(Uri.parse("bridget://question/$id")).putExtra("action_id",id)
        val flags=PendingIntent.FLAG_UPDATE_CURRENT or if(free) PendingIntent.FLAG_MUTABLE else PendingIntent.FLAG_IMMUTABLE
        val pending=PendingIntent.getBroadcast(context,id.hashCode(),intent,flags)
        val builder=NotificationCompat.Action.Builder(0,label,pending).setAuthenticationRequired(true)
        if(free) builder.addRemoteInput(RemoteInput.Builder("answer").setLabel(context.getString(R.string.user_input_free)).build())
        return builder.build()
    }
    private fun show(context:Context,store:LocalStore,item:JSONObject,binding:String) {
        val id=item.getString("request_id");val manager=NotificationManagerCompat.from(context)
        if(!item.optBoolean("can_respond")) {manager.cancel(notificationId(id));return}
        val posted="q-notified:"+LocalStore.digest((binding+id+item.getString("revision")).toByteArray())
        if(store.get(posted)!=null) return
        val system=context.getSystemService(NotificationManager::class.java)
        system.createNotificationChannel(NotificationChannel(CHANNEL,context.getString(R.string.questions),NotificationManager.IMPORTANCE_DEFAULT))
        val openIntent=Intent(context,MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP).putExtra("question_id",id)
        val open=PendingIntent.getActivity(context,id.hashCode(),openIntent,PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        val publicVersion=NotificationCompat.Builder(context,CHANNEL).setSmallIcon(R.drawable.ic_notification).setContentTitle(context.getString(R.string.app_name)).setContentText(context.getString(R.string.user_input_notification)).build()
        val questions=inputQuestions(item)
        val builder=NotificationCompat.Builder(context,CHANNEL).setSmallIcon(R.drawable.ic_notification).setContentTitle(context.getString(R.string.user_input_notification))
            .setContentText(questions.first().question).setStyle(NotificationCompat.BigTextStyle().bigText(questions.first().question))
            .setVisibility(NotificationCompat.VISIBILITY_PRIVATE).setPublicVersion(publicVersion).setContentIntent(open).setAutoCancel(true).setOnlyAlertOnce(true)
        if(questions.size==1) {
            val q=questions.first()
            if(!q.multiple && q.options.size<=2 && q.options.all {it.label.length<=35}) q.options.forEach {option->builder.addAction(action(context,store,item,binding,option.label,option.value,false))}
            if(q.custom) builder.addAction(action(context,store,item,binding,context.getString(R.string.user_input_quick),null,true))
        }
        if(MobileNotifications.permitted(context)) {
            try {manager.notify(notificationId(id),builder.build())} catch(_:SecurityException){return}
            store.put(posted,JSONObject().put("at",System.currentTimeMillis()).put("request_id",id).put("binding",binding))
        }
    }
    fun reply(context:Context,intent:Intent,done:()->Unit) {
        executor.execute {
            android.util.Log.i("BridgetQuestionReply","reply_received")
            try {LocalStore(context.applicationContext).use {store->runBlocking {withTimeout(20_000) {
                if(context.getSystemService(KeyguardManager::class.java).isDeviceLocked) {android.util.Log.i("BridgetQuestionReply","device_locked");return@withTimeout}
                val id=intent.getStringExtra("action_id") ?: return@withTimeout
                if(runCatching {UUID.fromString(id).toString()==id}.getOrDefault(false).not()) return@withTimeout
                val action=store.get("q-action:$id") ?: return@withTimeout
                val session=store.get("session") ?: return@withTimeout
                if(action.getString("binding")!=MobileNotifications.binding(session) || action.getLong("expires")<System.currentTimeMillis()) return@withTimeout
                val client=BridgetClient(store)
                val original=action.getJSONObject("item")
                val q=inputQuestions(original).singleOrNull() ?: throw MobileFailure("invalid_request")
                val selected=action.optString("selected").takeIf {it.isNotEmpty() && it!="null"}
                val text=RemoteInput.getResultsFromIntent(intent)?.getCharSequence("answer")?.toString().orEmpty()
                val answer=InputAnswer(selected?.let(::listOf).orEmpty(),text)
                if(!inputAnswerValid(q,answer)) throw MobileFailure("invalid_answer")
                val answers=inputAnswerJson(mapOf(q.id to answer))
                val prior=store.get("command:$id")
                if(prior!=null && prior.getJSONObject("envelope").getJSONObject("answers").toString()!=answers.toString()) throw MobileFailure("envelope_mismatch")
                // Sauvegarde avant le premier accès réseau : une coupure ne consomme pas la réponse.
                store.put(store.userInputDraftKey(original),JSONObject().put("answers",answers).put("updated_at",System.currentTimeMillis()))
                val entry=prior ?: store.enqueueUserInput(original,answers,id)
                if(prior?.optString("state") in setOf("accepted","outcome_unknown")) {
                    try {client.reconcile(entry)} catch(e:MobileFailure) {if(e.code=="not_found") client.send(entry) else throw e}
                    NotificationManagerCompat.from(context).cancel(notificationId(original.getString("request_id")))
                    return@withTimeout
                }
                val item=client.read("/v1/user-inputs/"+action.getString("request_id")).getJSONObject("item")
                if(!item.optBoolean("can_respond") || item.getString("revision")!=action.getString("revision")) {
                    store.setCommandState(id,"rejected");throw MobileFailure("question_already_resolved")
                }
                client.send(entry)
                NotificationManagerCompat.from(context).cancel(notificationId(item.getString("request_id")))
            }}}}
            catch(error:Exception) {
                android.util.Log.w("BridgetQuestionReply","reply_unconfirmed:"+((error as? MobileFailure)?.code ?: error.javaClass.simpleName))
                runCatching {
                    val open=PendingIntent.getActivity(context,181,Intent(context,MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP),PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
                    val notification=NotificationCompat.Builder(context,CHANNEL).setSmallIcon(R.drawable.ic_notification).setContentTitle(context.getString(R.string.user_input_reply_pending))
                        .setContentText(context.getString(R.string.user_input_reply_recovery)).setVisibility(NotificationCompat.VISIBILITY_PRIVATE).setContentIntent(open).setAutoCancel(true).build()
                    if(MobileNotifications.permitted(context)) try {NotificationManagerCompat.from(context).notify(181,notification)} catch(_:SecurityException) {Unit}
                }
            }
            finally {done()}
        }
    }
}
class UserInputReplyReceiver:BroadcastReceiver() {
    override fun onReceive(context:Context,intent:Intent) {
        if(intent.action!="app.cartae.bridget.ANSWER_QUESTION") return
        val pending=goAsync();UserInputNotifications.reply(context.applicationContext,intent) {pending.finish()}
    }
}
