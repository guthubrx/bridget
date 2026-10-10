// E3d (ronde r3) - fenêtre « résultat capturé / exécution durable encore active » : existe-t-elle sans coupure ? Que devient-elle si le daemon est coupé dedans ?
// Partie 1 (sans coupure) : 6 racines avec un descendant rapide ; échantillon 40 ms ; décalage entre capture du résultat et clôture de l'exécution.
// Partie 2 : mêmes arbres, SIGTERM réel du daemon dès que la fenêtre est vue (tâche avec résultat ET exécution ACTIVE) ; relance après 2,5 s ; 45 s d'observation.
// Aucun modèle, binaire release r8, aucune écriture de base.
import * as OS from "node:os";
import * as Path from "node:path";
import { startT3Host, codexFullAccessFact } from "./t3host.ts";
import { Fx, Checks, pause, BIN, BIN_SHA256, HERE, sha256File, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, THREAD_B } from "./common149.ts";

process.env.NATIVE149_CODEX_PROVIDER = Path.join(HERE, "codex149b.py");
const PORT = 14799;
const ACTIVE = ["queued", "starting", "running", "waiting_approval", "waiting_user_input", "interrupting"];
const checks = new Checks("E3d fenêtre résultat capturé / exécution active (r3)");
const T0S = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire release r8 : empreinte différente du reçu");
if (!(await freePort(PORT))) throw new Error(`port ${PORT} occupé`);
const host = await startT3Host(PORT);
const fx = await Fx.create("exec-lag149", { port: PORT });
notes.fixtureRoot = fx.root;
const base = { agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", cwd: fx.work };
const rowOf = (id: string) => fx.tasks().find((t: any) => t.task_id === id);
const untilT = async (fn: () => any, ms: number, step = 100) => { const end = Date.now() + ms; for (;;) { const v = fn(); if (v) return v; if (Date.now() >= end) return undefined; await pause(step); } };
let curPeer: any;
let mcpA: any;
let cA: any;
let exitCode = 0;

/** Un instantané cohérent : tâches + exécutions (deux lectures SQL rapprochées). */
const snapshot = () => {
  const tasks = fx.tasks();
  const execs = new Map(fx.execs().map((e: any) => [e.execution_id, e]));
  return tasks.map((t: any) => {
    const e: any = execs.get(`execution-${t.mission}`);
    const hasResult = typeof t.result === "string" && t.result.length > 0;
    return { id: t.task_id as string, short: String(t.task_id).slice(0, 8), parent: t.parent_task_id ? String(t.parent_task_id).slice(0, 8) : null, state: t.state as string, hasResult, exec: e?.state as string | undefined, execReason: e?.reason as string | undefined, sent: t.result_sent as boolean };
  });
};

async function remount(label: string) { mcpA = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label }); }

try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  notes.daemonPid = await fx.startDaemon();
  curPeer = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  cA = (await host.issue(THREAD_A)).config;
  host.publish(cA, codexFullAccessFact(cA, "run-l-1", fx.work), host.startRun(THREAD_A, "run-l-1"));
  await remount("L");

  // ------------------------------------------------ Partie 1 : sans coupure
  const roots: string[] = [];
  for (let i = 1; i <= 6; i += 1) {
    const d = await mcpA.call("bridget_delegate", { ...base, request_id: `lag-${i}`, task: `NONCE_l${i} NESTED_149[fast]` });
    roots.push(d.payload.task_id);
  }
  const firstResult = new Map<string, number>();
  const firstClosed = new Map<string, number>();
  const t0 = Date.now();
  let maxActiveWithResult = new Map<string, number>();
  while (Date.now() - t0 < 25000) {
    const s = snapshot();
    const now = Date.now() - t0;
    for (const t of s) {
      if (t.hasResult && !firstResult.has(t.id)) firstResult.set(t.id, now);
      if (t.exec && !ACTIVE.includes(t.exec) && !firstClosed.has(t.id)) firstClosed.set(t.id, now);
      if (t.hasResult && t.exec && ACTIVE.includes(t.exec)) maxActiveWithResult.set(t.id, now - (firstResult.get(t.id) ?? now));
    }
    if (s.length >= 12 && s.every((t) => t.hasResult && t.exec && !ACTIVE.includes(t.exec)) && s.every((t) => t.state === "result_available" && t.sent)) break;
    await pause(40);
  }
  const final1 = snapshot();
  const lags = [...firstResult].map(([id, r]) => ({ id: id.slice(0, 8), resultMs: r, closedMs: firstClosed.get(id) ?? null, lagMs: firstClosed.has(id) ? (firstClosed.get(id)! - r) : null }));
  notes.part1 = { tasks: final1.length, lags, maxLagMs: Math.max(...lags.map((l) => l.lagMs ?? 99999)), final: final1.map((t) => [t.short, t.state, t.exec, t.sent]) };
  checks.add("L1", "REEL", "sans coupure : toute tâche qui a son résultat voit son exécution durable CLOSE (aucune exécution restée active après 25 s), 12 tâches (6 racines + 6 descendants) toutes `result_available` et remises",
    final1.length === 12 && final1.every((t) => t.hasResult && t.exec && !ACTIVE.includes(t.exec) && t.state === "result_available" && t.sent),
    { tasks: final1.length, notClosed: final1.filter((t) => !t.exec || ACTIVE.includes(t.exec)).map((t) => [t.short, t.state, t.exec]), maxLagMs: (notes.part1 as any).maxLagMs });
  checks.add("L2", "OBS", "OBSERVATION : décalage maximal entre la capture du résultat et la clôture de l'exécution durable, sans coupure (ms) - mesure, ni PASS ni FAIL",
    true, { maxLagMs: (notes.part1 as any).maxLagMs, lags });

  // ------------------------------------------------ Partie 2 : coupure dans la fenêtre
  const attempts: any[] = [];
  let hit: any;
  for (let attempt = 1; attempt <= 6 && !hit; attempt += 1) {
    const ids: string[] = [];
    const msgs: string[] = [];
    for (let i = 1; i <= 3; i += 1) {
      const d = await mcpA.call("bridget_delegate", { ...base, request_id: `cut-${attempt}-${i}`, task: `NONCE_c${attempt}${i} NESTED_149[fast]` });
      ids.push(d.payload.task_id);
      msgs.push(d.payload.message_id);
    }
    const tStart = Date.now();
    let window: any;
    while (Date.now() - tStart < 12000 && !window) {
      const s = snapshot();
      window = s.find((t) => t.hasResult && t.exec && ACTIVE.includes(t.exec) && !t.sent);
      if (window) break;
      await pause(15);
    }
    attempts.push({ attempt, windowSeen: !!window, window: window && { task: window.short, parent: window.parent, state: window.state, exec: window.exec, sent: window.sent } });
    if (!window) { await pause(6000); continue; }
    // coupure IMMÉDIATE (SIGTERM réel, PID vérifié)
    const before = snapshot();
    curPeer.close();
    await mcpA.close();
    await fx.stopDaemon();
    await pause(2500);
    await fx.startDaemon();
    curPeer = await fx.registerParent(THREAD_A);
    await fx.registerParent(THREAD_B);
    const tl: Array<{ atMs: number; v: any }> = [];
    const tw = Date.now();
    let last = "";
    while (Date.now() - tw < 45000) {
      const s = snapshot().filter((t) => ids.some((r) => r === t.id || t.parent === r.slice(0, 8)));
      const v = s.map((t) => [t.short, t.parent, t.state, t.exec, t.sent]);
      const j = JSON.stringify(v);
      if (j !== last) { tl.push({ atMs: Date.now() - tw, v }); last = j; }
      await pause(500);
    }
    const after = snapshot().filter((t) => ids.some((r) => r === t.id || t.parent === r.slice(0, 8)));
    hit = { attempt, window: attempts[attempts.length - 1].window, beforeCut: before.filter((t) => ids.some((r) => r === t.id || t.parent === r.slice(0, 8))).map((t) => [t.short, t.parent, t.state, t.exec, t.sent]), timeline: tl, after: after.map((t) => ({ short: t.short, parent: t.parent, state: t.state, exec: t.exec, sent: t.sent })), ids: ids.map((i) => i.slice(0, 8)) };
    await remount("L2");
  }
  notes.part2 = { attempts, hit };
  checks.add("L3", "OBS", "OBSERVATION : la fenêtre « résultat capturé / exécution encore active » a été atteinte par l'API puis coupée par un SIGTERM réel du daemon",
    true, { attempts: attempts.length, windowSeen: attempts.map((a) => a.windowSeen) });
  if (hit) {
    const staleActive = hit.after.filter((t: any) => t.exec && ACTIVE.includes(t.exec));
    const waiting = hit.after.filter((t: any) => t.state === "waiting_for_children");
    checks.add("L4", "REEL", "après coupure dans la fenêtre : aucune racine ne reste indéfiniment `waiting_for_children` (45 s) et aucune exécution durable ne reste ACTIVE pour une tâche qui a son résultat",
      waiting.length === 0 && staleActive.length === 0, { windowTask: hit.window, waitingRoots: waiting, staleActive, final: hit.after });
  } else {
    checks.add("L4", "OBS", "fenêtre non atteinte en 6 essais : aucune conclusion tirée (non prouvé)", true, { attempts });
  }
  await mcpA.close().catch(() => undefined);
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
await writeResults("exec-lag149", { schema: "native-network-recipes/1", startedAt: T0S, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
