import groovy.json.JsonSlurper
plugins {
 id("com.android.application")
 id("org.jetbrains.kotlin.android")
 id("org.jetbrains.kotlin.plugin.compose")
}
val firebaseConfig = System.getenv("BRIDGET_FIREBASE_CONFIG")?.let { path ->
 val file = file(path)
 require(file.isFile && file.length() <= 65536) { "Private Firebase application configuration unavailable" }
 @Suppress("UNCHECKED_CAST")
 (JsonSlurper().parse(file) as Map<String, Any>)
}
fun firebaseValue(key: String): String {
 val value = firebaseConfig?.get(key) as? String ?: ""
 require(value.length <= 512 && value.all { it.isLetterOrDigit() || it in "_-:." }) { "Invalid public Firebase option" }
 return "\"$value\""
}
val mobileVersion = rootProject.projectDir.resolve("../Cargo.toml").readLines()
 .firstOrNull { it.startsWith("version = ") }?.substringAfter('"')?.substringBefore('"')
 ?: error("Workspace release version missing")
require(mobileVersion.matches(Regex("[0-9]{1,3}[.][0-9]{1,3}[.][0-9]{1,3}")))
val mobileVersionParts = mobileVersion.split(".").map(String::toInt)
require(mobileVersionParts.all { it in 0..999 })
android {
 namespace = "app.cartae.bridget"
 compileSdk = 35
 defaultConfig {
  applicationId = "app.cartae.bridget"
  minSdk = 28
  targetSdk = 35
  versionCode = mobileVersionParts[0] * 1_000_000 + mobileVersionParts[1] * 1_000 + mobileVersionParts[2]
  versionName = mobileVersion
  buildConfigField("String", "FIREBASE_APP_ID", firebaseValue("application_id"))
  buildConfigField("String", "FIREBASE_API_KEY", firebaseValue("api_key"))
  buildConfigField("String", "FIREBASE_SENDER_ID", firebaseValue("sender_id"))
  buildConfigField("String", "FIREBASE_PROJECT_ID", firebaseValue("project_id"))
  testInstrumentationRunner = if (providers.gradleProperty("bridgetUpgradeProof").orNull == "true")
   "app.cartae.bridget.UpgradeVerificationInstrumentation" else "androidx.test.runner.AndroidJUnitRunner"
 }
 buildFeatures { compose = true; buildConfig = true }
 compileOptions { sourceCompatibility = JavaVersion.VERSION_17; targetCompatibility = JavaVersion.VERSION_17 }
 kotlinOptions { jvmTarget = "17" }
 buildTypes { release { isMinifyEnabled = true; proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt")) } }
}
dependencyLocking { lockAllConfigurations() }
dependencies {
 implementation(platform("androidx.compose:compose-bom:2025.04.01"))
 implementation("androidx.activity:activity-compose:1.10.1")
 implementation("androidx.fragment:fragment:1.8.9")
 implementation("androidx.core:core-ktx:1.16.0")
 implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.9.0")
 implementation("androidx.lifecycle:lifecycle-runtime-compose:2.9.0")
 implementation("androidx.compose.material3:material3")
 implementation("androidx.compose.ui:ui-tooling-preview")
 implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
 implementation("com.google.firebase:firebase-messaging:24.1.1")
 implementation("com.journeyapps:zxing-android-embedded:4.3.0")
 debugImplementation("androidx.compose.ui:ui-tooling")
 testImplementation("junit:junit:4.13.2")
 androidTestImplementation("androidx.test:runner:1.6.2")
 androidTestImplementation("androidx.test.ext:junit:1.2.1")
}
