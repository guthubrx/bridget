// E2 / E3 (ronde r3) - deux hypothèses de la revue de sources à DIAGNOSTIQUER sur le binaire release r8, sans modifier la production.
//   E3  descendant `cancelling`/`cancelled` (ou `result_available`) dont l'exécution durable reste `running` quand le daemon est coupé :
//       prepare_restart ne ferme que `failed/unreachable` ; descendants_busy n'ignore que ce couple -> la racine attend-elle sans fin ?
//   E2  tâche `queued` dont le propriétaire est un enfant natif (qui ne se reconnecte jamais) : tick saute ; la racine attend-elle sans fin ?
// E3  : états obtenus par l'API et des SIGTERM réels (fournisseur qui ignore SIGTERM => annulation lente ; fournisseur à turn/completed tardif).
// E2  : la fenêtre (admission -> tick) n'est pas atteignable par SIGTERM (l'arrêt gracieux dure 1 s et laisse le tick finir) ; état durable COHÉRENT
//       rejoué par craft-queued.py (transaction d'insertion exacte) sur la base privée, daemon ARRÊTÉ. Niveau : SIMULÉ.
import * as OS from "node:os";
import * as Path from "node:path";
import * as ChildProcess from "node:child_process";
import { startT3Host, codexFullAccessFact } from "./t3host.ts";
import { Fx, Checks, pause, BIN, BIN_SHA256, HERE, sha256File, lineageArgs, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, THREAD_B } from "./common149.ts";

process.env.NATIVE149_CODEX_PROVIDER = Path.join(HERE, "codex149b.py");
const PORT = 14798;
const ACTIVE = ["queued", "starting", "running", "waiting_approval", "waiting_user_input", "interrupting"];
const checks = new Checks("E2/E3 hypothèses de la revue de sources (r3)");
const T0S = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire release r8 : empreinte différente du reçu");
if (!(await freePort(PORT))) throw new Error(`port ${PORT} occupé`);
const host = await startT3Host(PORT);
const fx = await Fx.create("e2e3-149", { port: PORT });
notes.fixtureRoot = fx.root;
const base = { agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", cwd: fx.work };
const prompts = (nonce: string) => fx.evidence().filter((e: any) => e.event === "prompt" && e.nonce === nonce);
const starts = () => fx.evidence().filter((e: any) => e.event === "started");
const rowOf = (id: string) => fx.tasks().find((t: any) => t.task_id === id);
const nestedOf = (id: string) => fx.tasks().filter((t: any) => t.parent_task_id === id);
const execOfTask = (t: any) => fx.execs().find((e: any) => e.execution_id === `execution-${t?.mission}`);
const brief = (t: any) => t ? { id: String(t.task_id).slice(0, 8), state: t.state, error: t.error ?? null, sent: t.result_sent } : null;
const untilT = async (fn: () => any, ms: number, step = 250) => { const end = Date.now() + ms; for (;;) { const v = fn(); if (v) return v; if (Date.now() >= end) return undefined; await pause(step); } };
const alive = (pid: number) => { try { process.kill(pid, 0); return true; } catch { return false; } };
let curPeer: any;
let cA: any;
let exitCode = 0;
let mcpA: any;

/** Échantillonne un état (JSON) toutes les `step` ms ; ne garde que les changements. */
async function timeline(ms: number, step: number, probe: () => unknown) {
  const out: Array<{ atMs: number; v: unknown }> = [];
  const t0 = Date.now();
  let last = "";
  while (Date.now() - t0 < ms) {
    const v = probe();
    const j = JSON.stringify(v);
    if (j !== last) { out.push({ atMs: Date.now() - t0, v }); last = j; }
    await pause(step);
  }
  return out;
}

async function cutAndRestart(settleMs = 2500, reconnect = true) {
  curPeer?.close();
  await mcpA?.close().catch(() => undefined);
  await fx.stopDaemon();
  await pause(settleMs);
  const hook = (cutAndRestart as any).between as undefined | (() => Promise<void> | void);
  if (hook) { await hook(); (cutAndRestart as any).between = undefined; }
  await fx.startDaemon();
  if (reconnect) {
    curPeer = await fx.registerParent(THREAD_A);
    await fx.registerParent(THREAD_B);
  }
}

async function remountMcp(label: string) {
  mcpA = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label });
}

try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  notes.daemonPid = await fx.startDaemon();
  curPeer = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  cA = (await host.issue(THREAD_A)).config;
  host.publish(cA, codexFullAccessFact(cA, "run-e-1", fx.work), host.startRun(THREAD_A, "run-e-1"));
  await remountMcp("E");

  // ================================================== E3c CONTRÔLE : annulation individuelle d'un descendant, daemon sain
  const dK = await mcpA.call("bridget_delegate", { ...base, request_id: "e3-control", task: "NONCE_k1 NESTED_149[SLOW_149:25000]" });
  const rootK = dK.payload.task_id as string;
  const readyK = await untilT(() => { const r = rowOf(rootK); const n = nestedOf(rootK)[0]; return r?.state === "waiting_for_children" && n?.state === "working" ? n : undefined; }, 20000);
  const cancelK = await mcpA.call("bridget_task_cancel", { task_id: readyK?.task_id });
  notes.e3ControlCancelApi = { isError: cancelK.isError, payload: cancelK.payload };
  let cliCancel: any;
  if (cancelK.isError) {
    cliCancel = await fx.cli(lineageArgs("cancel", THREAD_A, fx.work, ["--task", readyK?.task_id, "--request-id", "49000000-0000-4000-8000-0000000000e1"]));
    notes.e3ControlCancelCli = { code: cliCancel.code, json: cliCancel.json };
  }
  const tlK = await timeline(25000, 250, () => ({ root: brief(rowOf(rootK)), nested: brief(nestedOf(rootK)[0]), nestedExec: execOfTask(nestedOf(rootK)[0])?.state, delivered: curPeer.deliveries.filter((d: any) => d.in_reply_to === dK.payload.message_id).length }));
  const finK = tlK[tlK.length - 1].v as any;
  notes.e3Control = { timeline: tlK };
  checks.add("E3c", "REEL", "CONTRÔLE : annulation individuelle d'un descendant avec le daemon SAIN - le descendant devient `cancelled`, son exécution est close (non active), la racine sort de `waiting_for_children` (result_available) et son résultat est remis UNE fois",
    !!readyK && finK.nested?.state === "cancelled" && !ACTIVE.includes(finK.nestedExec) && finK.root?.state === "result_available" && finK.delivered === 1,
    { cancelApi: cancelK.isError ? `refus ${cancelK.code ?? cancelK.payload?.text} -> CLI ${cliCancel?.code}` : "ok (MCP propriétaire racine)", final: finK, steps: tlK.length });

  // ================================================== E3 : coupure PENDANT l'annulation lente d'un descendant (fournisseur qui ignore SIGTERM)
  const dE = await mcpA.call("bridget_delegate", { ...base, request_id: "e3-cancelling", task: "NONCE_k2 NESTED_149[IGNORE_TERM_149:25000 SLOW_149:25000]" });
  const rootE = dE.payload.task_id as string;
  const readyE = await untilT(() => { const r = rowOf(rootE); const n = nestedOf(rootE)[0]; return r?.state === "waiting_for_children" && n?.state === "working" ? n : undefined; }, 20000);
  const nestedE = readyE?.task_id as string;
  const nestedEPid = prompts("k2g")[0]?.pid as number | undefined;
  const cancelE = await mcpA.call("bridget_task_cancel", { task_id: nestedE });
  if (cancelE.isError) notes.e3CancelCli = await fx.cli(lineageArgs("cancel", THREAD_A, fx.work, ["--task", nestedE, "--request-id", "49000000-0000-4000-8000-0000000000e2"]));
  const tlE1 = await timeline(6000, 200, () => ({ nested: brief(rowOf(nestedE)), nestedExec: execOfTask(rowOf(nestedE))?.state, root: rowOf(rootE)?.state }));
  const windowOk = (tlE1[tlE1.length - 1].v as any).nested?.state === "cancelling" && ACTIVE.includes((tlE1[tlE1.length - 1].v as any).nestedExec);
  notes.e3BeforeCut = { cancelApiError: cancelE.isError, timeline: tlE1, windowOk, nestedProviderAlive: nestedEPid ? alive(nestedEPid) : null };
  checks.add("E3.a", "REEL", "fenêtre atteinte par l'API : le descendant est `cancelling` (annulation lente, fournisseur sourd à SIGTERM) avec son exécution durable ACTIVE, la racine attend toujours",
    windowOk && (tlE1[tlE1.length - 1].v as any).root === "waiting_for_children", tlE1.slice(-2));
  const startsBeforeE = starts().length;
  await cutAndRestart(2500);
  await remountMcp("E2nd");
  const tlE2 = await timeline(45000, 500, () => ({ root: brief(rowOf(rootE)), nested: brief(rowOf(nestedE)), nestedExec: execOfTask(rowOf(nestedE))?.state, delivered: curPeer.deliveries.filter((d: any) => d.in_reply_to === dE.payload.message_id).length, live: 0 }));
  const finE = tlE2[tlE2.length - 1].v as any;
  notes.e3AfterCut = { timeline: tlE2, startsBefore: startsBeforeE, startsAfter: starts().length, prompts: ["k2", "k2g"].map((n) => prompts(n).length) };
  const stuckE = finE.root?.state === "waiting_for_children";
  checks.add("E3.b", "REEL", "APRÈS coupure en pleine annulation : la racine ne reste PAS indéfiniment en `waiting_for_children` (45 s) - le descendant `cancelling` devient `cancelled`, la racine est `result_available` et remise au plus UNE fois, aucun nouveau lancement ni tour",
    !stuckE && finE.nested?.state === "cancelled" && finE.root?.state === "result_available" && finE.delivered === 1 && starts().length === startsBeforeE && prompts("k2").length === 1 && prompts("k2g").length === 1,
    { final: finE, stuck: stuckE, timelineSteps: tlE2.length, startsDelta: starts().length - startsBeforeE, prompts: notes.e3AfterCut && (notes.e3AfterCut as any).prompts });
  checks.add("E3.c", "REEL", "hygiène des exécutions durables : l'exécution du descendant `cancelled` n'est plus ACTIVE après la relance (sinon : écart F3 - ligne d'exécution restée active pour une tâche terminale ; sans effet observé sur la racine)",
    !ACTIVE.includes(finE.nestedExec), { nestedState: finE.nested?.state, nestedExec: finE.nestedExec, rootResolvedAnyway: finE.root?.state });

  // ================================================== E3b : coupure dans la fenêtre « résultat du descendant capturé / turn/completed pas encore reçu »
  await remountMcp("E3b");
  const dL = await mcpA.call("bridget_delegate", { ...base, request_id: "e3b-late", task: "NONCE_k3 NESTED_149[LATE_COMPLETE_149:9000]" });
  const rootL = dL.payload.task_id as string;
  const tlL1 = await timeline(8000, 150, () => ({ root: rowOf(rootL)?.state, nested: brief(nestedOf(rootL)[0]), nestedExec: execOfTask(nestedOf(rootL)[0])?.state }));
  const lastL1 = tlL1[tlL1.length - 1].v as any;
  const nestedL = nestedOf(rootL)[0]?.task_id as string | undefined;
  const windowL = lastL1.nested?.state === "result_available" && ACTIVE.includes(lastL1.nestedExec) && lastL1.root === "waiting_for_children";
  notes.e3bBeforeCut = { timeline: tlL1, windowL };
  checks.add("E3b.a", "OBS", "OBSERVATION (ne compte ni PASS ni FAIL) : la fenêtre « résultat du descendant capturé / exécution encore active » n'est PAS atteignable avec turn/completed tardif - le résultat n'est capturé qu'à la fin du tour ; le descendant est `working` avec exécution `running` à la coupure",
    true, { windowReached: windowL, lastBeforeCut: lastL1 });
  await cutAndRestart(2500);
  await remountMcp("E3b2");
  const tlL2 = await timeline(45000, 500, () => ({ root: brief(rowOf(rootL)), nested: brief(nestedL ? rowOf(nestedL) : undefined), nestedExec: nestedL ? execOfTask(rowOf(nestedL))?.state : null, delivered: curPeer.deliveries.filter((d: any) => d.in_reply_to === dL.payload.message_id).length }));
  const finL = tlL2[tlL2.length - 1].v as any;
  notes.e3bAfterCut = { timeline: tlL2 };
  const stuckL = finL.root?.state === "waiting_for_children";
  checks.add("E3b.b", "REEL", "APRÈS coupure pendant un tour tardif du descendant (exécution `running`) : la racine ne reste PAS indéfiniment `waiting_for_children` (45 s), l'exécution du descendant est close (unreachable), résultat remis au plus une fois",
    !stuckL && !ACTIVE.includes(finL.nestedExec) && finL.delivered <= 1, { final: finL, stuck: stuckL, nestedExecActive: ACTIVE.includes(finL.nestedExec) });

  // ================================================== E2 (SIMULÉ) : tâche `queued` dont le propriétaire est un enfant natif
  await remountMcp("E2");
  const dX = await mcpA.call("bridget_delegate", { ...base, request_id: "e2-ext-done", task: "NONCE_x1" });
  await untilT(() => rowOf(dX.payload.task_id)?.state === "result_available" && rowOf(dX.payload.task_id)?.result_sent ? true : undefined, 20000);
  const dS = await mcpA.call("bridget_delegate", { ...base, request_id: "e2-nested-queued", task: "NONCE_k4 NESTED_149[HB_149:60000]" });
  const rootS = dS.payload.task_id as string;
  const readyS = await untilT(() => { const r = rowOf(rootS); const n = nestedOf(rootS)[0]; return r?.state === "waiting_for_children" && n?.state === "working" ? n : undefined; }, 20000);
  const nestedS = readyS?.task_id as string;
  const execsPreE2 = JSON.stringify(fx.execs());
  const doneX = rowOf(dX.payload.task_id);
  let craftedQ: any;
  let craftedX: any;
  (cutAndRestart as any).between = () => {
    const db = `${fx.state}/bridget.db`;
    // Q : clone ENVELOPPE d'un descendant réel -> même propriétaire (l'enfant natif), même parent (la racine)
    craftedQ = JSON.parse(ChildProcess.execFileSync("/usr/bin/python3", ["-I", Path.join(HERE, "craft-queued.py"), db, nestedS, "nested-e2-queued"], { encoding: "utf8" }));
    // X : clone d'une tâche EXTERNE (propriétaire = le fil T3 A) terminée -> contrôle « queued externe doit reprendre »
    craftedX = JSON.parse(ChildProcess.execFileSync("/usr/bin/python3", ["-I", Path.join(HERE, "craft-queued.py"), db, dX.payload.task_id, "e2-ext-queued"], { encoding: "utf8" }));
  };
  await cutAndRestart(2500);
  notes.e2Crafted = { nestedQueued: craftedQ, externalQueued: craftedX };
  const qState = () => rowOf(craftedQ.task_id);
  const xState = () => rowOf(craftedX.task_id);
  const tlS = await timeline(45000, 500, () => ({ root: brief(rowOf(rootS)), nested: brief(rowOf(nestedS)), queuedNested: brief(qState()), queuedExternal: brief(xState()), xDelivered: curPeer.deliveries.filter((d: any) => String(d.body) === "fixture149-answer:x1" && d.in_reply_to === craftedX.mission).length }));
  const finS = tlS[tlS.length - 1].v as any;
  notes.e2After = { timeline: tlS, startsDelta: starts().length };
  const stuckS = finS.root?.state === "waiting_for_children";
  checks.add("E2.a", "SIMULE", "CONTRÔLE : la tâche `queued` EXTERNE (propriétaire = le fil T3 A reconnecté) REPREND après la relance : elle est lancée, termine (`result_available`) et son résultat est remis UNE fois ; la tâche terminée d'origine n'est PAS rouverte",
    finS.queuedExternal?.state === "result_available" && finS.xDelivered === 1 && rowOf(dX.payload.task_id)?.state === "result_available" && rowOf(dX.payload.task_id)?.result_sent === true && doneX?.mission === rowOf(dX.payload.task_id)?.mission && prompts("x1").filter((p: any) => p).length === 2,
    { external: finS.queuedExternal, delivered: finS.xDelivered, originalState: rowOf(dX.payload.task_id)?.state, promptsX1: prompts("x1").length });
  checks.add("E2.b", "SIMULE", "E2 : la tâche `queued` dont le propriétaire est un enfant natif (jamais reconnecté) NE bloque PAS indéfiniment la racine : au bout de 45 s la racine n'est plus `waiting_for_children` et la tâche `queued` n'est plus `queued` (sinon : BLOCK)",
    !stuckS && finS.queuedNested?.state !== "queued", { final: finS, stuck: stuckS, execsUnchangedByCraft: JSON.stringify(fx.execs()) !== execsPreE2 });

  // ================================================== E2c (SIMULÉ) : variante RÉALISTE - le propriétaire de la tâche `queued` est en plein tour (`working`) au moment de la coupure
  await remountMcp("E2c");
  const dR = await mcpA.call("bridget_delegate", { ...base, request_id: "e2c-midturn", task: "NONCE_k5 NESTED_149[HB_149:60000] HB_149:60000" });
  const rootR = dR.payload.task_id as string;
  const readyR = await untilT(() => { const r = rowOf(rootR); const n = nestedOf(rootR)[0]; return r?.state === "working" && n?.state === "working" ? n : undefined; }, 20000);
  let craftedR: any;
  (cutAndRestart as any).between = () => {
    craftedR = JSON.parse(ChildProcess.execFileSync("/usr/bin/python3", ["-I", Path.join(HERE, "craft-queued.py"), `${fx.state}/bridget.db`, readyR?.task_id, "nested-e2c-queued"], { encoding: "utf8" }));
  };
  await cutAndRestart(2500);
  const tlR = await timeline(20000, 500, () => ({ root: brief(rowOf(rootR)), nested: brief(rowOf(readyR?.task_id)), queued: brief(rowOf(craftedR.task_id)), notices: curPeer.deliveries.filter((d: any) => String(d.body).includes(rootR)).length }));
  const finR = tlR[tlR.length - 1].v as any;
  notes.e2cAfter = { crafted: craftedR, timeline: tlR };
  checks.add("E2c", "SIMULE", "variante réaliste : propriétaire de la tâche `queued` en plein tour au moment de la coupure (racine `working`) -> la racine devient `failed` (unreachable, ou interrompue par l'arrêt coopératif du wrapper), le nettoyage ANNULE la tâche `queued` (plus de `queued`), UNE notice d'échec au parent, aucun lancement",
    finR.root?.state === "failed" && finR.queued?.state === "cancelled" && finR.notices === 1 && prompts("k5").length === 1,
    { final: finR, promptsK5: prompts("k5").length });

  // ================================================== Libération : seul `cancel` sort une racine bloquée (preuve de l'issue de secours)
  const stuckRoots = [[rootE, "E3"], [rootL, "E3b"], [rootS, "E2"], [rootR, "E2c"]].filter(([id]) => rowOf(id as string)?.state === "waiting_for_children");
  await remountMcp("E-release");
  const released: any[] = [];
  for (const [id, tag] of stuckRoots as Array<[string, string]>) {
    const c = await mcpA.call("bridget_task_cancel", { task_id: id });
    await untilT(() => rowOf(id)?.state === "cancelled" ? true : undefined, 25000);
    released.push({ tag, cancelIsError: c.isError, state: rowOf(id)?.state, resultLost: rowOf(id)?.result == null || undefined });
  }
  notes.release = released;
  checks.add("E.rel", "REEL", "issue de secours : toute racine restée bloquée ne se termine que par `cancel` du parent (cancelled) - ou aucune racine n'est restée bloquée",
    released.every((r) => r.state === "cancelled"), { stuckBeforeRelease: stuckRoots.map(([, t]) => t), released });
  await mcpA.close();
  curPeer.close();
} catch (error) {
  exitCode = 1;
  checks.add("EXC", "REEL", "exception non prévue", false, { message: String((error as Error).stack ?? error).slice(0, 900) });
} finally {
  // les orphelins bornés (IGNORE_TERM 25 s, SLOW 25 s) finissent seuls ; attente bornée puis SIGTERM individuel (cleanup)
  notes.stopDaemonFinal = await fx.stopDaemon().catch((e: unknown) => String(e));
  const waitEnd = Date.now() + 40000;
  while ((await fx.liveFixtureProcesses()).some((p: any) => p.kind === "provider") && Date.now() < waitEnd) await pause(1000);
  notes.cleanup = await fx.cleanup().catch((e: unknown) => String(e));
  await host.stop().catch(() => undefined);
}
const summary = checks.summary();
const live = (notes.cleanup as any)?.remainingAfter ?? [];
await writeResults("e2e3-149", { schema: "native-network-recipes/1", startedAt: T0S, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
