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
    @Test fun htmlBlocksUseTheSameParsedTreeAsTheVisibleContent() {
        val parser=org.commonmark.parser.Parser.builder().build()
        val blocks=markdownRenderBlocks(parser.parse("Avant\n\n```html\n<h1>Bonjour</h1>\n```\n\n```python\nprint('x')\n```"))
        assertEquals(listOf("<h1>Bonjour</h1>\n"),blocks.filter {it.language=="html"}.map {it.code})
        assertTrue(markdownRenderBlocks(parser.parse("Du texte qui cite `html`.")).none {it.code!=null})
        val pages=(1..10).joinToString("\n\n") {"```html\n<p>$it</p>\n```"}
        assertEquals(10,markdownRenderBlocks(parser.parse(pages)).count {it.language=="html"})
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

    @Test fun interactiveDocumentCannotEscapeItsOpaqueSandboxOrAcquireNetwork() {
        val hostile="\" ></iframe><script>parent.document.body.textContent='bad'</script>"
        val wrapped=isolatedHtmlPreviewDocument(hostile)
        assertTrue(wrapped.contains("sandbox=\"allow-scripts\""))
        assertFalse(wrapped.contains("allow-same-origin"))
        assertFalse(wrapped.contains("</iframe><script>"))
        assertTrue(wrapped.contains("&lt;/iframe&gt;"))
        assertTrue(wrapped.contains("connect-src"))
        assertThrows(IllegalArgumentException::class.java) {isolatedHtmlPreviewDocument("x".repeat(262145))}
    }
    @Test fun codeBlocksKeepTheirExactBytesAndProseOrderWithoutDuplication() {
        val root=org.commonmark.parser.Parser.builder().build().parse("Avant\n\n```python\nprint('é')  \n\n```\n\nAprès\n")
        val blocks=markdownRenderBlocks(root)
        assertEquals(3,blocks.size)
        assertNotNull(blocks[0].prose);assertNotNull(blocks[2].prose)
        assertEquals("python",blocks[1].language)
        assertEquals("print('é')  \n\n",blocks[1].code)
        assertNull(blocks[1].prose)
    }
}
