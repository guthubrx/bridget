package app.cartae.bridget
import org.junit.Assert.*
import org.junit.Test
class Updates179Test {
 @Test fun versionsAreOrderedNumericallyWithoutDowngrade() {
  assertTrue(MobileRelease.versionCode("0.10.0") > MobileRelease.versionCode("0.9.99"))
  assertTrue(MobileRelease.versionCode("1.0.0") > MobileRelease.versionCode("0.999.999"))
  for (invalid in listOf("v1.0.0","1.0","1.00.0","1.0.0-beta","1000.0.0","1.0.0/evil"))
   assertTrue(runCatching { MobileRelease.versionCode(invalid) }.isFailure)
 }
 @Test fun downloadOriginCannotCarryCredentialsOrChangeProtocol() {
  assertTrue(MobileRelease.allowedDownload("https://release-assets.githubusercontent.com/asset"))
  for (url in listOf("http://github.com/a","https://github.com.evil/a","https://user@github.com/a",
   "https://github.com:444/a","file:///a","https://example.com/a","https://github.com/a#fragment"))
   assertFalse(url, MobileRelease.allowedDownload(url))
 }
}
