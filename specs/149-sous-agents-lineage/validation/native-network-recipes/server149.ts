// T036 (partie 2) - Serveur T3 COMPLET réel (bin.ts, base SQLite privée, auth bearer scopée, RPC WebSocket)
// lisant le daemon Bridget 149 RÉEL (binaire debug fixe) via T3CODE_BRIDGET_EXECUTABLE.
// Les tâches sont admises pour de vrai (MCP T3 réel + fait de permissions, enfants Codex fermés sans modèle).
// SIMULÉ (nommé) : le wrapper T3 qui garde la connexion/le binding du fil côté daemon (trames réelles
// Register/T3ThreadBindingFact émises par la recette) et l'adaptateur qui publie le fait de permissions.
import * as OS from "node:os";
import { startT3Host, codexFullAccessFact } from "./t3host.ts";
import { Fx, Checks, pause, BIN, BIN_SHA256, sha256File, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, THREAD_B, readOnlyFact, waitTask, counts, settle } from "./common149.ts";
import { startT3Server, createProjectAndThreads } from "./t3server.ts";

const MCP_PORT = 14796;
const SERVER_PORT = 15756;
const checks = new Checks("T036 serveur T3 réel <-> daemon natif");
const T0 = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire debug 149 : empreinte différente du reçu");
for (const p of [MCP_PORT, SERVER_PORT]) if (!(await freePort(p))) throw new Error(`port ${p} occupé`);

const host = await startT3Host(MCP_PORT);
const fx = await Fx.create("server149", { port: MCP_PORT });
let server: Awaited<ReturnType<typeof startT3Server>> | undefined;
let exitCode = 0;
const alive = (pid: number) => { try { process.kill(pid, 0); return true; } catch { return false; } };
try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  notes.daemonPid = await fx.startDaemon();
  let peerA = await fx.registerParent(THREAD_A);
  let peerB = await fx.registerParent(THREAD_B);
  const { config: cA } = await host.issue(THREAD_A);
  const { config: cB } = await host.issue(THREAD_B);
  host.publish(cA, codexFullAccessFact(cA, "run-a-1", fx.work), host.startRun(THREAD_A, "run-a-1"));
  host.publish(cB, readOnlyFact(cB, "run-b-1", fx.work), host.startRun(THREAD_B, "run-b-1"));
  const mcpA = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "A" });
  const mcpB = await fx.mcp({ endpoint: cB.endpoint, authorization: cB.authorizationHeader, label: "B" });
  const base = { agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", cwd: fx.work };
  // racine A avec enfant imbriqué actif (WAIT) : le résultat de la racine reste retenu
  const dRoot = await mcpA.call("bridget_delegate", { ...base, request_id: "srv-a-root", task: "NONCE_r1 NESTED_149[WAIT_149]" });
  const dB = await mcpB.call("bridget_delegate", { ...base, request_id: "srv-b-1", task: "NONCE_b1 lire" });
  const rootTask = dRoot.payload.task_id as string;
  const bTask = dB.payload.task_id as string;
  await pause(4500);

  server = await startT3Server(fx, SERVER_PORT);
  notes.serverPid = server.pid;
  const read = server.tokens.read;
  const operate = server.tokens.operate;
  const proj = await createProjectAndThreads(server, fx.work, [{ id: THREAD_A, title: "Conversation A" }, { id: THREAD_B, title: "Conversation B" }]);
  checks.add("V0", "REEL", "serveur T3 réel démarré sur base privée, projet + deux conversations créés par RPC", proj.created.status === 200 && proj.results.slice(1).every((r: any) => r.ok), { results: proj.results.map((r: any) => r.ok ? "ok" : r.error ?? r.status) });
  const ctx = (thread: string) => ({ projectId: proj.projectId, threadId: thread });
  const sr = (thread: string, extra: object) => server!.call(read, "bridget.lineage.read", { ...ctx(thread), ...extra });

  // Conversations de premier niveau AVANT toute lecture Lineage (project.create ajoute lui-même un « New thread »).
  const topBefore = server.sql("select thread_id from orchestration_v2_projection_threads").map((t: any) => t.thread_id).sort();
  // ---- lecture : conversation A / conversation B (même projet)
  const listA = await sr(THREAD_A, { action: "list" });
  const listB = await sr(THREAD_B, { action: "list" });
  const tasksA = listA.value?.tasks ?? [];
  const tasksB = listB.value?.tasks ?? [];
  const nested = tasksA.find((t: any) => t.parent_task_id === rootTask);
  const root = tasksA.find((t: any) => t.task_id === rootTask);
  checks.add("V1", "REEL", "A : racine + enfant imbriqué (lien parent_task_id), modèle/protocole/posture du moteur ; la racine attend ses enfants (waiting_for_children), résultat retenu",
    tasksA.length === 2 && root?.status === "waiting_for_children" && root.result_available === false && nested?.parent_agent_id === root.child_agent_id && nested.status === "working" && root.model === "fixture-model-149" && root.execution_protocol === "codex_app_server" && root.posture === "development" && root.journal_available === true,
    { rootStatus: root?.status, nestedStatus: nested?.status, tasks: tasksA.length });
  checks.add("V2", "REEL", "S149-19 via serveur T3 : B (même projet) ne voit que sa tâche, jamais celles de A",
    tasksB.length === 1 && tasksB[0].task_id === bTask && !tasksB.some((t: any) => [rootTask, nested?.task_id].includes(t.task_id)), { b: tasksB.map((t: any) => t.task_id.slice(0, 8)) });
  const showCross = await sr(THREAD_B, { action: "show", taskId: rootTask });
  const journalCross = await sr(THREAD_B, { action: "journal", taskId: rootTask });
  const cancelCross = await server.call(operate, "bridget.lineage.cancel", { ...ctx(THREAD_B), taskId: rootTask, requestId: "49000000-0000-4000-8000-0000000000d1" });
  checks.add("V3", "REEL", "B ne peut ni show, ni journal, ni cancel la tâche de A (task_unavailable) ; token read ne peut pas cancel",
    [showCross, journalCross, cancelCross].every((r) => r.error?.code === "task_unavailable"), { show: showCross.error?.code, journal: journalCross.error?.code, cancel: cancelCross.error?.code });
  const readCancel = await server.call(read, "bridget.lineage.cancel", { ...ctx(THREAD_A), taskId: rootTask, requestId: "49000000-0000-4000-8000-0000000000d2" });
  checks.add("V3b", "REEL", "scope : le jeton orchestration:read est refusé sur cancel (aucun effet natif)",
    readCancel.error !== undefined && readCancel.value === undefined && fx.tasks().find((t: any) => t.task_id === rootTask).state !== "cancelled", { error: readCancel.error?.tag ?? readCancel.error?.message });

  // ---- projection T3 : fils virtuels, zéro effet fournisseur, zéro conversation de premier niveau
  const threads = server.sql("select thread_id, title, json_extract(payload_json,'$.bridgetTaskRef.taskId') as task, json_extract(payload_json,'$.lineage.relationshipToParent') as rel from orchestration_v2_projection_threads");
  const virtual = threads.filter((t: any) => t.task);
  const topLevel = threads.filter((t: any) => !t.task);
  const zero = (table: string) => server!.sql(`select count(*) n from ${table}`)[0].n;
  const providerTables = ["orchestration_v2_projection_runs", "orchestration_v2_projection_run_attempts", "orchestration_v2_projection_provider_turns", "orchestration_v2_projection_provider_sessions", "orchestration_v2_projection_provider_threads", "orchestration_v2_projection_provider_session_bindings", "orchestration_v2_projection_runtime_requests", "orchestration_v2_effect_outbox", "orchestration_v2_thread_launch_workflows", "projection_thread_messages"];
  const providerCounts = Object.fromEntries(providerTables.map((t) => [t.replace("orchestration_v2_", ""), zero(t)]));
  checks.add("V4", "REEL", "projection T3 : un fil virtuel marqué par tâche native (3, relation subagent) ; l'ensemble des conversations de premier niveau est identique avant/après lecture (aucune conversation créée par la lecture)",
    virtual.length === 3 && virtual.every((t: any) => t.rel === "subagent") && new Set(virtual.map((t: any) => t.task)).size === 3 && JSON.stringify(topLevel.map((t: any) => t.thread_id).sort()) === JSON.stringify(topBefore),
    { virtual: virtual.length, topLevelBefore: topBefore.length, topLevelAfter: topLevel.length, topTitles: topLevel.map((t: any) => t.title) });
  checks.add("V5", "REEL", "zéro effet fournisseur côté T3 : 0 run, 0 tentative, 0 tour, 0 session, 0 effet en attente, 0 message",
    Object.values(providerCounts).every((n) => n === 0), providerCounts);

  // ---- détails : résultat retenu puis libéré, journal réel
  const showNested = await sr(THREAD_A, { action: "show", taskId: nested.task_id });
  const showRoot = await sr(THREAD_A, { action: "show", taskId: rootTask });
  checks.add("V6", "REEL", "tant que l'enfant imbriqué travaille, la racine n'expose aucun résultat (retenu) ; l'enfant lui-même n'a pas de résultat",
    showRoot.value?.result === null && showRoot.value?.task?.status === "waiting_for_children" && showNested.value?.result === null, { rootResult: showRoot.value?.result, nestedResult: showNested.value?.result });
  const journal = await sr(THREAD_A, { action: "journal", taskId: rootTask });
  checks.add("V7", "REEL", "journal réel de l'enfant actif lisible via T3 (événements du daemon, séquence ordonnée)",
    (journal.value?.events?.length ?? 0) >= 1 && journal.value.events.every((e: any, i: number, all: any[]) => i === 0 || e.seq > all[i - 1].seq), { events: journal.value?.events?.length, gap: journal.value?.gap, error: journal.error?.code });

  // ---- watch temps réel : ready seq 0 puis signal sans corps sur mutation
  const watchPromise = server.call(read, "bridget.lineage.watch", ctx(THREAD_A), { take: 2, waitMs: 12000 });
  await pause(1500);
  const dMore = await mcpA.call("bridget_delegate", { ...base, request_id: "srv-a-2", task: "NONCE_r2 WAIT_149" });
  const watch = await watchPromise;
  checks.add("V8", "REEL", "bridget.lineage.watch via T3 : ready seq 0 premier, puis changed sans corps ni UUID après nouvelle tâche réelle",
    watch.items?.[0]?.status === "ready" && watch.items[0].seq === 0 && ["changed", "resync"].includes(watch.items?.[1]?.status) && !JSON.stringify(watch.items[1]).includes(THREAD_A), { statuses: (watch.items ?? []).map((i: any) => i.status), keys: Object.keys(watch.items?.[1] ?? {}) });
  const task2 = dMore.payload.task_id as string;
  await pause(1200);

  // ---- annulation depuis T3 : native, enfant réellement arrêté, rien côté fournisseur T3
  const pidsBefore = fx.evidence().filter((e: any) => e.event === "started").map((e: any) => e.pid);
  const rid = "49000000-0000-4000-8000-0000000000e1";
  const c1 = await server.call(operate, "bridget.lineage.cancel", { ...ctx(THREAD_A), taskId: task2, requestId: rid });
  const c2 = await server.call(operate, "bridget.lineage.cancel", { ...ctx(THREAD_A), taskId: task2, requestId: rid });
  const c3 = await server.call(operate, "bridget.lineage.cancel", { ...ctx(THREAD_A), taskId: rootTask, requestId: rid });
  const t2 = await waitTask(fx, task2, ["cancelled"]);
  await pause(3500);
  const wait2Pid = fx.evidence().find((e: any) => e.event === "prompt" && e.nonce === "r2")?.pid;
  checks.add("V9", "REEL", "cancel via T3 : reçu natif, rejeu identique, request_id sur autre tâche = envelope_mismatch ; tâche cancelled et PID enfant réellement terminé",
    c1.value?.status && JSON.stringify(c1.value) === JSON.stringify(c2.value) && c3.error?.code === "envelope_mismatch" && t2?.state === "cancelled" && wait2Pid !== undefined && !alive(wait2Pid),
    { receipt: c1.value, replaySame: JSON.stringify(c1.value) === JSON.stringify(c2.value), mismatch: c3.error?.code, state: t2?.state, childPid: wait2Pid, alive: wait2Pid ? alive(wait2Pid) : null });
  const providerAfter = Object.fromEntries(providerTables.map((t) => [t.replace("orchestration_v2_", ""), zero(t)]));
  checks.add("V10", "REEL", "l'annulation depuis T3 ne crée aucun run/tour/session fournisseur côté T3 (tables toujours à zéro)",
    Object.values(providerAfter).every((n) => n === 0), providerAfter);

  // ---- lectures répétées : quiètes
  const quietBefore = { c: counts(fx), db: fx.tasks().map((t: any) => [t.task_id, t.state, t.updated_at]).sort() };
  for (let i = 0; i < 100; i += 1) {
    await sr(THREAD_A, { action: "list" });
    if (i % 10 === 0) await sr(THREAD_A, { action: "show", taskId: rootTask });
  }
  await pause(500);
  const quietAfter = { c: counts(fx), db: fx.tasks().map((t: any) => [t.task_id, t.state, t.updated_at]).sort() };
  checks.add("V11", "REEL", "110 lectures T3 -> daemon : aucun lancement, aucun tour, aucune mutation (états et updated_at identiques)",
    JSON.stringify(quietBefore) === JSON.stringify(quietAfter), { before: quietBefore.c, after: quietAfter.c });

  // ---- coupure du daemon puis reprise : la projection se reconstruit sans doublon ni relance
  const beforeCut = { threads: virtual.length, started: counts(fx).started, generation: listA.value?.generation, ids: tasksA.map((t: any) => t.task_id).sort() };
  const rowsBefore = fx.tasks().map((t: any) => [t.task_id, t.state]).sort();
  peerA.close();
  peerB.close();
  notes.stopDaemon = await fx.stopDaemon();
  const down = await sr(THREAD_A, { action: "list" });
  checks.add("V12", "REEL", "daemon coupé : T3 refuse explicitement (aucun contenu présenté comme courant)",
    down.value === undefined && down.error !== undefined, { error: down.error?.code ?? down.error?.tag, message: down.error?.message });
  await fx.startDaemon();
  peerA = await fx.registerParent(THREAD_A);
  peerB = await fx.registerParent(THREAD_B);
  await pause(2500);
  const up = await sr(THREAD_A, { action: "list" });
  const threadsAfter = server.sql("select json_extract(payload_json,'$.bridgetTaskRef.taskId') as task from orchestration_v2_projection_threads").filter((t: any) => t.task);
  const idsAfter = (up.value?.tasks ?? []).map((t: any) => t.task_id).sort();
  checks.add("V13", "REEL", "reprise sans relance : même génération native, mêmes tâches, aucun fil virtuel en double côté T3",
    up.value?.generation === beforeCut.generation && JSON.stringify(idsAfter.filter((i: string) => beforeCut.ids.includes(i))) === JSON.stringify(beforeCut.ids) && new Set(threadsAfter.map((t: any) => t.task)).size === threadsAfter.length,
    { sameGeneration: up.value?.generation === beforeCut.generation, tasksBefore: beforeCut.ids.length, tasksAfter: idsAfter.length, virtualThreadsAfter: threadsAfter.length });
  const rowsAfter = fx.tasks().map((t: any) => [t.task_id, t.state]).sort();
  notes.restartTaskStates = { before: rowsBefore, after: rowsAfter };
  notes.restartCounts = { startedBeforeCut: beforeCut.started, startedAfterRestart: counts(fx).started, promptsAfter: counts(fx).prompts };

  // ---- nettoyage natif : annuler la racine (descendance comprise)
  const cleanupCancel = await server.call(operate, "bridget.lineage.cancel", { ...ctx(THREAD_A), taskId: rootTask, requestId: "49000000-0000-4000-8000-0000000000e2" });
  await waitTask(fx, rootTask, ["cancelled"]);
  await pause(3500);
  const taskStates = fx.tasks().map((t: any) => t.state);
  notes.finalStates = taskStates;
  notes.cancelReceipt = cleanupCancel.value ?? cleanupCancel.error;
  await mcpA.close();
  await mcpB.close();
  peerA.close();
  peerB.close();
  notes.finalCounts = counts(fx);
  notes.serverDb = { virtualThreads: threadsAfter.length, providerCounts };
} catch (error) {
  exitCode = 1;
  checks.add("EXC", "REEL", "exception non prévue", false, { message: String((error as Error).stack ?? error).slice(0, 900) });
} finally {
  if (server) notes.serverStop = await server.stop().catch((e) => String(e));
  notes.daemonStop = await fx.stopDaemon().catch((e) => String(e));
  notes.cleanup = await fx.cleanup().catch((e) => String(e));
  await host.stop().catch(() => undefined);
}
const summary = checks.summary();
const live = (notes.cleanup as any)?.remainingAfter ?? [];
await writeResults("server149", { schema: "native-network-recipes/1", startedAt: T0, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
