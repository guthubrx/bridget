package app.cartae.bridget

import org.junit.Assert.*
import org.junit.Test

class ProjectPresentationTest {
    @Test fun uniqueNamesStayShortWithoutChangingTheirIdentity() {
        val p=ProjectIdentity("mac", "/Users/test/projects/64.bridget")
        assertEquals("64.bridget",projectTitle(p.root))
        assertEquals("BR",projectMonogram(p.root))
        assertEquals("",projectQualifier(p,listOf(p,p)))
        assertEquals("/Users/test/projects/64.bridget",p.root)
    }
    @Test fun sameNamesOnDifferentHostsAndParentsAreDistinguished() {
        val a=ProjectIdentity("mac","/one/projects/app")
        val b=ProjectIdentity("mac","/two/projects/app")
        val c=ProjectIdentity("server","/one/projects/app")
        val all=listOf(a,b,c)
        val labels=all.map { projectQualifier(it,all) }
        assertEquals(3,labels.toSet().size)
        assertTrue(labels.all { it.isNotBlank() })
        assertEquals(labels.first(),projectQualifier(a,all.reversed()))
        assertFalse(labels.first().contains("/one/"))
    }
}
