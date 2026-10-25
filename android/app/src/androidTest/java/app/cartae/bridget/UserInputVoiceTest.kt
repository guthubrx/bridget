package app.cartae.bridget
import android.app.Activity
import android.app.Instrumentation
import android.content.Intent
import android.speech.RecognizerIntent
import android.view.accessibility.AccessibilityNodeInfo
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.concurrent.atomic.AtomicBoolean
@RunWith(AndroidJUnit4::class)
class UserInputVoiceTest {
    private fun find(node:AccessibilityNodeInfo?,predicate:(AccessibilityNodeInfo)->Boolean):AccessibilityNodeInfo? {
        if(node==null)return null
        if(predicate(node)) return node
        for(i in 0 until node.childCount) find(node.getChild(i),predicate)?.let {return it}
        return null
    }
    private fun waitNode(predicate:(AccessibilityNodeInfo)->Boolean):AccessibilityNodeInfo? {
        val inst=InstrumentationRegistry.getInstrumentation();val end=System.currentTimeMillis()+25_000
        while(System.currentTimeMillis()<end) {
            find(inst.uiAutomation.rootInActiveWindow,predicate)?.let {return it}
            Thread.sleep(150)
        };return null
    }
    @Test fun dictatedAnswerStaysEditableAndIsNotSentAutomatically() {
        val inst=InstrumentationRegistry.getInstrumentation();val context=inst.targetContext
        val fixture=File(context.filesDir,"qa181-voice.json")
        assumeTrue("Explicit private voice fixture required",fixture.isFile)
        val config=JSONObject(fixture.readText());val real=InstrumentationRegistry.getArguments().getString("voiceMode")=="real"
        val observed=AtomicBoolean(false)
        val monitor=object:Instrumentation.ActivityMonitor() {
            override fun onStartActivity(intent:Intent):Instrumentation.ActivityResult? {
                if(intent.action!=RecognizerIntent.ACTION_RECOGNIZE_SPEECH) return null
                observed.set(true)
                if(!real) return Instrumentation.ActivityResult(Activity.RESULT_OK,Intent().putStringArrayListExtra(RecognizerIntent.EXTRA_RESULTS,arrayListOf("Réponse dictée contrôlée")))
                // L’audio réel vient du microphone émulé injectAudio (hôte désactivé).
                // Aucun descripteur de fichier ne peut traverser startActivity.
                return null
            }
        }
        inst.addMonitor(monitor)
        try {
            inst.startActivitySync(Intent(context,MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK).putExtra("question_id",config.getString("request_id")))
            val initial=waitNode {it.className?.toString()=="android.widget.EditText"} ?: error("Answer field unavailable")
            val clear=android.os.Bundle().apply {putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE,"")}
            initial.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT,clear)
            val button=waitNode {it.text?.toString() in setOf("Dicter ma réponse","Dictate my answer")} ?: error("Voice button unavailable")
            var target=button
            while(!target.isClickable && target.parent!=null) target=target.parent
            assertTrue(target.performAction(AccessibilityNodeInfo.ACTION_CLICK))
            val field=waitNode {it.className?.toString()=="android.widget.EditText" && !it.text.isNullOrBlank()}
            val text=field?.text?.toString().orEmpty()
            val result=JSONObject().put("mode",if(real) "real_recognizer_emulator_microphone" else "controlled_activity_result").put("intent_observed",observed.get()).put("transcript",text)
            File(context.filesDir,"qa181-voice-result.json").writeText(result.toString())
            inst.uiAutomation.takeScreenshot()?.let {bitmap->File(context.filesDir,"qa181-voice-result.png").outputStream().use {bitmap.compress(android.graphics.Bitmap.CompressFormat.PNG,100,it)}}
            assertTrue("Speech intent not observed",observed.get())
            assertTrue("No transcript from recognition: $result",text.isNotBlank())
            assertTrue(field!!.isEnabled)
            if(real) assertTrue("Unexpected real transcript: $text",text.lowercase().contains("solution")) else assertEquals("Réponse dictée contrôlée",text)
            LocalStore(context).use {store->
                val entries=store.list("command:").filter {it.getJSONObject("envelope").optString("request_id")==config.getString("request_id")}
                assertTrue("Dictation must never send automatically",entries.isEmpty())
            }
        } finally {inst.removeMonitor(monitor)}
    }
}
