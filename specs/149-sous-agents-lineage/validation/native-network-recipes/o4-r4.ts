// O4.6 (ronde r4) - redémarrage rapide du daemon avec un fournisseur dont l'interruption est bloquée (SLOW_149 : sommeil non interruptible).
// Binaire release r9 (NATIVE149_BIN). Sans modèle, sans Cargo. Un SEUL scénario : SIGTERM du daemon, relance IMMÉDIATE (sans attendre le wrapper).
// Attendu (correctif r9) : le nouveau daemon attend l'arrêt coopératif du wrapper (<= 8 s, jamais SIGKILL natif), aucun orphelin, un seul prompt.
import * as FS from "node:fs";
import * as ChildProcess from "node:child_process";
import * as OS from "node:os";
import * as Path from "node:path";
import { startT3Host, codexFullAccessFact } from "./t3host.ts";
import { Fx, Checks, pause, BIN, BIN_SHA256, HERE, sha256File, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, THREAD_B } from "./common149.ts";

process.env.NATIVE149_CODEX_PROVIDER = Path.join(HERE, "codex149b.py");
const PORT = 14816;
const checks = new Checks("O4.6 redémarrage rapide, fournisseur à interruption bloquée (r4)");
const T0S = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire release r9 : empreinte différente du reçu");
if (!(await freePort(PORT))) throw new Error(`port ${PORT} occupé`);
const host = await startT3Host(PORT);
const fx = await Fx.create("o4-r4", { port: PORT });
notes.fixtureRoot = fx.root;
const alive = (pid: number) => { try { process.kill(pid, 0); return true; } catch { return false; } };
const ps = (pid: number, field: string) => { try { return ChildProcess.execFileSync("/bin/ps", ["-p", String(pid), "-o", `${field}=`], { encoding: "utf8" }).trim(); } catch { return ""; } };
const markers = () => { try { return FS.readdirSync(Path.join(fx.state, "managed")).sort(); } catch { return []; } };
const base = { agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", cwd: fx.work };
let exitCode = 0;
let curPeer: any;
try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  notes.daemon1Pid = await fx.startDaemon();
  curPeer = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  const cA = (await host.issue(THREAD_A)).config;
  host.publish(cA, codexFullAccessFact(cA, "run-o4r4-1", fx.work), host.startRun(THREAD_A, "run-o4r4-1"));
  const mcp = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "O4R4" });
  const d = await mcp.call("bridget_delegate", { ...base, request_id: "o4r4-b1", task: "NONCE_o4r4 SLOW_149:20000" });
  await pause(2500);
  const prompt = fx.evidence().find((e: any) => e.event === "prompt" && e.nonce === "o4r4");
  const providerPid: number = prompt?.pid;
  const wrapperPid: number = prompt?.ppid;
  const tree = (await fx.processTree()).filter((p: any) => p.kind === "provider" || p.kind === "wrapper");
  const wRow = tree.find((p: any) => p.pid === wrapperPid);
  const pRow = tree.find((p: any) => p.pid === providerPid);
  const daemon1 = fx.daemon.pid;
  const before = {
    taskState: fx.tasks().find((t: any) => t.task_id === d.payload?.task_id)?.state,
    daemon1, wrapperPid, providerPid,
    wrapper: wRow && { ppid: wRow.ppid, pgid: wRow.pgid, birth: ps(wrapperPid, "lstart") },
    provider: pRow && { ppid: pRow.ppid, pgid: pRow.pgid, birth: ps(providerPid, "lstart") },
    markers: markers(), startsBefore: fx.evidence().filter((e: any) => e.event === "started").length,
  };
  notes.before = before;

  const samples: Array<{ ms: number; w: boolean; p: boolean }> = [];
  let wDead: number | undefined, pDead: number | undefined, stopSampling = false;
  const t0perf = performance.now();
  const sampler = (async () => {
    while (!stopSampling && performance.now() - t0perf < 30000) {
      const ms = Math.round(performance.now() - t0perf);
      const w = alive(wrapperPid), p = alive(providerPid);
      samples.push({ ms, w, p });
      if (!w && wDead === undefined) wDead = ms;
      if (!p && pDead === undefined) pDead = ms;
      if (wDead !== undefined && pDead !== undefined) break;
      await pause(20);
    }
  })();
  const t0epoch = Date.now() / 1000;
  await mcp.close();
  curPeer.close();
  const stopped = await fx.stopDaemon(); // SIGTERM individuel vérifié, PID parent = recette
  const daemon1GoneMs = Math.round(performance.now() - t0perf);
  const wrapperAliveAtDaemonGone = alive(wrapperPid);
  const providerAliveAtDaemonGone = alive(providerPid);
  // Relance IMMÉDIATE : aucune attente de l'arrêt du wrapper.
  const t1perf = performance.now();
  let daemon2Pid: number | undefined, startError: string | null = null;
  try { daemon2Pid = await fx.startDaemon(); } catch (e) { startError = String((e as Error).message); }
  const daemon2ReadyMs = Math.round(performance.now() - t0perf);
  const startDurationMs = Math.round(performance.now() - t1perf);
  const wrapperAliveAtDaemon2Ready = alive(wrapperPid);
  const providerAliveAtDaemon2Ready = alive(providerPid);
  if (!startError) { curPeer = await fx.registerParent(THREAD_A); await fx.registerParent(THREAD_B); }
  await sampler;
  stopSampling = true;
  await pause(3000);
  const ev = fx.evidence().filter((e: any) => e.pid === providerPid && e.t >= t0epoch - 0.05);
  const rel = (t: number) => +(t - t0epoch).toFixed(2);
  const events = ev.filter((e: any) => ["sigterm", "interrupt", "stdin_eof", "answered"].includes(e.event)).map((e: any) => ({ event: e.event, atS: rel(e.t), ignored: e.ignored, exact: e.exact }));
  const task = fx.tasks().find((t: any) => t.task_id === d.payload?.task_id);
  const exe = fx.execs().find((e: any) => e.execution_id === `execution-${task?.mission}`);
  const live = (await fx.liveFixtureProcesses()).filter((p: any) => p.kind !== "other").map((p: any) => ({ pid: p.pid, ppid: p.ppid, kind: p.kind }));
  const daemon2Log = FS.existsSync(Path.join(fx.root, "logs/daemon-2.log")) ? FS.readFileSync(Path.join(fx.root, "logs/daemon-2.log"), "utf8").split("\n").filter((l) => /managed|native|restart|stale|reconcil|refus|cooper/i.test(l)).slice(0, 12).map((l) => l.slice(0, 220)) : [];
  const after = {
    stopped, daemon1GoneMs: daemon1GoneMs, wrapperAliveAtDaemonGone, providerAliveAtDaemonGone,
    daemon2Pid, startError, daemon2ReadyMs, startDurationMs, wrapperAliveAtDaemon2Ready, providerAliveAtDaemon2Ready,
    wrapperDeadMs: wDead ?? null, providerDeadMs: pDead ?? null, events,
    prompts: fx.evidence().filter((e: any) => e.event === "prompt" && e.nonce === "o4r4").length,
    startsAfter: fx.evidence().filter((e: any) => e.event === "started").length,
    task: { state: task?.state, error: task?.error ?? null, sent: task?.result_sent }, exec: exe && { state: exe.state, reason: exe.reason },
    markersAfter: markers(), live, daemon2LogHints: daemon2Log,
  };
  notes.after = after;
  notes.samplesCount = samples.length;

  const shape = before.provider?.ppid === wrapperPid && before.provider?.pgid === providerPid && before.wrapper?.pgid !== before.provider?.pgid && before.wrapper?.ppid === daemon1;
  checks.add("O4r4.0", "REEL", "arbre avant coupure : daemon -> wrapper (groupe propre) -> fournisseur fermé (groupe PROPRE, ppid = wrapper) ; tâche `working` ; un marqueur de wrapper natif présent",
    shape && before.taskState === "working" && before.markers.length >= 1, { before });
  checks.add("O4r4.1", "REEL", "SIGTERM du daemon (PID individuel vérifié) : le daemon sort et le wrapper est encore vivant à ce moment (arrêt coopératif en cours, pas tué par le daemon)",
    stopped === "stopped_by_sigterm" && wrapperAliveAtDaemonGone === true, { stopped, daemon1GoneMs, wrapperAliveAtDaemonGone, providerAliveAtDaemonGone });
  checks.add("O4r4.2", "REEL", "relance immédiate : le nouveau daemon n'est prêt qu'APRÈS la mort du wrapper (attente coopérative), au plus 8 s après le SIGTERM du fournisseur ; aucun SIGKILL imposé (le fournisseur a reçu `sigterm` de son wrapper, non ignoré)",
    !startError && wDead !== undefined && wrapperAliveAtDaemon2Ready === false && after.events.some((e: any) => e.event === "sigterm" && e.ignored === false) && (wDead ?? 99999) <= 8000 + 500,
    { startError, wrapperDeadMs: wDead, daemon2ReadyMs, startDurationMs, events });
  checks.add("O4r4.3", "REEL", "aucun orphelin : wrapper ET fournisseur morts avant 8,5 s ; plus aucun processus de la fixture 3 s plus tard",
    wDead !== undefined && pDead !== undefined && wDead <= 8500 && pDead <= 8500 && live.length === 0, { wrapperDeadMs: wDead, providerDeadMs: pDead, live });
  checks.add("O4r4.4", "REEL", "aucun doublon : UN seul prompt (nonce), aucun nouveau lancement de fournisseur après la relance",
    after.prompts === 1 && after.startsAfter === before.startsBefore, { prompts: after.prompts, startsBefore: before.startsBefore, startsAfter: after.startsAfter });
  checks.add("O4r4.5", "OBS", "OBSERVATION : état durable après relance (tâche, exécution, marqueurs) - ne compte ni PASS ni FAIL",
    true, { task: after.task, exec: after.exec, markersBefore: before.markers, markersAfter: after.markersAfter, daemon2LogHints: daemon2Log });
  curPeer?.close();
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
await writeResults("o4-r4", { schema: "native-network-recipes/1", startedAt: T0S, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
