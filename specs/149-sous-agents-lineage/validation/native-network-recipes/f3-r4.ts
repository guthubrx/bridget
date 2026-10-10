// F3 (ronde r4) - clôture CAS de l'exécution durable AVANT que la tâche passe `cancelled`, sur le binaire release r9. Sans modèle.
//  F3.1 daemon sain : annulation d'un descendant ; invariant échantillonné (100 ms) : jamais `cancelled` avec exécution ACTIVE ;
//  F3.2 coupure du daemon (SIGTERM réel) PENDANT l'annulation lente (fournisseur sourd à SIGTERM) puis relance : même invariant, exécution fermée,
//       racine débloquée, résultat remis une fois ; une exécution `completed` témoin reste identique (état, motif, révision, génération).
// Trois tables durables capturées avant/après : native_delegations, executions, agent_links.
import * as OS from "node:os";
import * as Path from "node:path";
import { startT3Host, codexFullAccessFact } from "./t3host.ts";
import { Fx, Checks, pause, BIN, BIN_SHA256, HERE, sha256File, lineageArgs, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, THREAD_B } from "./common149.ts";

process.env.NATIVE149_CODEX_PROVIDER = Path.join(HERE, "codex149b.py");
const PORT = 14818;
const ACTIVE = ["queued", "starting", "running", "waiting_approval", "waiting_user_input", "interrupting"];
const checks = new Checks("F3 clôture CAS de l'exécution avant `cancelled` (r4)");
const T0S = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire release r9 : empreinte différente du reçu");
if (!(await freePort(PORT))) throw new Error(`port ${PORT} occupé`);
const host = await startT3Host(PORT);
const fx = await Fx.create("f3-r4", { port: PORT });
notes.fixtureRoot = fx.root;
const base = { agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", cwd: fx.work };
const rowOf = (id: string) => fx.tasks().find((t: any) => t.task_id === id);
const nestedOf = (id: string) => fx.tasks().find((t: any) => t.parent_task_id === id);
const execOfTask = (t: any) => fx.execs().find((e: any) => e.execution_id === `execution-${t?.mission}`);
const untilT = async (fn: () => any, ms: number, step = 100) => { const end = Date.now() + ms; for (;;) { const v = fn(); if (v) return v; if (Date.now() >= end) return undefined; await pause(step); } };
const tables = () => ({
  tasks: fx.tasks().map((t: any) => ({ id: String(t.task_id).slice(0, 8), parent: t.parent_task_id ? String(t.parent_task_id).slice(0, 8) : null, state: t.state, error: t.error ?? null, sent: t.result_sent, cleanup: t.cleanup_done })),
  execs: fx.execs().map((e: any) => ({ id: e.execution_id.slice(0, 18), state: e.state, reason: e.reason, rev: e.revision, gen: String(e.generation) })),
  links: fx.sql("select substr(link_id,1,8) as id, role, state, revision from agent_links order by created_at, link_id"),
});
/** Annulation d'un descendant : MCP du propriétaire racine, sinon CLI Lineage humaine (même chemin que e2e3-149.ts). */
async function cancelTask(taskId: string, rid: string) {
  const viaMcp = await mcpA.call("bridget_task_cancel", { task_id: taskId });
  if (!viaMcp.isError) return { path: "mcp", ok: true, mcpRefusal: null as unknown };
  const cli = await fx.cli(lineageArgs("cancel", THREAD_A, fx.work, ["--task", taskId, "--request-id", rid]));
  return { path: "cli", ok: cli.code === 0, mcpRefusal: viaMcp.code ?? viaMcp.payload?.text ?? "refus", cliCode: cli.code };
}
let curPeer: any;
let exitCode = 0;
let mcpA: any;
let cA: any;
/** Échantillonne l'invariant « jamais `cancelled` avec exécution active » ; retourne le nombre de violations et le dernier état. */
async function watchInvariant(nestedId: string, ms: number) {
  const t0 = Date.now();
  let violations = 0, samples = 0, last: any, firstCancelledAtMs: number | null = null, execAtFirstCancelled: string | null = null;
  const seen: string[] = [];
  while (Date.now() - t0 < ms) {
    const n = rowOf(nestedId); const e = execOfTask(n);
    samples += 1;
    const key = `${n?.state}/${e?.state}`;
    if (seen[seen.length - 1] !== key) seen.push(key);
    if (n?.state === "cancelled") {
      if (firstCancelledAtMs === null) { firstCancelledAtMs = Date.now() - t0; execAtFirstCancelled = e?.state ?? null; }
      if (e && ACTIVE.includes(e.state)) violations += 1;
    }
    last = { nested: n?.state, exec: e?.state, reason: e?.reason };
    await pause(100);
  }
  return { violations, samples, last, seen, firstCancelledAtMs, execAtFirstCancelled };
}
try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  notes.daemonPid = await fx.startDaemon();
  curPeer = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  cA = (await host.issue(THREAD_A)).config;
  host.publish(cA, codexFullAccessFact(cA, "run-f3r4-1", fx.work), host.startRun(THREAD_A, "run-f3r4-1"));
  mcpA = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "F3R4" });

  // témoin terminal : tâche externe `completed` + accusée
  const dC = await mcpA.call("bridget_delegate", { ...base, request_id: "f3r4-completed", task: "NONCE_c1" });
  await untilT(() => rowOf(dC.payload.task_id)?.state === "result_available" && rowOf(dC.payload.task_id)?.result_sent ? true : undefined, 20000);
  const completedExecId = `execution-${rowOf(dC.payload.task_id)?.mission}`;
  const completedBefore = fx.execs().find((e: any) => e.execution_id === completedExecId);

  // ---------------- F3.1 daemon sain
  const d1 = await mcpA.call("bridget_delegate", { ...base, request_id: "f3r4-healthy", task: "NONCE_h1 NESTED_149[HB_149:60000]" });
  const root1 = d1.payload.task_id as string;
  const ready1 = await untilT(() => { const r = rowOf(root1); const n = nestedOf(root1); return r?.state === "waiting_for_children" && n?.state === "working" ? n : undefined; }, 25000);
  const c1 = await cancelTask(ready1?.task_id, "49000000-0000-4000-8000-0000000004f1");
  const inv1 = await watchInvariant(ready1?.task_id, 15000);
  const fin1 = { nested: rowOf(ready1?.task_id)?.state, exec: execOfTask(rowOf(ready1?.task_id)), root: rowOf(root1)?.state };
  notes.f31 = { cancel: c1, inv: inv1, fin: { nested: fin1.nested, exec: fin1.exec && [fin1.exec.state, fin1.exec.reason], root: fin1.root } };
  checks.add("F3.1", "REEL", "daemon sain : le descendant annulé devient `cancelled` ; à chaque échantillon (100 ms) l'exécution n'est JAMAIS active quand la tâche est `cancelled` (clôture CAS AVANT) ; exécution finale non active ; racine débloquée",
    !!ready1 && c1.ok && fin1.nested === "cancelled" && inv1.violations === 0 && inv1.firstCancelledAtMs !== null && fin1.exec && !ACTIVE.includes(fin1.exec.state) && fin1.root === "result_available",
    notes.f31);

  // ---------------- F3.2 coupure pendant l'annulation lente
  const d2 = await mcpA.call("bridget_delegate", { ...base, request_id: "f3r4-cut", task: "NONCE_k2 NESTED_149[IGNORE_TERM_149:6000 SLOW_149:6000]" });
  const root2 = d2.payload.task_id as string;
  const ready2 = await untilT(() => { const r = rowOf(root2); const n = nestedOf(root2); return r?.state === "waiting_for_children" && n?.state === "working" ? n : undefined; }, 25000);
  const nested2 = ready2?.task_id as string;
  const c2 = await cancelTask(nested2, "49000000-0000-4000-8000-0000000004f2");
  const window = await untilT(() => { const n = rowOf(nested2); const e = execOfTask(n); return n?.state === "cancelling" && e && ACTIVE.includes(e.state) ? { n: n.state, e: e.state, reason: e.reason } : undefined; }, 8000, 50);
  const tablesBefore = tables();
  const startsBefore = fx.evidence().filter((e: any) => e.event === "started").length;
  notes.f32Before = { cancel: c2, window, tables: tablesBefore };
  checks.add("F3.2.a", "REEL", "fenêtre atteinte par l'API : le descendant est `cancelling` avec exécution durable ACTIVE, racine `waiting_for_children`",
    !!window && rowOf(root2)?.state === "waiting_for_children", { window, root: rowOf(root2)?.state });
  await mcpA.close();
  curPeer.close();
  await fx.stopDaemon();
  await pause(2500);
  await fx.startDaemon();
  curPeer = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  const inv2 = await watchInvariant(nested2, 25000);
  const tablesAfter = tables();
  notes.f32After = { inv: inv2, tables: tablesAfter };
  const fin2 = { nested: rowOf(nested2), exec: execOfTask(rowOf(nested2)), root: rowOf(root2) };
  const deliveries2 = curPeer.deliveries.filter((d: any) => d.in_reply_to === d2.payload.message_id).length;
  checks.add("F3.2.b", "REEL", "après coupure en pleine annulation : le descendant est `cancelled`, son exécution est FERMÉE (non active) - jamais `cancelled` avec exécution active à aucun échantillon ; la racine n'attend plus (`result_available`), résultat remis UNE fois ; aucun nouveau lancement",
    fin2.nested?.state === "cancelled" && fin2.exec && !ACTIVE.includes(fin2.exec.state) && inv2.violations === 0 && fin2.root?.state === "result_available" && deliveries2 === 1 && fx.evidence().filter((e: any) => e.event === "started").length === startsBefore,
    { nested: fin2.nested?.state, exec: fin2.exec && [fin2.exec.state, fin2.exec.reason, fin2.exec.revision], root: fin2.root?.state, deliveries: deliveries2, inv: { violations: inv2.violations, seen: inv2.seen, firstCancelledAtMs: inv2.firstCancelledAtMs }, startsDelta: fx.evidence().filter((e: any) => e.event === "started").length - startsBefore });
  const completedAfter = fx.execs().find((e: any) => e.execution_id === completedExecId);
  checks.add("F3.2.c", "REEL", "exécution terminale `completed` (témoin) INCHANGÉE (état, motif, révision, génération) ; aucune exécution terminale rouverte ; trois tables capturées avant/après (voir notes)",
    JSON.stringify(completedAfter) === JSON.stringify(completedBefore) && completedAfter?.state === "completed" &&
      tablesBefore.execs.filter((e: any) => !ACTIVE.includes(e.state)).every((b: any) => { const a: any = tablesAfter.execs.find((x: any) => x.id === b.id); return a && a.state === b.state && a.rev === b.rev && a.gen === b.gen && a.reason === b.reason; }),
    { completedBefore: completedBefore && [completedBefore.state, completedBefore.reason, completedBefore.revision], completedAfter: completedAfter && [completedAfter.state, completedAfter.reason, completedAfter.revision], execsAfter: tablesAfter.execs, linksAfter: tablesAfter.links });
  curPeer.close();
} catch (error) {
  exitCode = 1;
  checks.add("EXC", "REEL", "exception non prévue", false, { message: String((error as Error).stack ?? error).slice(0, 900) });
} finally {
  notes.stopDaemonFinal = await fx.stopDaemon().catch((e: unknown) => String(e));
  const waitEnd = Date.now() + 40000;
  while ((await fx.liveFixtureProcesses()).some((p: any) => p.kind === "provider") && Date.now() < waitEnd) await pause(1000);
  notes.cleanup = await fx.cleanup().catch((e: unknown) => String(e));
  await host.stop().catch(() => undefined);
}
const summary = checks.summary();
const live = (notes.cleanup as any)?.remainingAfter ?? [];
await writeResults("f3-r4", { schema: "native-network-recipes/1", startedAt: T0S, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
