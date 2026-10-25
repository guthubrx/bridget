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
import android.widget.TextView
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
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
import org.commonmark.node.AbstractVisitor
import org.commonmark.node.FencedCodeBlock
import org.commonmark.parser.Parser
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
internal fun RichMarkdown(text: String, modifier: Modifier=Modifier) {
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
    var rendered by remember(text) { mutableStateOf<CharSequence>(text) }
    LaunchedEffect(text,renderer) { rendered=withContext(Dispatchers.Default) { renderer.toMarkdown(text) } }
    AndroidView(modifier=modifier.fillMaxWidth(),factory={ TextView(it).apply {
        textSize=16f; setTextIsSelectable(true); setLineSpacing(0f,1.22f)
    } },update={ it.setTextColor(color); if(rendered is Spanned) renderer.setParsedMarkdown(it,rendered as Spanned) else it.text=rendered })
    val previews=remember(text) { htmlCodeBlocks(text) }
    var preview by remember(text) { mutableStateOf<String?>(null) }
    previews.forEachIndexed { index,html ->
        TextButton(onClick={preview=html}) { Text(stringResource(R.string.preview_html)+(if(previews.size>1) " ${index+1}" else "")) }
    }
    preview?.let { HtmlPreview(it) { preview=null } }
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

/** Parser CommonMark partagé, au plus quatre aperçus de 256 Kio, sans accès réseau. */
internal fun htmlCodeBlocks(text: String): List<String> {
    val result=mutableListOf<String>()
    Parser.builder().build().parse(text).accept(object: AbstractVisitor() {
        override fun visit(block: FencedCodeBlock) {
            if(result.size<4 && block.info.trim().lowercase() in setOf("html","htm") && block.literal.toByteArray().size<=262_144) result+=block.literal
        }
    })
    return result
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
            Text(stringResource(R.string.preview_html_static),modifier=Modifier.padding(horizontal=12.dp),style=MaterialTheme.typography.labelSmall)
            if(!htmlPreviewEngineAllowed(engineVersion)) {
                Text(stringResource(R.string.preview_html_update_required),modifier=Modifier.padding(24.dp))
                TextButton(onClick={
                    updateFailed=runCatching { context.startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW,
                        android.net.Uri.parse("https://play.google.com/store/apps/details?id=com.google.android.webview"))) }.isFailure
                }) { Text(stringResource(R.string.preview_html_update_action)) }
                if(updateFailed) Text(stringResource(R.string.preview_html_update_manual),modifier=Modifier.padding(24.dp))
                Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(16.dp)) {
                    HighlightedCode(html,"html")
                }
            } else AndroidView(modifier=Modifier.weight(1f).fillMaxWidth(),factory={ context -> WebView(context).apply {
                settings.javaScriptEnabled=false;settings.allowFileAccess=false;settings.allowContentAccess=false
                settings.domStorageEnabled=false;settings.blockNetworkLoads=true;settings.mixedContentMode=android.webkit.WebSettings.MIXED_CONTENT_NEVER_ALLOW
                webViewClient=object: WebViewClient() {
                    override fun shouldOverrideUrlLoading(view: WebView,request: WebResourceRequest)=true
                    override fun shouldInterceptRequest(view: WebView,request: WebResourceRequest): WebResourceResponse =
                        WebResourceResponse("text/plain","UTF-8",ByteArrayInputStream(byteArrayOf()))
                }
            } },update={ view -> if(view.tag!=html) {
                view.tag=html
                view.loadDataWithBaseURL("https://preview.invalid/",htmlPreviewDocument(html),"text/html","UTF-8",null)
            } },onRelease={it.stopLoading();it.destroy()})
        } }
    }
}

internal fun htmlPreviewDocument(html: String): String {
    require(html.toByteArray().size<=262_144)
    return "<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src data:; base-uri 'none'; form-action 'none'; frame-src 'none'\"><style>body{font-family:system-ui;margin:16px;overflow-wrap:anywhere}img,svg{max-width:100%;height:auto}pre{overflow:auto}</style></head><body>"+html+"</body></html>"
}
