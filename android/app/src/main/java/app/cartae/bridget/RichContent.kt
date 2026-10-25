package app.cartae.bridget

import android.graphics.Typeface
import android.text.SpannableString
import android.text.Spanned
import android.text.style.ForegroundColorSpan
import android.text.style.StyleSpan
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient
import android.webkit.WebChromeClient
import android.webkit.PermissionRequest
import android.webkit.GeolocationPermissions
import android.webkit.ValueCallback
import android.webkit.WebSettings
import android.net.Uri
import android.content.ClipData
import android.content.ClipboardManager
import android.widget.TextView
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.ui.window.Dialog
import androidx.lifecycle.compose.LifecycleResumeEffect
import androidx.compose.ui.window.DialogProperties
import dev.snipme.highlights.Highlights
import dev.snipme.highlights.model.BoldHighlight
import dev.snipme.highlights.model.ColorHighlight
import dev.snipme.highlights.model.SyntaxLanguage
import dev.snipme.highlights.model.SyntaxThemes
import io.noties.markwon.AbstractMarkwonPlugin
import io.noties.markwon.Markwon
import io.noties.markwon.MarkwonConfiguration
import io.noties.markwon.ext.tasklist.TaskListPlugin
import io.noties.markwon.ext.strikethrough.StrikethroughPlugin
import io.noties.markwon.ext.tables.TablePlugin
import io.noties.markwon.syntax.SyntaxHighlight
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.commonmark.node.FencedCodeBlock
import java.io.ByteArrayInputStream

internal fun syntaxLanguage(info: String?): SyntaxLanguage? = when(info?.trim()?.substringBefore(' ')?.lowercase()) {
    "sh","bash","zsh","shell","console" -> SyntaxLanguage.SHELL
    "py","python" -> SyntaxLanguage.PYTHON
    "js","javascript","json","jsonc" -> SyntaxLanguage.JAVASCRIPT
    "ts","typescript","tsx" -> SyntaxLanguage.TYPESCRIPT
    "rs","rust" -> SyntaxLanguage.RUST
    "kt","kotlin","kts" -> SyntaxLanguage.KOTLIN
    "c++","cpp" -> SyntaxLanguage.CPP
    "c#","csharp" -> SyntaxLanguage.CSHARP
    else -> info?.let(SyntaxLanguage::getByName)?.takeUnless { it==SyntaxLanguage.DEFAULT }
}

/** La taille colorée est bornée ; les textes longs restent complets en texte simple. */
private fun coloredCode(code: String, info: String?, dark: Boolean): CharSequence {
    val lang=syntaxLanguage(info) ?: return code
    if(code.length>32_768) return code
    val theme=if(dark) SyntaxThemes.darcula(true) else SyntaxThemes.darcula(false)
    val result=SpannableString(code)
    val highlights=Highlights.Builder().code(code).language(lang).theme(theme).build().getHighlights()
    for(span in highlights) {
        val start=span.location.start.coerceIn(0,code.length)
        val end=span.location.end.coerceIn(start,code.length)
        when(span) {
            is ColorHighlight -> result.setSpan(ForegroundColorSpan(0xff000000.toInt() or span.rgb),start,end,Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
            is BoldHighlight -> result.setSpan(StyleSpan(Typeface.BOLD),start,end,Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
        }
    }
    return result
}

@Composable
internal fun RichMarkdown(text: String, modifier: Modifier=Modifier, onPreviewVisibility: (Boolean)->Unit = {}) {
    val context=LocalContext.current
    val dark=isSystemInDarkTheme()
    val color=MaterialTheme.colorScheme.onSurface.toArgb()
    val renderer=remember(context,dark) {
        Markwon.builder(context).usePlugin(TablePlugin.create(context)).usePlugin(StrikethroughPlugin.create()).usePlugin(TaskListPlugin.create(context)).usePlugin(object: AbstractMarkwonPlugin() {
            override fun configureConfiguration(builder: MarkwonConfiguration.Builder) {
                builder.syntaxHighlight(SyntaxHighlight { language,code -> coloredCode(code,language,dark) })
                builder.linkResolver { view,link ->
                    val uri=android.net.Uri.parse(link)
                    if(uri.scheme in setOf("https","http")) runCatching {
                        view.context.startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW,uri))
                    }
                }
            }
        }).build()
    }
    val blocks=remember(text,renderer) {markdownRenderBlocks(renderer.parse(text))}
    var preview by remember(text) {mutableStateOf<String?>(null)}
    Column(modifier.fillMaxWidth(),verticalArrangement=Arrangement.spacedBy(10.dp)) {
        blocks.forEach {block ->
            if(block.code!=null) {
                Surface(shape=androidx.compose.foundation.shape.RoundedCornerShape(12.dp),color=MaterialTheme.colorScheme.surfaceContainer) {
                    Column(Modifier.fillMaxWidth().padding(horizontal=12.dp)) {
                        var copied by remember(block.code) {mutableStateOf(false)}
                        Row(Modifier.fillMaxWidth(),verticalAlignment=androidx.compose.ui.Alignment.CenterVertically) {
                            Text(block.language.ifBlank {stringResource(R.string.code_label)}.take(40),modifier=Modifier.weight(1f),style=MaterialTheme.typography.labelMedium)
                            TextButton(onClick={
                                context.getSystemService(ClipboardManager::class.java).setPrimaryClip(ClipData.newPlainText("Bridget code",block.code));copied=true
                            }) {Text(stringResource(if(copied) R.string.code_copied else R.string.code_copy))}
                        }
                        Box(Modifier.fillMaxWidth().padding(bottom=12.dp)) {HighlightedCode(block.code,block.language)}
                        if(block.language.trim().lowercase() in setOf("html","htm") && block.code.toByteArray().size<=262_144)
                            OutlinedButton(onClick={preview=block.code},modifier=Modifier.fillMaxWidth().padding(bottom=8.dp)) {Text(stringResource(R.string.preview_html))}
                    }
                }
            } else block.prose?.let {node ->
                var rendered by remember(node,renderer) {mutableStateOf<CharSequence>("")}
                LaunchedEffect(node,renderer) {rendered=withContext(Dispatchers.Default) {renderer.render(node)}}
                AndroidView(modifier=Modifier.fillMaxWidth(),factory={TextView(it).apply {
                    textSize=16f;setTextIsSelectable(true);setLineSpacing(0f,1.22f)
                }},update={it.setTextColor(color);if(rendered is Spanned) renderer.setParsedMarkdown(it,rendered as Spanned) else it.text=rendered})
            }
        }
    }
    preview?.let {
        DisposableEffect(Unit) {onPreviewVisibility(true);onDispose {onPreviewVisibility(false)}}
        HtmlPreview(it) {preview=null}
    }
}

internal data class MarkdownRenderBlock(val prose: org.commonmark.node.Document?=null,val code: String?=null,val language: String="")

/** Les blocs racine restent dans leur ordre CommonMark ; listes et citations gardent leur structure. */
internal fun markdownRenderBlocks(root: org.commonmark.node.Node): List<MarkdownRenderBlock> {
    val result=mutableListOf<MarkdownRenderBlock>()
    var prose=org.commonmark.node.Document()
    var node=root.firstChild
    fun flush() {if(prose.firstChild!=null) {result.add(MarkdownRenderBlock(prose=prose));prose=org.commonmark.node.Document()}}
    while(node!=null) {
        val next=node.next
        if(node is FencedCodeBlock) {flush();result.add(MarkdownRenderBlock(code=node.literal,language=node.info.trim()))}
        else {node.unlink();prose.appendChild(node)}
        node=next
    }
    flush();return result
}

@Composable
internal fun HighlightedCode(code: String, language: String?=null) {
    val dark=isSystemInDarkTheme();val color=MaterialTheme.colorScheme.onSurface.toArgb()
    var rendered by remember(code) { mutableStateOf<CharSequence>(code) }
    LaunchedEffect(code,language,dark) { rendered=withContext(Dispatchers.Default) { coloredCode(code,language,dark) } }
    AndroidView(modifier=Modifier.fillMaxWidth(),factory={ TextView(it).apply {
        textSize=13f;typeface=Typeface.MONOSPACE;setTextIsSelectable(true)
    } },update={it.setTextColor(color);it.text=rendered})
}

internal fun isMarkdownPreview(title: String,media: String=""): Boolean = media=="text/markdown" || title.substringAfterLast('.').lowercase() in setOf("md","markdown")
internal fun isHtmlPreview(title: String,media: String=""): Boolean = media=="text/html" || title.substringAfterLast('.').lowercase() in setOf("html","htm")
internal fun isTextPreview(title: String, media: String): Boolean = media in setOf("text/html","text/markdown") || media=="text/plain" &&
    title.substringAfterLast('.').lowercase() in setOf("html","htm","md","markdown","txt","json","py","js","ts","rs","sh","kt","java","css","sql","yaml","yml")

/** Baseline WebView stable vérifiée le 25 octobre 2026 ; version inconnue refusée. */
internal fun htmlPreviewEngineAllowed(version: String?): Boolean {
    if(version==null || !Regex("[0-9]{1,6}(\\.[0-9]{1,6}){3}").matches(version)) return false
    val parts=version.split('.').map(String::toInt)
    val minimum=listOf(156,0,8078,48)
    for(i in parts.indices) if(parts[i]!=minimum[i]) return parts[i]>minimum[i]
    return true
}

@Composable
internal fun HtmlPreview(html: String,onClose: () -> Unit) {
    val context=LocalContext.current
    var engineVersion by remember { mutableStateOf<String?>(null) }
    var updateFailed by remember { mutableStateOf(false) }
    var mode by rememberSaveable {mutableIntStateOf(0)}
    var rendererFailed by remember {mutableStateOf(false)}
    LifecycleResumeEffect(Unit) {
        engineVersion=runCatching { WebView.getCurrentWebViewPackage()?.versionName }.getOrNull()
        onPauseOrDispose { }
    }

    Dialog(onDismissRequest=onClose,properties=DialogProperties(usePlatformDefaultWidth=false)) {
        Surface(Modifier.fillMaxSize()) { Column(Modifier.fillMaxSize().statusBarsPadding().navigationBarsPadding()) {
            Row(Modifier.fillMaxWidth().padding(horizontal=12.dp),horizontalArrangement=Arrangement.SpaceBetween) {
                Text(stringResource(R.string.preview_html),modifier=Modifier.padding(vertical=16.dp))
                TextButton(onClick=onClose) { Text(stringResource(R.string.dismiss)) }
            }
            Row(Modifier.fillMaxWidth().padding(horizontal=8.dp),horizontalArrangement=Arrangement.spacedBy(6.dp)) {
                listOf(R.string.preview_interactive,R.string.preview_static,R.string.code_label).forEachIndexed {index,label ->
                    FilterChip(selected=mode==index,onClick={mode=index;rendererFailed=false},label={Text(stringResource(label))})
                }
            }
            Text(stringResource(if(mode==0) R.string.preview_html_isolated else R.string.preview_html_static),modifier=Modifier.padding(horizontal=12.dp),style=MaterialTheme.typography.labelSmall)
            if(mode==2 || rendererFailed || !htmlPreviewEngineAllowed(engineVersion)) {
                if(rendererFailed) Text(stringResource(R.string.preview_failed),Modifier.padding(16.dp))
                if(mode!=2 && !htmlPreviewEngineAllowed(engineVersion)) {
                Text(stringResource(R.string.preview_html_update_required),modifier=Modifier.padding(24.dp))
                TextButton(onClick={
                    updateFailed=runCatching { context.startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW,
                        android.net.Uri.parse("https://play.google.com/store/apps/details?id=com.google.android.webview"))) }.isFailure
                }) { Text(stringResource(R.string.preview_html_update_action)) }
                if(updateFailed) Text(stringResource(R.string.preview_html_update_manual),modifier=Modifier.padding(24.dp))
                }
                Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(16.dp)) {
                    HighlightedCode(html,"html")
                }
            } else key(mode) {
                AndroidView(modifier=Modifier.weight(1f).fillMaxWidth(),factory={context ->
                    isolatedPreviewWebView(context,mode==0) {rendererFailed=true}
                },update={view -> if(view.tag!=html) {
                    view.tag=html
                    view.loadDataWithBaseURL("https://preview.invalid/",if(mode==0) isolatedHtmlPreviewDocument(html) else htmlPreviewDocument(html),"text/html","UTF-8",null)
                }},onRelease={it.stopLoading();it.destroy()})
            }
        } }
    }
}

internal fun htmlPreviewDocument(html: String): String {
    require(html.toByteArray().size<=262_144)
    return "<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src data:; base-uri 'none'; form-action 'none'; frame-src 'none'\"><style>body{font-family:system-ui;margin:16px;overflow-wrap:anywhere}img,svg{max-width:100%;height:auto}pre{overflow:auto}</style></head><body>"+html+"</body></html>"
}

/** Le document non fiable vit dans une origine opaque ; aucune capacité native n'est ajoutée. */
internal fun isolatedHtmlPreviewDocument(html: String): String {
    require(html.toByteArray(Charsets.UTF_8).size<=262_144)
    val child="""<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; img-src data:; connect-src 'none'; media-src data:; font-src 'none'; frame-src 'none'; worker-src 'none'; base-uri 'none'; form-action 'none'"><style>body{font-family:system-ui;margin:16px;overflow-wrap:anywhere}img,svg{max-width:100%;height:auto}pre{overflow:auto}</style></head><body>"""+html+"</body></html>"
    val escaped=child.replace("&","&amp;").replace("\"","&quot;").replace("<","&lt;").replace(">","&gt;")
    return """<!doctype html><html><head><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; frame-src 'self' about:; script-src 'unsafe-inline'; style-src 'unsafe-inline'; img-src data:; connect-src 'none'; base-uri 'none'; form-action 'none'"><style>html,body,iframe{margin:0;width:100%;height:100%;border:0;overflow:hidden}iframe{display:block}</style></head><body><iframe title="Bridget HTML" sandbox="allow-scripts" referrerpolicy="no-referrer" srcdoc=""""+escaped+""""></iframe></body></html>"""
}

/** Cette vue ne reçoit ni session, ni pont natif, ni gestionnaire de message privilégié. */
internal fun isolatedPreviewWebView(context: android.content.Context, interactive: Boolean, failed: () -> Unit = {}): WebView = WebView(context).apply {
    // WRAP_CONTENT force Chromium à calculer une hauteur CSS nulle pour un cadre plein écran.
    layoutParams=android.view.ViewGroup.LayoutParams(android.view.ViewGroup.LayoutParams.MATCH_PARENT,android.view.ViewGroup.LayoutParams.MATCH_PARENT)
    settings.javaScriptEnabled=interactive
    settings.allowFileAccess=false;settings.allowContentAccess=false
    settings.domStorageEnabled=false;settings.databaseEnabled=false
    settings.blockNetworkLoads=true;settings.cacheMode=WebSettings.LOAD_NO_CACHE
    settings.mixedContentMode=WebSettings.MIXED_CONTENT_NEVER_ALLOW
    settings.javaScriptCanOpenWindowsAutomatically=false;settings.setSupportMultipleWindows(true)
    settings.mediaPlaybackRequiresUserGesture=true
    android.webkit.CookieManager.getInstance().setAcceptThirdPartyCookies(this,false)
    removeJavascriptInterface("searchBoxJavaBridge_");removeJavascriptInterface("accessibility");removeJavascriptInterface("accessibilityTraversal")
    setDownloadListener {_,_,_,_,_-> }
    webChromeClient=object: WebChromeClient() {
        override fun onPermissionRequest(request: PermissionRequest) {request.deny()}
        override fun onGeolocationPermissionsShowPrompt(origin: String,callback: GeolocationPermissions.Callback) {callback.invoke(origin,false,false)}
        override fun onShowFileChooser(view: WebView,callback: ValueCallback<Array<Uri>>,params: FileChooserParams): Boolean {callback.onReceiveValue(null);return true}
        override fun onCreateWindow(view: WebView,isDialog: Boolean,isUserGesture: Boolean,result: android.os.Message)=false
        override fun onJsAlert(view: WebView,url: String,message: String,result: android.webkit.JsResult): Boolean {result.cancel();return true}
        override fun onJsConfirm(view: WebView,url: String,message: String,result: android.webkit.JsResult): Boolean {result.cancel();return true}
        override fun onJsPrompt(view: WebView,url: String,message: String,defaultValue: String,result: android.webkit.JsPromptResult): Boolean {result.cancel();return true}
    }
    webViewClient=object: WebViewClient() {
        override fun shouldOverrideUrlLoading(view: WebView,request: WebResourceRequest)=true
        override fun shouldInterceptRequest(view: WebView,request: WebResourceRequest): WebResourceResponse =
            WebResourceResponse("text/plain","UTF-8",ByteArrayInputStream(byteArrayOf()))
        override fun onRenderProcessGone(view: WebView,detail: android.webkit.RenderProcessGoneDetail): Boolean {failed();return true}
    }
}
