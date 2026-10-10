// Recette 149 r5 - borne C0/C1 du schéma RÉEL BridgetSessionIdentity (v1) sur les 65536 unités UTF-16.
// Oracle : l'ancienne règle /[\u0000-\u001f\u007f-\u009f]/ ; outil jetable, aucune logique de production.
import * as Schema from "effect/Schema";
import { BridgetSessionIdentity } from "@t3tools/contracts";
const decode = Schema.decodeUnknownExit(BridgetSessionIdentity);
const oldRule = /[\u0000-\u001f\u007f-\u009f]/;
const base = { version: 1, environmentId: "env", threadId: "t", providerSessionId: "s", providerInstanceId: "i" };
let mismatches = 0, rejected = 0, accepted = 0;
const bad: number[] = [];
for (let c = 0; c < 65536; c += 1) {
  const ch = String.fromCharCode(c);
  const ok = decode({ ...base, environmentId: `e${ch}x` })._tag === "Success";
  const expectOk = !oldRule.test(ch);
  if (ok !== expectOk) { mismatches += 1; bad.push(c); }
  ok ? accepted++ : rejected++;
}
const edge = (c: number) => decode({ ...base, providerSessionId: String.fromCharCode(c) })._tag === "Success";
console.log(JSON.stringify({ mismatches, bad: bad.slice(0, 10), accepted, rejected, expectedRejected: 32 + 33,
  edges: { "0x1f": edge(0x1f), "0x20": edge(0x20), "0x7e": edge(0x7e), "0x7f": edge(0x7f), "0x9f": edge(0x9f), "0xa0": edge(0xa0) },
  loneSurrogateAccepted: edge(0xd800), astralAccepted: decode({ ...base, environmentId: "e😀" })._tag === "Success" }));
