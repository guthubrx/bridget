package app.cartae.bridget

import android.content.Intent
import androidx.activity.compose.setContent
import androidx.compose.runtime.mutableStateOf
import android.view.accessibility.AccessibilityNodeInfo
import android.os.SystemClock
import android.view.MotionEvent
import android.view.ViewGroup
import android.webkit.WebView
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.json.JSONTokener
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

@RunWith(AndroidJUnit4::class)
class HtmlIsolationTest {
    private val inst=InstrumentationRegistry.getInstrumentation()
    private fun evaluate(view: WebView, script: String): String {
        val result=AtomicReference<String>();val done=CountDownLatch(1)
        inst.runOnMainSync {view.evaluateJavascript(script) {result.set(it);done.countDown()}}
        assertTrue("WebView evaluation timed out",done.await(5,TimeUnit.SECONDS))
        return result.get()
    }
    private fun probe(view: WebView, predicate: (JSONObject)->Boolean): JSONObject {
        val until=SystemClock.uptimeMillis()+12_000
        var last=JSONObject()
        while(SystemClock.uptimeMillis()<until) {
            val raw=evaluate(view,"JSON.stringify(window.__probe || null)")
            val value=JSONTokener(raw).nextValue()
            if(value is String && value!="null") last=JSONObject(value)
            if(predicate(last)) return last
            Thread.sleep(100)
        }
        error("Probe was not reached: $last")
    }
    @Test fun interactiveCounterWorksWhileParentStorageNativeAndNetworkAccessAreDenied() {
        val context=inst.targetContext
        assertTrue("Patched WebView required for qualification",htmlPreviewEngineAllowed(WebView.getCurrentWebViewPackage()?.versionName))
        val activity=inst.startActivitySync(Intent(context,MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        lateinit var view:WebView
        val child="""<button id="counter" style="position:fixed;left:16px;top:16px;width:160px;height:64px">0</button><script>
const r={count:0};
function report(){parent.postMessage(r,'*')}
function blocked(name,fn){try{fn();r[name]=false}catch(e){r[name]=true}}
blocked('parent',()=>parent.document.body.textContent='escape');
blocked('storage',()=>localStorage.setItem('secret','escape'));
r.native=typeof Android==='undefined' && typeof bridget==='undefined';r.opaque=self.origin==='null';
r.popup=window.open('https://blocked.invalid/popup')===null;
blocked('session_storage',()=>sessionStorage.setItem('secret','escape'));
blocked('cookies',()=>document.cookie='qa183=escape');
try{const ws=new WebSocket('wss://blocked.invalid/socket');ws.onopen=()=>{r.websocket=false;report()};ws.onerror=()=>{r.websocket=true;report()}}catch(e){r.websocket=true}
try{top.location='https://blocked.invalid/navigation'}catch(e){};
const download=document.createElement('a');download.download='qa183-escape.txt';download.href='data:text/plain,escape';download.click();
alert('must not block');r.dialog=prompt('must not block')===null;
const urls={network:'https://blocked.invalid/network',file:'file:///data/data/app.cartae.bridget/files/session',content:'content://settings/system'};
Promise.all(Object.entries(urls).map(async([name,url])=>{try{await fetch(url);r[name]=false}catch(e){r[name]=true}})).then(report);
try{const w=new Worker('data:text/javascript,postMessage(1)');w.onmessage=()=>{r.worker=false;report()};w.onerror=()=>{r.worker=true;report()}}catch(e){r.worker=true}
if(navigator.mediaDevices) navigator.mediaDevices.getUserMedia({audio:true}).then(()=>{r.permission=false;report()},()=>{r.permission=true;report()});else r.permission=true;
if(navigator.mediaDevices) navigator.mediaDevices.getUserMedia({video:true}).then(()=>{r.camera=false;report()},()=>{r.camera=true;report()});else r.camera=true;
document.getElementById('counter').onclick=()=>{r.count++;document.getElementById('counter').textContent=r.count;report()};
setInterval(report,100);
</script>"""
        try {
            inst.runOnMainSync {
                view=isolatedPreviewWebView(activity,true)
                activity.addContentView(view,ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT,ViewGroup.LayoutParams.MATCH_PARENT))
                view.loadDataWithBaseURL("https://preview.invalid/",isolatedHtmlPreviewDocument(child),"text/html","UTF-8",null)
            }
            // Cet observateur n'existe que dans le test. Il ne fournit aucune capacité à l'enfant.
            val until=SystemClock.uptimeMillis()+5000
            while(SystemClock.uptimeMillis()<until && evaluate(view,"document.readyState")!="\"complete\"") Thread.sleep(100)
            val geometry=JSONObject(JSONTokener(evaluate(view,"JSON.stringify({height:innerHeight,frame:document.querySelector('iframe').getBoundingClientRect().height})")).nextValue() as String)
            assertTrue("Opaque frame must fill a visible viewport: $geometry",geometry.getDouble("frame")>=geometry.getDouble("height")-1 && geometry.getDouble("height")>0)
            evaluate(view,"window.addEventListener('message',e=>{window.__probe=e.data});true")
            val initial=probe(view) {it.has("network") && it.has("permission") && it.has("camera") && it.has("worker") && it.has("websocket")}
            for(key in listOf("parent","storage","native","opaque","popup","network","file","content","worker","permission","camera","session_storage","cookies","websocket","dialog"))
                assertTrue("Capability was not denied: $key — $initial",initial.optBoolean(key))
            assertEquals(0,initial.getInt("count"))
            val density=context.resources.displayMetrics.density
            inst.runOnMainSync {
                val now=SystemClock.uptimeMillis()
                for(action in listOf(MotionEvent.ACTION_DOWN,MotionEvent.ACTION_UP)) {
                    val event=MotionEvent.obtain(now,now+50,action,80*density,45*density,0)
                    view.dispatchTouchEvent(event);event.recycle()
                }
            }
            val clicked=probe(view) {it.optInt("count")==1}
            assertEquals(1,clicked.getInt("count"))
            assertEquals("\"https://preview.invalid/\"",evaluate(view,"location.href"))
            assertEquals("true",evaluate(view,"(()=>{try{document.querySelector('iframe').contentWindow.document;return false}catch(e){return true}})()"))
            File(context.filesDir,"qa183-html-isolation.json").writeText(clicked.toString())
            inst.uiAutomation.takeScreenshot()?.let {bitmap->File(context.filesDir,"qa183-html-isolation.png").outputStream().use {bitmap.compress(android.graphics.Bitmap.CompressFormat.PNG,100,it)}}
        } finally {inst.runOnMainSync {runCatching {view}.getOrNull()?.let {web -> (web.parent as? ViewGroup)?.removeView(web);web.stopLoading();web.destroy()}}}
    }
    @Test fun aScriptFailureDoesNotOwnTheNativeCloseControl() {
        val activity=inst.startActivitySync(Intent(inst.targetContext,MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)) as MainActivity
        val open=mutableStateOf(true)
        fun find(node:AccessibilityNodeInfo?):AccessibilityNodeInfo? {
            if(node==null)return null
            if(node.text?.toString() in setOf("Fermer","Dismiss")) return node
            for(i in 0 until node.childCount) find(node.getChild(i))?.let {return it}
            return null
        }
        try {
            inst.runOnMainSync {activity.setContent {androidx.compose.material3.MaterialTheme {
                if(open.value) HtmlPreview("<h1>Page de recette en erreur</h1><script>throw new Error('qa183 failure')</script>") {open.value=false}
                else androidx.compose.material3.Text("Preview closed")
            }}}
            val until=SystemClock.uptimeMillis()+8000
            var close:AccessibilityNodeInfo?=null
            while(SystemClock.uptimeMillis()<until && close==null) {close=find(inst.uiAutomation.rootInActiveWindow);Thread.sleep(100)}
            assertNotNull("Native close control unavailable",close)
            var target=close!!
            while(!target.isClickable && target.parent!=null) target=target.parent
            assertTrue(target.performAction(AccessibilityNodeInfo.ACTION_CLICK))
            inst.waitForIdleSync()
            assertFalse("Failed page prevented closing",open.value)
        } finally {inst.runOnMainSync {activity.finish()}}
    }
}
