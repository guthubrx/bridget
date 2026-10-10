#!/usr/bin/env node
// Recette 149 - matrice réseau réelle (WebSocket RPC du serveur T3 de recette).
// Couches RÉELLES : client Effect RPC, auth bearer scopée, serveur T3 WT, lecteur
// BridgetReader, BridgetLineage, projection SQL privée. Couche SIMULÉE nommée :
// CLI daemon = bridget_fixture.mjs (magasin JSON privé). Sortie : PASS/FAIL par cas.
// Usage : node matrix149.mjs <groupe: scopes|inputs|roots|cancel|all>
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const C = process.env.RECIPE149_CACHE ?? "/Users/moi/.cache/bridget149-ui";
const R = new URL(".", import.meta.url).pathname;
const { projectId: P, threadId: T } = JSON.parse(readFileSync(`${C}/ids.json`, "utf8"));
const { projectId2: P2, threadId2: T2 } = JSON.parse(readFileSync(`${C}/ids2.json`, "utf8"));
const U = (n) => `00000000-0000-4000-8000-${String(n).padStart(12, "0")}`;
const uuid = () => crypto.randomUUID();
const tok = (n) => `${C}/tok-${n}.txt`;

function rpc(token, method, payload, extra = []) {
  try {
    const out = execFileSync(process.execPath, ["--no-warnings", `${R}rpc149.ts`, "call", token, method, JSON.stringify(payload), ...extra], { encoding: "utf8", timeout: 30000 });
    return JSON.parse(out.trim().split("\n").pop());
  } catch (e) {
    return { harness_error: String(e.stdout ?? e.message).slice(0, 300) };
  }
}
function raw(token, method, payload) {
  try {
    const out = execFileSync(process.execPath, ["--no-warnings", `${R}rpc149.ts`, "raw", token, method, JSON.stringify(payload)], { encoding: "utf8", timeout: 30000 });
    return JSON.parse(out.trim().split("\n").pop());
  } catch (e) {
    return { harness_error: String(e.stdout ?? e.message).slice(0, 300) };
  }
}
const ctx = (extra = {}) => ({ projectId: P, threadId: T, ...extra });
const code = (r) => r.error?.code ?? r.error?.tag ?? r.transport_error ?? (r.error ? "error" : null);
const refused = (r) => r.error !== undefined || r.transport_error !== undefined;
const summary = (r) => r.exit ? `${r.exit}${r.causeText ? " " + r.causeText.replace(/\\+/g, "").slice(0, 110) : ""}` : r.value ? `ok(${r.value.status ?? "value"})` : r.items ? `items=${r.items.length}${r.error ? " err=" + code(r) : ""}` : `${r.transport_error ?? r.error?.tag ?? "?"}/${r.error?.code ?? r.error?.message?.slice(0, 50) ?? r.body?.slice(0, 60) ?? ""}`;

const cases = [];
const add = (group, id, title, run, expect) => cases.push({ group, id, title, run, expect });

// --- scopes ---
add("scopes", "S1", "read: lineage.read list", () => rpc(tok("read"), "bridget.lineage.read", ctx({ action: "list" })), (r) => r.value?.status === "ok");
add("scopes", "S2", "read: lineage.read show T3", () => rpc(tok("read"), "bridget.lineage.read", ctx({ action: "show", taskId: U(3) })), (r) => r.value?.task?.task_id === U(3) && r.value.result?.includes("Résultat de recette 149"));
add("scopes", "S3", "read: lineage.read journal T3", () => rpc(tok("read"), "bridget.lineage.read", ctx({ action: "journal", taskId: U(3) })), (r) => r.value?.events?.length === 2);
add("scopes", "S4", "read: lineage.watch ready seq0", () => rpc(tok("read"), "bridget.lineage.watch", ctx(), ["--take", "1"]), (r) => r.items?.[0]?.status === "ready" && r.items[0].seq === 0);
add("scopes", "S5", "read: lineage.journal flux", () => rpc(tok("read"), "bridget.lineage.journal", ctx({ taskId: U(3) }), ["--take", "1"]), (r) => r.items?.[0]?.task_id === U(3));
add("scopes", "S6", "read: lineage.cancel REFUSÉ (operate requis)", () => rpc(tok("read"), "bridget.lineage.cancel", ctx({ taskId: U(1), requestId: uuid() })), (r) => refused(r) && !r.value);
add("scopes", "S7", "operate: lineage.cancel tâche terminale T3 -> état courant sans effet", () => rpc(tok("operate"), "bridget.lineage.cancel", ctx({ taskId: U(3), requestId: uuid() })), (r) => r.value?.status === "result_available");
add("scopes", "S8", "settings:write seul: lineage.read REFUSÉ", () => rpc(tok("noorch"), "bridget.lineage.read", ctx({ action: "list" })), (r) => refused(r) && !r.value);
add("scopes", "S9", "settings:write seul: lineage.cancel REFUSÉ", () => rpc(tok("noorch"), "bridget.lineage.cancel", ctx({ taskId: U(1), requestId: uuid() })), (r) => refused(r) && !r.value);
add("scopes", "S10", "sans jeton: ticket WS refusé", () => rpc("none", "bridget.lineage.read", ctx({ action: "list" })), (r) => r.harness_error !== undefined || refused(r));

// --- inputs (autorité inconnue / extra / invalide) ---
// Trame brute : le serveur reçoit exactement la charge ; refus attendu côté serveur (Exit Failure).
const bad = (id, title, method, payload, tk = "operate") => add("inputs", id, title, () => raw(tok(tk), method, payload), (r) => r.exit === "Failure");
bad("I1", "propriété extra projectRoot (autorité forgée)", "bridget.lineage.read", ctx({ action: "list", projectRoot: "/etc" }));
bad("I2", "propriété extra workspaceRoot", "bridget.lineage.read", ctx({ action: "list", workspaceRoot: "/tmp" }));
bad("I3", "propriété extra agentId/rootOwner", "bridget.lineage.read", ctx({ action: "list", rootOwnerAgentId: U(901) }));
bad("I4", "action inconnue", "bridget.lineage.read", ctx({ action: "delete" }));
bad("I5", "taskId non UUID canonique", "bridget.lineage.read", ctx({ action: "show", taskId: "pas-un-uuid" }));
bad("I6", "taskId UUID majuscules non canonique", "bridget.lineage.read", ctx({ action: "show", taskId: "AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA" }));
bad("I7", "limit hors borne show (16385)", "bridget.lineage.read", ctx({ action: "show", taskId: U(3), limit: 16385 }));
bad("I8", "limit journal hors borne (101)", "bridget.lineage.read", ctx({ action: "journal", taskId: U(3), limit: 101 }));
bad("I9", "afterSeq négatif", "bridget.lineage.read", ctx({ action: "journal", taskId: U(3), afterSeq: -1 }));
bad("I10", "cancel sans requestId", "bridget.lineage.cancel", ctx({ taskId: U(1) }));
bad("I11", "cancel requestId non UUID", "bridget.lineage.cancel", ctx({ taskId: U(1), requestId: "abc" }));
bad("I12", "cancel propriété extra force", "bridget.lineage.cancel", ctx({ taskId: U(1), requestId: uuid(), force: true }));
bad("I13", "watch propriété extra", "bridget.lineage.watch", ctx({ afterSeq: 3 }));

// --- roots / tâches / contexte ---
const code_is = (c) => (r) => code(r) === c;
add("roots", "R1", "projectId inexistant", () => rpc(tok("operate"), "bridget.lineage.read", { projectId: uuid(), threadId: T, action: "list" }), (r) => refused(r));
add("roots", "R2", "thread inexistant", () => rpc(tok("operate"), "bridget.lineage.read", { projectId: P, threadId: uuid(), action: "list" }), (r) => refused(r));
add("roots", "R3", "thread de proj1 avec projectId proj2 -> project_mismatch", () => rpc(tok("operate"), "bridget.lineage.read", { projectId: P2, threadId: T, action: "list" }), code_is("project_mismatch"));
add("roots", "R4", "fil étranger (root proj2 != root store) -> project_mismatch natif", () => rpc(tok("operate"), "bridget.lineage.read", { projectId: P2, threadId: T2, action: "list" }), code_is("project_mismatch"));
add("roots", "R5", "show tâche inconnue -> task_unavailable", () => rpc(tok("operate"), "bridget.lineage.read", ctx({ action: "show", taskId: uuid() })), code_is("task_unavailable"));
add("roots", "R6", "journal tâche inconnue -> task_unavailable", () => rpc(tok("operate"), "bridget.lineage.read", ctx({ action: "journal", taskId: uuid() })), code_is("task_unavailable"));
add("roots", "R7", "cancel tâche inconnue -> task_unavailable", () => rpc(tok("operate"), "bridget.lineage.cancel", ctx({ taskId: uuid(), requestId: uuid() })), code_is("task_unavailable"));
add("roots", "R8", "show depuis fil étranger -> project_mismatch (pas d'existence révélée)", () => rpc(tok("operate"), "bridget.lineage.read", { projectId: P2, threadId: T2, action: "show", taskId: U(3) }), code_is("project_mismatch"));
add("roots", "R9", "cancel depuis fil étranger -> project_mismatch", () => rpc(tok("operate"), "bridget.lineage.cancel", { projectId: P2, threadId: T2, taskId: U(3), requestId: uuid() }), code_is("project_mismatch"));
add("roots", "R10", "watch depuis fil étranger refusé", () => rpc(tok("operate"), "bridget.lineage.watch", { projectId: P2, threadId: T2 }, ["--take", "1"]), (r) => refused(r) || r.items?.length === 0);
add("roots", "R11", "journal tâche sans journal (T4) -> journal_unavailable", () => rpc(tok("operate"), "bridget.lineage.read", ctx({ action: "journal", taskId: U(4) })), code_is("journal_unavailable"));
add("roots", "R12", "show offset hors résultat -> result_offset_invalid", () => rpc(tok("operate"), "bridget.lineage.read", ctx({ action: "show", taskId: U(3), offset: 99999 })), code_is("result_offset_invalid"));

// --- cancel : idempotence / enveloppe ---
const rid = uuid();
add("cancel", "C1", "cancel T4 (échouée) -> état courant failed, pas cancelled inventé", () => rpc(tok("operate"), "bridget.lineage.cancel", ctx({ taskId: U(4), requestId: rid })), (r) => r.value?.status === "failed");
add("cancel", "C2", "rejeu identique même request_id -> même reçu", () => rpc(tok("operate"), "bridget.lineage.cancel", ctx({ taskId: U(4), requestId: rid })), (r) => r.value?.status === "failed");
add("cancel", "C3", "même request_id, autre tâche -> envelope_mismatch", () => rpc(tok("operate"), "bridget.lineage.cancel", ctx({ taskId: U(3), requestId: rid })), code_is("envelope_mismatch"));

const want = process.argv[2] ?? "all";
let pass = 0, fail = 0;
for (const c of cases.filter((x) => want === "all" || x.group === want)) {
  const r = c.run();
  const ok = (() => { try { return Boolean(c.expect(r)); } catch { return false; } })();
  ok ? pass++ : fail++;
  console.log(`${ok ? "PASS" : "FAIL"} ${c.id.padEnd(4)} ${c.title} => ${summary(r)}`);
}
console.log(`TOTAL pass=${pass} fail=${fail}`);
process.exit(fail === 0 ? 0 : 1);
