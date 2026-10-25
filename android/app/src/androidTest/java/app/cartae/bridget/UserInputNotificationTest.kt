package app.cartae.bridget
import android.app.NotificationManager
import android.view.accessibility.AccessibilityNodeInfo
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import kotlinx.coroutines.runBlocking
import java.io.File
@RunWith(AndroidJUnit4::class)
class UserInputNotificationTest {
    private fun find(node:AccessibilityNodeInfo?,predicate:(AccessibilityNodeInfo)->Boolean):AccessibilityNodeInfo? {
        if(node==null)return null
        if(predicate(node)) return node
        for(i in 0 until node.childCount) find(node.getChild(i),predicate)?.let {return it};return null
    }
    @Test fun actualNotificationChoiceIsSentOnceAndConfirmed()=runBlocking {
        val inst=InstrumentationRegistry.getInstrumentation();val context=inst.targetContext
        val fixture=File(context.filesDir,"qa181-notification.json")
        assumeTrue("Explicit QA notification fixture required",fixture.isFile)
        val id=JSONObject(fixture.readText()).getString("request_id")
        val offline=InstrumentationRegistry.getArguments().getString("offline")=="true"
        val direct=InstrumentationRegistry.getArguments().getString("direct")=="true"
        val free=InstrumentationRegistry.getArguments().getString("free")=="true"
        val notificationAction=if(direct) context.getSystemService(NotificationManager::class.java).activeNotifications.firstOrNull {it.id==(0x18100000 xor id.hashCode())}?.notification?.actions?.firstOrNull {if(free) !it.remoteInputs.isNullOrEmpty() else it.title.toString()=="Valider"} else null
        if(!direct) assertTrue(inst.uiAutomation.performGlobalAction(android.accessibilityservice.AccessibilityService.GLOBAL_ACTION_NOTIFICATIONS))
        var clicked=false
        if(direct) {
            assertNotNull("No real notification PendingIntent",notificationAction)
            if(free) {
                val fill=android.content.Intent();val results=android.os.Bundle().apply {putCharSequence("answer","Réponse libre depuis la notification")}
                android.app.RemoteInput.addResultsToIntent(notificationAction!!.remoteInputs,fill,results)
                notificationAction.actionIntent.send(context,0,fill)
            } else notificationAction!!.actionIntent.send()
            clicked=true
        }
        repeat(if(direct) 0 else 5) {
            val root=inst.uiAutomation.rootInActiveWindow
            val choice=find(root) {it.className?.toString()=="android.widget.Button" && (it.text?.toString()=="Valider" || it.contentDescription?.toString()=="Valider")}
            if(choice!=null && !clicked) {assertTrue(choice.performAction(AccessibilityNodeInfo.ACTION_CLICK));clicked=true}
            else if(!clicked) find(root) {it.className?.toString()=="android.widget.Button" && it.contentDescription?.toString()=="Expand"}?.performAction(AccessibilityNodeInfo.ACTION_CLICK)
            Thread.sleep(700)
        }
        assertTrue("Notification action not found",clicked)
        val deadline=System.currentTimeMillis()+20_000;var applied=false;var result=JSONObject()
        if(offline) {
            LocalStore(context).use {store->
                var saved:JSONObject?=null
                while(System.currentTimeMillis()<deadline && saved==null) {
                    saved=store.list("command:").firstOrNull {it.getJSONObject("envelope").optString("request_id")==id}
                    if(saved==null) Thread.sleep(150)
                }
                assertNotNull("Offline notification reply must be saved before network access",saved)
                assertEquals("local",saved!!.getString("state"))
                assertTrue(inputAnswerText(saved).contains("Valider"))
                File(context.filesDir,"qa181-notification-offline.json").writeText(JSONObject().put("request_id",id).put("command_id",saved!!.getJSONObject("envelope").getString("command_id")).put("state","local").toString())
            }
            return@runBlocking
        }
        LocalStore(context).use {store->
            while(System.currentTimeMillis()<deadline && !applied) {
                result=BridgetClient(store).read("/v1/user-inputs/$id").getJSONObject("item");applied=result.optString("state")=="applied"
                if(!applied) Thread.sleep(500)
            }
            File(context.filesDir,"qa181-notification-result.json").writeText(JSONObject().put("clicked",clicked).put("state",result.optString("state")).toString())
            val commands=store.list("command:").filter {it.getJSONObject("envelope").optString("request_id")==id}
            assertEquals("Exactly one durable command",1,commands.size)
            assertTrue("Provider did not confirm: $result",applied)
            val answer=result.getJSONObject("answers").getJSONObject("notification")
            if(free) {assertEquals(0,answer.getJSONArray("selected").length());assertEquals("Réponse libre depuis la notification",answer.getString("text"))}
            else assertEquals("Valider",answer.getJSONArray("selected").getString(0))
        }
        Unit
    }
}
