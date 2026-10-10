// T036 - Interop réseau native 149 : T3 réel (MCP/HTTP/registre/faits de permissions) <-> daemon Bridget
// 149 réel (binaire debug fixe) + clients `bridget mcp` réels + CLI `bridget lineage` réelle.
// Aucun modèle. Enfant = serveur app-server Codex FERMÉ (codex149.py), PID et lancements comptés.
// Usage : node interop149.ts   (Node 24, umask 077, cwd quelconque)
import * as OS from "node:os";
import * as FS from "node:fs";
import * as Path from "node:path";
import * as ChildProcess from "node:child_process";
import { startT3Host, codexFullAccessFact } from "./t3host.ts";
import { Fx, Checks, pause, BIN, BIN_SHA256, sha256File, lineageArgs, lineageWatch, writeResults, freePort } from "./fx.mjs";
import { THREAD_A, THREAD_B, THREAD_C, THREAD_D, rawSession, rawStatus, rawTool, readOnlyFact, waitTask, counts, sha, settle } from "./common149.ts";

const PORT = 14796;
const checks = new Checks("T036 interop 149");
const T0 = new Date().toISOString();
const notes: Record<string, unknown> = {};
if (sha256File(BIN) !== BIN_SHA256) throw new Error("binaire debug 149 : empreinte différente du reçu");
if (!(await freePort(PORT))) throw new Error(`port ${PORT} occupé`);

const host = await startT3Host(PORT);
const fx = await Fx.create("interop149", { port: PORT });
notes.fixtureRoot = fx.root;
notes.projectRoot = fx.work;
let exitCode = 0;
try {
  await fx.writeRegistry();
  await fx.publishRuntime();
  const daemonPid = await fx.startDaemon();
  notes.daemonPid = daemonPid;

  // ---------------------------------------------------------------- Phase 1 : deux sessions authentifiées
  const peerA = await fx.registerParent(THREAD_A);
  const peerB = await fx.registerParent(THREAD_B);
  const peers = { A: peerA, B: peerB };
  const a1 = await host.issue(THREAD_A);
  const b1 = await host.issue(THREAD_B);
  let cA = a1.config;
  const cB = b1.config;
  const runA = host.startRun(THREAD_A, "run-a-1");
  const runB = host.startRun(THREAD_B, "run-b-1");
  checks.add("S1.1", "REEL", "deux credentials distincts, même endpoint et même instance fournisseur, sessions fournisseur distinctes",
    cA.endpoint === cB.endpoint && cA.providerInstanceId === cB.providerInstanceId && cA.providerSessionId !== cB.providerSessionId && cA.authorizationHeader !== cB.authorizationHeader,
    { sameEndpoint: cA.endpoint === cB.endpoint, sameInstance: cA.providerInstanceId === cB.providerInstanceId });
  checks.add("S1.2", "REEL", "les deux agents Bridget dérivent de leur fil et diffèrent",
    peerA.agentId !== peerB.agentId, { a: peerA.agentId.slice(0, 8), b: peerB.agentId.slice(0, 8) });

  const v1a = await rawSession(cA);
  const v1keys = Object.keys(v1a.structured ?? {}).sort();
  checks.add("S1.3", "REEL", "bridget_session sans fait : enveloppe v1 identité seule (5 champs camelCase, pas de permissions)",
    v1a.structured?.version === 1 && JSON.stringify(v1keys) === JSON.stringify(["environmentId", "providerInstanceId", "providerSessionId", "threadId", "version"]) &&
      v1a.structured.threadId === THREAD_A && v1a.structured.providerSessionId === cA.providerSessionId, { keys: v1keys, version: v1a.structured?.version });

  let mcpA = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "A" });
  const mcpB = await fx.mcp({ endpoint: cB.endpoint, authorization: cB.authorizationHeader, label: "B" });
  const whoA = await mcpA.call("bridget_who", { scope: "global" });
  const whoB = await mcpB.call("bridget_who", { scope: "global" });
  checks.add("S1.4", "REEL", "bridget_who via T3 réel résout A et B sur leurs propres agents (identité148)",
    !whoA.isError && !whoB.isError && JSON.stringify(whoA.payload).includes(peerA.agentId) && JSON.stringify(whoB.payload).includes(peerB.agentId), { aErr: whoA.isError, bErr: whoB.isError });
  const posts = host.seen.filter((r) => r.method === "POST").length;
  const deletes = host.seen.filter((r) => r.method === "DELETE").length;
  checks.add("S1.5", "REEL", "chaque attestation ouvre puis ferme sa session de transport HTTP (DELETE par session, statuts 2xx)",
    deletes >= 2 && host.seen.every((r) => r.status >= 200 && r.status < 300), { posts, deletes, statuses: [...new Set(host.seen.map((r) => r.status))] });

  // ---------------------------------------------------------------- Phase 2 : v1 ne donne aucun droit d'écriture 149
  const before1 = counts(fx, peers);
  const d1 = await mcpA.call("bridget_delegate", { request_id: "interop-v1-inherit", agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", task: "NONCE_v1a écrire", cwd: fx.work });
  checks.add("S2.1", "REEL", "G-P-07(c) : admission inherit avec la seule enveloppe v1 -> permission_attestation_unavailable",
    d1.isError && d1.payload?.text === "permission_attestation_unavailable", { text: d1.payload?.text, code: d1.code });
  const d1b = await mcpA.call("bridget_delegate", { request_id: "interop-v1-dev", agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", task: "NONCE_v1b écrire", cwd: fx.work, posture: "development" });
  const after1 = counts(fx, peers);
  checks.add("S2.2", "REEL", "v1 + posture development : même refus nommé, aucune ligne, aucun lancement, aucune remise, aucun grant",
    d1b.isError && d1b.payload?.text === "permission_attestation_unavailable" && after1.rows === before1.rows && after1.started === 0 && after1.deliveries.A === 0,
    { text: d1b.payload?.text, before: before1, after: after1 });
  const stUnknown = await mcpA.call("bridget_task_status", { task_id: "14900000-0000-4000-8000-0000000000aa" });
  checks.add("S2.3", "REEL", "status/identité148 restent fonctionnels en v1 (tâche inconnue -> task_unavailable, pas de crash)",
    stUnknown.isError && /task_unavailable/.test(JSON.stringify(stUnknown.payload)), { payload: stUnknown.payload });
  const capV1 = await mcpA.call("bridget_capabilities");
  const codexCapV1 = (capV1.payload?.providers ?? []).find((p: any) => p.agent_type === "fixture-codex-149");
  notes.catalogueV1 = { inherit: codexCapV1?.inherit, inherit_refusal: codexCapV1?.inherit_refusal, development: codexCapV1?.development, development_refusal: codexCapV1?.development_refusal };

  // ---------------------------------------------------------------- Phase 3 : fait v2 publié par l'adaptateur (simulé), consommé pour de vrai
  const revA = host.publish(cA, codexFullAccessFact(cA, "run-a-1", fx.work), runA);
  const v2a = await rawSession(cA);
  const v2keys = Object.keys(v2a.structured ?? {}).sort();
  const permKeys = Object.keys(v2a.structured?.permissions ?? {}).sort();
  checks.add("S3.1", "REEL", "bridget_session avec fait publié : enveloppe v2 stricte (identité camelCase + permissions snake_case, revision 1)",
    v2a.structured?.version === 2 && revA === 1 && v2a.structured.permissions.revision === 1 && v2a.structured.permissions.provider_session_id === cA.providerSessionId &&
      v2a.structured.permissions.run_id === "run-a-1" && v2a.structured.permissions.provider_policy.sandbox_policy.type === "dangerFullAccess",
    { v2keys, permKeys, revision: v2a.structured?.permissions?.revision });
  const serialized = JSON.stringify(v2a);
  checks.add("S3.2", "REEL", "l'enveloppe v2 ne contient ni token, ni endpoint, ni en-tête d'autorisation (<= 64 KiB)",
    !serialized.includes(cA.authorizationHeader.slice(7)) && !serialized.includes("Bearer") && !serialized.includes(cA.endpoint) && (v2a.bytes ?? 0) < 65536, { bytes: v2a.bytes });
  const v1b = await rawSession(cB);
  checks.add("S3.3", "REEL", "le fait de A n'est jamais prêté à B : B (même processus fournisseur) reste en v1",
    v1b.structured?.version === 1 && v1b.structured.threadId === THREAD_B, { version: v1b.structured?.version });

  // admission A : inherit
  const reqA = { request_id: "interop-a-1", agent_type: "fixture-codex-149", model: "fixture-model-149", effort: "high", task: "NONCE_a1 écrire le fichier de recette", cwd: fx.work };
  const dA = await mcpA.call("bridget_delegate", reqA);
  checks.add("S3.4", "REEL", "A (v2 full-access) : bridget_delegate sans posture admis, posture effective development, 1 ligne durable",
    !dA.isError && dA.payload?.posture === "development" && dA.payload?.status === "queued", { status: dA.payload?.status, posture: dA.payload?.posture, err: dA.payload?.text });
  const capV2 = await mcpA.call("bridget_capabilities");
  notes.catalogueV2FullAccessCodexParent = Object.fromEntries((capV2.payload?.providers ?? []).filter((p: any) => ["fixture-codex-149", "fixture-claude-149", "codex", "claude"].includes(p.agent_type)).map((p: any) => [p.agent_type, { protocol: p.protocol, inherit: p.inherit, inherit_refusal: p.inherit_refusal, development: p.development, development_refusal: p.development_refusal, discovery: p.discovery }]));
  notes.catalogueScope = { cwd_scope: capV2.payload?.cwd_scope, max_depth: capV2.payload?.max_depth, max_children: capV2.payload?.max_children };
  const taskA = dA.payload.task_id as string;
  const doneA = await waitTask(fx, taskA, ["result_available"]);
  await pause(400);
  const evA = fx.evidence();
  const promptA = evA.find((e: any) => e.event === "prompt");
  checks.add("S3.5", "REEL", "le moteur a lancé UN processus enfant réel et UN tour ; résultat corrélé disponible",
    doneA?.state === "result_available" && evA.filter((e: any) => e.event === "started").length === 1 && evA.filter((e: any) => e.event === "prompt").length === 1 && doneA.result === "fixture149-answer:a1",
    { state: doneA?.state, pid: evA.find((e: any) => e.event === "started")?.pid, result: doneA?.result });
  checks.add("S3.6", "REEL", "la politique de l'enfant est celle du fait parent (dangerFullAccess + approval never), figée dans le snapshot",
    promptA?.sandbox_policy?.type === "dangerFullAccess" && promptA?.approval === "never" && doneA?.permission_snapshot !== undefined,
    { sandbox: promptA?.sandbox_policy, approval: promptA?.approval, snapshotKeys: Object.keys(doneA?.permission_snapshot ?? {}) });
  const dlvA = peerA.deliveries.filter((d: any) => d.in_reply_to === dA.payload.message_id);
  checks.add("S3.7", "REEL", "exactement une remise corrélée (in_reply_to = mission, from = enfant, to = A), ACK réel, rien chez B",
    dlvA.length === 1 && dlvA[0].from === dA.payload.child_agent_id && dlvA[0].to === peerA.agentId && peerA.acks.length >= 1 && peerB.deliveries.length === 0,
    { aDeliveries: peerA.deliveries.length, matching: dlvA.length, bDeliveries: peerB.deliveries.length, acks: peerA.acks.length });
  const snapshotA = sha(doneA?.permission_snapshot);

  // B : même processus fournisseur, droits différents, sans emprunt
  const dB0 = await mcpB.call("bridget_delegate", { ...reqA, request_id: "interop-b-0" });
  checks.add("S3.8", "REEL", "B sans fait : refus nommé, A inchangé (aucun emprunt du fait de A)",
    dB0.isError && dB0.payload?.text === "permission_attestation_unavailable" && counts(fx).rows === 1, { text: dB0.payload?.text, rows: counts(fx).rows });
  const revB = host.publish(cB, readOnlyFact(cB, "run-b-1", fx.work), runB);
  const v2b = await rawSession(cB);
  const dBdev = await mcpB.call("bridget_delegate", { ...reqA, request_id: "interop-b-dev", posture: "development", task: "NONCE_b0 écrire" });
  checks.add("S3.9", "REEL", "S149-02/25 : B lecteur (sandbox readOnly) demande development -> permission_not_inherited, aucune ligne, aucun lancement, aucune invitation au grant",
    v2b.structured?.version === 2 && dBdev.isError && dBdev.payload?.text === "permission_not_inherited" && counts(fx).rows === 1 && counts(fx).started === 1 && !/grant/i.test(JSON.stringify(dBdev.payload)),
    { text: dBdev.payload?.text, rows: counts(fx).rows, started: counts(fx).started, revB });
  const dB1 = await mcpB.call("bridget_delegate", { ...reqA, request_id: "interop-b-1", task: "NONCE_b1 lire seulement" });
  const taskB = dB1.payload?.task_id as string | undefined;
  if (taskB) await waitTask(fx, taskB, ["result_available", "failed"]);
  await pause(300);
  const promptsB = fx.evidence().filter((e: any) => e.event === "prompt" && e.nonce === "b1");
  checks.add("S3.10", "REEL", "B (readOnly) sans posture : admis en lecture ; l'enfant de B reçoit un sandbox readOnly, celui de A est resté dangerFullAccess (S149-25)",
    !dB1.isError && dB1.payload?.posture === "discovery" && promptsB[0]?.sandbox_policy?.type === "readOnly",
    { posture: dB1.payload?.posture, sandboxB: promptsB[0]?.sandbox_policy, err: dB1.payload?.text });
  notes.snapshots = { A: snapshotA, B: sha(fx.tasks().find((t: any) => t.task_id === taskB)?.permission_snapshot) };

  // identité étrangère
  const bStatusA = await mcpB.call("bridget_task_status", { task_id: taskA });
  const bCancelA = await mcpB.call("bridget_task_cancel", { task_id: taskA });
  checks.add("S3.11", "REEL", "B ne lit ni n'annule la tâche de A (task_unavailable, aucune fuite de champ)",
    bStatusA.isError && bCancelA.isError && /task_unavailable/.test(JSON.stringify(bStatusA.payload)) && /task_unavailable/.test(JSON.stringify(bCancelA.payload)) && !JSON.stringify(bStatusA.payload).includes("fixture149-answer"),
    { status: bStatusA.payload, cancel: bCancelA.payload });
  const rows3 = counts(fx).rows;
  const bReplayA = await mcpB.call("bridget_delegate", { ...reqA, request_id: "interop-a-1" });
  checks.add("S3.12", "REEL", "B rejoue le request_id de A : jamais la tâche de A (soit refus, soit tâche distincte propre à B)",
    bReplayA.isError || (bReplayA.payload?.task_id !== taskA && bReplayA.payload?.child_agent_id !== dA.payload.child_agent_id),
    { isError: bReplayA.isError, sameTask: bReplayA.payload?.task_id === taskA, text: bReplayA.payload?.text, rowsBefore: rows3, rowsAfter: counts(fx).rows });

  // ---------------------------------------------------------------- Phase 4 : rotation et révocation
  const oldA = cA;
  const a2 = await host.rotate(THREAD_A);
  cA = a2.config;
  const oldStatus = await rawStatus(oldA);
  const oldWho = await mcpA.call("bridget_who", { scope: "global" });
  checks.add("S4.1", "REEL", "rotation : l'ancien credential est refusé par T3 (HTTP 401) et par Rust (t3_session_unavailable), sans repli",
    oldStatus === 401 && oldWho.isError && oldWho.code === "t3_session_unavailable", { http: oldStatus, code: oldWho.code });
  await mcpA.close();
  mcpA = await fx.mcp({ endpoint: cA.endpoint, authorization: cA.authorizationHeader, label: "A2" });
  const whoA2 = await mcpA.call("bridget_who", { scope: "global" });
  const v1a2 = await rawSession(cA);
  checks.add("S4.2", "REEL", "le nouveau credential retrouve la même identité Bridget (stable) mais aucun fait hérité : v1",
    !whoA2.isError && JSON.stringify(whoA2.payload).includes(peerA.agentId) && v1a2.structured?.version === 1 && cA.providerSessionId !== oldA.providerSessionId,
    { version: v1a2.structured?.version, sameAgent: JSON.stringify(whoA2.payload).includes(peerA.agentId) });
  const refusedAfterRotation = await mcpA.call("bridget_delegate", { ...reqA, request_id: "interop-a-rot", task: "NONCE_rot" });
  const stA2 = await mcpA.call("bridget_task_status", { task_id: taskA });
  checks.add("S4.3", "REEL", "après rotation sans fait : nouvelle admission refusée (v1) mais lecture de la tâche admise intacte",
    refusedAfterRotation.isError && refusedAfterRotation.payload?.text === "permission_attestation_unavailable" && stA2.payload?.status === "result_available" && stA2.payload?.result === "fixture149-answer:a1",
    { refusal: refusedAfterRotation.payload?.text, status: stA2.payload?.status });
  // révocation de B
  await host.revokeThread(THREAD_B);
  const bStatus = await rawStatus(cB);
  const whoBr = await mcpB.call("bridget_who", { scope: "global" });
  const whoA2b = await mcpA.call("bridget_who", { scope: "global" });
  checks.add("S4.4", "REEL", "révocation de B : HTTP 401 + t3_session_unavailable ; le voisin A (nouveau credential) reste servi",
    bStatus === 401 && whoBr.isError && whoBr.code === "t3_session_unavailable" && !whoA2b.isError, { http: bStatus, code: whoBr.code, neighborOk: !whoA2b.isError });
  // identité fausse / absente
  const forged = { ...cA, authorizationHeader: `Bearer ${"x".repeat(43)}` };
  const mcpForged = await fx.mcp({ endpoint: cA.endpoint, authorization: forged.authorizationHeader, label: "forged" });
  const whoForged = await mcpForged.call("bridget_who", { scope: "global" });
  const wrongPort = await fx.mcp({ endpoint: "http://127.0.0.1:1/mcp", authorization: cA.authorizationHeader, label: "wrongport" });
  const whoWrongPort = await wrongPort.call("bridget_who", { scope: "global" });
  const rowsBeforeForged = counts(fx).rows;
  const cC = (await host.issue(THREAD_C)).config; // credential T3 valide, aucun wrapper Bridget enregistré pour ce fil
  const mcpC = await fx.mcp({ endpoint: cC.endpoint, authorization: cC.authorizationHeader, label: "C" });
  const whoC = await mcpC.call("bridget_who", { scope: "global" });
  const dC = await mcpC.call("bridget_delegate", { ...reqA, request_id: "interop-c-1" });
  checks.add("S4.5", "REEL", "identités fausses : token forgé, endpoint hors runtime, credential valide sans binding Bridget -> t3_session_unavailable, jamais de repli PID",
    [whoForged, whoWrongPort, whoC].every((r) => r.isError && r.code === "t3_session_unavailable") && dC.isError && counts(fx).rows === rowsBeforeForged,
    { forged: whoForged.code, wrongPort: whoWrongPort.code, unboundC: whoC.code, delegateC: dC.code ?? dC.payload?.text });
  for (const m of [mcpForged, wrongPort, mcpC]) await m.close();

  // ---------------------------------------------------------------- Phase 5 : cycle de vie du fait (A, credential courant)
  const runA2 = host.startRun(THREAD_A, "run-a-2");
  const revA2 = host.publish(cA, codexFullAccessFact(cA, "run-a-2", fx.work), runA2);
  host.endRun(THREAD_A, cA, "run-a-2");
  const tomb = await rawSession(cA);
  const dTomb = await mcpA.call("bridget_delegate", { ...reqA, request_id: "interop-a-tomb", task: "NONCE_tomb" });
  const stTomb = await mcpA.call("bridget_task_status", { task_id: taskA });
  checks.add("S5.1", "REEL", "G-P-07(b) côté T3 : fait retiré (fin de run) -> refus nommé permission_attestation_unavailable, jamais l'enveloppe v1",
    tomb.isError && tomb.namedCode === "permission_attestation_unavailable" && tomb.structured === undefined, { rawIsError: tomb.isError, namedCode: tomb.namedCode, hasStructured: tomb.structured !== undefined });
  const rowsTomb = counts(fx);
  checks.add("S5.1b", "REEL", "côté Rust (montage MCP T3) : même credential -> admission refusée fermée, zéro ligne, zéro lancement",
    dTomb.isError && counts(fx).rows === rowsTomb.rows && counts(fx).started === rowsTomb.started, { delegate: dTomb.code ?? dTomb.payload?.text, status: stTomb.code ?? stTomb.payload?.status });
  notes.observationTombstoneRustFront = { delegateCode: dTomb.code, delegateText: String(dTomb.payload?.text).slice(0, 60), statusOfAdmittedTaskThroughT3MountedMcp: stTomb.payload?.status ?? ("refused:" + (stTomb.code ?? stTomb.payload?.text)) };
  const runA3 = host.startRun(THREAD_A, "run-a-3");
  const rev3 = host.publish(cA, codexFullAccessFact(cA, "run-a-3", fx.work), runA3);
  const stale = host.publish(cA, codexFullAccessFact(cA, "run-OTHER", fx.work), runA3);
  const staleSession = await rawSession(cA);
  checks.add("S5.2", "REEL", "fait lié à un autre run que le run actif : refus nommé ; la revision a augmenté à chaque publication du même credential",
    staleSession.isError && staleSession.namedCode === "permission_attestation_unavailable" && rev3 === 2 && stale === 3, { rev3, stale, code: staleSession.namedCode });
  host.publish(cA, codexFullAccessFact(cA, "run-a-3", fx.work, { interaction_mode: "plan" }), runA3);
  const planned = await rawSession(cA);
  const snapshotAAfter = sha(fx.tasks().find((t: any) => t.task_id === taskA)?.permission_snapshot);
  checks.add("S5.3", "REEL", "changement de mode (plan) : nouvelle revision pour les futures admissions ; le snapshot de la tâche déjà admise est inchangé (invariant 5)",
    planned.structured?.permissions?.interaction_mode === "plan" && planned.structured.permissions.revision === 4 && snapshotAAfter === snapshotA, { revision: planned.structured?.permissions?.revision, snapshotUnchanged: snapshotAAfter === snapshotA });
  host.publish(cA, codexFullAccessFact(cA, "run-a-3", fx.work), runA3);

  // ---------------------------------------------------------------- Phase 6 : lecteur Lineage (S149-19), deux conversations dans le MÊME projet
  const listA = await fx.cli(lineageArgs("list", THREAD_A, fx.work));
  const listB = await fx.cli(lineageArgs("list", THREAD_B, fx.work));
  const idsA = (listA.json?.tasks ?? []).map((t: any) => t.task_id);
  const idsB = (listB.json?.tasks ?? []).map((t: any) => t.task_id);
  checks.add("S6.1", "REEL", "S149-19 : A voit seulement sa tâche, B (même projet, même processus) seulement la sienne ; aucune ne voit l'autre",
    listA.code === 0 && listB.code === 0 && idsA.includes(taskA) && !idsA.includes(taskB) && idsB.includes(taskB as string) && !idsB.includes(taskA), { a: idsA.length, b: idsB.length });
  const showCross = await fx.cli(lineageArgs("show", THREAD_B, fx.work, ["--task", taskA, "--offset", "0", "--limit", "100"]));
  const jrnCross = await fx.cli(lineageArgs("journal", THREAD_B, fx.work, ["--task", taskA, "--after-seq", "0", "--limit", "10"]));
  const cancelCross = await fx.cli(lineageArgs("cancel", THREAD_B, fx.work, ["--task", taskA, "--request-id", "49000000-0000-4000-8000-0000000000b1"]));
  checks.add("S6.2", "REEL", "B ne peut ni show, ni journal, ni cancel la tâche de A via la CLI lecteur (task_unavailable, exit 2)",
    [showCross, jrnCross, cancelCross].every((r) => r.code === 2 && r.json?.code === "task_unavailable"), { show: showCross.json?.code, journal: jrnCross.json?.code, cancel: cancelCross.json?.code });
  const wrongRoot = await fx.cli(lineageArgs("list", THREAD_A, "/tmp"));
  const unknownThread = await fx.cli(lineageArgs("list", "89000000-0000-4000-8000-0000000000ff", fx.work));
  const notBound = await fx.cli(lineageArgs("list", THREAD_C, fx.work));
  checks.add("S6.3", "REEL", "mauvaise racine -> project_mismatch ; fil inconnu et fil sans binding -> binding_unavailable ; aucune donnée rendue",
    wrongRoot.json?.code === "project_mismatch" && unknownThread.json?.code === "binding_unavailable" && notBound.json?.code === "binding_unavailable" && !JSON.stringify([wrongRoot.json, unknownThread.json, notBound.json]).includes(taskA),
    { wrongRoot: wrongRoot.json?.code, unknown: unknownThread.json?.code, unbound: notBound.json?.code });
  const showA = await fx.cli(lineageArgs("show", THREAD_A, fx.work, ["--task", taskA, "--offset", "0", "--limit", "100"]));
  checks.add("S6.4", "REEL", "show de la tâche terminale par son propre fil : résultat exact servi",
    showA.code === 0 && showA.json?.result === "fixture149-answer:a1", { code: showA.code, result: showA.json?.result });

  // Lecture silencieuse : 100 list + 100 show sans aucune mutation, aucun lancement, aucune remise
  const settled = await settle(fx, [peerA, peerB]);
  notes.settledBeforeQuiet = settled;
  const quietBefore = { ...counts(fx, peers), db: sha(fx.sql("select task_id,payload from native_delegations order by task_id")) };
  for (let i = 0; i < 100; i += 1) {
    await fx.cli(lineageArgs("list", THREAD_A, fx.work));
    if (i % 2 === 0) await fx.cli(lineageArgs("show", THREAD_A, fx.work, ["--task", taskA, "--offset", "0", "--limit", "100"]));
  }
  const quietAfter = { ...counts(fx, peers), db: sha(fx.sql("select task_id,payload from native_delegations order by task_id")) };
  checks.add("S6.5", "REEL", "150 lectures Lineage : aucun lancement, aucune remise, aucun tour, base byte-stable (lecture quiète, aucun provider turn)",
    JSON.stringify(quietBefore) === JSON.stringify(quietAfter), { quietBefore, quietAfter });

  // Trois tâches de plus pour A afin de paginer réellement (4 tâches au total)
  for (const n of [1, 2, 3]) {
    const d = await mcpA.call("bridget_delegate", { ...reqA, request_id: `interop-a-pg${n}`, task: `NONCE_pg${n} page` });
    await waitTask(fx, d.payload.task_id, ["result_available", "failed"]);
  }
  await settle(fx, [peerA, peerB]);
  // Pagination en snapshot : pages de 1 tâche, même génération/séquence, pas de doublon ; une mutation entre deux pages abandonne le snapshot
  const fullList = await fx.cli(lineageArgs("list", THREAD_A, fx.work));
  const pages: any[] = [];
  let cursor: string | null = null;
  for (let i = 0; i < 20; i += 1) {
    const page: any = await fx.cli(lineageArgs("list", THREAD_A, fx.work, ["--limit", "1", ...(cursor ? ["--cursor", cursor] : [])]));
    pages.push(page.json);
    cursor = page.json?.next_cursor ?? null;
    if (!cursor) break;
  }
  const pagedIds = pages.flatMap((p) => (p?.tasks ?? []).map((t: any) => t.task_id));
  checks.add("S6.9", "REEL", "pagination en snapshot : pages de 1 tâche, même génération et même seq, union = liste complète, aucun doublon",
    pages.length === (fullList.json?.tasks ?? []).length && pages.length >= 4 && new Set(pagedIds).size === pagedIds.length && JSON.stringify([...pagedIds].sort()) === JSON.stringify((fullList.json?.tasks ?? []).map((t: any) => t.task_id).sort()) && pages.every((p) => p.seq === fullList.json.seq && p.generation === fullList.json.generation),
    { pages: pages.length, tasks: fullList.json?.tasks?.length, seq: fullList.json?.seq });
  const page1: any = await fx.cli(lineageArgs("list", THREAD_A, fx.work, ["--limit", "1"]));
  const mutate = await mcpA.call("bridget_delegate", { ...reqA, request_id: "interop-a-snap", task: "NONCE_sn1" });
  const page2: any = await fx.cli(lineageArgs("list", THREAD_A, fx.work, ["--limit", "1", "--cursor", page1.json?.next_cursor ?? "x"]));
  checks.add("S6.10", "REEL", "mutation entre deux pages : page 2 rend snapshot_changed (le staging est à abandonner), jamais un snapshot mélangé",
    !mutate.isError && page1.json?.next_cursor && page2.json?.code === "snapshot_changed", { mutated: !mutate.isError, page2: page2.json?.code ?? page2.json?.status });
  await waitTask(fx, mutate.payload.task_id, ["result_available", "failed"]);
  const grammar = [
    ["lineage", "inspect"], ["lineage", "inspect", "--json", "--t3-thread", THREAD_A, "--project-root", fx.work, "--action", "list", "--limit", "0"],
    ["lineage", "inspect", "--json", "--t3-thread", THREAD_A, "--project-root", "relatif", "--action", "list"],
    ["lineage", "inspect", "--json", "--t3-thread", THREAD_A, "--project-root", fx.work, "--action", "list", "--inconnu", "x"],
    ["lineage", "inspect", "--json", "--t3-thread", THREAD_A, "--project-root", fx.work, "--action", "show", "--task", "pas-un-uuid"],
    ["lineage", "inspect", "--json", "--t3-thread", THREAD_A, "--project-root", fx.work, "--action", "list", "--task", taskA],
  ];
  const grammarResults = [];
  for (const g of grammar) grammarResults.push(await fx.cli(g));
  checks.add("S6.11", "REEL", "grammaire CLI fermée : formes ouvertes ou hors bornes -> exit 2, invalid_request, retryable false",
    grammarResults.every((r) => r.code === 2 && r.json?.code === "invalid_request" && r.json?.retryable === false), { codes: grammarResults.map((r) => [r.code, r.json?.code]) });

  // Watch scopé : A reçoit le signal d'une mutation de sa racine, B reste silencieux
  const watchA = lineageWatch(fx, THREAD_A);
  const watchB = lineageWatch(fx, THREAD_B);
  const readyA = await watchA.waitLines(1);
  const readyB = await watchB.waitLines(1);
  checks.add("S6.6", "REEL", "watch : ready seq 0 premier pour chaque racine",
    readyA && readyB && watchA.lines[0].status === "ready" && watchA.lines[0].seq === 0 && watchB.lines[0].status === "ready", { a: watchA.lines[0], b: watchB.lines[0] });
  const dWait = await mcpA.call("bridget_delegate", { ...reqA, request_id: "interop-a-wait", task: "NONCE_w1 WAIT_149" });
  const taskW = dWait.payload?.task_id as string | undefined;
  await pause(2500);
  const aSignals = watchA.lines.length;
  const bSignals = watchB.lines.length;
  checks.add("S6.7", "REEL", "mutation de la racine de A (nouvelle tâche) : A est signalé (corps absent), B ne reçoit rien (aucun UUID de fil ni contenu)",
    !dWait.isError && aSignals > 1 && bSignals === 1 && !JSON.stringify(watchA.lines.slice(1)).includes(taskW ?? "x"), { aLines: watchA.lines.map((l: any) => l.status), bLines: bSignals });
  notes.watchLinesA = watchA.lines.map((l: any) => ({ status: l.status, seq: l.seq, keys: Object.keys(l) }));
  const cancelReq = "49000000-0000-4000-8000-0000000000c1";
  const cx1 = await fx.cli(lineageArgs("cancel", THREAD_A, fx.work, ["--task", taskW!, "--request-id", cancelReq]));
  const cx2 = await fx.cli(lineageArgs("cancel", THREAD_A, fx.work, ["--task", taskW!, "--request-id", cancelReq]));
  const cx3 = await fx.cli(lineageArgs("cancel", THREAD_A, fx.work, ["--task", taskA, "--request-id", cancelReq]));
  const cancelled = await waitTask(fx, taskW!, ["cancelled"]);
  checks.add("S6.8", "REEL", "cancel CLI : reçu natif, rejeu identique = même reçu, même request_id sur une autre tâche = envelope_mismatch, tâche réellement cancelled",
    cx1.code === 0 && JSON.stringify(cx1.json) === JSON.stringify(cx2.json) && cx3.json?.code === "envelope_mismatch" && cancelled?.state === "cancelled", { first: cx1.json, replay: cx2.json, mismatch: cx3.json?.code, state: cancelled?.state });
  await pause(600);
  await watchA.close();
  await watchB.close();

  // ---------------------------------------------------------------- Phase 7 : aucune autorité en trop, liaison projet/conversation
  const refusedLike = (r: any) => r.isError === true || r.error !== undefined;
  const rowsBeforeForge = counts(fx);
  const forgedArgs: Array<[string, unknown]> = [["parent", peerB.agentId], ["permissions", { runtime_mode: "full-access" }], ["sandbox", "danger-full-access"], ["endpoint", "http://127.0.0.1:1/mcp"], ["credential", "Bearer x"], ["proof", { endpoint: "x" }], ["agent_id", peerB.agentId]];
  const forgedResults: Record<string, unknown> = {};
  let forgedRefused = 0;
  for (const [key, value] of forgedArgs) {
    const r = await mcpA.call("bridget_delegate", { ...reqA, request_id: `interop-forge-${key}`, task: `NONCE_fg${key}`, [key]: value });
    forgedResults[key] = r.error?.message ?? r.payload?.text ?? r.code;
    if (refusedLike(r)) forgedRefused += 1;
  }
  const afterForge = counts(fx);
  checks.add("S7.1", "REEL", "aucune autorité en trop : parent, permissions, sandbox, endpoint, credential, proof, agent_id forgés dans les arguments -> 7 refus (arguments fermés), zéro ligne, zéro lancement",
    forgedRefused === forgedArgs.length && afterForge.rows === rowsBeforeForge.rows && afterForge.started === rowsBeforeForge.started, { refused: forgedRefused, of: forgedArgs.length, rows: [rowsBeforeForge.rows, afterForge.rows], messages: forgedResults });
  const project2 = FS.realpathSync(FS.mkdtempSync(Path.join(fx.tmpRoot, "work2-")));
  ChildProcess.execFileSync("/usr/bin/git", ["init", "-q", project2], { env: { PATH: "/usr/bin:/bin", HOME: fx.tmpRoot, GIT_CONFIG_GLOBAL: "/dev/null", GIT_CONFIG_SYSTEM: "/dev/null" } });
  const escapeLink = Path.join(fx.work, "escape-link");
  FS.symlinkSync(project2, escapeLink);
  const cwdTmp = await mcpA.call("bridget_delegate", { ...reqA, request_id: "interop-cwd-tmp", task: "NONCE_cw1", cwd: "/tmp" });
  const cwdOther = await mcpA.call("bridget_delegate", { ...reqA, request_id: "interop-cwd-other", task: "NONCE_cw2", cwd: project2 });
  const cwdLink = await mcpA.call("bridget_delegate", { ...reqA, request_id: "interop-cwd-link", task: "NONCE_cw3", cwd: escapeLink });
  const cwdDotdot = await mcpA.call("bridget_delegate", { ...reqA, request_id: "interop-cwd-dotdot", task: "NONCE_cw4", cwd: `${fx.work}/../` });
  checks.add("S7.2", "REEL", "projet attesté : cwd hors projet (/tmp), autre dépôt git, lien symbolique sortant, .. -> refus d'admission avant tout lancement",
    [cwdTmp, cwdOther, cwdLink, cwdDotdot].every(refusedLike) && ["cw1", "cw2", "cw3", "cw4"].every((n) => fx.evidence().filter((e: any) => e.event === "prompt" && e.nonce === n).length === 0),
    { tmp: cwdTmp.payload?.text, otherRepo: cwdOther.payload?.text, symlink: cwdLink.payload?.text, dotdot: cwdDotdot.payload?.text });
  // deuxième projet, quatrième conversation
  const peerD = await fx.registerParent(THREAD_D, { project: project2 });
  const cD = (await host.issue(THREAD_D)).config;
  host.publish(cD, codexFullAccessFact(cD, "run-d-1", project2), host.startRun(THREAD_D, "run-d-1"));
  const mcpD = await fx.mcp({ endpoint: cD.endpoint, authorization: cD.authorizationHeader, label: "D" });
  const dIntoP1 = await mcpD.call("bridget_delegate", { ...reqA, request_id: "interop-d-p1", task: "NONCE_d1", cwd: fx.work });
  const dInP2 = await mcpD.call("bridget_delegate", { ...reqA, request_id: "interop-d-p2", task: "NONCE_d2", cwd: project2 });
  const taskD = dInP2.payload?.task_id as string | undefined;
  if (taskD) await waitTask(fx, taskD, ["result_available", "failed"]);
  const listD = await fx.cli(lineageArgs("list", THREAD_D, project2));
  const dWrongRoot = await fx.cli(lineageArgs("list", THREAD_D, fx.work));
  const aWrongRoot = await fx.cli(lineageArgs("list", THREAD_A, project2));
  const dShowA = await fx.cli(lineageArgs("show", THREAD_D, project2, ["--task", taskA, "--offset", "0", "--limit", "100"]));
  checks.add("S7.3", "REEL", "conversation D d'un AUTRE projet : ne peut pas lancer dans le projet de A (refus), lance dans le sien ; sa liste Lineage ne contient que sa tâche ; racines croisées -> project_mismatch ; tâche de A introuvable",
    refusedLike(dIntoP1) && !refusedLike(dInP2) && (listD.json?.tasks ?? []).length === 1 && listD.json.tasks[0].task_id === taskD && dWrongRoot.json?.code === "project_mismatch" && aWrongRoot.json?.code === "project_mismatch" && dShowA.json?.code === "task_unavailable",
    { intoOtherProject: dIntoP1.payload?.text, ownProject: dInP2.payload?.status, listD: listD.json?.tasks?.length, dWrongRoot: dWrongRoot.json?.code, aWrongRoot: aWrongRoot.json?.code, dShowA: dShowA.json?.code });
  await mcpD.close();
  peerD.close();

  // ---------------------------------------------------------------- Bilan et nettoyage
  const final = counts(fx, peers);
  notes.finalCounts = final;
  notes.httpSeen = { posts: host.seen.filter((r) => r.method === "POST").length, deletes: host.seen.filter((r) => r.method === "DELETE").length, statuses: Object.fromEntries([...new Set(host.seen.map((r) => r.status))].map((s) => [s, host.seen.filter((r) => r.status === s).length])) };
  notes.evidencePids = fx.evidence().filter((e: any) => e.event === "started").map((e: any) => e.pid);
  await mcpA.close();
  await mcpB.close();
  peerA.close();
  peerB.close();
  notes.stopDaemon = await fx.stopDaemon();
} catch (error) {
  exitCode = 1;
  checks.add("EXC", "REEL", "exception non prévue", false, { message: String((error as Error).stack ?? error).slice(0, 800) });
} finally {
  notes.cleanup = await fx.cleanup().catch((e) => String(e));
  await host.stop().catch(() => undefined);
}
const summary = checks.summary();
const live = (notes.cleanup as any)?.remainingAfter ?? [];
await writeResults("interop149", {
  schema: "native-network-recipes/1", startedAt: T0, finishedAt: new Date().toISOString(),
  binary: { path: BIN, sha256: BIN_SHA256 }, node: process.version, host: OS.hostname(), umask: "077",
  summary, notes, checks: checks.items, residualProcesses: live,
});
console.log(JSON.stringify({ summary, residualProcesses: live.length }));
process.exit(exitCode || (summary.fail > 0 || live.length > 0 ? 1 : 0));
