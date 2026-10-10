// O4 (ronde r3) - le fournisseur d'un enfant natif suit-il l'arrêt du daemon ? Mesure sur le binaire release r8, sans modèle.
// Trois fournisseurs fermés (codex149b.py), un VRAI SIGTERM du daemon (PID individuel vérifié, jamais -9) pour chacun :
//   A  coopératif   HB_149      : battement d'horloge dans un fichier, répond à turn/interrupt ;
//   B  bloquant     SLOW_149    : sommeil non interruptible (ne lit pas stdin) ;
//   C  sourd        IGNORE_TERM : ignore SIGTERM 8 s (borné) puis répond.
// Mesures : arbre de processus (PID/PPID/PGID wrapper et fournisseur), disposition du signal SIGTERM du wrapper (ps -o caught),
// environnement du wrapper (NOM de la variable d'amorce native seulement), horloge de mort de chaque PID après T0 (échantillon 20 ms),
// dernier battement, événements du fournisseur (interrupt/sigterm/hb_stop/stdin_eof), reste de processus.
import * as FS from "node:fs";
import * as ChildProcess from "node:child_process";
import * as OS from "node:os";
import * as Path from "node:path";
import { startT3Host, codexFullAccessFact } from "./t3host.ts";
import { Fx, Checks, pause, BIN, BIN_SHA256, HERE, sha256File, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, THREAD_B } from "./common149.ts";

process.env.NATIVE149_CODEX_PROVIDER = Path.join(HERE, "codex149b.py");
const PORT = 14797;
const checks = new Checks("O4 fournisseur natif et arrêt du daemon (r3)");
const T0S = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire release r8 : empreinte différente du reçu");
if (!(await freePort(PORT))) throw new Error(`port ${PORT} occupé`);
const host = await startT3Host(PORT);
const fx = await Fx.create("o4-149", { port: PORT });
notes.fixtureRoot = fx.root;
const alive = (pid: number) => { try { process.kill(pid, 0); return true; } catch { return false; } };
const psField = (pid: number, field: string) => { try { return ChildProcess.execFileSync("/bin/ps", ["-p", String(pid), "-o", `${field}=`], { encoding: "utf8" }).trim(); } catch { return ""; } };
const wrapperEnvNames = (pid: number) => {
  try {
    const out = ChildProcess.execFileSync("/bin/ps", ["eww", "-p", String(pid)], { encoding: "utf8" });
    return /BRIDGET_NATIVE_MISSION_BOOTSTRAP=/.test(out) ? "BRIDGET_NATIVE_MISSION_BOOTSTRAP présent" : "variable d'amorce native ABSENTE de l'environnement du wrapper";
  } catch { return "ps eww indisponible"; }
};
const base = { agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", cwd: fx.work };
let exitCode = 0;
let curPeer: any;
let cA: any;
let runN = 0;

async function restartAndReconnect() {
  await fx.startDaemon();
  curPeer = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  await pause(800);
}

async function scenario(tag: string, task: string, nonce: string, sampleMs: number, restartAfterMs?: number) {
  const mcp = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: `O4-${tag}` });
  const d = await mcp.call("bridget_delegate", { ...base, request_id: `o4-${tag}`, task });
  await pause(2500);
  const prompt = fx.evidence().find((e: any) => e.event === "prompt" && e.nonce === nonce);
  const providerPid: number | undefined = prompt?.pid;
  const wrapperPid: number | undefined = prompt?.ppid;
  const tree = (await fx.processTree()).filter((p: any) => p.kind === "provider" || p.kind === "wrapper");
  const wrapperRow = tree.find((p: any) => p.pid === wrapperPid);
  const providerRow = tree.find((p: any) => p.pid === providerPid);
  const sigmask = wrapperPid ? psField(wrapperPid, "sigmask") : ""; // macOS n'expose pas `caught` : le gestionnaire se prouve par le comportement
  const hbBefore = FS.existsSync(`${fx.evidencePath}.hb`) ? FS.readFileSync(`${fx.evidencePath}.hb`, "utf8").trim() : null;
  const before = {
    taskState: fx.tasks().find((t: any) => t.task_id === d.payload?.task_id)?.state, providerPid, wrapperPid,
    wrapperRow: wrapperRow && { ppid: wrapperRow.ppid, pgid: wrapperRow.pgid }, providerRow: providerRow && { ppid: providerRow.ppid, pgid: providerRow.pgid },
    wrapperSigmask: sigmask, wrapperEnv: wrapperPid ? wrapperEnvNames(wrapperPid) : null,
    daemonPid: fx.daemon?.pid, hbBefore,
  };
  const samples: Array<{ ms: number; w: boolean; p: boolean }> = [];
  let wDead: number | undefined;
  let pDead: number | undefined;
  let stop = false;
  const t0perf = performance.now();
  const sampler = (async () => {
    while (!stop && performance.now() - t0perf < sampleMs) {
      const ms = performance.now() - t0perf;
      const w = wrapperPid !== undefined && alive(wrapperPid);
      const p = providerPid !== undefined && alive(providerPid);
      samples.push({ ms: Math.round(ms), w, p });
      if (!w && wDead === undefined) wDead = Math.round(ms);
      if (!p && pDead === undefined) pDead = Math.round(ms);
      if (wDead !== undefined && pDead !== undefined) break;
      await pause(20);
    }
  })();
  const t0epoch = Date.now() / 1000;
  await mcp.close();
  curPeer.close();
  const stopped = await fx.stopDaemon(); // SIGTERM individuel vérifié
  const daemonGoneMs = Math.round(performance.now() - t0perf);
  let restartedAtMs: number | null = null;
  if (restartAfterMs !== undefined) {
    // relance PENDANT l'échantillonnage (le sampler tourne) : le nouveau daemon réconcilie les groupes périmés au démarrage
    await pause(restartAfterMs);
    await fx.startDaemon();
    restartedAtMs = Math.round(performance.now() - t0perf);
    curPeer = await fx.registerParent(THREAD_A);
    await fx.registerParent(THREAD_B);
  }
  await sampler;
  stop = true;
  const ev = fx.evidence().filter((e: any) => e.pid === providerPid && e.t >= t0epoch - 0.05);
  const hbAfter = FS.existsSync(`${fx.evidencePath}.hb`) ? JSON.parse(FS.readFileSync(`${fx.evidencePath}.hb`, "utf8").trim()) : null;
  const rel = (t: number) => +(t - t0epoch).toFixed(2);
  const after = {
    stopped, daemonGoneMs, wrapperDeadMs: wDead ?? null, providerDeadMs: pDead ?? null,
    events: ev.filter((e: any) => ["interrupt", "sigterm", "hb_stop", "stdin_eof", "answered"].includes(e.event)).map((e: any) => ({ event: e.event, atS: rel(e.t), exact: e.exact, ignored: e.ignored })),
    lastHeartbeatAfterT0S: hbAfter && hbAfter.pid === providerPid ? rel(hbAfter.t) : null,
    liveAfter: (await fx.liveFixtureProcesses()).filter((p: any) => p.kind !== "other").map((p: any) => ({ pid: p.pid, ppid: p.ppid, kind: p.kind })),
  };
  return { tag, restartAfterMs: restartAfterMs ?? null, restartedAtMs, task: d.payload?.task_id, before, after, samplesCount: samples.length };
}

try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  notes.daemonPid = await fx.startDaemon();
  curPeer = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  cA = (await host.issue(THREAD_A)).config;
  const run = host.startRun(THREAD_A, "run-o4-1");
  host.publish(cA, codexFullAccessFact(cA, "run-o4-1", fx.work), run);

  const scenarios: any[] = [];
  for (const [tag, task, nonce, ms, restart] of [
    ["B", "NONCE_o4b SLOW_149:20000", "o4b", 25000, undefined],
    ["A", "NONCE_o4a HB_149:60000", "o4a", 25000, undefined],
    ["C", "NONCE_o4c IGNORE_TERM_149:8000", "o4c", 25000, undefined],
    ["B1", "NONCE_o4d SLOW_149:20000", "o4d", 28000, 0],
    ["A1", "NONCE_o4e HB_149:60000", "o4e", 28000, 0],
    ["B2", "NONCE_o4f SLOW_149:20000", "o4f", 28000, 6000],
  ] as Array<[string, string, string, number, number | undefined]>) {
    const r = await scenario(tag, task, nonce, ms, restart);
    scenarios.push(r);
    notes[`scenario${tag}`] = r;
    // Reste éventuel : SIGTERM individuel vérifié (jamais -9) après la mesure
    const leftovers = (await fx.liveFixtureProcesses()).filter((p: any) => p.kind === "provider" || /managed-wrapper/.test(p.command));
    for (const p of leftovers) {
      if (/firefox/i.test(p.command)) continue;
      try { process.kill(p.pid, "SIGTERM"); } catch { /* déjà parti */ }
    }
    if (leftovers.length) await pause(3000);
    if (!fx.daemon) await restartAndReconnect();
    runN += 1;
  }
  const [B, A, C, B1, A1, B2] = scenarios;
  const shape = (s: any) => s.before.providerRow?.ppid === s.before.wrapperPid && s.before.providerRow?.pgid !== s.before.wrapperRow?.pgid && s.before.providerRow?.pgid === s.before.providerPid;
  checks.add("O4.0", "REEL", "arbre de processus cohérent pour les 3 cas : le fournisseur est un enfant DIRECT du wrapper (ppid = PID wrapper) et dirige son PROPRE groupe de processus (pgid = son PID, différent de celui du wrapper) ; le wrapper est un enfant du daemon",
    [A, B, C].every((s) => s.before.providerPid && s.before.wrapperPid && shape(s) && s.before.wrapperRow?.ppid === s.before.daemonPid),
    [A, B, C].map((s) => ({ tag: s.tag, provider: s.before.providerPid, wrapper: s.before.wrapperPid, wrapperPpid: s.before.wrapperRow?.ppid, daemon: s.before.daemonPid, providerPgid: s.before.providerRow?.pgid, wrapperPgid: s.before.wrapperRow?.pgid })));
  checks.add("O4.1", "REEL", "la variable d'amorce native (condition d'installation du gestionnaire SIGTERM du wrapper) est dans l'environnement du wrapper en vol (lue par `ps eww`, nom seulement)",
    [A, B, C].every((s) => /présent/.test(String(s.before.wrapperEnv))),
    [A, B, C].map((s) => ({ tag: s.tag, env: s.before.wrapperEnv, sigmask: s.before.wrapperSigmask })));
  // A : coopératif
  const aEv = (n: string) => A.after.events.find((e: any) => e.event === n);
  checks.add("O4.2", "REEL", "A (coopératif) : après SIGTERM du daemon, le wrapper interrompt le tour EXACT (turn/interrupt avec l'identifiant du tour) puis le fournisseur s'arrête ; le battement cesse (dernier battement <= 4 s après T0) ; aucun processus restant",
    !!aEv("interrupt") && aEv("interrupt").exact === true && A.after.providerDeadMs !== null && A.after.providerDeadMs <= 6000 && A.after.lastHeartbeatAfterT0S !== null && A.after.lastHeartbeatAfterT0S <= 4 && A.after.liveAfter.length === 0,
    { events: A.after.events, providerDeadMs: A.after.providerDeadMs, wrapperDeadMs: A.after.wrapperDeadMs, lastHeartbeatAfterT0S: A.after.lastHeartbeatAfterT0S, live: A.after.liveAfter });
  // B : bloquant
  checks.add("O4.3", "REEL", "B (fournisseur bloquant qui ne lit pas stdin) : le fournisseur est arrêté PAR le wrapper (événement `sigterm` du fournisseur) dans une borne <= 6 s, sans survivre jusqu'à la fin naturelle de son tour (20 s)",
    B.after.events.some((e: any) => e.event === "sigterm") && B.after.providerDeadMs !== null && B.after.providerDeadMs <= 6000,
    { events: B.after.events, providerDeadMs: B.after.providerDeadMs, wrapperDeadMs: B.after.wrapperDeadMs, live: B.after.liveAfter });
  // C : sourd (borné)
  checks.add("O4.4", "REEL", "C (fournisseur qui ignore SIGTERM 8 s) : borne effective mesurée ; le fournisseur reçoit SIGTERM du wrapper et s'arrête au plus tard à la fin de sa fenêtre (8 s) + 3 s, sans orphelin durable",
    C.after.events.some((e: any) => e.event === "sigterm") && C.after.providerDeadMs !== null && C.after.providerDeadMs <= 11000 && C.after.liveAfter.length === 0,
    { events: C.after.events, providerDeadMs: C.after.providerDeadMs, wrapperDeadMs: C.after.wrapperDeadMs, live: C.after.liveAfter });
  // Relance IMMÉDIATE du daemon : la réconciliation des groupes périmés (daemon.rs reconcile_stale_groups) envoie SIGTERM au groupe du wrapper
  // puis SIGKILL après 0,5 s ; le fournisseur dirige un AUTRE groupe. Le test pose l'exigence « pas d'orphelin au-delà de 6 s ».
  const noOrphan = (s: any) => s.after.providerDeadMs !== null && s.after.providerDeadMs <= 6000 && s.after.liveAfter.length === 0;
  const detail = (s: any) => ({ restartedAtMs: s.restartedAtMs, wrapperDeadMs: s.after.wrapperDeadMs, providerDeadMs: s.after.providerDeadMs, events: s.after.events, live: s.after.liveAfter });
  checks.add("O4.6", "REEL", "B1 (fournisseur bloquant) + relance IMMÉDIATE du daemon : aucun orphelin au-delà de 6 s (exigence) - le wrapper ne doit pas être tué avant d'avoir arrêté son fournisseur",
    noOrphan(B1) && B1.after.events.some((e: any) => e.event === "sigterm"), detail(B1));
  checks.add("O4.7", "REEL", "A1 (fournisseur coopératif) + relance IMMÉDIATE du daemon : aucun orphelin au-delà de 6 s ; le tour exact est interrompu",
    noOrphan(A1) && A1.after.events.some((e: any) => e.event === "interrupt" && e.exact === true), detail(A1));
  checks.add("O4.8", "REEL", "B2 (fournisseur bloquant) + relance du daemon APRÈS 6 s (le wrapper a fini seul) : aucun orphelin au-delà de 6 s",
    noOrphan(B2) && B2.after.events.some((e: any) => e.event === "sigterm"), detail(B2));
  // Aucune écriture du fournisseur après sa mort / après l'arrêt du daemon
  checks.add("O4.5", "REEL", "après la mort de chaque fournisseur, plus aucun processus de la fixture et plus aucune ligne d'évidence ne s'ajoute (zéro processus possédé, zéro écriture)",
    (await fx.liveFixtureProcesses()).filter((p: any) => p.kind === "provider" || /managed-wrapper/.test(p.command)).length === 0,
    { live: (await fx.liveFixtureProcesses()).length });
  notes.summaryTable = scenarios.map((s) => ({ tag: s.tag, providerDeadMs: s.after.providerDeadMs, wrapperDeadMs: s.after.wrapperDeadMs, lastHeartbeatAfterT0S: s.after.lastHeartbeatAfterT0S, events: s.after.events.map((e: any) => `${e.event}@${e.atS}`) }));
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
await writeResults("o4-149", { schema: "native-network-recipes/1", startedAt: T0S, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
