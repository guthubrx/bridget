package app.cartae.bridget

import dev.snipme.highlights.model.SyntaxLanguage
import org.junit.Assert.*
import org.junit.Test

class RichContentTest {
    @Test fun commonLanguagesAndUnknownCodeRemainExplicit() {
        assertEquals(SyntaxLanguage.SHELL,syntaxLanguage("bash"))
        assertEquals(SyntaxLanguage.PYTHON,syntaxLanguage("python"))
        assertEquals(SyntaxLanguage.TYPESCRIPT,syntaxLanguage("ts"))
        assertEquals(SyntaxLanguage.RUST,syntaxLanguage("rust"))
        assertNull(syntaxLanguage("unknown-language"))
    }
    @Test fun htmlPreviewComesFromCodeBlocksNotMentionedFences() {
        assertEquals(listOf("<h1>Bonjour</h1>\n"),htmlCodeBlocks("Avant\n\n```html\n<h1>Bonjour</h1>\n```\n\n```python\nprint('x')\n```"))
        assertTrue(htmlCodeBlocks("Du texte qui cite `html`.").isEmpty())
        assertEquals(4,htmlCodeBlocks((1..10).joinToString("\n\n") { "```html\n<p>$it</p>\n```" }).size)
    }
    @Test fun previewPolicyDoesNotTrustClosingTagsOrRemoteResources() {
        val html=htmlPreviewDocument("</head><script>fetch('https://example.com')</script><img src='file:///private'>")
        assertTrue(html.indexOf("Content-Security-Policy")<html.indexOf("<script>"))
        assertTrue(html.contains("default-src 'none'"))
        assertTrue(html.contains("form-action 'none'"))
        assertTrue(html.contains("frame-src 'none'"))
    }
    @Test fun onlyBoundedTextDocumentsHavePreview() {
        assertTrue(isTextPreview("page.html","text/html"))
        assertTrue(isTextPreview("guide.md","text/markdown"))
        assertTrue(isTextPreview("page.HTML","text/plain"))
        assertFalse(isTextPreview("page.html","application/javascript"))
        assertFalse(isTextPreview("data.bin","text/plain"))
        assertThrows(IllegalArgumentException::class.java) { htmlPreviewDocument("x".repeat(262_145)) }
    }
    @Test fun oldOrUnknownWebViewCannotRenderHtml() {
        for(version in listOf(null,"","155.0.8059.43","156.0.8078.47","unknown","156.0.8078.+48","157.-1.0.0","9999999.0.0.0")) {
            assertFalse("Unsafe or unknown WebView: $version",htmlPreviewEngineAllowed(version))
        }
    }
    @Test fun patchedWebViewAndNewerBranchesCanRenderHtml() {
        for(version in listOf("156.0.8078.48","156.0.8078.49","156.0.8079.1","157.0.8103.14")) {
            assertTrue("Patched WebView: $version",htmlPreviewEngineAllowed(version))
        }
    }
}
