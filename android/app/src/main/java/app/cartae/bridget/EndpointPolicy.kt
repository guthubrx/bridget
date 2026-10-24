package app.cartae.bridget
import java.net.URI
object EndpointPolicy {
 fun requireEndpoint(value: String): String {
  val uri = URI(value)
  require(uri.scheme == "https" && uri.rawUserInfo == null && uri.port == -1 &&
   uri.rawPath.isNullOrEmpty() && uri.rawQuery == null && uri.rawFragment == null)
  val host = uri.host ?: throw IllegalArgumentException("invalid_endpoint")
  require(host.length <= 253 && host.contains('.') && !host.all { it.isDigit() || it == '.' } && host == host.lowercase())
  require(host.split('.').all { part -> part.length in 1..63 && part.first() != '-' && part.last() != '-' &&
   part.all { it in 'a'..'z' || it in '0'..'9' || it == '-' } })
  return value
 }
}
