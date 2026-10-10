#!/usr/bin/env node
// Recette UI 149 — fixture CLI du daemon Bridget (saut simulé nommé).
// Processus réel NDJSON qui implémente le contrat fermé de
// specs/149-sous-agents-lineage/contracts/lineage.md sur un magasin JSON privé.
// Toutes les couches en amont (UI web, WS, serveur T3, orchestration, lecteur)
// restent réelles. Ce script est la seule couche simulée.
// Magasin : variable BRIDGET149_UI_STORE (chemin absolu du store.json). 0700.
import { readFileSync, writeFileSync, existsSync, statSync, renameSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const EXIT_OK = 0;
const EXIT_REFUSAL = 2; // refus métier nommé
const EXIT_UNAVAILABLE = 3; // indisponibilité technique

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const STATUSES = ["queued", "starting", "mission_pending", "working", "waiting_for_children", "cancelling", "result_available", "failed", "cancelled"];

const storePath = () => {
  const p = process.env.BRIDGET149_UI_STORE;
  if (!p || !p.startsWith("/")) fail("store_unavailable", "BRIDGET149_UI_STORE absent");
  return p;
};

function loadStore() {
  const p = storePath();
  if (!existsSync(p)) emitErrorAndExit("store_unavailable", "magasin absent");
  const raw = JSON.parse(readFileSync(p, "utf8"));
  // verrou de recette : un seul consommateur à la fois
  const lock = p + ".lock";
  if (existsSync(lock) && Date.now() - statSync(lock).mtimeMs < 5000) {
    emitErrorAndExit("store_unavailable", "magasin verrouillé");
  }
  return raw;
}

function emitErrorAndExit(code, detail, retryable) {
  const body = { version: 1, status: "error", code };
  if (retryable !== undefined) body.retryable = retryable;
  process.stderr.write(`fixture149: ${detail}\n`);
  process.stdout.write(JSON.stringify(body) + "\n");
  process.exit(code === "unavailable" || code === "timeout" || code === "store_unavailable" ? EXIT_UNAVAILABLE : EXIT_REFUSAL);
}

function fail(code, detail, retryable) {
  // variante silencieuse pour usage pré-sortie
  process.stderr.write(`fixture149: ${detail}\n`);
  process.exit(EXIT_REFUSAL);
}

function parseArgs(argv) {
  const known = new Set(["--t3-thread", "--project-root", "--action", "--task", "--cursor", "--limit", "--offset", "--after-seq", "--follow", "--request-id", "--json"]);
  const seen = new Map();
  let i = 0;
  while (i < argv.length) {
    const flag = argv[i];
    if (!known.has(flag)) emitErrorAndExit("invalid_request", `option inconnue ${flag}`);
    if (seen.has(flag)) emitErrorAndExit("invalid_request", `option dupliquée ${flag}`);
    if (flag === "--json") { seen.set(flag, true); i += 1; continue; }
    if (flag === "--follow") { seen.set(flag, true); i += 1; continue; }
    const value = argv[i + 1];
    if (value === undefined || value.startsWith("--")) emitErrorAndExit("invalid_request", `valeur manquante ${flag}`);
    seen.set(flag, value);
    i += 2;
  }
  if (!seen.has("--json")) emitErrorAndExit("invalid_request", "--json requis");
  return seen;
}

function requireCanonicalUuid(value, label) {
  if (!UUID_RE.test(value)) emitErrorAndExit("invalid_request", `${label} n'est pas un UUID canonique`);
  return value;
}

function boundedInt(value, min, max, label) {
  if (!/^\d+$/.test(value)) emitErrorAndExit("invalid_request", `${label} non entier`);
  const n = Number(value);
  if (!Number.isSafeInteger(n) || n < min || n > max) emitErrorAndExit("invalid_request", `${label} hors intervalle ${min}..${max}`);
  return n;
}

function utf8Boundary(buf, offset) {
  if (offset === 0) return true;
  if (offset >= buf.length) return false;
  const b = buf[offset];
  // octet de continuation (10xxxxxx) = frontière invalide
  return (b & 0xc0) !== 0x80;
}

function taskView(task) {
  // projection exacte de BridgetLineageTask — champs figés, sans consigne complète
  return {
    task_id: task.task_id,
    parent_task_id: task.parent_task_id,
    parent_agent_id: task.parent_agent_id,
    child_agent_id: task.child_agent_id,
    child_instance_id: task.child_instance_id,
    created_at: task.created_at,
    updated_at: task.updated_at,
    started_at: task.started_at,
    completed_at: task.completed_at,
    agent_type: task.agent_type,
    execution_protocol: task.execution_protocol,
    model: task.model,
    effort: task.effort,
    cwd: task.cwd,
    posture: task.posture,
    title: task.title,
    status: task.status,
    error: task.error,
    result_available: task.status === "result_available",
    journal_available: task.journal_available,
  };
}

const sortedTasks = (store) =>
  [...store.tasks].sort((a, b) => a.created_at - b.created_at || (a.task_id < b.task_id ? -1 : 1));

function actionList(store, args) {
  const threadId = requireCanonicalUuid(args.get("--t3-thread"), "t3-thread");
  const root = args.get("--project-root");
  if (root !== store.project_root) emitErrorAndExit("project_mismatch", "root étranger");
  if (!UUID_RE.test(threadId)) emitErrorAndExit("invalid_request", "thread non UUID");
  const limit = args.has("--limit") ? boundedInt(args.get("--limit"), 1, 100, "limit") : 50;
  const cursor = args.has("--cursor") ? args.get("--cursor") : null;
  if (cursor !== null && cursor.length > 2048) emitErrorAndExit("invalid_request", "cursor trop long");
  // curseur opaque de recette : base64("page:<offset>")
  let offset = 0;
  if (cursor !== null && (store.bump_on_cursor ?? 0) > 0) {
    // contrôle de recette : mutation du magasin ENTRE deux pages (le curseur devient périmé)
    mutateStore(storePath(), (fresh) => { fresh.seq += 1; fresh.bump_on_cursor -= 1; });
    Object.assign(store, JSON.parse(readFileSync(storePath(), "utf8")));
  }
  if (cursor !== null) {
    try {
      const decoded = Buffer.from(cursor, "base64").toString("utf8");
      const m = decoded.match(/^page:(\d+):([0-9a-f-]{36}):(\d+)$/);
      if (!m || m[2] !== store.generation || Number(m[3]) !== store.seq) {
        emitErrorAndExit("snapshot_changed", "génération du curseur périmée");
      }
      offset = Number(m[1]);
    } catch {
      emitErrorAndExit("invalid_request", "cursor illisible");
    }
  }
  const all = sortedTasks(store).map(taskView);
  const page = all.slice(offset, offset + limit);
  const nextOffset = offset + page.length;
  const nextCursor = nextOffset < all.length
    ? Buffer.from(`page:${nextOffset}:${store.generation}:${store.seq}`).toString("base64")
    : null;
  process.stdout.write(JSON.stringify({
    version: 1,
    status: "ok",
    generation: store.generation,
    seq: store.seq,
    root_owner_agent_id: store.root_owner_agent_id,
    tasks: page,
    next_cursor: nextCursor,
  }) + "\n");
}

function findTask(store, taskId, root) {
  if (root !== store.project_root) emitErrorAndExit("project_mismatch", "root étranger");
  const task = store.tasks.find((t) => t.task_id === taskId);
  if (!task) emitErrorAndExit("task_unavailable", "tâche absente du root");
  return task;
}

function actionShow(store, args) {
  if (!args.has("--task")) emitErrorAndExit("invalid_request", "show exige --task");
  const taskId = requireCanonicalUuid(args.get("--task"), "task");
  const offset = args.has("--offset") ? boundedInt(args.get("--offset"), 0, Number.MAX_SAFE_INTEGER, "offset") : 0;
  const limit = args.has("--limit") ? boundedInt(args.get("--limit"), 1, 16384, "limit") : 16384;
  const task = findTask(store, taskId, args.get("--project-root"));
  const resultText = store.results[taskId] ?? null;
  const buf = resultText === null ? Buffer.alloc(0) : Buffer.from(resultText, "utf8");
  if (resultText === null && task.status === "result_available") {
    emitErrorAndExit("store_unavailable", "résultat attendu absent du magasin");
  }
  if (offset > buf.length) emitErrorAndExit("result_offset_invalid", "offset hors résultat");
  if (!utf8Boundary(buf, offset)) emitErrorAndExit("result_offset_invalid", "offset hors frontière UTF-8");
  const window = buf.subarray(offset, Math.min(offset + limit, buf.length));
  const next = offset + window.length < buf.length ? offset + window.length : null;
  process.stdout.write(JSON.stringify({
    version: 1,
    status: "ok",
    generation: store.generation,
    seq: store.seq,
    root_owner_agent_id: store.root_owner_agent_id,
    task: taskView(task),
    result: window.length === 0 ? null : window.toString("utf8"),
    result_offset: offset,
    result_next_offset: next,
    result_total_bytes: buf.length,
  }) + "\n");
}

function journalPage(store, taskId, afterSeq, limit) {
  const events = (store.journals[taskId] ?? []).filter((e) => e.seq > afterSeq).slice(0, limit);
  const lastSeq = events.length > 0 ? events[events.length - 1].seq : afterSeq;
  const durable = store.journal_next_seq[taskId] ?? 0;
  const caughtUp = lastSeq >= durable;
  const gap = store.gaps[taskId] ?? null;
  return {
    version: 1,
    status: "ok",
    task_id: taskId,
    events,
    next_seq: Math.max(lastSeq, afterSeq),
    caught_up: caughtUp,
    gap: gap,
  };
}

function actionJournal(store, args) {
  if (!args.has("--task")) emitErrorAndExit("invalid_request", "journal exige --task");
  const taskId = requireCanonicalUuid(args.get("--task"), "task");
  const task = findTask(store, taskId, args.get("--project-root"));
  if (!task.journal_available) emitErrorAndExit("journal_unavailable", "journal absent pour cette tâche");
  const afterSeq = args.has("--after-seq") ? boundedInt(args.get("--after-seq"), 0, Number.MAX_SAFE_INTEGER, "after-seq") : 0;
  const limit = args.has("--limit") ? boundedInt(args.get("--limit"), 1, 100, "limit") : 50;
  const follow = args.has("--follow");
  const emitPage = () => process.stdout.write(JSON.stringify(journalPage(store, taskId, afterSeq, limit)) + "\n");
  if (!follow) { emitPage(); return; }
  // --follow : première page bornée puis pages live à chaque mutation du magasin
  const poll = () => {
    try {
      const fresh = JSON.parse(readFileSync(storePath(), "utf8"));
      return fresh;
    } catch {
      return store; // lecture transitoire : on garde la dernière vue cohérente
    }
  };
  // Première page bornée, puis seulement les événements au-delà du dernier next_seq
  // émis (le lecteur T3 refuse tout rejeu : event.seq <= précédent => invalid_output).
  const first = journalPage(store, taskId, afterSeq, limit);
  process.stdout.write(JSON.stringify(first) + "\n");
  let emittedSeq = first.next_seq;
  let emittedGap = JSON.stringify(first.gap);
  const timer = setInterval(() => {
    const fresh = poll();
    const page = journalPage(fresh, taskId, emittedSeq, limit);
    const gapNow = JSON.stringify(page.gap);
    if (page.events.length > 0 || gapNow !== emittedGap) {
      emittedSeq = page.next_seq;
      emittedGap = gapNow;
      process.stdout.write(JSON.stringify(page) + "\n");
    }
  }, 250);
  // plafond de sécurité 20 min (un suivi réel reste ouvert tant que le consommateur vit ; r2 : 60 s coupait le flux)
  const stop = setTimeout(() => { clearInterval(timer); process.exit(EXIT_OK); }, 20 * 60_000);
  process.on("SIGTERM", () => { clearInterval(timer); clearTimeout(stop); process.exit(EXIT_OK); });
  // maintient le flux ouvert pour le lecteur T3
  setInterval(() => {}, 1000);
}

function actionWatch(store, args) {
  const root = args.get("--project-root");
  if (root !== store.project_root) emitErrorAndExit("project_mismatch", "root étranger");
  const threadId = requireCanonicalUuid(args.get("--t3-thread"), "t3-thread");
  // ready seq 0 obligatoire et non coalescable, puis changed par seq croissante
  process.stdout.write(JSON.stringify({ version: 1, generation: store.generation, seq: 0, status: "ready" }) + "\n");
  let lastSeq = store.seq;
  let lastGeneration = store.generation;
  let killed = false;
  const poll = () => {
    if (killed) return;
    let fresh;
    try {
      fresh = JSON.parse(readFileSync(storePath(), "utf8"));
    } catch {
      return;
    }
    if (fresh.generation !== lastGeneration) {
      process.stdout.write(JSON.stringify({ version: 1, generation: fresh.generation, seq: fresh.seq, status: "resync" }) + "\n");
      lastGeneration = fresh.generation;
      lastSeq = fresh.seq;
      return;
    }
    if (fresh.seq > lastSeq) {
      process.stdout.write(JSON.stringify({ version: 1, generation: fresh.generation, seq: fresh.seq, status: "changed" }) + "\n");
      lastSeq = fresh.seq;
    }
  };
  const timer = setInterval(poll, 200);
  process.on("SIGTERM", () => { killed = true; clearInterval(timer); process.exit(EXIT_OK); });
  process.on("SIGHUP", () => { killed = true; clearInterval(timer); process.exit(EXIT_OK); });
  // flux vivant : le lecteur T3 l'interrompt à la disparition du contexte
  setInterval(() => {}, 1000);
}

function actionCancel(store, args) {
  if (!args.has("--task") || !args.has("--request-id")) emitErrorAndExit("invalid_request", "cancel exige --task et --request-id");
  const taskId = requireCanonicalUuid(args.get("--task"), "task");
  const requestId = requireCanonicalUuid(args.get("--request-id"), "request-id");
  const task = findTask(store, taskId, args.get("--project-root"));
  const envelope = JSON.stringify({ task_id: taskId, request_id: requestId });
  const receipts = store.cancel_receipts ?? {};
  const previous = receipts[requestId];
  if (previous !== undefined) {
    if (previous.envelope !== envelope) emitErrorAndExit("envelope_mismatch", "rejeu avec enveloppe différente");
    // rejeu identique : reçu natif stable, aucun nouvel effet
    process.stdout.write(JSON.stringify({ version: 1, task_id: taskId, status: previous.status }) + "\n");
    return;
  }
  if (["result_available", "failed", "cancelled"].includes(task.status)) {
    // déjà terminale : état courant observé, pas cancelled inventé
    const status = task.status;
    persistReceipt(store, requestId, envelope, status);
    process.stdout.write(JSON.stringify({ version: 1, task_id: taskId, status }) + "\n");
    return;
  }
  // transition cancelling ; le scénario pousse ensuite cancelled via le magasin
  mutateStore(storePath(), (fresh) => {
    const target = fresh.tasks.find((t) => t.task_id === taskId);
    if (target) { target.status = "cancelling"; target.updated_at = nowSeconds(); }
    fresh.cancel_receipts = { ...(fresh.cancel_receipts ?? {}), [requestId]: { envelope, status: "cancelling" } };
    bumpSeq(fresh);
  });
  process.stdout.write(JSON.stringify({ version: 1, task_id: taskId, status: "cancelling" }) + "\n");
}

function persistReceipt(store, requestId, envelope, status) {
  mutateStore(storePath(), (fresh) => {
    fresh.cancel_receipts = { ...(fresh.cancel_receipts ?? {}), [requestId]: { envelope, status } };
    bumpSeq(fresh);
  });
}

function nowSeconds() { return Math.floor(Date.now() / 1000); }

function bumpSeq(store) {
  store.seq = (store.seq ?? 0) + 1;
}

function mutateStore(path, mutate) {
  // mutation atomique simple de recette : lecture, mutation, écriture renommée
  const tmp = path + ".tmp";
  const fresh = JSON.parse(readFileSync(path, "utf8"));
  mutate(fresh);
  writeFileSync(tmp, JSON.stringify(fresh, null, 2), { mode: 0o600 });
  renameSync(tmp, path);
}

// --- legacy 148 minimal : thread inspect/watch, mêmes gardes, vue de fil partagé ---
function threadInspect(store, args) {
  const root = args.get("--project-root");
  if (root !== store.project_root) emitErrorAndExit("project_mismatch", "root étranger");
  const action = args.get("--action");
  if (action === "list") {
    process.stdout.write(JSON.stringify({
      version: 1,
      result: {
        status: "listed",
        threads: (store.shared_threads ?? []).map((t) => ({
          thread_id: t.thread_id, title: t.title, last_activity_at: t.last_activity_at,
        })),
        snapshot_seq: store.shared_seq ?? 1,
      },
    }) + "\n");
    return;
  }
  emitErrorAndExit("invalid_request", "action thread non couverte par la fixture");
}

// --- routage principal ---
// Formes réelles : bridget lineage inspect --action X | bridget lineage watch |
// bridget lineage cancel | bridget thread inspect | bridget thread watch | bridget mcp
const argv = process.argv.slice(2);
const namespace = argv[0];
const verb = argv[1];
let rest = argv.slice(1);
if (namespace === "lineage" || namespace === "thread") {
  if (verb === "inspect" || verb === "watch" || verb === "cancel") rest = argv.slice(2);
  else if (verb?.startsWith("--")) rest = argv.slice(1);
  else emitErrorAndExit("invalid_request", `verbe inconnu ${verb ?? "absent"}`);
} else {
  rest = argv.slice(1);
}
const args = parseArgs(rest);
const store = loadStore();
// Contrôle de recette « daemon lent » : store.hang_ms fait dormir l'appel (hors watch/follow)
// pour prouver le timeout 6 s du lecteur T3 (S149-18). Absent = comportement normal.
if (Number.isInteger(store.hang_ms) && store.hang_ms > 0 && verb !== "watch" && !args.has("--follow")) {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, store.hang_ms);
}

if (namespace === "lineage") {
  const action = args.get("--action");
  if (verb === "cancel" || (args.has("--request-id") && !action)) actionCancel(store, args);
  else if (verb === "watch" && !action) actionWatch(store, args);
  else if (action === "list") actionList(store, args);
  else if (action === "show") actionShow(store, args);
  else if (action === "journal") actionJournal(store, args);
  else emitErrorAndExit("invalid_request", "action lineage inconnue");
} else if (namespace === "thread") {
  const action = args.get("--action");
  if (verb === "watch" && !action) {
    // watch humain 148 : mêmes événements ready/changed, magasin partagé
    actionWatch(store, args);
  } else if (action === "list" || action === "list_recent") threadInspect(store, args);
  else emitErrorAndExit("invalid_request", "action thread inconnue");
} else if (namespace === "mcp") {
  // le chemin MCP privé réel exige le binaire natif 149 ; la fixture le refuse nommément
  emitErrorAndExit("unavailable", "MCP privé : binaire natif 149 requis (recette CLI fixture)");
} else {
  emitErrorAndExit("invalid_request", `namespace inconnu ${namespace}`);
}
// Les modes streaming (watch, journal --follow) gardent leur propre boucle ; un
// exit immédiat les tuerait juste après `ready` (constat r2 : exit 0 en 31 ms).
const streaming = verb === "watch" || args.has("--follow");
if (!streaming) process.exit(EXIT_OK);
