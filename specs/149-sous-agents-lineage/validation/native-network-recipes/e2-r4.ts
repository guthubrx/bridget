// E2 (ronde r4) - rejeu de la transaction durable COHÉRENTE de r3 (craft-queued.py) sur le binaire release r9.
// État : racine `waiting_for_children` avec résultat RÉEL capturé + descendant `working` ; on ajoute (SIMULÉ, daemon arrêté, transaction exacte d'admission)
// une tâche `queued` dont le propriétaire est l'enfant natif (propriétaire perdu) et une tâche `queued` EXTERNE (contrôle).
// Attendu : parent hors ligne -> descendants `failed`/unreachable, racine `result_available` (résultat intact, non remis) ; reconnexion -> UNE remise ;
// queued externe reprend normalement ; 2e redémarrage : aucun rejeu. AUCUNE écriture DB après la relance.
import * as OS from "node:os";
import * as Path from "node:path";
import * as ChildProcess from "node:child_process";
import { startT3Host, codexFullAccessFact } from "./t3host.ts";
import { Fx, Checks, pause, BIN, BIN_SHA256, HERE, sha256File, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, THREAD_B } from "./common149.ts";

process.env.NATIVE149_CODEX_PROVIDER = Path.join(HERE, "codex149b.py");
const PORT = 14817;
const ACTIVE = ["queued", "starting", "running", "waiting_approval", "waiting_user_input", "interrupting"];
const checks = new Checks("E2 queued à propriétaire natif perdu, transaction durable cohérente (r4)");
const T0S = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire release r9 : empreinte différente du reçu");
if (!(await freePort(PORT))) throw new Error(`port ${PORT} occupé`);
const host = await startT3Host(PORT);
const fx = await Fx.create("e2-r4", { port: PORT });
notes.fixtureRoot = fx.root;
const base = { agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", cwd: fx.work };
const prompts = (nonce: string) => fx.evidence().filter((e: any) => e.event === "prompt" && e.nonce === nonce);
const starts = () => fx.evidence().filter((e: any) => e.event === "started");
const rowOf = (id: string) => fx.tasks().find((t: any) => t.task_id === id);
const brief = (t: any) => t ? { id: String(t.task_id).slice(0, 8), state: t.state, error: t.error ?? null, sent: t.result_sent, result: t.result ?? null } : null;
const untilT = async (fn: () => any, ms: number, step = 250) => { const end = Date.now() + ms; for (;;) { const v = fn(); if (v) return v; if (Date.now() >= end) return undefined; await pause(step); } };
const snapTasks = () => fx.tasks().map((t: any) => ({ id: String(t.task_id).slice(0, 8), parent: t.parent_task_id ? String(t.parent_task_id).slice(0, 8) : null, state: t.state, error: t.error ?? null, sent: t.result_sent, hasResult: typeof t.result === "string" && t.result.length > 0 }));
let curPeer: any;
let exitCode = 0;
try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  notes.daemonPid = await fx.startDaemon();
  curPeer = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  const cA = (await host.issue(THREAD_A)).config;
  host.publish(cA, codexFullAccessFact(cA, "run-e2r4-1", fx.work), host.startRun(THREAD_A, "run-e2r4-1"));
  const mcp = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "E2R4" });
  // contrôle externe terminé et accusé (modèle pour le queued externe)
  const dX = await mcp.call("bridget_delegate", { ...base, request_id: "e2r4-ext-done", task: "NONCE_x1" });
  await untilT(() => rowOf(dX.payload.task_id)?.state === "result_available" && rowOf(dX.payload.task_id)?.result_sent ? true : undefined, 20000);
  // racine réelle : résultat capturé + descendant vivant
  const dR = await mcp.call("bridget_delegate", { ...base, request_id: "e2r4-root", task: "NONCE_e1 NESTED_149[HB_149:60000]" });
  const rootId = dR.payload.task_id as string;
  const ready = await untilT(() => {
    const r = rowOf(rootId); const n = fx.tasks().find((t: any) => t.parent_task_id === rootId);
    const e = r && fx.execs().find((x: any) => x.execution_id === `execution-${r.mission}`);
    return r?.state === "waiting_for_children" && r?.result === "fixture149-answer:e1" && n?.state === "working" && e?.state === "completed" ? n : undefined;
  }, 25000);
  const nestedId = ready?.task_id as string;
  const rootBefore = rowOf(rootId);
  const execsBefore = fx.execs();
  const tasksBefore = snapTasks();
  const startsBefore = starts().length;
  checks.add("E2r4.a", "REEL", "avant coupure : racine `waiting_for_children` avec résultat RÉEL capturé (fixture149-answer:e1), exécution de la racine `completed`, descendant `working` avec exécution active",
    !!ready && rootBefore?.result === "fixture149-answer:e1" && execsBefore.some((e: any) => e.execution_id === `execution-${ready.mission}` && ACTIVE.includes(e.state)),
    { root: brief(rootBefore), nested: brief(ready), execs: execsBefore.map((e: any) => [e.execution_id.slice(0, 14), e.state, e.reason]) });
  await mcp.close();
  curPeer.close(); // parent HORS LIGNE à la coupure ET à la relance
  await fx.stopDaemon();
  await pause(2500);
  // ---- SIMULATION (transaction exacte d'admission, daemon ARRÊTÉ) : deux lignes `queued`
  const craft = (src: string, rid: string) => JSON.parse(ChildProcess.execFileSync("/usr/bin/python3", ["-I", Path.join(HERE, "craft-queued.py"), `${fx.state}/bridget.db`, src, rid], { encoding: "utf8" }));
  const craftedQ = craft(nestedId, "e2r4-nested-queued");
  const craftedX = craft(dX.payload.task_id, "e2r4-ext-queued");
  const dbWritesEnd = Date.now(); // dernière écriture DB de la recette
  const preRestart = { tasks: snapTasks(), execs: fx.execs() };
  notes.crafted = { nestedQueued: craftedQ, externalQueued: craftedX };
  checks.add("E2r4.b", "SIMULE", "état durable simulé COHÉRENT (transaction d'admission rejouée, daemon arrêté) : `queued` imbriqué (parent = la racine, propriétaire = l'enfant natif) et `queued` externe, sans enfant, sans exécution ; la racine et le descendant réels ne sont pas modifiés",
    craftedQ.state === "queued" && craftedQ.parent_task_id === rootId && craftedX.state === "queued" && preRestart.execs.length === execsBefore.length && rowOf(rootId)?.state === "waiting_for_children",
    { queuedNested: { id: craftedQ.task_id?.slice(0, 8), parent: craftedQ.parent_task_id?.slice(0, 8) }, queuedExternal: craftedX.task_id?.slice(0, 8), execsUnchanged: preRestart.execs.length === execsBefore.length });
  const cutAt = Date.now();
  await fx.startDaemon(); // aucune écriture DB de la recette après ce point
  const resolved = await untilT(() => { const r = rowOf(rootId); return r && r.state !== "waiting_for_children" ? r : undefined; }, 30000);
  const resolveMs = Date.now() - cutAt;
  const off = { root: rowOf(rootId), nested: rowOf(nestedId), qN: rowOf(craftedQ.task_id), qX: rowOf(craftedX.task_id), execs: fx.execs() };
  await pause(6000);
  const off2 = rowOf(rootId);
  notes.offline = { resolveMs, tasks: snapTasks(), execs: off.execs.map((e: any) => [e.execution_id.slice(0, 14), e.state, e.reason]), startsDelta: starts().length - startsBefore };
  checks.add("E2r4.c", "REEL", "parent HORS LIGNE après relance (effets du daemon réel, sans écriture DB de la recette) : le `queued` imbriqué à propriétaire perdu n'est plus `queued` (fermé `failed`), le descendant réel est `failed`, la racine sort de `waiting_for_children` -> `result_available`, résultat capturé INTACT, `result_sent` faux, stable 6 s ; aucun lancement",
    !!resolved && off.qN?.state === "failed" && off.nested?.state === "failed" && off.root?.state === "result_available" && off.root?.result === "fixture149-answer:e1" && off.root?.result_sent === false && off2?.state === "result_available" && off2?.result_sent === false && starts().length === startsBefore,
    { resolveMs, root: brief(off.root), nested: brief(off.nested), queuedNested: brief(off.qN), queuedExternal: brief(off.qX), startsDelta: starts().length - startsBefore });
  const beforeIds = new Map(execsBefore.map((e: any) => [e.execution_id, e]));
  const reopened = off.execs.filter((e: any) => { const b: any = beforeIds.get(e.execution_id); return b && !ACTIVE.includes(b.state) && ACTIVE.includes(e.state); });
  checks.add("E2r4.d", "REEL", "exécutions durables : aucune exécution rouverte, aucune encore active, aucune ligne en plus ; celle du descendant est fermée (unreachable/daemon_restart ou failed)",
    reopened.length === 0 && off.execs.every((e: any) => !ACTIVE.includes(e.state)) && off.execs.length === execsBefore.length,
    { before: execsBefore.length, after: off.execs.length, reopened: reopened.length, active: off.execs.filter((e: any) => ACTIVE.includes(e.state)).length });
  // ---- reconnexion du parent : UNE remise de la racine ; le queued externe reprend
  curPeer = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  const gotRoot = () => curPeer.deliveries.filter((d: any) => d.in_reply_to === dR.payload.message_id);
  await untilT(() => gotRoot().length >= 1 && rowOf(craftedX.task_id)?.state === "result_available" && rowOf(craftedX.task_id)?.result_sent ? true : undefined, 25000);
  await pause(6000);
  const rootD = gotRoot();
  const xD = curPeer.deliveries.filter((d: any) => d.in_reply_to === craftedX.mission);
  const leak = curPeer.deliveries.filter((d: any) => /e1g/.test(String(d.body)));
  const rootSent = rowOf(rootId);
  notes.reconnect = { root: rootD.map((d: any) => ({ body: d.body })), external: xD.map((d: any) => ({ body: d.body })), acks: curPeer.acks.length, tasks: snapTasks() };
  checks.add("E2r4.e", "REEL", "reconnexion : le résultat EXACT de la racine est remis UNE seule fois (expéditeur = l'enfant), `result_sent` vrai, rien du descendant remis, aucun doublon 6 s ; le `queued` EXTERNE (contrôle) REPREND (`result_available`, UNE remise) sans faux échec ; la tâche externe d'origine n'est pas rouverte (2 tours x1)",
    rootD.length === 1 && rootD[0].body === "fixture149-answer:e1" && rootD[0].from === rootSent?.child && rootSent?.result_sent === true && leak.length === 0 &&
      rowOf(craftedX.task_id)?.state === "result_available" && xD.length === 1 && rowOf(dX.payload.task_id)?.state === "result_available" && prompts("x1").length === 2,
    { rootDeliveries: rootD.length, rootBody: rootD[0]?.body, fromChild: rootD[0]?.from === rootSent?.child, rootSent: rootSent?.result_sent, external: brief(rowOf(craftedX.task_id)), externalDeliveries: xD.length, promptsX1: prompts("x1").length, leaked: leak.length });
  // ---- 2e redémarrage : ACK / rejeu
  const tasksPre2 = JSON.stringify(snapTasks());
  const execsPre2 = JSON.stringify(fx.execs());
  const startsPre2 = starts().length;
  const delPre2 = curPeer.deliveries.length;
  curPeer.close();
  await fx.stopDaemon();
  await pause(2500);
  await fx.startDaemon();
  curPeer = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  await pause(9000);
  checks.add("E2r4.f", "REEL", "2e redémarrage (résultats accusés) : aucune remise rejouée (racine, queued externe), tâches et exécutions durables identiques, aucun lancement, aucun fournisseur vivant",
    curPeer.deliveries.length === 0 && JSON.stringify(snapTasks()) === tasksPre2 && JSON.stringify(fx.execs()) === execsPre2 && starts().length === startsPre2 && (await fx.liveProviders()).length === 0,
    { redelivered: curPeer.deliveries.length, tasksEqual: JSON.stringify(snapTasks()) === tasksPre2, execsEqual: JSON.stringify(fx.execs()) === execsPre2, startsDelta: starts().length - startsPre2, deliveriesBefore: delPre2 });
  curPeer.close();
} catch (error) {
  exitCode = 1;
  checks.add("EXC", "REEL", "exception non prévue", false, { message: String((error as Error).stack ?? error).slice(0, 900) });
} finally {
  notes.stopDaemonFinal = await fx.stopDaemon().catch((e: unknown) => String(e));
  notes.cleanup = await fx.cleanup().catch((e: unknown) => String(e));
  await host.stop().catch(() => undefined);
}
const summary = checks.summary();
const live = (notes.cleanup as any)?.remainingAfter ?? [];
await writeResults("e2-r4", { schema: "native-network-recipes/1", startedAt: T0S, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
