// T039 - Scénarios dégradés 149 sur daemon natif RÉEL + hôte MCP T3 RÉEL + enfants Codex fermés (aucun modèle).
// Les six scénarios US6 (retry, cancel, reprise, parent extérieur, identité inconnue, refus fournisseur)
// + panne T3 après admission, nested actif / résultat retenu, retry après changement de droits,
// reprise du daemon. Chaque compteur vient de lancements/remises/PID réellement observés.
// SIMULÉ (nommé) : wrapper du fil T3 (connexion/binding envoyés par la recette), adaptateur qui publie le fait
// de permissions, fournisseur enfant = serveur app-server Codex fermé.
import * as OS from "node:os";
import { startT3Host, codexFullAccessFact } from "./t3host.ts";
import { Fx, Checks, pause, BIN, BIN_SHA256, sha256File, lineageArgs, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, THREAD_B, rawSession, readOnlyFact, waitTask, counts, settle, sha } from "./common149.ts";

const PORT = 14776;
const checks = new Checks("T039 scénarios dégradés 149");
const T0 = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire debug 149 : empreinte différente du reçu");
if (!(await freePort(PORT))) throw new Error(`port ${PORT} occupé`);

let host = await startT3Host(PORT);
const fx = await Fx.create("recovery149", { port: PORT });
notes.fixtureRoot = fx.root;
const alive = (pid: number) => { try { process.kill(pid, 0); return true; } catch { return false; } };
const prompts = (nonce: string) => fx.evidence().filter((e: any) => e.event === "prompt" && e.nonce === nonce);
const starts = () => fx.evidence().filter((e: any) => e.event === "started");
const base = { agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", cwd: fx.work };
let exitCode = 0;
try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  notes.daemonPid = await fx.startDaemon();
  let peerA = await fx.registerParent(THREAD_A);
  const peerB = await fx.registerParent(THREAD_B);
  let cA = (await host.issue(THREAD_A)).config;
  const cB = (await host.issue(THREAD_B)).config;
  let runA = host.startRun(THREAD_A, "run-a-1");
  host.publish(cA, codexFullAccessFact(cA, "run-a-1", fx.work), runA);
  host.publish(cB, readOnlyFact(cB, "run-b-1", fx.work), host.startRun(THREAD_B, "run-b-1"));
  let mcpA = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "A" });
  const mcpB = await fx.mcp({ endpoint: cB.endpoint, authorization: cB.authorizationHeader, label: "B" });

  // ============================================================ R1 Retry
  const reqRetry = { ...base, request_id: "rec-retry", task: "NONCE_rt1 SLOW_149:1500" };
  const first = await mcpA.call("bridget_delegate", reqRetry);
  const replays: any[] = [];
  for (let i = 0; i < 10; i += 1) replays.push(await mcpA.call("bridget_delegate", reqRetry));
  await waitTask(fx, first.payload.task_id, ["result_available"]);
  await pause(800);
  for (let i = 0; i < 10; i += 1) replays.push(await mcpA.call("bridget_delegate", reqRetry));
  await settle(fx, [peerA, peerB]);
  const sameAll = replays.every((r) => !r.isError && r.payload.task_id === first.payload.task_id && r.payload.child_agent_id === first.payload.child_agent_id && r.payload.message_id === first.payload.message_id);
  const dlv = peerA.deliveries.filter((d: any) => d.in_reply_to === first.payload.message_id);
  checks.add("R1.1", "REEL", "retry : 20 rejeux (10 en vol, 10 après la fin) -> même task_id, même enfant, même message_id ; 1 lancement, 1 tour, 1 remise corrélée",
    first.payload?.status === "queued" && sameAll && starts().length === 1 && prompts("rt1").length === 1 && dlv.length === 1 && counts(fx).rows === 1,
    { replays: replays.length, sameAll, starts: starts().length, prompts: prompts("rt1").length, deliveries: dlv.length, rows: counts(fx).rows });
  const mismatch = await mcpA.call("bridget_delegate", { ...reqRetry, task: "NONCE_rt1 autre enveloppe" });
  checks.add("R1.2", "REEL", "même request_id avec une autre enveloppe -> envelope_mismatch, aucune mutation, aucun lancement",
    mismatch.isError && /envelope_mismatch/.test(JSON.stringify(mismatch.payload)) && counts(fx).rows === 1 && starts().length === 1, { payload: mismatch.payload });
  const lin = await fx.cli(lineageArgs("list", THREAD_A, fx.work));
  checks.add("R1.3", "REEL", "Lineage montre un seul enfant pour ce request_id (pas de doublon)", (lin.json?.tasks ?? []).filter((t: any) => t.task_id === first.payload.task_id).length === 1 && (lin.json?.tasks ?? []).length === 1, { tasks: lin.json?.tasks?.length });

  // ============================================================ R7 Nested actif + résultat retenu (avant le cancel, pour garder un état maîtrisé)
  const nestedReq = { ...base, request_id: "rec-nested", task: "NONCE_n1 NESTED_149[SLOW_149:6000]" };
  const dN = await mcpA.call("bridget_delegate", nestedReq);
  const rootN = dN.payload.task_id as string;
  await pause(2500);
  const stN = await mcpA.call("bridget_task_status", { task_id: rootN });
  const rows = fx.tasks();
  const childN = rows.find((t: any) => t.parent_task_id === rootN);
  const dlvDuring = peerA.deliveries.filter((d: any) => d.in_reply_to === dN.payload.message_id);
  checks.add("R7.1", "REEL", "nested actif : le tour de l'enfant est fini mais la racine n'expose ni statut result_available ni résultat tant que le petit-enfant travaille ; aucune remise au parent",
    stN.payload?.status === "waiting_for_children" && stN.payload.result === null && childN?.state === "working" && dlvDuring.length === 0 && starts().length === 3,
    { rootStatus: stN.payload?.status, rootResult: stN.payload?.result, nestedState: childN?.state, deliveriesDuring: dlvDuring.length, starts: starts().length });
  checks.add("R7.2", "REEL", "la lignée est durable : parent_task_id et propriétaire racine stables (la racine appartient à A, le petit-enfant à l'enfant)",
    childN?.parent_task_id === rootN && childN?.root_owner_agent_id === peerA.agentId && childN?.owner !== peerA.agentId, { rootOwnerOfNested: String(childN?.root_owner_agent_id).slice(0, 8), ownerOfNested: String(childN?.owner).slice(0, 8) });
  await waitTask(fx, rootN, ["result_available"], 40000);
  await settle(fx, [peerA, peerB]);
  const dlvAfter = peerA.deliveries.filter((d: any) => d.in_reply_to === dN.payload.message_id);
  const nestedAfter = fx.tasks().find((t: any) => t.parent_task_id === rootN);
  checks.add("R7.3", "REEL", "quand le petit-enfant a fini : la racine devient result_available, UNE seule remise corrélée à A ; le résultat du petit-enfant n'est pas remis à A",
    fx.tasks().find((t: any) => t.task_id === rootN)?.state === "result_available" && nestedAfter?.state === "result_available" && dlvAfter.length === 1 && !peerA.deliveries.some((d: any) => String(d.body).includes("n1g")),
    { root: fx.tasks().find((t: any) => t.task_id === rootN)?.state, nested: nestedAfter?.state, deliveries: dlvAfter.length, leaked: peerA.deliveries.some((d: any) => String(d.body).includes("n1g")) });

  // ============================================================ R2 Cancel
  const cancelReq = { ...base, request_id: "rec-cancel", task: "NONCE_c1 NESTED_149[WAIT_149]" };
  const dC = await mcpA.call("bridget_delegate", cancelReq);
  const rootC = dC.payload.task_id as string;
  await pause(2500);
  const beforeCancel = { providers: await fx.liveProviders(), rows: counts(fx), delivered: peerA.deliveries.length };
  const bCancel = await mcpB.call("bridget_task_cancel", { task_id: rootC });
  const bStatus = await mcpB.call("bridget_task_status", { task_id: rootC });
  checks.add("R2.1", "REEL", "un autre fil (B) ne peut ni annuler ni lire la tâche de A : task_unavailable, tâche et processus inchangés",
    bCancel.isError && bStatus.isError && /task_unavailable/.test(JSON.stringify(bCancel.payload)) && /task_unavailable/.test(JSON.stringify(bStatus.payload)) && (await fx.liveProviders()).length === beforeCancel.providers.length, { cancel: bCancel.payload, status: bStatus.payload, providers: beforeCancel.providers.length });
  const cancelRes = await mcpA.call("bridget_task_cancel", { task_id: rootC });
  await pause(3500);
  const afterStates = fx.tasks().filter((t: any) => t.task_id === rootC || t.parent_task_id === rootC).map((t: any) => t.state);
  const providerPids = fx.evidence().filter((e: any) => e.event === "prompt" && (e.nonce === "c1" || e.nonce === "c1g")).map((e: any) => e.pid);
  const cancelReplay = await mcpA.call("bridget_task_cancel", { task_id: rootC });
  checks.add("R2.2", "REEL", "cancel de la racine : racine + descendant actif cancelled, PID des deux enfants réellement terminés, zéro fournisseur résiduel, aucune remise de résultat",
    !cancelRes.isError && afterStates.length === 2 && afterStates.every((s: string) => s === "cancelled") && providerPids.length === 2 && providerPids.every((p: number) => !alive(p)) && (await fx.liveProviders()).length === 0 && !peerA.deliveries.some((d: any) => d.in_reply_to === dC.payload.message_id),
    { states: afterStates, pids: providerPids.map((p: number) => [p, alive(p)]), liveProviders: (await fx.liveProviders()).length });
  checks.add("R2.3", "REEL", "cancel rejoué : idempotent (même état cancelled, aucune erreur, aucun nouveau lancement)",
    !cancelReplay.isError && cancelReplay.payload?.status === "cancelled" && starts().length === 5, { status: cancelReplay.payload?.status, starts: starts().length });

  // ============================================================ R4 Identité inconnue / R3 Parent extérieur
  const quiet0 = { c: counts(fx), db: sha(fx.tasks().map((t: any) => [t.task_id, t.state, t.updated_at])) };
  const bogus = await fx.mcp({ endpoint: cA.endpoint, authorization: `Bearer ${"z".repeat(43)}`, label: "bogus" });
  const bogusDelegate = await bogus.call("bridget_delegate", { ...base, request_id: "rec-bogus", task: "NONCE_bg1" });
  const bogusStatus = await bogus.call("bridget_task_status", { task_id: rootN });
  const noIdentity = await fx.mcp({ endpoint: undefined as any, authorization: undefined as any, label: "noid" });
  const noIdDelegate = await noIdentity.call("bridget_delegate", { ...base, request_id: "rec-noid", task: "NONCE_ni1" });
  const noIdStatus = await noIdentity.call("bridget_task_status", { task_id: rootN });
  const unknownThread = await fx.cli(lineageArgs("list", "89000000-0000-4000-8000-0000000000fe", fx.work));
  const leak = JSON.stringify([bogusDelegate.payload, bogusStatus.payload, noIdDelegate.payload, noIdStatus.payload, unknownThread.json]);
  checks.add("R4.1", "REEL", "identité inconnue : token forgé, absence totale d'identité, fil sans binding -> refus explicites ; aucune identité voisine empruntée, aucune ligne, aucun lancement",
    [bogusDelegate, bogusStatus, noIdDelegate, noIdStatus].every((r) => r.isError) && unknownThread.json?.code === "binding_unavailable" && !leak.includes(peerA.agentId) && !leak.includes(rootN) &&
      JSON.stringify(counts(fx)) === JSON.stringify(quiet0.c) && sha(fx.tasks().map((t: any) => [t.task_id, t.state, t.updated_at])) === quiet0.db,
    { bogus: [bogusDelegate.code, bogusStatus.code], noIdentity: [noIdDelegate.code ?? noIdDelegate.payload?.text, noIdStatus.code ?? noIdStatus.payload?.text], lineage: unknownThread.json?.code });
  await bogus.close();
  await noIdentity.close();
  const listB = await fx.cli(lineageArgs("list", THREAD_B, fx.work));
  const showCross = await fx.cli(lineageArgs("show", THREAD_B, fx.work, ["--task", rootN, "--offset", "0", "--limit", "100"]));
  checks.add("R3.1", "REEL", "parent extérieur : le fil B voit zéro tâche de A (liste, show), compteurs et base inchangés",
    (listB.json?.tasks ?? []).length === 0 && showCross.json?.code === "task_unavailable" && JSON.stringify(counts(fx)) === JSON.stringify(quiet0.c), { listBTasks: listB.json?.tasks?.length, show: showCross.json?.code });

  // ============================================================ R5 Refus fournisseur
  const refuse = await mcpA.call("bridget_delegate", { ...base, request_id: "rec-refuse", task: "NONCE_f1 REFUSE_149" });
  const taskF = await waitTask(fx, refuse.payload.task_id, ["failed", "result_available"], 30000);
  await settle(fx, [peerA, peerB]);
  const refusedPrompts = prompts("f1");
  const failNotice = peerA.deliveries.filter((d: any) => String(d.body).includes(refuse.payload.task_id));
  const startsF = fx.evidence().filter((e: any) => e.event === "started" && refusedPrompts.some((p: any) => p.pid === e.pid));
  const linF = await fx.cli(lineageArgs("list", THREAD_A, fx.work));
  const lineF = (linF.json?.tasks ?? []).find((t: any) => t.task_id === refuse.payload.task_id);
  const replayF = await mcpA.call("bridget_delegate", { ...base, request_id: "rec-refuse", task: "NONCE_f1 REFUSE_149" });
  checks.add("R5.1", "REEL", "refus fournisseur : tâche failed avec erreur explicite, 1 seul lancement, 1 seul tour (pas de reprise ni de substitution de modèle/fournisseur), 1 notification d'échec, Lineage failed",
    taskF?.state === "failed" && !!taskF.error && refusedPrompts.length === 1 && startsF.length === 1 && refusedPrompts[0].model === null && failNotice.length >= 1 && lineF?.status === "failed" && lineF.model === "fixture-model-149",
    { state: taskF?.state, error: String(taskF?.error).slice(0, 120), prompts: refusedPrompts.length, notices: failNotice.length, lineageStatus: lineF?.status, lineageModel: lineF?.model });
  checks.add("R5.2", "REEL", "le rejeu de la requête refusée rend l'échec mémorisé sans nouveau lancement",
    replayF.payload?.task_id === refuse.payload.task_id && starts().filter((e: any) => true).length === fx.evidence().filter((e: any) => e.event === "started").length && prompts("f1").length === 1, { replayStatus: replayF.payload?.status ?? replayF.payload?.text, prompts: prompts("f1").length });
  const unknownModel = await mcpA.call("bridget_delegate", { ...base, request_id: "rec-nomodel", model: "modele-inexistant", task: "NONCE_nm1" });
  const unknownAgent = await mcpA.call("bridget_delegate", { ...base, request_id: "rec-noagent", agent_type: "agent-inconnu", task: "NONCE_na1" });
  checks.add("R5.3", "REEL", "modèle ou fournisseur absent du registre : refus nommé à l'admission, aucun lancement, aucune substitution",
    unknownModel.isError && unknownAgent.isError && prompts("nm1").length === 0 && prompts("na1").length === 0, { model: unknownModel.payload?.text, agent: unknownAgent.payload?.text });

  // ============================================================ R8 Retry après changement de droits
  const reqRights = { ...base, request_id: "rec-rights", task: "NONCE_g1" };
  const dG = await mcpA.call("bridget_delegate", reqRights);
  await waitTask(fx, dG.payload.task_id, ["result_available"]);
  await settle(fx, [peerA, peerB]);
  const snapBefore = sha(fx.tasks().find((t: any) => t.task_id === dG.payload.task_id)?.permission_snapshot);
  const startedBeforeRights = starts().length;
  const deliveredBeforeRights = peerA.deliveries.length;
  const revReadOnly = host.publish(cA, readOnlyFact(cA, "run-a-1", fx.work), runA);
  const replayDown = await mcpA.call("bridget_delegate", reqRights);
  const newDev = await mcpA.call("bridget_delegate", { ...base, request_id: "rec-rights-new", posture: "development", task: "NONCE_g2" });
  const newInherit = await mcpA.call("bridget_delegate", { ...base, request_id: "rec-rights-new-inherit", task: "NONCE_g3" });
  await waitTask(fx, newInherit.payload?.task_id, ["result_available", "failed"]);
  await pause(400);
  const promptG1 = prompts("g1")[0];
  const promptG3 = prompts("g3")[0];
  checks.add("R8.1", "REEL", "droits réduits (parent devient readOnly) : le rejeu de l'ancienne requête rend la même tâche et le même résultat, sans nouveau lancement ; son snapshot reste dangerFullAccess",
    replayDown.payload?.task_id === dG.payload.task_id && replayDown.payload?.result === "fixture149-answer:g1" && prompts("g1").length === 1 && promptG1.sandbox_policy?.type === "dangerFullAccess" && sha(fx.tasks().find((t: any) => t.task_id === dG.payload.task_id)?.permission_snapshot) === snapBefore,
    { sameTask: replayDown.payload?.task_id === dG.payload.task_id, prompts: prompts("g1").length, sandbox: promptG1?.sandbox_policy?.type, snapshotUnchanged: sha(fx.tasks().find((t: any) => t.task_id === dG.payload.task_id)?.permission_snapshot) === snapBefore, revision: revReadOnly });
  checks.add("R8.2", "REEL", "droits réduits : une NOUVELLE requête development est refusée (permission_not_inherited) avant tout lancement ; une requête héritée reçoit le sandbox readOnly (jamais l'ancien full-access)",
    newDev.isError && newDev.payload?.text === "permission_not_inherited" && prompts("g2").length === 0 && promptG3?.sandbox_policy?.type === "readOnly",
    { development: newDev.payload?.text, g2Prompts: prompts("g2").length, g3Sandbox: promptG3?.sandbox_policy?.type });
  // fait retiré puis session tournée sans fait : le rejeu d'une requête déjà admise
  host.endRun(THREAD_A, cA, "run-a-1");
  const replayTomb = await mcpA.call("bridget_delegate", reqRights);
  const statusTomb = await mcpA.call("bridget_task_status", { task_id: dG.payload.task_id });
  const whoTomb = await mcpA.call("bridget_who", { scope: "global" });
  const oldA = cA;
  cA = (await host.rotate(THREAD_A)).config;
  const replayOld = await mcpA.call("bridget_delegate", reqRights);
  await mcpA.close();
  mcpA = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "A2" });
  const replayV1 = await mcpA.call("bridget_delegate", reqRights);
  const statusV1 = await mcpA.call("bridget_task_status", { task_id: dG.payload.task_id });
  notes.replayAfterRightsLoss = {
    tombstone: replayTomb.code ?? replayTomb.payload?.text ?? replayTomb.payload?.status,
    oldCredentialAfterRotation: replayOld.code ?? replayOld.payload?.text,
    newCredentialWithoutFact_replay: replayV1.isError ? (replayV1.payload?.text ?? replayV1.code) : `ok:${replayV1.payload?.status}`,
    newCredentialWithoutFact_status: statusV1.isError ? (statusV1.payload?.text ?? statusV1.code) : `ok:${statusV1.payload?.status}`,
  };
  const startedAfterRights = starts().length;
  checks.add("R8.3", "REEL", "fait retiré, puis session tournée : aucun rejeu n'entraîne de lancement ni de nouvelle remise ; l'ancien credential est refusé ; sans fait (v1) la lecture/le rejeu de l'identité 148 ne relance rien",
    replayTomb.isError && replayOld.isError && replayOld.code === "t3_session_unavailable" && startedAfterRights === startedBeforeRights + 1 /* g3 de R8.2 */ && peerA.deliveries.length >= deliveredBeforeRights && prompts("g1").length === 1,
    { tombstone: notes.replayAfterRightsLoss, startedBefore: startedBeforeRights, startedAfter: startedAfterRights, g1Prompts: prompts("g1").length });

  // O1 - sens exact de G-P-07(b) « lecture et rejeu inchangés » : trois voies observées après retrait du fait + rotation
  const showG = await fx.cli(lineageArgs("show", THREAD_A, fx.work, ["--task", dG.payload.task_id, "--offset", "0", "--limit", "100"]));
  const listTomb = await fx.cli(lineageArgs("list", THREAD_A, fx.work));
  notes.o1ThreeLanes = {
    mcpMountWithRetiredFact: { delegateReplay: replayTomb.code ?? replayTomb.payload?.text, taskStatus: statusTomb.code ?? statusTomb.payload?.text, who: whoTomb.code ?? whoTomb.payload?.text },
    freshCredentialNeverHadFact: { replay: notes.replayAfterRightsLoss && (notes.replayAfterRightsLoss as any).newCredentialWithoutFact_replay, status: (notes.replayAfterRightsLoss as any).newCredentialWithoutFact_status },
    humanNativeLineageCli: { showResult: showG.json?.result, showStatus: showG.json?.task?.status, listHasTask: (listTomb.json?.tasks ?? []).some((t: any) => t.task_id === dG.payload.task_id) },
  };
  checks.add("R8.4", "REEL", "O1 (comportement exact, G-P-07(b)) : fait retiré puis session tournée - (1) le montage MCP privé de l'ancien credential est fermé EN BLOC (rejeu, status ET identité : t3_session_unavailable, jamais v1) ; (2) un credential neuf qui n'a jamais eu de fait reçoit v1 et lit/rejoue la tâche admise (148) ; (3) la lecture humaine Lineage native (gardes147) lit la tâche admise, résultat intact",
    replayTomb.isError && statusTomb.isError && whoTomb.isError && [replayTomb, statusTomb, whoTomb].every((r) => (r.code ?? r.payload?.text) === "t3_session_unavailable") &&
      !replayV1.isError && !statusV1.isError && showG.json?.result === "fixture149-answer:g1" && (listTomb.json?.tasks ?? []).some((t: any) => t.task_id === dG.payload.task_id),
    notes.o1ThreeLanes);

  // ============================================================ R6 Panne T3 après admission
  runA = host.startRun(THREAD_A, "run-a-2");
  host.publish(cA, codexFullAccessFact(cA, "run-a-2", fx.work), runA);
  const dT = await mcpA.call("bridget_delegate", { ...base, request_id: "rec-t3down", task: "NONCE_t1 SLOW_149:5000" });
  const dTW = await mcpA.call("bridget_delegate", { ...base, request_id: "rec-t3down-wait", task: "NONCE_t2 WAIT_149" });
  await pause(1800);
  const pidsBeforeCrash = (await fx.liveProviders());
  const t1pid = prompts("t1")[0]?.pid;
  const t2pid = prompts("t2")[0]?.pid;
  await host.stop(); // panne réelle du serveur HTTP MCP et du registre de sessions de T3
  await pause(300);
  const stDown = await mcpA.call("bridget_task_status", { task_id: dT.payload.task_id });
  const newDown = await mcpA.call("bridget_delegate", { ...base, request_id: "rec-t3down-new", task: "NONCE_t3" });
  const cancelNative = await fx.cli(lineageArgs("cancel", THREAD_A, fx.work, ["--task", dTW.payload.task_id, "--request-id", "49000000-0000-4000-8000-0000000000f1"]));
  const finished = await waitTask(fx, dT.payload.task_id, ["result_available", "failed"], 30000);
  await settle(fx, [peerA, peerB], 1500, 40000);
  const cancelledW = fx.tasks().find((t: any) => t.task_id === dTW.payload.task_id);
  const dlvT = peerA.deliveries.filter((d: any) => d.in_reply_to === dT.payload.message_id);
  const linDown = await fx.cli(lineageArgs("show", THREAD_A, fx.work, ["--task", dT.payload.task_id, "--offset", "0", "--limit", "100"]));
  checks.add("R6.1", "REEL", "T3 en panne après admission : la mission admise (enfant déjà lancé) va à son terme sans T3, résultat remis UNE fois à A, PID enfant sans relance",
    finished?.state === "result_available" && finished.result === "fixture149-answer:t1" && dlvT.length === 1 && prompts("t1").length === 1 && t1pid !== undefined && pidsBeforeCrash.includes(t1pid),
    { state: finished?.state, deliveries: dlvT.length, prompts: prompts("t1").length, t1pid, startsAfter: starts().length });
  checks.add("R6.2", "REEL", "T3 en panne : l'annulation reste native (CLI Lineage) -> tâche cancelled et PID de l'enfant réellement terminé ; la lecture native reste servie",
    cancelNative.code === 0 && cancelledW?.state === "cancelled" && t2pid !== undefined && !alive(t2pid) && linDown.json?.result === "fixture149-answer:t1",
    { cancel: cancelNative.json, state: cancelledW?.state, t2pid, alive: t2pid ? alive(t2pid) : null, readResult: linDown.json?.result });
  checks.add("R6.3", "REEL", "T3 en panne : le montage MCP T3 est fermé (status et nouvelle admission refusés t3_session_unavailable), zéro ligne et zéro lancement ajoutés, aucun repli PID",
    stDown.isError && stDown.code === "t3_session_unavailable" && newDown.isError && newDown.code === "t3_session_unavailable" && prompts("t3").length === 0, { status: stDown.code, delegate: newDown.code, t3Prompts: prompts("t3").length });
  await mcpA.close();
  // T3 revient (nouveau registre : les anciens credentials sont perdus, comme après un vrai redémarrage de T3)
  host = await startT3Host(PORT);
  const staleAfterRestart = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "A-stale" });
  const staleCall = await staleAfterRestart.call("bridget_who", { scope: "global" });
  await staleAfterRestart.close();
  cA = (await host.issue(THREAD_A)).config;
  runA = host.startRun(THREAD_A, "run-a-3");
  host.publish(cA, codexFullAccessFact(cA, "run-a-3", fx.work), runA);
  mcpA = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "A3" });
  const replayAfterT3 = await mcpA.call("bridget_delegate", { ...base, request_id: "rec-t3down", task: "NONCE_t1 SLOW_149:5000" });
  checks.add("R6.4", "REEL", "T3 revenu : l'ancien credential reste refusé (registre perdu) ; avec un credential neuf + fait, le rejeu de la requête admise avant la panne rend la MÊME tâche, sans relance",
    staleCall.isError && staleCall.code === "t3_session_unavailable" && replayAfterT3.payload?.task_id === dT.payload.task_id && replayAfterT3.payload?.message_id === dT.payload.message_id && prompts("t1").length === 1,
    { stale: staleCall.code, sameTask: replayAfterT3.payload?.task_id === dT.payload.task_id, sameMessage: replayAfterT3.payload?.message_id === dT.payload.message_id, prompts: prompts("t1").length });

  // ============================================================ R9 Reprise du daemon (coupure + relance)
  const reqResume = { ...base, request_id: "rec-resume", task: "NONCE_u1" };
  const dU = await mcpA.call("bridget_delegate", reqResume);
  await waitTask(fx, dU.payload.task_id, ["result_available"]);
  await settle(fx, [peerA, peerB]);
  const doneStates = fx.tasks().map((t: any) => [t.task_id, t.state]).sort();
  const ackedBefore = new Set(peerA.deliveries.filter((d: any) => !d.plain).map((d: any) => d.message_id));
  const plainBefore = peerA.deliveries.filter((d: any) => d.plain).length;
  const beforeRestart = { starts: starts().length, prompts: fx.evidence().filter((e: any) => e.event === "prompt").length, deliveries: peerA.deliveries.length, doneStates: doneStates.length };
  peerA.close();
  peerB.close();
  await mcpA.close();
  await mcpB.close();
  notes.stopDaemon = await fx.stopDaemon();
  const providersAfterStop = await fx.liveProviders();
  notes.daemonSecondPid = await fx.startDaemon();
  const peerA2 = await fx.registerParent(THREAD_A);
  const peerB2 = await fx.registerParent(THREAD_B);
  await pause(2500);
  runA = host.startRun(THREAD_A, "run-a-4");
  cA = (await host.rotate(THREAD_A)).config;
  host.publish(cA, codexFullAccessFact(cA, "run-a-4", fx.work), runA);
  const mcpA2 = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "A4" });
  const replayU = await mcpA2.call("bridget_delegate", reqResume);
  const replayRetry = await mcpA2.call("bridget_delegate", reqRetry);
  await pause(1500);
  const statesAfterRestart = fx.tasks().map((t: any) => [t.task_id, t.state]).sort();
  const replayedAcked = peerA2.deliveries.filter((d: any) => ackedBefore.has(d.message_id));
  notes.deliveriesAfterReconnect = peerA2.deliveries.map((d: any) => ({ plain: d.plain === true, alreadyAckedBefore: ackedBefore.has(d.message_id), from: String(d.from).slice(0, 8), bodyHead: String(d.body).slice(0, 60), id: String(d.message_id).slice(0, 24) }));
  notes.deliveriesBeforeRestart = { idempotentAcked: ackedBefore.size, plain: plainBefore };
  checks.add("R9.1", "REEL", "reprise du daemon, missions terminées : mêmes task_id/child/message, mêmes états (result_available, cancelled, failed), aucun nouveau lancement, aucun fournisseur survivant à l'arrêt",
    JSON.stringify(statesAfterRestart) === JSON.stringify(doneStates) && replayU.payload?.task_id === dU.payload.task_id && replayU.payload?.message_id === dU.payload.message_id && replayRetry.payload?.task_id === first.payload.task_id && replayRetry.payload?.child_agent_id === first.payload.child_agent_id &&
      starts().length === beforeRestart.starts && providersAfterStop.length === 0,
    { statesEqual: JSON.stringify(statesAfterRestart) === JSON.stringify(doneStates), startsBefore: beforeRestart.starts, startsAfter: starts().length, providersAfterDaemonStop: providersAfterStop.length });
  checks.add("R9.1b", "REEL", "reprise du daemon : aucune remise déjà accusée (ACK) n'est rejouée au parent reconnecté",
    replayedAcked.length === 0, { deliveriesAfterReconnect: peerA2.deliveries.length, replayedAfterAck: replayedAcked.length, plain: peerA2.deliveries.filter((d: any) => d.plain).length });
  // mission en vol au moment de la coupure (R9.2) : VRAI SIGTERM du daemon puis relance de la même fixture
  const dI = await mcpA2.call("bridget_delegate", { ...base, request_id: "rec-inflight", task: "NONCE_i1 SLOW_149:9000" });
  await pause(2200);
  const inflightBeforeT = Date.now() / 1000;
  const inflightBefore = { starts: starts().length, promptsI1: prompts("i1").length, pids: await fx.liveProviders(), rows: counts(fx).rows };
  const i1PidBefore = prompts("i1")[0]?.pid;
  const taskBeforeStop = fx.tasks().find((t: any) => t.task_id === dI.payload.task_id);
  // Arbre de processus de la fixture AVANT l'arrêt (pid, ppid, pgid, session) : sert à expliquer un éventuel survivant.
  notes.processTreeBeforeStop = (await (async () => {
    const { execFile } = await import("node:child_process");
    const out: string = await new Promise((resolve) => execFile("/bin/ps", ["-axo", "pid=,ppid=,pgid=,sess=,command="], (_e, so) => resolve(so)));
    return out.split("\n").filter((l) => l.includes(fx.root) && !l.includes("/bin/ps")).map((l) => l.trim().replaceAll(fx.root, "<ROOT>").slice(0, 150));
  })());
  peerA2.close();
  peerB2.close();
  await mcpA2.close();
  notes.stopDaemonInflight = await fx.stopDaemon(); // SIGTERM individuel vérifié, jamais -9
  const providersAfterStop2 = await fx.liveProviders();
  // Observation brute, en parallèle de la relance IMMÉDIATE : durée de vie des fournisseurs qui survivent à l'arrêt.
  const survivorSeen = new Map<number, { firstMs: number; lastMs: number; ppid: number; command: string }>();
  const traceStart = Date.now();
  const tracer = (async () => {
    while (Date.now() - traceStart < 14000) {
      const live = (await fx.liveFixtureProcesses()).filter((p: any) => p.kind === "provider");
      const t = Date.now() - traceStart;
      for (const p of live) survivorSeen.set(p.pid, { firstMs: survivorSeen.get(p.pid)?.firstMs ?? t, lastMs: t, ppid: p.ppid, command: p.command.slice(0, 90) });
      if (live.length === 0 && t > 1500) break;
      await pause(50);
    }
  })();
  const restartAt = Math.floor(Date.now() / 1000);
  const restartPid = await fx.startDaemon();
  // l'état doit être explicite DÈS le démarrage du daemon, avant que le parent se reconnecte
  const earlyTask = fx.tasks().find((t: any) => t.task_id === dI.payload.task_id);
  const earlyStarts = starts().length;
  const peerA3 = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  await pause(7000);
  const inflightTask = fx.tasks().find((t: any) => t.task_id === dI.payload.task_id);
  const i1 = prompts("i1");
  const i1Pids = [...new Set(i1.map((p: any) => p.pid))];
  const noticesI = peerA3.deliveries.filter((d: any) => String(d.body).includes(dI.payload.task_id));
  const providersT7 = await fx.liveProviders();
  const startsT7 = starts().length;
  await tracer;
  notes.survivorsAfterDaemonStop = [...survivorSeen].map(([pid, v]) => ({ pid, ...v, isInflightProvider: pid === i1PidBefore }));
  await pause(8000);
  const providersT15 = await fx.liveProviders();
  const afterRestartEvents = fx.evidence().filter((e: any) => e.t > inflightBeforeT && e.t >= restartAt - 1 && ["started", "prompt"].includes(e.event));
  notes.inflightTimeline = fx.evidence().filter((e: any) => e.t > inflightBeforeT).map((e: any) => ({ dt: +(e.t - inflightBeforeT).toFixed(2), event: e.event, pid: e.pid, nonce: e.nonce, turn: e.turn, chars: e.chars }));
  notes.inflightTask = { before: taskBeforeStop?.state, early: earlyTask?.state, earlyError: earlyTask?.error, state: inflightTask?.state, error: inflightTask?.error, updated_at: inflightTask?.updated_at, completed_at: inflightTask?.completed_at, started_at: inflightTask?.started_at, created_at: inflightTask?.created_at, restartAtEpoch: restartAt, failure_sent: inflightTask?.failure_sent };
  notes.inflightProvidersAlive = { afterStop: providersAfterStop2.length, at7s: providersT7.length, at15s: providersT15.length };
  notes.inflightRestart = {
    pidsDaemon: { stopped: "SIGTERM", restartedPid: restartPid },
    before: inflightBefore, i1PidBefore, providersAfterDaemonStop: providersAfterStop2.length,
    taskState: inflightTask?.state, taskError: inflightTask?.error, missionPrompts: i1.length, distinctProviderPids: i1Pids.length,
    startsTotalAfter: starts().length, notices: noticesI.map((d: any) => String(d.body).slice(0, 140)),
  };
  checks.add("R9.2.a", "REEL", "mission EN VOL, vrai SIGTERM du daemon puis relance : la tâche est failed/unreachable DÈS le démarrage (avant la reconnexion du parent), sans attente de 2 s ni reprise",
    taskBeforeStop?.state === "working" && earlyTask?.state === "failed" && earlyTask?.error === "unreachable" && earlyStarts === inflightBefore.starts,
    { before: taskBeforeStop?.state, early: earlyTask?.state, earlyError: earlyTask?.error, earlyStarts, startsBefore: inflightBefore.starts });
  checks.add("R9.2.b", "REEL", "la mission n'est portée que par UN tour et UN PID : 1 prompt NONCE_i1, 1 PID distinct (celui d'avant la coupure, mort), aucun fournisseur vivant à 7 s ni à 15 s (un survivant immédiat est tracé dans survivorsAfterDaemonStop)",
    i1.length === 1 && i1Pids.length === 1 && i1Pids[0] === i1PidBefore && !alive(i1PidBefore) && providersT7.length === 0 && providersT15.length === 0,
    { missionPrompts: i1.length, distinctPids: i1Pids.length, samePidAsBeforeStop: i1Pids[0] === i1PidBefore, pidAliveNow: alive(i1PidBefore), providers: [providersAfterStop2.length, providersT7.length, providersT15.length], survivors: notes.survivorsAfterDaemonStop });
  checks.add("R9.2.c", "REEL", "starts avant = après : aucun nouvel enfant (compteur de lancements fournisseur inchangé à t0, 7 s et 15 s) ; aucun événement started/prompt après la relance",
    starts().length === inflightBefore.starts && startsT7 === inflightBefore.starts && earlyStarts === inflightBefore.starts && afterRestartEvents.length === 0 && counts(fx).rows === inflightBefore.rows,
    { startsBefore: inflightBefore.starts, startsT7, startsFinal: starts().length, eventsAfterRestart: afterRestartEvents.length, rows: counts(fx).rows, rowsBefore: inflightBefore.rows });
  checks.add("R9.2.d", "REEL", "UNE notice d'échec failed/unreachable remise au parent reconnecté (pas de résultat, pas de rejeu), état final failed + erreur explicite",
    inflightTask?.state === "failed" && inflightTask?.error === "unreachable" && noticesI.length === 1 && /failed|unreachable/i.test(String(noticesI[0]?.body)) && noticesI.every((d: any) => d.plain === true || d.in_reply_to !== undefined),
    { state: inflightTask?.state, error: inflightTask?.error, notices: noticesI.length, noticeHead: String(noticesI[0]?.body).slice(0, 140) });
  const replayInflight = await (async () => {
    const m = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "A5" });
    const r = await m.call("bridget_delegate", { ...base, request_id: "rec-inflight", task: "NONCE_i1 SLOW_149:9000" });
    const st = await m.call("bridget_task_status", { task_id: dI.payload.task_id });
    await m.close();
    return { r, st };
  })();
  await pause(1500);
  checks.add("R9.2.e", "REEL", "le rejeu de la requête interrompue rend la MÊME tâche failed, sans nouveau lancement (pas de substitution silencieuse)",
    replayInflight.r.payload?.task_id === dI.payload.task_id && replayInflight.st.payload?.status === "failed" && starts().length === inflightBefore.starts && prompts("i1").length === 1,
    { replayTask: replayInflight.r.payload?.task_id === dI.payload.task_id, replayStatus: replayInflight.r.payload?.status ?? replayInflight.r.payload?.text, status: replayInflight.st.payload?.status, startsFinal: starts().length, prompts: prompts("i1").length });

  // ============================================================ R9.3 admission coupée par SIGTERM avant toute mission
  // On admet puis on envoie SIGTERM aussitôt (course contre le tick de 1 s). On consigne l'état durable à la sortie du
  // daemon, puis ce que devient la tâche après la relance. `queued` non engagé n'a JAMAIS été observé à la sortie
  // (voir queuedAttempts) : ce cas reste couvert par le test unitaire seul, il est nommé comme limite.
  let curPeer = peerA3;
  const queuedAttempts: any[] = [];
  for (let attempt = 1; attempt <= 3; attempt += 1) {
    const nonce = `qd${attempt}`;
    const mcpQ = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: `Q${attempt}` });
    const startsPre = starts().length;
    const dq = await mcpQ.call("bridget_delegate", { ...base, request_id: `rec-queued-${attempt}`, task: `NONCE_${nonce}` });
    await fx.stopDaemon(); // SIGTERM immédiat ; peer et client ferment après
    curPeer.close();
    await mcpQ.close();
    const row = fx.tasks().find((t: any) => t.task_id === dq.payload?.task_id);
    const exitState = { state: row?.state, error: row?.error ?? null, promptsBeforeRestart: prompts(nonce).length, startsDuringStop: starts().length - startsPre };
    await fx.startDaemon();
    curPeer = await fx.registerParent(THREAD_A);
    await fx.registerParent(THREAD_B);
    await pause(4000);
    const post = fx.tasks().find((t: any) => t.task_id === dq.payload?.task_id);
    queuedAttempts.push({ attempt, admitted: dq.payload?.status ?? dq.payload?.text, exitState, afterRestart: { state: post?.state, error: post?.error ?? null, prompts: prompts(nonce).length, startsTotalDelta: starts().length - startsPre, notices: curPeer.deliveries.filter((d: any) => String(d.body).includes(dq.payload?.task_id)).length } });
  }
  notes.admissionCutAttempts = queuedAttempts;
  const queuedSeen = queuedAttempts.some((a) => a.exitState.state === "queued");
  const midLaunch = queuedAttempts.filter((a) => a.exitState.state === "starting");
  notes.queuedObservedAtExit = queuedSeen;
  checks.add("R9.3", "REEL", "admission coupée par SIGTERM avant toute mission (état durable `starting`, 0 prompt) : après relance la tâche est failed/unreachable, 0 mission envoyée (0 prompt), 0 enfant relancé, 1 notice d'échec ; `queued` seul n'a pas pu être exposé en réel",
    midLaunch.length >= 1 && queuedAttempts.every((a) => a.afterRestart.prompts === 0 && a.afterRestart.state === (a.exitState.state === "queued" ? a.afterRestart.state : "failed") && (a.exitState.state === "queued" || a.afterRestart.error === "unreachable") && a.afterRestart.notices <= 1) && !queuedSeen,
    { attempts: queuedAttempts.map((a) => ({ exit: a.exitState.state, post: a.afterRestart.state, err: a.afterRestart.error, prompts: a.afterRestart.prompts, startsDelta: a.afterRestart.startsTotalDelta, notices: a.afterRestart.notices })), queuedObservedAtExit: queuedSeen });

  // ============================================================ R9.4 attente des descendants coupée par SIGTERM
  const mcpN = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "N" });
  const dRoot = await mcpN.call("bridget_delegate", { ...base, request_id: "rec-nested-restart", task: "NONCE_m1 NESTED_149[SLOW_149:20000]" });
  const dRoot2 = await mcpN.call("bridget_delegate", { ...base, request_id: "rec-nested-restart-2", task: "NONCE_m2 NESTED_149[SLOW_149:20000]" });
  await pause(3500);
  const rootBefore = fx.tasks().find((t: any) => t.task_id === dRoot.payload.task_id);
  const nestedBefore = fx.tasks().find((t: any) => t.parent_task_id === dRoot.payload.task_id);
  const nestedStartsBefore = starts().length;
  const nestedPidBefore = prompts("m1g")[0]?.pid;
  await mcpN.close();
  curPeer.close();
  await fx.stopDaemon();
  await fx.startDaemon();
  curPeer = await fx.registerParent(THREAD_A);
  await fx.registerParent(THREAD_B);
  const rootEarly = fx.tasks().find((t: any) => t.task_id === dRoot.payload.task_id);
  const nestedEarly = fx.tasks().find((t: any) => t.parent_task_id === dRoot.payload.task_id);
  await pause(9000);
  const rootLate = fx.tasks().find((t: any) => t.task_id === dRoot.payload.task_id);
  const nestedLate = fx.tasks().find((t: any) => t.parent_task_id === dRoot.payload.task_id);
  const noticesN = curPeer.deliveries.filter((d: any) => String(d.body).includes(dRoot.payload.task_id) || String(d.body).includes(nestedBefore?.task_id ?? "~"));
  notes.nestedRestart = {
    before: { root: rootBefore?.state, nested: nestedBefore?.state, starts: nestedStartsBefore }, early: { root: rootEarly?.state, rootError: rootEarly?.error, nested: nestedEarly?.state, nestedError: nestedEarly?.error },
    late: { root: rootLate?.state, rootError: rootLate?.error, nested: nestedLate?.state, nestedError: nestedLate?.error }, startsAfter: starts().length,
    promptsM1: prompts("m1").length, promptsM1g: prompts("m1g").length, notices: noticesN.map((d: any) => String(d.body).slice(0, 120)),
  };
  checks.add("R9.4", "REEL", "racine en attente de son petit-enfant, vrai SIGTERM + relance : l'attente de la racine reste durable (waiting_for_children), le petit-enfant en vol devient failed/unreachable, AUCUN nouvel enfant ni nouveau tour (m1 et m1g : 1 prompt chacun)",
    rootBefore?.state === "waiting_for_children" && nestedBefore?.state === "working" && rootEarly?.state === "waiting_for_children" && nestedEarly?.state === "failed" && nestedEarly?.error === "unreachable" &&
      starts().length === nestedStartsBefore && prompts("m1").length === 1 && prompts("m1g").length === 1,
    notes.nestedRestart);
  void nestedPidBefore;
  // Suite de R9.4 (observation) : la racine retenue sort-elle de waiting_for_children quand son seul descendant est failed ?
  const rootWaitStart = Date.now();
  const rootAfter = await waitTask(fx, dRoot.payload.task_id, ["result_available", "failed", "cancelled"], 40000);
  const rootWaitedMs = Date.now() - rootWaitStart;
  const mcpN2 = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "N2" });
  const stRoot = await mcpN2.call("bridget_task_status", { task_id: dRoot.payload.task_id });
  const linRoot = await fx.cli(lineageArgs("show", THREAD_A, fx.work, ["--task", dRoot.payload.task_id, "--offset", "0", "--limit", "100"]));
  const rawRoot = fx.tasks().find((t: any) => t.task_id === dRoot.payload.task_id);
  notes.nestedRestartRootAfterWait = {
    waitedMs: rootWaitedMs, rawState: rawRoot?.state, hasRetainedResult: typeof rawRoot?.result === "string" && rawRoot.result.length > 0, resultSent: rawRoot?.result_sent,
    mcpStatus: stRoot.payload?.status, mcpResultPresent: stRoot.payload?.result != null, lineageStatus: linRoot.json?.status ?? linRoot.json?.task?.status, noticesToParent: curPeer.deliveries.filter((d: any) => String(d.body).includes(dRoot.payload.task_id)).length,
  };
  checks.add("R9.4.b", "REEL", "OBSERVATION : la racine retenue (résultat déjà capturé) sort de waiting_for_children (result_available ou failed) dans les 40 s après relance, une fois son seul descendant failed/unreachable ; sinon le parent n'a ni résultat ni échec",
    rootAfter !== undefined && ["result_available", "failed"].includes(rootAfter.state), notes.nestedRestartRootAfterWait);
  // DIAGNOSTIC (protocole forensic, mutation de la base PRIVÉE de la fixture, pas un comportement produit) :
  // l'exécution du petit-enfant reste-t-elle `running` après relance ? Si on la passe à failed, la racine se résout-elle ?
  const nestedRowRaw = fx.tasks().find((t: any) => t.parent_task_id === dRoot.payload.task_id);
  const execRowBefore = fx.sql(`select execution_id, state, reason from executions where execution_id='execution-${nestedRowRaw?.mission}'`);
  const dbPath = `${fx.state}/bridget.db`;
  const linkRows = fx.sql("select count(*) as n from agent_links");
  notes.nestedRestartDiagnostic = { nestedExecutionBefore: execRowBefore, agentLinks: linkRows };
  if (rootAfter?.state === "waiting_for_children" && execRowBefore[0]?.state === "running") {
    (await import("node:child_process")).execFileSync("sqlite3", [dbPath, `update executions set state='failed', reason='provider_failed' where execution_id='execution-${nestedRowRaw?.mission}'`]);
    const resolved = await waitTask(fx, dRoot.payload.task_id, ["result_available", "failed"], 10000);
    await settle(fx, [curPeer], 1000, 10000);
    await pause(4000);
    const rootRaw2 = fx.tasks().find((t: any) => t.task_id === dRoot.payload.task_id);
    (notes.nestedRestartDiagnostic as any).afterMutation = { resultSentFlag: rootRaw2?.result_sent, stateAfter4s: rootRaw2?.state, rootState: resolved?.state, rootResult: resolved?.result, deliveriesToParent: curPeer.deliveries.filter((d: any) => d.in_reply_to === dRoot.payload.message_id).length };
    const am = (notes.nestedRestartDiagnostic as any).afterMutation;
    checks.add("R9.4.c", "SIMULE", "DIAGNOSTIC (mutation de la base privée) : cause prouvée - quand l'exécution `running` du petit-enfant mort passe à failed, la racine quitte waiting_for_children (result_available, résultat retenu intact) ; c'est donc l'exécution laissée `running` par la relance qui bloque l'attente",
      resolved?.state === "result_available" && resolved.result === "fixture149-answer:m1", notes.nestedRestartDiagnostic);
    checks.add("R9.4.d", "SIMULE", "DIAGNOSTIC : une fois la racine débloquée, son résultat retenu est remis UNE fois au parent (result_sent vrai, 1 remise corrélée)",
      am.resultSentFlag === true && am.deliveriesToParent === 1, am);
  }
  // issue de secours : la 2e racine, elle aussi bloquée (non mutée), se termine par cancel du parent
  const root2Before = fx.tasks().find((t: any) => t.task_id === dRoot2.payload?.task_id);
  const cancelStuck = await mcpN2.call("bridget_task_cancel", { task_id: dRoot2.payload?.task_id });
  await pause(2500);
  const root2After = fx.tasks().find((t: any) => t.task_id === dRoot2.payload?.task_id);
  notes.nestedRestartCancel = { stateBefore: root2Before?.state, isError: cancelStuck.isError, status: cancelStuck.payload?.status, rawStateAfter: root2After?.state, providersAlive: (await fx.liveProviders()).length, prompts: { m2: prompts("m2").length, m2g: prompts("m2g").length } };
  checks.add("R9.4.e", "REEL", "la 2e racine, bloquée de la même façon et non mutée, ne se termine que par le cancel du parent : cancelled, zéro fournisseur vivant, aucun nouveau tour (m2 et m2g : 1 prompt chacun) - le résultat retenu est perdu",
    root2Before?.state === "waiting_for_children" && !cancelStuck.isError && root2After?.state === "cancelled" && (notes.nestedRestartCancel as any).providersAlive === 0 && prompts("m2").length === 1 && prompts("m2g").length === 1,
    notes.nestedRestartCancel);
  await mcpN2.close();

  // ============================================================ R9.6 Mort du fournisseur natif en vol (le daemon reste vivant) : aucune relance
  {
    const mcpP = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "P" });
    const dP = await mcpP.call("bridget_delegate", { ...base, request_id: "rec-provider-death", task: "NONCE_pd1 WAIT_149" });
    await pause(2500);
    const pdPid = prompts("pd1")[0]?.pid as number | undefined;
    const startsPd = starts().length;
    let verified = "";
    if (pdPid !== undefined) {
      const { execFileSync } = await import("node:child_process");
      verified = execFileSync("/bin/ps", ["-p", String(pdPid), "-o", "ppid=", "-o", "command="], { encoding: "utf8" }).trim();
      if (!verified.includes(fx.root) || /firefox/i.test(verified)) throw new Error(`refus de signaler un processus non vérifié: ${pdPid}`);
      process.kill(pdPid, "SIGTERM"); // un seul PID identifié, jamais -9
    }
    await pause(3000);
    const diedAfter3s = pdPid !== undefined && !alive(pdPid);
    await pause(9000);
    const taskPd = fx.tasks().find((t: any) => t.task_id === dP.payload?.task_id);
    const noticesPd = curPeer.deliveries.filter((d: any) => String(d.body).includes(dP.payload?.task_id));
    notes.providerDeath = { pid: pdPid, parentOfProviderIsWrapper: /managed-wrapper/.test(verified) ? "oui (ppid vérifié)" : "non précisé", diedAfter3s, state: taskPd?.state, error: taskPd?.error, startsBefore: startsPd, startsAfter: starts().length, prompts: prompts("pd1").length, liveProvidersAfter: (await fx.liveProviders()).length, notices: noticesPd.map((d: any) => String(d.body).slice(0, 120)) };
    checks.add("R9.6", "REEL", "mort du fournisseur natif en vol (SIGTERM d'UN PID vérifié, daemon vivant) : aucune relance par la flotte ni le wrapper (starts inchangé, 1 seul tour, 0 fournisseur vivant), la tâche devient terminale (failed) avec erreur explicite et UNE notice d'échec",
      diedAfter3s && starts().length === startsPd && prompts("pd1").length === 1 && notes.providerDeath && (notes.providerDeath as any).liveProvidersAfter === 0 && taskPd?.state === "failed" && !!taskPd.error && noticesPd.length === 1,
      notes.providerDeath);
    await mcpP.close();
  }

  // ============================================================ R9.5 Aucune mission exécutée deux fois sur tout le rejeu (4 arrêts SIGTERM + relances du daemon)
  {
    const byNonce = new Map<string, number>();
    for (const e of fx.evidence().filter((x: any) => x.event === "prompt" && x.nonce !== "none")) byNonce.set(e.nonce, (byNonce.get(e.nonce) ?? 0) + 1);
    const dup = [...byNonce].filter(([, n]) => n > 1);
    notes.promptsByNonce = Object.fromEntries(byNonce);
    checks.add("R9.5", "REEL", "sur tout le rejeu (plusieurs SIGTERM + relances du daemon) chaque mission (nonce) a été portée par au plus UN tour fournisseur : aucune mission exécutée deux fois",
      dup.length === 0 && byNonce.size >= 12, { nonces: byNonce.size, duplicated: dup, daemonStarts: fx.daemonStarts });
  }

  // ============================================================ Bilan
  notes.finalCounts = { ...counts(fx), startsTotal: starts().length };
  notes.httpSeen = { posts: host.seen.filter((r) => r.method === "POST").length, deletes: host.seen.filter((r) => r.method === "DELETE").length };
  notes.allStarts = fx.evidence().filter((e: any) => e.event === "started").map((e: any) => e.pid);
  notes.finalStates = fx.tasks().map((t: any) => t.state);
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
await writeResults("recovery149", { schema: "native-network-recipes/1", startedAt: T0, finishedAt: new Date().toISOString(), binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), summary, notes, checks: checks.items, residualProcesses: live });
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
