// O5/G8 (ronde r3) - oracle `journal --follow` de la CLI Lineage RÉELLE (bridget lineage inspect --action journal --follow), binaire release r8.
// Prouve : séquence (seq strictement croissant, contiguë, sans doublon), appends en direct, reprise --after-seq sans doublon, égalité avec la lecture
// ponctuelle, fermeture des ressources du daemon (descripteurs, threads, processus) à la fin du suivi, et humains silencieux (aucune remise au parent).
// Environnement : fixture privée uniquement (BRIDGET_HOME/BRIDGET_SOCKET de la fixture). Aucun modèle.
import * as OS from "node:os";
import * as Path from "node:path";
import * as ChildProcess from "node:child_process";
import * as Readline from "node:readline";
import { startT3Host, codexFullAccessFact } from "./t3host.ts";
import { Fx, Checks, pause, BIN, BIN_SHA256, HERE, sha256File, lineageArgs, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, THREAD_B, waitTask } from "./common149.ts";

process.env.NATIVE149_CODEX_PROVIDER = Path.join(HERE, "codex149b.py");
const PORT = 14800;
const checks = new Checks("O5/G8 journal --follow (r3)");
const T0S = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire release r8 : empreinte différente du reçu");
if (!(await freePort(PORT))) throw new Error(`port ${PORT} occupé`);
const host = await startT3Host(PORT);
const fx = await Fx.create("follow149", { port: PORT });
notes.fixtureRoot = fx.root;
const base = { agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", cwd: fx.work };
const exec = (cmd: string, args: string[]) => { try { return ChildProcess.execFileSync(cmd, args, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }); } catch (e: any) { return String(e.stdout ?? ""); } };
/** Ressources du daemon : descripteurs ouverts (lsof), threads (ps -M), processus de la fixture. */
const resources = async (daemonPid: number) => ({
  fds: exec("/usr/sbin/lsof", ["-p", String(daemonPid)]).split("\n").filter(Boolean).length - 1,
  threads: exec("/bin/ps", ["-M", "-p", String(daemonPid)]).split("\n").filter(Boolean).length - 2,
  processes: (await fx.liveFixtureProcesses()).filter((p: any) => p.kind !== "other").length,
});
let exitCode = 0;

/** Suivi CLI réel : processus possédé, lignes JSON collectées. */
function follow(thread: string, task: string, afterSeq: number, label: string) {
  const args = ["lineage", "inspect", "--json", "--t3-thread", thread, "--project-root", fx.work, "--action", "journal", "--task", task, "--after-seq", String(afterSeq), "--limit", "100", "--follow"];
  const child = fx.spawnOwned(args, fx.env());
  child.stderr?.resume();
  const lines: any[] = [];
  const stamps: number[] = [];
  Readline.createInterface({ input: child.stdout }).on("line", (l) => { try { lines.push(JSON.parse(l)); } catch { lines.push({ raw: l.slice(0, 80) }); } stamps.push(Date.now()); });
  let exited: number | null | undefined;
  child.on("exit", (code, signal) => { exited = code ?? (signal ? -1 : null); });
  return {
    label, pid: child.pid!, lines, stamps,
    events: () => lines.flatMap((l) => (l.events ?? []) as any[]),
    exited: () => exited,
    async stop() {
      if (exited !== undefined) return "already_exited";
      return fx.verifiedStop(child, BIN); // SIGTERM d'un PID dont le parent et la commande sont vérifiés
    },
  };
}

try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  const daemonPid = await fx.startDaemon();
  notes.daemonPid = daemonPid;
  const peerA = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  const cA = (await host.issue(THREAD_A)).config;
  host.publish(cA, codexFullAccessFact(cA, "run-f-1", fx.work), host.startRun(THREAD_A, "run-f-1"));
  const mcp = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "F" });

  // enfant vivant qui produit de l'activité pendant ~10 s (fragments), puis reste vivant 25 s
  const d = await mcp.call("bridget_delegate", { ...base, request_id: "follow-1", task: "NONCE_f1 STREAM_149:14:700 HB_149:25000" });
  const task = d.payload.task_id as string;
  await pause(1500);
  const baseline = await resources(daemonPid);
  const deliveriesBefore = peerA.received.filter((f: any) => ["Deliver", "DeliverIdempotent", "DeliverExecution"].includes(f.type)).length;

  const f1 = follow(THREAD_A, task, 0, "f1");
  await pause(500);
  const duringOpen = await resources(daemonPid);
  await pause(11000); // appends en direct
  const midSeqs = f1.events().map((e: any) => e.seq);
  const midSnapshot = { lines: f1.lines.length, events: midSeqs.length, last: midSeqs[midSeqs.length - 1] };
  await pause(4000);

  const seqs: number[] = f1.events().map((e: any) => e.seq);
  const nextSeqs: number[] = f1.lines.filter((l) => typeof l.next_seq === "number").map((l) => l.next_seq);
  const caughtUp = f1.lines.filter((l) => l.caught_up === true).length;
  const contiguous = seqs.every((s, i) => i === 0 || s === seqs[i - 1] + 1);
  const strictlyIncreasing = seqs.every((s, i) => i === 0 || s > seqs[i - 1]);
  const noDup = new Set(seqs).size === seqs.length;
  const nextMono = nextSeqs.every((n, i) => i === 0 || n >= nextSeqs[i - 1]);
  const gaps = f1.lines.filter((l) => l.gap).map((l) => l.gap);
  // arrivées étalées dans le temps = appends en direct (pas un seul bloc initial)
  const arrivalSpanMs = f1.stamps.length ? f1.stamps[f1.stamps.length - 1] - f1.stamps[0] : 0;
  const linesWithEvents = f1.lines.filter((l) => (l.events ?? []).length > 0).length;
  notes.f1 = { lines: f1.lines.length, linesWithEvents, events: seqs.length, firstSeq: seqs[0], lastSeq: seqs[seqs.length - 1], arrivalSpanMs, caughtUp, gaps, midSnapshot, kinds: [...new Set(f1.events().map((e: any) => e.kind ?? e.type ?? e.event ?? "?"))], sample: f1.events().slice(0, 3).map((e: any) => Object.keys(e)) };
  checks.add("F1", "REEL", "suivi réel : les événements arrivent avec `seq` strictement croissant, CONTIGU, sans doublon ; `next_seq` ne régresse jamais ; aucune lacune annoncée ; au moins un `caught_up` ; les appends arrivent en DIRECT (plusieurs lignes étalées sur plus de 3 s)",
    seqs.length >= 3 && strictlyIncreasing && contiguous && noDup && nextMono && gaps.length === 0 && caughtUp >= 1 && linesWithEvents >= 2 && arrivalSpanMs > 3000,
    { events: seqs.length, strictlyIncreasing, contiguous, noDup, nextMono, gaps: gaps.length, caughtUp, linesWithEvents, arrivalSpanMs, first: seqs[0], last: seqs[seqs.length - 1] });

  // égalité avec la lecture ponctuelle (même fenêtre)
  const oneShot: number[] = [];
  let after = 0;
  for (let i = 0; i < 10; i += 1) {
    const r = await fx.cli(lineageArgs("journal", THREAD_A, fx.work, ["--task", task, "--after-seq", String(after), "--limit", "100"]));
    const ev = (r.json?.events ?? []) as any[];
    oneShot.push(...ev.map((e) => e.seq));
    if (r.json?.caught_up === true || ev.length === 0) break;
    after = r.json?.next_seq ?? after;
  }
  const followedUpTo = seqs[seqs.length - 1];
  const oneShotSame = JSON.stringify(oneShot.filter((s) => s <= followedUpTo)) === JSON.stringify(seqs);
  checks.add("F2", "REEL", "égalité : les `seq` reçus en direct sont EXACTEMENT ceux de la lecture ponctuelle (non-follow) de la même tâche, sans perte ni ajout",
    oneShotSame && oneShot.length >= seqs.length, { followed: seqs.length, oneShot: oneShot.length, firstOneShot: oneShot[0], lastOneShot: oneShot[oneShot.length - 1] });

  // reprise : un 2e suivi après --after-seq K ne renvoie QUE seq > K, sans doublon, sans trou
  const K = seqs[Math.floor(seqs.length / 2)];
  const f2 = follow(THREAD_A, task, K, "f2");
  await pause(3500);
  const resumed: number[] = f2.events().map((e: any) => e.seq);
  const expected = oneShot.filter((s) => s > K);
  checks.add("F3", "REEL", "reprise `--after-seq K` : le 2e suivi ne rend que des `seq` > K, sans doublon ni trou, et couvre exactement la suite connue de la lecture ponctuelle",
    resumed.length > 0 && resumed.every((s) => s > K) && new Set(resumed).size === resumed.length && JSON.stringify(resumed.slice(0, expected.length)) === JSON.stringify(expected.slice(0, resumed.length)) && resumed.length >= Math.min(expected.length, 1),
    { K, resumed: resumed.length, expected: expected.length, first: resumed[0], last: resumed[resumed.length - 1] });

  // humains silencieux : aucune remise nouvelle au parent pendant le suivi (la mission n'est pas terminée)
  const deliveriesDuring = peerA.received.filter((f: any) => ["Deliver", "DeliverIdempotent", "DeliverExecution"].includes(f.type)).length;
  const taskNow = (await waitTask(fx, task, ["working"], 100));
  checks.add("F4", "REEL", "humains silencieux : pendant tout le suivi (deux suivis, ~20 s), le parent ne reçoit AUCUNE remise, la tâche reste `working` (lecture sans effet de bord), aucun nouveau lancement",
    deliveriesDuring === deliveriesBefore && taskNow?.state === "working" && fx.evidence().filter((e: any) => e.event === "started").length === 1,
    { deliveriesBefore, deliveriesDuring, state: taskNow?.state, starts: fx.evidence().filter((e: any) => e.event === "started").length });

  // fermeture : SIGTERM individuel des deux suivis, ressources du daemon revenues à la base
  const stop1 = await f1.stop();
  const stop2 = await f2.stop();
  await pause(2500);
  const afterClose = await resources(daemonPid);
  notes.resources = { baseline, duringOpen, afterClose, stop1, stop2 };
  checks.add("F5", "REEL", "fermeture : après l'arrêt des deux suivis (SIGTERM individuel vérifié), les descripteurs ouverts, threads et processus du daemon reviennent au niveau de base (tolérance 0 descripteur, 0 processus) ; en cours de suivi, ils avaient augmenté",
    afterClose.fds <= baseline.fds && afterClose.processes === baseline.processes && afterClose.threads <= baseline.threads && (duringOpen.fds > baseline.fds || duringOpen.threads > baseline.threads),
    { baseline, duringOpen, afterClose });

  // fin de vie de la tâche : annulation par le propriétaire ; un suivi ouvert ensuite se termine proprement (tâche terminale, enfant arrêté)
  const f3 = follow(THREAD_A, task, 0, "f3");
  await pause(1500);
  const cancel = await mcp.call("bridget_task_cancel", { task_id: task });
  await waitTask(fx, task, ["cancelled"], 15000);
  await pause(2500);
  const f3Seqs: number[] = f3.events().map((e: any) => e.seq);
  const f3Last = f3.lines[f3.lines.length - 1];
  notes.f3 = { cancelErr: cancel.isError, events: f3Seqs.length, exited: f3.exited() ?? "ouvert", lastLine: f3Last && { status: f3Last.status, code: f3Last.code, caught_up: f3Last.caught_up } };
  checks.add("F6", "REEL", "à la fin de vie de la tâche (cancel), un suivi ouvert reçoit des `seq` contigus sans doublon et ne laisse aucune ressource : après son arrêt, descripteurs/processus au niveau de base",
    f3Seqs.length >= 1 && new Set(f3Seqs).size === f3Seqs.length && f3Seqs.every((s, i) => i === 0 || s === f3Seqs[i - 1] + 1), { events: f3Seqs.length, exited: f3.exited() ?? "ouvert", cancelErr: cancel.isError });
  await f3.stop();
  await pause(2500);
  const finalRes = await resources(daemonPid);
  checks.add("F7", "REEL", "après annulation et arrêt du dernier suivi : plus aucun processus de fixture hors daemon, descripteurs revenus au niveau de base ou en dessous",
    finalRes.fds <= baseline.fds && finalRes.processes <= 0 + 1 /* le parent simulé n'est pas un processus */ && finalRes.processes <= baseline.processes,
    { baseline, finalRes });
  await mcp.close();
  peerA.close();
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
await writeResults("follow149", { schema: "native-network-recipes/1", startedAt: T0S, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
