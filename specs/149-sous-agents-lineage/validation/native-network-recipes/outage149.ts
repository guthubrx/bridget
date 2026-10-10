// Contrôle « panne de T3 après admission » (ronde r3) avec le VRAI serveur T3 (bin.ts, une seule instance, faux pair Codex capturé derrière un espion) :
// pendant la panne de T3, les effets natifs (status, cancel, résultat) ne déclenchent AUCUN tour côté T3.
// Compteurs T3 : trames `turn/start` / `thread/start` vues par l'espion du pair Codex de T3, runs et sessions fournisseur de la base T3.
// Aucun modèle. Daemon Bridget = binaire release r8.
import * as FS from "node:fs";
import * as OS from "node:os";
import * as Path from "node:path";
import * as Crypto from "node:crypto";
import { Fx, Checks, pause, BIN, BIN_SHA256, sha256File, lineageArgs, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, rawTool, waitTask, settle } from "./common149.ts";
import { startT3Server, createProjectAndThreads, NODE_BIN } from "./t3server.ts";

const T3_FIX = "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/provider/testFixtures";
const SERVER_PORT = 15757;
const checks = new Checks("Panne T3 après admission : zéro tour T3 (r3)");
const T0S = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire release r8 : empreinte différente du reçu");
if (!(await freePort(SERVER_PORT))) throw new Error(`port ${SERVER_PORT} occupé`);
const fx = await Fx.create("outage149", { port: SERVER_PORT });
notes.fixtureRoot = fx.root;
const fake = Path.join(fx.root, "fakebin");
FS.mkdirSync(fake, { recursive: true, mode: 0o700 });
for (const f of ["codexCollabMockPeer.mjs", "codexMultiAgentWire.json"]) FS.copyFileSync(Path.join(T3_FIX, f), Path.join(fake, f));
FS.copyFileSync(Path.join(import.meta.dirname, "codexspy.mjs"), Path.join(fake, "codexspy.mjs"));
FS.writeFileSync(Path.join(fake, "script.json"), JSON.stringify({ rootThreadId: "probe-thread", notifications: [], holdTurnOpen: true }), { mode: 0o600 });
FS.writeFileSync(Path.join(fake, "codex"), `#!/bin/sh\nexec "${NODE_BIN}/node" "${fake}/codexspy.mjs" "$@"\n`, { mode: 0o700 });
const spyLog = Path.join(fx.root, "evidence", "codex-spy.jsonl");
const alive = (pid: number) => { try { process.kill(pid, 0); return true; } catch { return false; } };
const spyLines = () => (FS.existsSync(spyLog) ? FS.readFileSync(spyLog, "utf8").split("\n").filter(Boolean).map((l) => { try { return JSON.parse(l); } catch { return {}; } }) : []);
const t3Frames = () => { const l = spyLines(); return { turnStart: l.filter((x) => x.method === "turn/start").length, threadStart: l.filter((x) => x.method === "thread/start").length, spawns: l.filter((x) => x.spy === "argv").length }; };
let server: Awaited<ReturnType<typeof startT3Server>> | undefined;
let exitCode = 0;
const t3Settings = () => {
  FS.mkdirSync(Path.join(fx.root, "t3home", "userdata"), { recursive: true, mode: 0o700 });
  FS.writeFileSync(Path.join(fx.root, "t3home", "userdata", "settings.json"), JSON.stringify({ providers: { codex: { enabled: true, binaryPath: Path.join(fake, "codex") }, claudeAgent: { enabled: false }, cursor: { enabled: false }, grok: { enabled: false }, opencode: { enabled: false } } }), { mode: 0o600 });
};
try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  notes.daemonPid = await fx.startDaemon();
  const peerA = await fx.registerParent(THREAD_A);
  t3Settings();
  const startServer = () => startT3Server(fx, SERVER_PORT, { extraEnv: { SPY_LOG: spyLog, T3_CODEX_COLLAB_SCRIPT: Path.join(fake, "script.json") } });
  server = await startServer();
  const proj = await createProjectAndThreads(server, fx.work, [{ id: THREAD_A, title: "Conversation A" }]);
  await server.dispatch(server.tokens.operate, {
    type: "message.dispatch", commandId: Crypto.randomUUID(), threadId: THREAD_A, messageId: Crypto.randomUUID(), text: "message de recette (aucun modèle)", attachments: [],
    createdBy: "user", creationSource: "web", dispatchMode: { type: "start_immediately" },
  });
  const until = Date.now() + 30000;
  while (!spyLines().some((l) => l.method === "turn/start") && Date.now() < until) await pause(100);
  await pause(800);
  const threadStart = spyLines().find((l) => l.method === "thread/start");
  const mcpCfg = threadStart?.params?.config?.["mcp_servers.t3-code"];
  if (!mcpCfg) throw new Error("credential non émis");
  const mcp = await fx.mcp({ endpoint: mcpCfg.url, authorization: mcpCfg.http_headers?.Authorization, label: "OUT" });
  const base = { agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", cwd: fx.work };
  const dWait = await mcp.call("bridget_delegate", { ...base, request_id: "out-wait", task: "NONCE_o1 WAIT_149" });
  const dSlow = await mcp.call("bridget_delegate", { ...base, request_id: "out-slow", task: "NONCE_o2 SLOW_149:7000" });
  await pause(2000);
  const waitPid = fx.evidence().find((e: any) => e.event === "prompt" && e.nonce === "o1")?.pid;
  const runsBefore = server.sql("select thread_id, status from orchestration_v2_projection_runs");
  const sessionsBefore = server.sql("select count(*) n from orchestration_v2_projection_provider_sessions")[0].n;
  const framesBefore = t3Frames();
  notes.before = { frames: framesBefore, runs: runsBefore.length, sessions: sessionsBefore, admitted: [dWait.payload?.status, dSlow.payload?.status] };

  // ---- panne réelle de T3 : SIGTERM vérifié du serveur
  await mcp.close();
  const t3Stop = await server.stop();
  server = undefined;
  await pause(500);
  notes.t3Stop = t3Stop;
  // effets natifs pendant la panne
  const readList = await fx.cli(lineageArgs("list", THREAD_A, fx.work));
  const waitTask1 = (readList.json?.tasks ?? []).find((t: any) => t.task_id === dWait.payload?.task_id);
  const cancel = await fx.cli(lineageArgs("cancel", THREAD_A, fx.work, ["--task", dWait.payload?.task_id, "--request-id", "49000000-0000-4000-8000-0000000000b1"]));
  const cancelled = await waitTask(fx, dWait.payload?.task_id, ["cancelled"], 15000);
  const slow = await waitTask(fx, dSlow.payload?.task_id, ["result_available", "failed"], 30000);
  await settle(fx, [peerA], 1500, 20000).catch(() => undefined);
  const dlvSlow = peerA.deliveries.filter((d: any) => d.in_reply_to === dSlow.payload?.message_id);
  const show = await fx.cli(lineageArgs("show", THREAD_A, fx.work, ["--task", dSlow.payload?.task_id, "--offset", "0", "--limit", "100"]));
  const framesDuring = t3Frames();
  notes.during = { list: (readList.json?.tasks ?? []).map((t: any) => [String(t.task_id).slice(0, 8), t.status]), cancel: cancel.json, cancelledState: cancelled?.state, slowState: slow?.state, deliveries: dlvSlow.length, showResult: show.json?.result, frames: framesDuring };
  checks.add("T3out.1", "REEL", "pendant la panne de T3 (SIGTERM du serveur) : lecture (liste) servie, annulation native de la tâche en attente (cancelled, PID enfant terminé), la tâche lente va à son terme et son résultat est remis UNE fois, lecture du résultat servie",
    t3Stop === "stopped_by_sigterm" && !!waitTask1 && cancel.code === 0 && cancelled?.state === "cancelled" && waitPid !== undefined && !alive(waitPid) && slow?.state === "result_available" && dlvSlow.length === 1 && show.json?.result === "fixture149-answer:o2",
    { t3Stop, listed: !!waitTask1, cancelCode: cancel.code, cancelled: cancelled?.state, waitPidAlive: waitPid ? alive(waitPid) : null, slow: slow?.state, deliveries: dlvSlow.length, result: show.json?.result });
  checks.add("T3out.2", "REEL", "ZÉRO tour T3 provoqué par ces effets : l'espion du pair Codex de T3 ne voit AUCUN nouveau `turn/start`, `thread/start` ni nouveau processus (compteurs identiques avant/pendant la panne, T3 étant arrêté) ; aucune tâche Bridget n'a créé de run T3",
    framesDuring.turnStart === framesBefore.turnStart && framesDuring.threadStart === framesBefore.threadStart && framesDuring.spawns === framesBefore.spawns && framesBefore.turnStart === 1,
    { before: framesBefore, during: framesDuring });
  // MCP T3 : fermé pendant la panne
  const mcpDown = await fx.mcp({ endpoint: mcpCfg.url, authorization: mcpCfg.http_headers?.Authorization, label: "OUT2" });
  const stDown = await mcpDown.call("bridget_task_status", { task_id: dSlow.payload?.task_id });
  const newDown = await mcpDown.call("bridget_delegate", { ...base, request_id: "out-new", task: "NONCE_o3" });
  await mcpDown.close();
  checks.add("T3out.3", "REEL", "montage MCP de T3 fermé pendant la panne (status et nouvelle admission : t3_session_unavailable), aucun lancement pour la nouvelle requête",
    stDown.isError && stDown.code === "t3_session_unavailable" && newDown.isError && newDown.code === "t3_session_unavailable" && fx.evidence().filter((e: any) => e.event === "prompt" && e.nonce === "o3").length === 0,
    { status: stDown.code, delegate: newDown.code });

  // ---- retour de T3 : les compteurs Bridget n'ont pas changé, T3 revient avec ses propres règles (observation)
  const promptsBeforeBack = fx.evidence().filter((e: any) => e.event === "prompt").length;
  server = await startServer();
  await pause(3000);
  const framesBack = t3Frames();
  const list = await server.call(server.tokens.read, "bridget.lineage.read", { projectId: proj.projectId, threadId: THREAD_A, action: "list" });
  const tasks = list.value?.tasks ?? [];
  notes.back = { framesBack, lineage: tasks.map((t: any) => [String(t.task_id).slice(0, 8), t.status]), t3SelfRecovery: "turn/start vus après le redémarrage de T3 = comportement propre de T3, pas un effet Bridget" };
  checks.add("T3out.4", "REEL", "T3 revenu : la lecture Lineage servie par T3 montre la tâche annulée et la tâche terminée ; aucun nouveau lancement ni tour côté Bridget (prompts inchangés)",
    tasks.some((t: any) => t.task_id === dWait.payload?.task_id && t.status === "cancelled") && tasks.some((t: any) => t.task_id === dSlow.payload?.task_id && t.status === "result_available") && fx.evidence().filter((e: any) => e.event === "prompt").length === promptsBeforeBack,
    { statuses: tasks.map((t: any) => t.status), promptsUnchanged: fx.evidence().filter((e: any) => e.event === "prompt").length === promptsBeforeBack });
  checks.add("T3out.5", "OBS", "OBSERVATION (ni PASS ni FAIL) : trames du pair Codex de T3 vues après le redémarrage de T3 (reprise propre de T3, indépendante de Bridget)",
    true, { before: framesBefore, during: framesDuring, afterT3Restart: framesBack });
  peerA.close();
} catch (error) {
  exitCode = 1;
  checks.add("EXC", "REEL", "exception non prévue", false, { message: String((error as Error).stack ?? error).slice(0, 900) });
} finally {
  notes.t3StopFinal = await server?.stop().catch((e: unknown) => String(e));
  notes.stopDaemonFinal = await fx.stopDaemon().catch((e: unknown) => String(e));
  notes.cleanup = await fx.cleanup().catch((e: unknown) => String(e));
}
const summary = checks.summary();
const live = (notes.cleanup as any)?.remainingAfter ?? [];
await writeResults("outage149", { schema: "native-network-recipes/1", startedAt: T0S, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
