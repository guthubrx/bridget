// T036 (partie 3) - UNE SEULE instance T3 réelle (bin.ts) : elle émet elle-même le credential MCP (vrai
// ProviderSessionManager + vrai adaptateur Codex V2 qui publie le fait de permissions) ET sert la lecture/annulation
// Lineage. Aucun hôte MCP simulé, aucun fait forgé par la recette (r1 : deux instances et fait publié par la recette).
// Aucun modèle : le « codex » configuré par settings.json est le faux pair app-server de T3 (testFixtures/codexCollabMockPeer.mjs,
// copié en lecture seule) derrière un espion qui consigne les trames reçues. Le tour reste ouvert (holdTurnOpen).
// SIMULÉ (nommé) : le pair Codex lui-même (réponses de session capturées), le wrapper du fil T3 côté daemon
// (trames réelles Register/T3ThreadBindingFact émises par la recette), les enfants (app-server Codex fermé).
import * as FS from "node:fs";
import * as OS from "node:os";
import * as Path from "node:path";
import * as Crypto from "node:crypto";
import { Fx, Checks, pause, BIN, BIN_SHA256, sha256File, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, rawTool, waitTask, counts, settle } from "./common149.ts";
import { startT3Server, createProjectAndThreads, NODE_BIN } from "./t3server.ts";

const T3_FIX = "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/provider/testFixtures";
const SERVER_PORT = 15756;
const checks = new Checks("T036 instance T3 unique (credential MCP + Lineage)");
const T0 = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire debug 149 : empreinte différente du reçu");
if (!(await freePort(SERVER_PORT))) throw new Error(`port ${SERVER_PORT} occupé`);

const fx = await Fx.create("unified149", { port: SERVER_PORT });
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
let server: Awaited<ReturnType<typeof startT3Server>> | undefined;
let exitCode = 0;
try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  notes.daemonPid = await fx.startDaemon();
  const peerA = await fx.registerParent(THREAD_A);
  // Le binaire « codex » est imposé en CHEMIN ABSOLU et les autres fournisseurs sont désactivés : aucun vrai binaire Codex,
  // Claude ou autre n'est lancé par ce serveur (un premier essai par PATH avait fait résoudre le vrai `codex` par le shell de connexion).
  FS.mkdirSync(Path.join(fx.root, "t3home", "userdata"), { recursive: true, mode: 0o700 });
  FS.writeFileSync(Path.join(fx.root, "t3home", "userdata", "settings.json"), JSON.stringify({ providers: { codex: { enabled: true, binaryPath: Path.join(fake, "codex") }, claudeAgent: { enabled: false }, cursor: { enabled: false }, grok: { enabled: false }, opencode: { enabled: false } } }), { mode: 0o600 });
  server = await startT3Server(fx, SERVER_PORT, { extraEnv: { SPY_LOG: spyLog, T3_CODEX_COLLAB_SCRIPT: Path.join(fake, "script.json") } });
  notes.serverPid = server.pid;
  const proj = await createProjectAndThreads(server, fx.work, [{ id: THREAD_A, title: "Conversation A" }]);
  const ctx = { projectId: proj.projectId, threadId: THREAD_A };
  const read = server.tokens.read;
  const operate = server.tokens.operate;
  const send = await server.dispatch(operate, {
    type: "message.dispatch", commandId: Crypto.randomUUID(), threadId: THREAD_A, messageId: Crypto.randomUUID(), text: "message de recette (aucun modèle)", attachments: [],
    createdBy: "user", creationSource: "web", dispatchMode: { type: "start_immediately" },
  });
  const until = Date.now() + 30000;
  while (!spyLines().some((l) => l.method === "turn/start") && Date.now() < until) await pause(100);
  await pause(800);
  const lines = spyLines();
  const threadStart = lines.find((l) => l.method === "thread/start");
  const mcpCfg = threadStart?.params?.config?.["mcp_servers.t3-code"];
  const methods = lines.map((l) => l.method ?? l.spy);
  const credential = mcpCfg ? { endpoint: mcpCfg.url as string, authorizationHeader: mcpCfg.http_headers?.Authorization as string } : undefined;
  notes.spyMethods = methods;
  notes.mcpEndpoint = mcpCfg?.url;
  checks.add("U0", "REEL", "une seule instance T3 (bin.ts) : la session Codex réelle de T3 (ProviderSessionManager + adaptateur V2) reçoit un credential MCP émis PAR CE SERVEUR - l'URL MCP de thread/start est l'origine même du serveur qui sert Lineage - et le tour démarre (turn/start)",
    send.ok !== undefined && credential?.endpoint === `${server.origin}/mcp` && /^Bearer \S{20,}$/.test(credential.authorizationHeader ?? "") && methods.includes("turn/start") && methods.indexOf("thread/start") < methods.indexOf("turn/start"),
    { dispatch: send.ok ? "ok" : send.error, endpoint: credential?.endpoint, sameOriginAsLineage: credential?.endpoint === `${server.origin}/mcp`, methods: methods.filter((m: string) => !["argv"].includes(m)), spawnedPeers: lines.filter((l) => l.spy === "argv").length });
  if (!credential) throw new Error("credential non émis : arrêt");

  // fait de permissions publié par le VRAI adaptateur (la recette ne publie rien)
  const sess = await rawTool({ endpoint: credential.endpoint, authorizationHeader: credential.authorizationHeader }, "bridget_session");
  const sv = sess.structured as any;
  notes.sessionEnvelope = sess.structured === undefined ? sess : { version: sv.version, keys: Object.keys(sv).sort(), permissions: sv.permissions ? { keys: Object.keys(sv.permissions).sort(), runtime_mode: sv.permissions.runtime_mode, driver: sv.permissions.driver, cwd: sv.permissions.cwd, sandbox: sv.permissions.provider_policy?.sandbox_policy?.type, source: sv.permissions.source } : null };
  checks.add("U1", "REEL", "bridget_session v2 sur le serveur réel : le fait de permissions est celui publié par l'adaptateur Codex RÉEL (driver codex_app_server, cwd = racine du projet, mode full-access du fil) - la recette n'a rien forgé",
    sv?.version === 2 && sv.threadId === THREAD_A && sv.permissions?.driver === "codex_app_server" && sv.permissions?.runtime_mode === "full-access" && sv.permissions?.cwd === fx.work && sv.permissions?.provider_policy?.sandbox_policy?.type === "dangerFullAccess",
    notes.sessionEnvelope);

  // client Bridget réel + admission héritée via ce credential
  const mcp = await fx.mcp({ endpoint: credential.endpoint, authorization: credential.authorizationHeader, label: "U" });
  const base = { agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", cwd: fx.work };
  const who = await mcp.call("bridget_who", { scope: "global" });
  const d1 = await mcp.call("bridget_delegate", { ...base, request_id: "uni-1", task: "NONCE_u1 NESTED_149[WAIT_149]" });
  const root = d1.payload?.task_id as string;
  await pause(3500);
  const d2 = await mcp.call("bridget_delegate", { ...base, request_id: "uni-2", task: "NONCE_u2" });
  const t2 = await waitTask(fx, d2.payload?.task_id, ["result_available", "failed"], 30000);
  await settle(fx, [peerA], 1500, 8000).catch(() => undefined);
  const dlv2 = peerA.deliveries.filter((d: any) => d.in_reply_to === d2.payload?.message_id);
  checks.add("U2", "REEL", "le credential émis par T3 pilote le daemon natif : identité résolue (bridget_who), admission héritée en full-access sans refus, enfant lancé (PID compté), résultat remis une fois au parent",
    !who.isError && JSON.stringify(who.payload).includes(peerA.agentId) && !d1.isError && !d2.isError && t2?.state === "result_available" && t2.result === "fixture149-answer:u2" && dlv2.length === 1,
    { whoErr: who.isError, d1: d1.payload?.status ?? d1.payload?.text, d2: d2.payload?.status ?? d2.payload?.text, state: t2?.state, deliveries: dlv2.length });

  // Lineage depuis LE MÊME serveur
  const list = await server.call(read, "bridget.lineage.read", { ...ctx, action: "list" });
  const tasks = list.value?.tasks ?? [];
  const rootRow = tasks.find((t: any) => t.task_id === root);
  const nested = tasks.find((t: any) => t.parent_task_id === root);
  checks.add("U3", "REEL", "Lineage servi par la MÊME instance T3 : la racine (en attente), son enfant imbriqué et la tâche terminée sont listés avec modèle, protocole et posture issus du vrai fait ; rien d'un autre fil",
    tasks.length === 3 && rootRow?.status === "waiting_for_children" && nested?.status === "working" && tasks.find((t: any) => t.task_id === d2.payload?.task_id)?.status === "result_available" && rootRow.model === "fixture-model-149" && rootRow.execution_protocol === "codex_app_server",
    { tasks: tasks.length, root: rootRow?.status, nested: nested?.status, posture: rootRow?.posture });

  // run réel du parent côté T3 ; aucun run/session pour les enfants Bridget
  const runs = server.sql("select thread_id, status from orchestration_v2_projection_runs");
  const sessions = server.sql("select count(*) n from orchestration_v2_projection_provider_sessions")[0].n;
  const virtual = server.sql("select thread_id, json_extract(payload_json,'$.bridgetTaskRef.taskId') as task from orchestration_v2_projection_threads").filter((t: any) => t.task);
  const childThreadIds = new Set(virtual.map((v: any) => v.thread_id));
  checks.add("U4", "REEL", "projection T3 : le parent a UN vrai run (créé par T3) ; les fils enfants Bridget sont des fils virtuels sans aucun run ; une seule session fournisseur (celle du parent)",
    runs.length === 1 && runs[0].thread_id === THREAD_A && virtual.length === 3 && !runs.some((r: any) => childThreadIds.has(r.thread_id)) && sessions === 1,
    { runs: runs.map((r: any) => [String(r.thread_id).slice(0, 8), r.status]), virtual: virtual.length, providerSessions: sessions });

  // annulation depuis T3 : native, PID enfant arrêté, le run du parent n'est pas touché
  const nestedPid = fx.evidence().find((e: any) => e.event === "prompt" && e.nonce === "u1g")?.pid;
  const rid = "49000000-0000-4000-8000-0000000000a1";
  const cancel = await server.call(operate, "bridget.lineage.cancel", { ...ctx, taskId: root, requestId: rid });
  const cancelled = await waitTask(fx, root, ["cancelled"], 15000);
  await pause(3500);
  const runsAfter = server.sql("select thread_id, status from orchestration_v2_projection_runs");
  checks.add("U5", "REEL", "cancel Lineage depuis la même instance T3 : reçu natif, racine + enfant imbriqué cancelled, PID enfant réellement terminé ; le run du parent côté T3 n'a pas changé",
    cancel.value?.status !== undefined && cancelled?.state === "cancelled" && nestedPid !== undefined && !alive(nestedPid) && JSON.stringify(runsAfter) === JSON.stringify(runs),
    { receipt: cancel.value, rootState: cancelled?.state, nestedPid, nestedAlive: nestedPid ? alive(nestedPid) : null, runsUnchanged: JSON.stringify(runsAfter) === JSON.stringify(runs) });

  // révocation réelle : interruption du run par T3 -> le credential n'ouvre plus rien
  const runId = (server.sql("select run_id from orchestration_v2_projection_runs") as any[])[0]?.run_id ?? (server.sql("select id from orchestration_v2_projection_runs") as any[])[0]?.id;
  const interrupt = runId ? await server.dispatch(operate, { type: "run.interrupt", commandId: Crypto.randomUUID(), threadId: THREAD_A, runId, holdQueue: true }) : { skipped: "run_id introuvable" };
  const intUntil = Date.now() + 20000;
  while (Date.now() < intUntil && server.sql("select status from orchestration_v2_projection_runs")[0]?.status === "running") await pause(250);
  await pause(1000);
  notes.spyMethodsAfterInterrupt = spyLines().map((l) => l.method ?? l.spy);
  const afterInterrupt = await rawTool({ endpoint: credential.endpoint, authorizationHeader: credential.authorizationHeader }, "bridget_session");
  const delegateAfter = await mcp.call("bridget_delegate", { ...base, request_id: "uni-after-interrupt", task: "NONCE_u3" });
  notes.revocation = { interrupt: (interrupt as any).ok ? "ok" : (interrupt as any).error ?? (interrupt as any).skipped, sessionAfter: afterInterrupt.httpStatus + ":" + (afterInterrupt.namedCode ?? afterInterrupt.code ?? (afterInterrupt.isError ? "error" : "ok")), delegateAfter: delegateAfter.code ?? delegateAfter.payload?.text ?? delegateAfter.payload?.status, u3Prompts: fx.evidence().filter((e: any) => e.event === "prompt" && e.nonce === "u3").length, runsStatusAfter: server.sql("select status from orchestration_v2_projection_runs") };
  checks.add("U6", "REEL", "interruption du run par T3 (vrai arrêt de session) : le credential émis ne donne plus d'identité ni d'admission (refus nommé), aucun enfant lancé pour la requête suivante",
    (afterInterrupt.isError || afterInterrupt.httpStatus >= 400) && delegateAfter.isError && notes.revocation && (notes.revocation as any).u3Prompts === 0,
    notes.revocation);
  await mcp.close();
  notes.finalCounts = counts(fx);
  notes.hostname = OS.hostname();
  peerA.close();
} catch (error) {
  exitCode = 1;
  checks.add("EXC", "REEL", "exception non prévue", false, { message: String((error as Error).stack ?? error).slice(0, 900) });
} finally {
  notes.t3Stop = await server?.stop().catch((e: unknown) => String(e));
  notes.stopDaemonFinal = await fx.stopDaemon().catch((e: unknown) => String(e));
  notes.cleanup = await fx.cleanup().catch((e: unknown) => String(e));
}
const summary = checks.summary();
const live = (notes.cleanup as any)?.remainingAfter ?? [];
await writeResults("unified149", { schema: "native-network-recipes/1", startedAt: T0, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
