package app.cartae.bridget

import android.app.Activity
import android.app.Instrumentation
import android.content.Intent
import android.speech.RecognizerIntent
import android.view.accessibility.AccessibilityNodeInfo
import android.graphics.Rect
import android.os.Bundle
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File

/** Installation privée d'une association de recette ; aucun secret ni compte dans les sources. */
@RunWith(AndroidJUnit4::class)
class MobileJourneyTest {
    private val inst=InstrumentationRegistry.getInstrumentation()
    private var imageNumber=0
    private val imageRun=System.currentTimeMillis().toString()
    private fun find(node:AccessibilityNodeInfo?,predicate:(AccessibilityNodeInfo)->Boolean):AccessibilityNodeInfo? {
        val seen=mutableSetOf<Int>();val deadline=android.os.SystemClock.uptimeMillis()+3000
        fun visit(value:AccessibilityNodeInfo?):AccessibilityNodeInfo? {
            if(value==null || seen.size>=512 || android.os.SystemClock.uptimeMillis()>deadline || !seen.add(value.hashCode()) || !value.isVisibleToUser) return null
            if(predicate(value)) return value
            for(i in 0 until value.childCount) visit(value.getChild(i))?.let {return it}
            return null
        }
        return visit(node)
    }
    private fun waitNode(predicate:(AccessibilityNodeInfo)->Boolean):AccessibilityNodeInfo {
        val end=System.currentTimeMillis()+25_000
        while(System.currentTimeMillis()<end) {
            find(inst.uiAutomation.rootInActiveWindow,predicate)?.let {return it}
            Thread.sleep(150)
        }
        screenshot("node-timeout")
        val labels=mutableListOf<String>()
        fun collect(node:AccessibilityNodeInfo?) {if(node==null)return;if(!node.text.isNullOrBlank() || !node.contentDescription.isNullOrBlank()) labels.add(node.className.toString()+" | "+node.text+" | "+node.contentDescription+" | enabled="+node.isEnabled);for(i in 0 until node.childCount) collect(node.getChild(i))}
        collect(inst.uiAutomation.rootInActiveWindow)
        File(inst.targetContext.filesDir,"qa183-node-timeout.txt").writeText(labels.joinToString("\n"))
        error("Expected UI node unavailable")
    }
    private fun screenshot(label:String):String {
        val name="qa183-journey-"+imageRun+"-"+(imageNumber++).toString().padStart(2,'0')+"-"+label+".png"
        inst.uiAutomation.takeScreenshot()?.let {bitmap->File(inst.targetContext.filesDir,name).outputStream().use {bitmap.compress(android.graphics.Bitmap.CompressFormat.PNG,100,it)}}
        return name
    }
    private fun tap(vararg labels:String) {
        var node=waitNode {it.text?.toString() in labels || it.contentDescription?.toString() in labels}
        while(!node.isClickable && node.parent!=null) node=node.parent
        val bounds=Rect();node.getBoundsInScreen(bounds)
        val before=screenshot("before")
        assertTrue("UI control was disabled: "+labels.joinToString(),node.isEnabled)
        android.os.ParcelFileDescriptor.AutoCloseInputStream(inst.uiAutomation.executeShellCommand("input tap ${bounds.centerX()} ${bounds.centerY()}")).use {it.readBytes()}
        File(inst.targetContext.filesDir,"qa183-actions.jsonl").appendText(JSONObject().put("action","touch").put("target",labels.joinToString(" | ")).put("x",bounds.centerX()).put("y",bounds.centerY()).put("bounds",bounds.toShortString()).put("at",System.currentTimeMillis()).put("before",before).toString()+"\n")
        Thread.sleep(450)
    }
    private fun verifyPreviewGeometry() {
        val done=java.util.concurrent.CountDownLatch(1)
        val layout=java.util.concurrent.atomic.AtomicReference<JSONObject>()
        inst.runOnMainSync {
            fun web(node:android.view.View):android.webkit.WebView? {
                if(node is android.webkit.WebView) return node
                if(node is android.view.ViewGroup) for(i in 0 until node.childCount) web(node.getChildAt(i))?.let {return it}
                return null
            }
            val view=android.view.inspector.WindowInspector.getGlobalWindowViews().mapNotNull(::web).lastOrNull()
            if(view==null) done.countDown() else {
                val facts=JSONObject().put("width",view.width).put("height",view.height).put("shown",view.isShown).put("hardware",view.isHardwareAccelerated).put("layer",view.layerType).put("javascript",view.settings.javaScriptEnabled).put("url",view.url).put("layout_height",view.layoutParams?.height).put("layout_width",view.layoutParams?.width).put("parent",view.parent?.javaClass?.name)
                view.evaluateJavascript("JSON.stringify({width:innerWidth,height:innerHeight,frame:document.querySelector('iframe')?.getBoundingClientRect().toJSON(),sourceLength:document.querySelector('iframe')?.srcdoc.length,compat:document.compatMode,clientHeight:document.documentElement.clientHeight,bodyHeight:getComputedStyle(document.body).height,rootHeight:getComputedStyle(document.documentElement).height,frameHeight:getComputedStyle(document.querySelector('iframe')).height})") {raw->
                    val decoded=org.json.JSONTokener(raw).nextValue()
                    if(decoded is String) layout.set(JSONObject(decoded))
                    facts.put("document",raw);File(inst.targetContext.filesDir,"qa183-preview-runtime.json").writeText(facts.toString());done.countDown()
                }
            }
        }
        assertTrue("Preview diagnostic timed out",done.await(5,java.util.concurrent.TimeUnit.SECONDS))
        val measured=layout.get() ?: error("Preview document geometry unavailable")
        assertTrue("Preview iframe has zero or incomplete height: $measured",measured.getJSONObject("frame").getDouble("height")>=measured.getDouble("height")-1)
        screenshot("preview-geometry-verified")
        // Le compteur tactile est qualifié dans HtmlIsolationTest et dans le circuit visible.
        // Certains moteurs ne publient pas les enfants de l'iframe opaque dans UiAutomation.
    }
    private fun editLastField(value:String) {
        val deadline=System.currentTimeMillis()+5000
        while(System.currentTimeMillis()<deadline && find(inst.uiAutomation.rootInActiveWindow) {it.contentDescription?.toString() in setOf("Fermer les conversations","Close conversations")}!=null) Thread.sleep(100)
        val fields=mutableListOf<AccessibilityNodeInfo>()
        fun collect(node:AccessibilityNodeInfo?) {if(node==null)return;if(node.isVisibleToUser && node.className?.toString()=="android.widget.EditText") fields.add(node);for(i in 0 until node.childCount) collect(node.getChild(i))}
        collect(inst.uiAutomation.rootInActiveWindow)
        assertTrue("No editable composer",fields.isNotEmpty())
        val changed=fields.last().performAction(AccessibilityNodeInfo.ACTION_SET_TEXT,Bundle().apply {putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE,value)})
        if(!changed) {screenshot("field-failed");File(inst.targetContext.filesDir,"qa183-field-failure.json").writeText(JSONObject().put("text",fields.last().text).put("enabled",fields.last().isEnabled).put("actions",fields.last().actionList.map {it.id}.joinToString()).toString())}
        assertTrue("Composer set-text unavailable",changed)
        Thread.sleep(200)
    }
    @Test fun drawerSwitchesKeepDraftsAndSearchVoiceAndPreviewReturnToTheSameConversation() {
        assumeTrue(InstrumentationRegistry.getArguments().getString("journeyStage")=="ui")
        LocalStore(inst.targetContext).use {assertNotNull("Private association setup required",it.get("qa183-association-backup"))}
        val activity=inst.startActivitySync(Intent(inst.targetContext,MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK))
        val a="Atelier · recette";val b="Mémo · recette"
        val initial=waitNode {it.contentDescription?.toString() in setOf("Ouvrir les conversations","Open conversations") || it.text?.toString()==a}
        if(initial.contentDescription?.toString() in setOf("Ouvrir les conversations","Open conversations")) tap("Ouvrir les conversations","Open conversations")
        tap(a)
        editLastField("Brouillon Atelier")
        screenshot("draft-a")
        tap("Ouvrir les conversations","Open conversations")
        waitNode {it.text?.toString()?.contains("Réponse attendue")==true || it.text?.toString()?.contains("Answer requested")==true}
        screenshot("drawer-question")
        tap(b);editLastField("Brouillon Memo")
        for(i in 0 until 10) {
            tap("Ouvrir les conversations","Open conversations");tap(if(i%2==0) a else b)
            val draft=if(i%2==0) "Brouillon Atelier" else "Brouillon Memo"
            waitNode {it.className?.toString()=="android.widget.EditText" && it.text?.toString()==draft}
        }
        tap("Ouvrir les conversations","Open conversations");tap(a)
        screenshot("switches-draft-restored")
        tap("Copier","Copy")
        val clipboard=inst.targetContext.getSystemService(android.content.ClipboardManager::class.java)
        val copied=clipboard.primaryClip?.getItemAt(0)?.text?.toString()
        assertEquals("message = 'Bonjour Bridget'\nprint(message)\n",copied)
        screenshot("exact-code-copied")
        tap("Rechercher dans ce fil","Search this conversation")
        // Le premier champ est celui de recherche ; le composeur reste le dernier.
        val search=waitNode {it.className?.toString()=="android.widget.EditText" && it.text?.toString()!="Brouillon Atelier"}
        assertTrue(search.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT,Bundle().apply {putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE,"Repère183")}))
        waitNode {it.text?.toString()?.contains("résultats")==true || it.text?.toString()?.contains("results")==true}
        tap("Suivant","Next");screenshot("search-result")
        tap("Rechercher dans ce fil","Search this conversation")
        val before=LocalStore(inst.targetContext).use {it.list("command:").size}
        val observed=java.util.concurrent.atomic.AtomicBoolean(false)
        val monitor=object:Instrumentation.ActivityMonitor() {
            override fun onStartActivity(intent:Intent):Instrumentation.ActivityResult? {
                if(intent.action!=RecognizerIntent.ACTION_RECOGNIZE_SPEECH) return null
                observed.set(true)
                return Instrumentation.ActivityResult(Activity.RESULT_OK,Intent().putStringArrayListExtra(RecognizerIntent.EXTRA_RESULTS,arrayListOf("complément dicté")))
            }
        }
        inst.addMonitor(monitor)
        try {tap("Dicter un brouillon","Dictate a draft");waitNode {it.text?.toString()=="Brouillon Atelier complément dicté"}}
        finally {inst.removeMonitor(monitor);File(inst.targetContext.filesDir,"qa183-voice-runtime.json").writeText(JSONObject().put("intent_observed",observed.get()).toString())}
        assertEquals("Dictation must not enqueue",before,LocalStore(inst.targetContext).use {it.list("command:").size})
        screenshot("dictated-editable")
        // La recherche suspend le suivi du bas ; le bouton natif le reprend.
        val latest=find(inst.uiAutomation.rootInActiveWindow) {it.text?.toString()?.contains("Derniers messages")==true || it.text?.toString()?.contains("Latest messages")==true}
        if(latest!=null) {var button=latest;while(!button!!.isClickable && button.parent!=null) button=button.parent;button.performAction(AccessibilityNodeInfo.ACTION_CLICK);Thread.sleep(800)}
        tap("Aperçu HTML","HTML preview")
        tap("Interactif","Interactive")
        verifyPreviewGeometry();screenshot("interactive-preview")
        tap("Statique","Static");screenshot("static-preview")
        tap("Code");screenshot("source-preview")
        tap("Fermer","Dismiss")
        waitNode {it.className?.toString()=="android.widget.EditText" && it.text?.toString()=="Brouillon Atelier complément dicté"}
        screenshot("returned-draft")
        File(inst.targetContext.filesDir,"qa183-ui-result.json").writeText(JSONObject().put("switches",10).put("drafts",true).put("search",true).put("controlled_dictation",true).put("automatic_send",false).put("preview_geometry",true).put("returned_draft",true).toString())
    }
    @Test fun previewModesCloseToTheSameDraft() {
        assumeTrue(InstrumentationRegistry.getArguments().getString("journeyStage")=="preview")
        LocalStore(inst.targetContext).use {assertNotNull(it.get("qa183-association-backup"))}
        inst.startActivitySync(Intent(inst.targetContext,MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK))
        val initial=waitNode {it.contentDescription?.toString() in setOf("Ouvrir les conversations","Open conversations") || it.text?.toString()=="Atelier · recette"}
        if(initial.contentDescription?.toString() in setOf("Ouvrir les conversations","Open conversations")) tap("Ouvrir les conversations","Open conversations")
        tap("Atelier · recette")
        editLastField("Brouillon à retrouver")
        val latest=find(inst.uiAutomation.rootInActiveWindow) {it.text?.toString()?.contains("Derniers messages")==true || it.text?.toString()?.contains("Latest messages")==true}
        if(latest!=null) {var node=latest;while(!node!!.isClickable && node.parent!=null) node=node.parent;node.performAction(AccessibilityNodeInfo.ACTION_CLICK)}
        tap("Aperçu HTML","HTML preview")
        verifyPreviewGeometry();screenshot("interactive")
        tap("Statique","Static");screenshot("static")
        tap("Code");screenshot("source")
        tap("Fermer","Dismiss")
        waitNode {it.text?.toString()=="Brouillon à retrouver"};screenshot("returned")
    }
    @Test fun prepareAssociationThroughTheNormalClient() = kotlinx.coroutines.runBlocking {
        assumeTrue(InstrumentationRegistry.getArguments().getString("journeyStage")=="prepare")
        val context=InstrumentationRegistry.getInstrumentation().targetContext
        val file=File(context.filesDir,"qa183-fixture-private.json")
        assertTrue("Explicit private native fixture required",file.isFile)
        val config=JSONObject(file.readText())
        LocalStore(context).use {store ->
            check(store.get("qa183-association-backup")==null) {"Previous association backup must be restored first"}
            val backup=JSONObject()
            for(key in listOf("session","pairing","renewal")) backup.put(key,store.get(key) ?: JSONObject.NULL)
            store.put("qa183-association-backup",backup)
            try {
                for(key in listOf("session","pairing","renewal")) store.remove(key)
                BridgetClient(store).pair(config.getJSONObject("pairing").toString())
                val agents=BridgetClient(store).read("/v1/agents?limit=50").getJSONArray("items")
                assertEquals(2,agents.length())
                File(context.filesDir,"qa183-session-private.json").writeText(store.get("session")!!.toString())
                File(context.filesDir,"qa183-setup-result.json").writeText(JSONObject().put("paired",true).put("agents",agents.length()).toString())
            } catch(error:Throwable) {
                for(key in listOf("session","pairing","renewal")) {if(backup.isNull(key)) store.remove(key) else store.put(key,backup.getJSONObject(key))}
                store.remove("qa183-association-backup");throw error
            }
        }
    }
    @Test fun restoreOriginalAssociation() {
        assumeTrue(InstrumentationRegistry.getArguments().getString("journeyStage")=="restore")
        val context=InstrumentationRegistry.getInstrumentation().targetContext
        LocalStore(context).use {store ->
            val backup=store.get("qa183-association-backup") ?: error("Association backup missing")
            for(key in listOf("session","pairing","renewal")) {if(backup.isNull(key)) store.remove(key) else store.put(key,backup.getJSONObject(key))}
            store.remove("qa183-association-backup")
        }
        File(context.filesDir,"qa183-session-private.json").delete()
        File(context.filesDir,"qa183-fixture-private.json").delete()
    }
}
