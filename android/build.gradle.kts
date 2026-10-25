buildscript {
 repositories { google(); mavenCentral() }
 dependencies { classpath("com.android.tools:r8:9.5.28") }
 // Patch plugin libraries before their classpath is resolved.
 configurations.configureEach {
  resolutionStrategy.activateDependencyLocking()
  resolutionStrategy.eachDependency {
   when {
    requested.group == "com.google.crypto.tink" && requested.name == "tink" -> useVersion("1.15.0")
    requested.group == "org.bouncycastle" -> useVersion("1.86")
    requested.group == "org.apache.commons" && requested.name == "commons-compress" -> useVersion("1.28.0")
    requested.group == "org.jdom" && requested.name == "jdom2" -> useVersion("2.0.6.1")
    requested.group == "org.bitbucket.b_c" && requested.name == "jose4j" -> useVersion("0.9.6")
    requested.group == "io.netty" -> useVersion("4.1.139.Final")
    requested.group == "com.google.protobuf" && requested.name != "protobuf-javalite" -> useVersion("3.25.8")
   }
  }
 }
}
plugins {
 id("com.android.application") version "8.9.2" apply false
 id("org.jetbrains.kotlin.android") version "2.4.21" apply false
 id("org.jetbrains.kotlin.plugin.compose") version "2.4.21" apply false
}
