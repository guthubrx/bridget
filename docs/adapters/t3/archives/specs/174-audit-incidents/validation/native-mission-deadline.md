# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/native-mission-deadline.md](../../../../../../../specs/174-audit-incidents/validation/native-mission-deadline.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

T3 ClaudeAdapter.ts, sendTurn (4881–5035), records turnState and queues the input.
This path does not impose a default turn expiry. Bridget must retain its explicit
native mission deadline because it coordinates durable remote work. The fix reuses
that existing authority instead of adding another timer or retry mechanism.

<a id="extrait-2"></a>

## Extrait 2

- Optimized macOS build: 1m25s; signed and verified.
  SHA-256 e817c59d0294cacf7294852a9b67bf34ec430b4352560ba265ee274dc0c2c57d.
  Active master PID 74260 reports build 5ac9e79599ef.
- Optimized Linux build: 1m51s.
  SHA-256 1d8a5cbdee32757f942322ae14490ee4cf24393b8add7e0c3fb1543b0f08e42b.
  Active worker PID 292436 uses the new executable.
- T3 and its bridge retain their original processes. Three live remote wrappers
  retained their PIDs and birth times across worker replacement.
- The incident task payload fingerprint remained identical after master activation.
  No business task was replayed, cancelled or edited.
- The first macOS status read preceded service readiness; the next read confirmed
  the expected build. No second restart was needed.
- Staging cleanup initially rejected a signing-only hash difference. The compiled
  executable received the same verified signature as the installed copy; their
  hashes then matched and the normal installed cleanup ran. No guard was weakened.
- Politique received the delivery guidance through Bridget. Recipient acknowledgment
  is not claimed.
