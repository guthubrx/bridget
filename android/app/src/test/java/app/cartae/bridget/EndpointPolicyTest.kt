package app.cartae.bridget
import org.junit.Assert.assertEquals
import org.junit.Test
class EndpointPolicyTest {
 @Test fun acceptsDedicatedHttpsOrigin() {
  assertEquals("https://bridget.example.net", EndpointPolicy.requireEndpoint("https://bridget.example.net"))
 }
 @Test fun refusesCredentialsPathsRedirectTargetsAndCleartext() {
  listOf("http://bridget.example.net", "https://user:secret@bridget.example.net",
   "https://bridget.example.net/pair", "https://bridget.example.net?token=secret",
   "https://bridget.example.net#secret", "https://bridget.example.net:8443",
   "https://localhost", "https://127.0.0.1", "https://-bad.example.net").forEach {
   check(runCatching { EndpointPolicy.requireEndpoint(it) }.isFailure) { "endpoint accepted" }
  }
 }
}
