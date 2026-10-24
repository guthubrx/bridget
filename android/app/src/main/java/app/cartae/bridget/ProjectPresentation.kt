package app.cartae.bridget

import java.security.MessageDigest

internal data class ProjectIdentity(val host: String, val root: String)
private fun projectPath(root: String): String = root.trimEnd('/').removeSuffix("/.git")
internal fun projectTitle(root: String): String = projectPath(root).substringAfterLast('/').removeSuffix(".git").ifEmpty { "/" }
internal fun projectQualifier(project: ProjectIdentity, peers: List<ProjectIdentity>): String {
    if (peers.distinct().count { projectTitle(it.root) == projectTitle(project.root) } <= 1) return ""
    val parent=projectPath(project.root).substringBeforeLast('/', "").substringAfterLast('/')
    val digest=MessageDigest.getInstance("SHA-256").digest((project.host+"\u0000"+project.root).toByteArray())
        .take(3).joinToString("") { "%02x".format(it.toInt() and 255) }
    return listOf(project.host,parent,digest).filter { it.isNotEmpty() }.joinToString(" · ")
}
internal fun projectMonogram(root: String): String = projectTitle(root).replace(Regex("^[0-9]+[._-]"), "").take(2).uppercase()
