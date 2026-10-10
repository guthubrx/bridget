// T040 / SC001 - UI web RÉELLE native (ronde r3) : serveur T3 complet (bin.ts) + UI web du worktree (Vite+) + VRAI daemon Bridget (release r8) +
// enfants Codex fermés (aucun modèle, « fixture » explicite). Une seule instance T3 émet le credential MCP ET sert Lineage (comme unified149.ts).
// Ce programme TIENT la fixture ; le navigateur est piloté à côté (outils preview T3) ; il réagit à des marqueurs dans <fixture>/ctl/ :
//   report       -> écrit ctl/report.json (tâches, exécutions, PID vivants)       restart-t3 -> SIGTERM vérifié du serveur T3 puis relance
//   restart-daemon -> SIGTERM vérifié du daemon puis relance + reconnexion        finish     -> nettoyage et sortie
// SIMULÉ (nommé) : wrapper du fil T3 côté daemon (trames Register/T3ThreadBindingFact émises ici), pair Codex de T3 (réponses capturées), enfants (app-server fermé).
import * as FS from "node:fs";
import * as OS from "node:os";
import * as Path from "node:path";
import * as Crypto from "node:crypto";
import * as ChildProcess from "node:child_process";
import { Fx, Checks, pause, BIN, BIN_SHA256, HERE, sha256File, writeResults, freePort } from "./fx.mjs";
import { THREAD_A } from "./common149.ts";
import { startT3Server, createProjectAndThreads, NODE_BIN, T3_WT } from "./t3server.ts";

process.env.NATIVE149_CODEX_PROVIDER = Path.join(HERE, "codex149b.py");
const T3_FIX = `${T3_WT}/apps/server/src/provider/testFixtures`;
const SERVER_PORT = 15758;
const WEB_PORT = 15778;
const checks = new Checks("T040 UI web réelle native (r3)");
const T0S = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire release r8 : empreinte différente du reçu");
for (const p of [SERVER_PORT, WEB_PORT]) if (!(await freePort(p))) throw new Error(`port ${p} occupé`);

const fx = await Fx.create("ui-native149", { port: SERVER_PORT });
notes.fixtureRoot = fx.root;
const ctl = Path.join(fx.root, "ctl");
FS.mkdirSync(ctl, { recursive: true, mode: 0o700 });
const fake = Path.join(fx.root, "fakebin");
FS.mkdirSync(fake, { recursive: true, mode: 0o700 });
for (const f of ["codexCollabMockPeer.mjs", "codexMultiAgentWire.json"]) FS.copyFileSync(Path.join(T3_FIX, f), Path.join(fake, f));
FS.copyFileSync(Path.join(HERE, "codexspy.mjs"), Path.join(fake, "codexspy.mjs"));
FS.writeFileSync(Path.join(fake, "script.json"), JSON.stringify({ rootThreadId: "probe-thread", notifications: [], holdTurnOpen: true }), { mode: 0o600 });
FS.writeFileSync(Path.join(fake, "codex"), `#!/bin/sh\nexec "${NODE_BIN}/node" "${fake}/codexspy.mjs" "$@"\n`, { mode: 0o700 });
const spyLog = Path.join(fx.root, "evidence", "codex-spy.jsonl");
const spyLines = () => (FS.existsSync(spyLog) ? FS.readFileSync(spyLog, "utf8").split("\n").filter(Boolean).map((l) => { try { return JSON.parse(l); } catch { return {}; } }) : []);
const alive = (pid: number) => { try { process.kill(pid, 0); return true; } catch { return false; } };
const marker = (n: string) => FS.existsSync(Path.join(ctl, n));
const eat = (n: string) => { try { FS.unlinkSync(Path.join(ctl, n)); } catch { /* absent */ } };
const base = { agent_type: "fixture-codex-149", effort: "high", cwd: fx.work };
let server: Awaited<ReturnType<typeof startT3Server>> | undefined;
let vite: ChildProcess.ChildProcess | undefined;
let exitCode = 0;
let peerA: any;

const startServer = () => startT3Server(fx, SERVER_PORT, { allowedOrigins: `http://localhost:${WEB_PORT}`, extraEnv: { SPY_LOG: spyLog, T3_CODEX_COLLAB_SCRIPT: Path.join(fake, "script.json") } });

try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  notes.daemonPid = await fx.startDaemon();
  peerA = await fx.registerParent(THREAD_A);
  FS.mkdirSync(Path.join(fx.root, "t3home", "userdata"), { recursive: true, mode: 0o700 });
  FS.writeFileSync(Path.join(fx.root, "t3home", "userdata", "settings.json"), JSON.stringify({ providers: { codex: { enabled: true, binaryPath: Path.join(fake, "codex") }, claudeAgent: { enabled: false }, cursor: { enabled: false }, grok: { enabled: false }, opencode: { enabled: false } } }), { mode: 0o600 });
  server = await startServer();
  const proj = await createProjectAndThreads(server, fx.work, [{ id: THREAD_A, title: "Conversation A (recette native)" }]);
  await server.dispatch(server.tokens.operate, {
    type: "message.dispatch", commandId: Crypto.randomUUID(), threadId: THREAD_A, messageId: Crypto.randomUUID(), text: "message de recette (aucun modèle)", attachments: [],
    createdBy: "user", creationSource: "web", dispatchMode: { type: "start_immediately" },
  });
  const until = Date.now() + 30000;
  while (!spyLines().some((l) => l.method === "turn/start") && Date.now() < until) await pause(100);
  await pause(800);
  const mcpCfg = spyLines().find((l) => l.method === "thread/start")?.params?.config?.["mcp_servers.t3-code"];
  if (!mcpCfg) throw new Error("credential non émis");
  const mcp = await fx.mcp({ endpoint: mcpCfg.url, authorization: mcpCfg.http_headers?.Authorization, label: "UI" });

  // UI web du worktree (Vite+), proxy vers CE serveur T3
  const viteEnv = { PATH: `${NODE_BIN}:/usr/bin:/bin:/usr/sbin:/sbin`, HOME: Path.join(fx.root, "web-user"), TMPDIR: Path.join(fx.root, "tmp"), PORT: String(WEB_PORT), T3CODE_PORT: String(SERVER_PORT) };
  FS.mkdirSync(viteEnv.HOME, { recursive: true, mode: 0o700 });
  const viteLog = FS.openSync(Path.join(fx.root, "logs", "vite.log"), "a", 0o600);
  vite = ChildProcess.spawn(`${T3_WT}/node_modules/.bin/vp`, ["dev"], { cwd: `${T3_WT}/apps/web`, env: viteEnv, stdio: ["ignore", viteLog, viteLog] });
  vite.on("error", () => undefined);
  const vUntil = Date.now() + 90000;
  for (;;) {
    const up = await fetch(`http://localhost:${WEB_PORT}/`).then((r) => r.status < 500, () => false);
    if (up) break;
    if (Date.now() >= vUntil) throw new Error("Vite indisponible");
    await pause(500);
  }
  const pairing = server.cli(["auth", "pairing", "create", "--ttl", "30m", "--base-url", `http://localhost:${WEB_PORT}`]);
  FS.writeFileSync(Path.join(ctl, "pairing.txt"), pairing + "\n", { mode: 0o600 });

  // enfants : R1 racine qui attend un descendant actif (journal en direct) ; R2 modèle « mini » qui se termine ; R3 actif, à arrêter depuis l'UI
  const mk = (id: string, model: string, task: string) => mcp.call("bridget_delegate", { ...base, model, request_id: id, task });
  const r1 = await mk("ui-r1", "fixture-model-149", "NONCE_ui1 NESTED_149[STREAM_149:200:1200 HB_149:900000]");
  const r2 = await mk("ui-r2", "fixture-model-149-mini", "NONCE_ui2 STREAM_149:5:600");
  const r3 = await mk("ui-r3", "fixture-model-149", "NONCE_ui3 STREAM_149:200:1200 HB_149:900000");
  notes.admitted = [r1, r2, r3].map((r) => r.payload?.status ?? r.payload?.text);
  FS.writeFileSync(Path.join(ctl, "info.json"), JSON.stringify({
    ready: true, serverPort: SERVER_PORT, webPort: WEB_PORT, threadId: THREAD_A, projectId: proj.projectId, daemonPid: fx.daemon?.pid,
    tasks: { r1: r1.payload?.task_id, r2: r2.payload?.task_id, r3: r3.payload?.task_id }, root: fx.root,
  }, null, 1), { mode: 0o600 });
  console.log("UI-NATIVE-READY", fx.root);

  const report = async () => {
    const tasks = fx.tasks().map((t: any) => ({ id: String(t.task_id).slice(0, 8), parent: t.parent_task_id ? String(t.parent_task_id).slice(0, 8) : null, state: t.state, error: t.error ?? null, model: t.request?.model ?? t.request?.Delegate?.model ?? null, sent: t.result_sent }));
    const prompts = fx.evidence().filter((e: any) => e.event === "prompt").map((e: any) => ({ nonce: e.nonce, pid: e.pid, alive: alive(e.pid), model: e.model }));
    const runs = server!.sql("select thread_id, status from orchestration_v2_projection_runs");
    const sessions = server!.sql("select count(*) n from orchestration_v2_projection_provider_sessions")[0].n;
    const t3Turns = spyLines().filter((l) => l.method === "turn/start").length;
    const deliveries = peerA.deliveries.map((d: any) => ({ in_reply_to: String(d.in_reply_to).slice(0, 8), body: String(d.body).slice(0, 40) }));
    FS.writeFileSync(Path.join(ctl, "report.json"), JSON.stringify({ at: new Date().toISOString(), tasks, prompts, t3: { runs, sessions, turnStartFrames: t3Turns }, deliveries, liveProviders: (await fx.liveProviders()).length }, null, 1), { mode: 0o600 });
  };
  await report();
  // boucle de service
  const deadline = Date.now() + 90 * 60 * 1000;
  while (Date.now() < deadline) {
    if (marker("report")) { eat("report"); await report(); }
    if (marker("restart-t3")) {
      eat("restart-t3");
      notes.t3Restart = await server!.stop();
      await pause(2500);
      server = await startServer();
      FS.writeFileSync(Path.join(ctl, "restart-t3.done"), new Date().toISOString(), { mode: 0o600 });
    }
    if (marker("restart-daemon")) {
      eat("restart-daemon");
      peerA.close();
      notes.daemonRestart = await fx.stopDaemon();
      await pause(2500);
      await fx.startDaemon();
      peerA = await fx.registerParent(THREAD_A);
      FS.writeFileSync(Path.join(ctl, "restart-daemon.done"), new Date().toISOString(), { mode: 0o600 });
    }
    if (marker("finish")) { eat("finish"); break; }
    await pause(500);
  }
  await report();
  checks.add("UI.0", "REEL", "fixture native tenue jusqu'au marqueur finish (serveur T3 réel + Vite + daemon réel r8)", true, { admitted: notes.admitted });
  await mcp.close();
} catch (error) {
  exitCode = 1;
  checks.add("EXC", "REEL", "exception non prévue", false, { message: String((error as Error).stack ?? error).slice(0, 900) });
} finally {
  if (vite && vite.exitCode === null) {
    const ps = ChildProcess.execFileSync("/bin/ps", ["-p", String(vite.pid), "-o", "ppid=", "-o", "command="], { encoding: "utf8" }).trim();
    if (Number(ps.split(/\s+/)[0]) === process.pid && !/firefox/i.test(ps)) { vite.kill("SIGTERM"); await pause(3000); }
  }
  notes.viteExit = vite ? (vite.exitCode ?? vite.signalCode) : null;
  // les processus fils de Vite (esbuild/workerd) : liste des restes liés au répertoire du worktree T3 lancés par ce Vite
  notes.t3Stop = await server?.stop().catch((e: unknown) => String(e));
  notes.stopDaemonFinal = await fx.stopDaemon().catch((e: unknown) => String(e));
  const waitEnd = Date.now() + 30000;
  while ((await fx.liveFixtureProcesses()).some((p: any) => p.kind === "provider") && Date.now() < waitEnd) await pause(1000);
  notes.cleanup = await fx.cleanup().catch((e: unknown) => String(e));
}
const summary = checks.summary();
const live = (notes.cleanup as any)?.remainingAfter ?? [];
await writeResults("ui-native149", { schema: "native-network-recipes/1", startedAt: T0S, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (live.length > 0 ? 1 : 0));
