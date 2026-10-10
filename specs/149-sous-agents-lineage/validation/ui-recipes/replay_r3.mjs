#!/usr/bin/env node
// Recette 149 r3 - rejeux ciblés des correctifs Sol (runtime T3) contre le VRAI serveur T3 de recette.
// Couches RÉELLES : serveur T3 WT, RPC WebSocket scopés, BridgetLineage, Orchestrator, projection SQL privée.
// Couche SIMULÉE nommée : bridget_fixture.mjs (daemon, magasin JSON privé). Données synthétiques [recette149].
// Mesure : événements d'orchestration (table orchestration_events) et disponibilité des fils virtuels (SQL lecture seule).
// Usage : RECIPE149_CACHE=... RECIPE149_T3_PORT=... node replay_r3.mjs <groupe|all>
//   groupes : tick seed cursor reads generation global unseed local known foreign forged
import { execFileSync } from "node:child_process";
import { readFileSync, copyFileSync, chmodSync } from "node:fs";

const C = process.env.RECIPE149_CACHE ?? "/Users/moi/.cache/bridget149-ui";
const R = new URL(".", import.meta.url).pathname;
const STORE = `${C}/store.json`;
const DB = `file:${C}/t3home/userdata/statev2.sqlite?mode=ro`;
const { projectId: P, threadId: T } = JSON.parse(readFileSync(`${C}/ids.json`, "utf8"));
const { projectId2: P2, threadId2: T2 } = JSON.parse(readFileSync(`${C}/ids2.json`, "utf8"));
const U = (n) => `00000000-0000-4000-8000-${String(n).padStart(12, "0")}`;
const VT = (n) => `thread:bridget-task:${U(n)}`;
const tok = (n) => `${C}/tok-${n}.txt`;
const uuid = () => crypto.randomUUID();

const sh = (file, args) => execFileSync(file, args, { encoding: "utf8", timeout: 120000, env: process.env }).trim();
const sql = (q) => sh("sqlite3", ["-readonly", DB, q]);
const rpc = (token, method, payload, extra = []) => {
  try {
    return JSON.parse(sh(process.execPath, ["--no-warnings", `${R}rpc149.ts`, "call", token, method, JSON.stringify(payload), ...extra]).split("\n").pop());
  } catch (e) { return { harness_error: String(e.stdout ?? e.message).slice(0, 300) }; }
};
const step = (...a) => sh(process.execPath, [`${R}scenario_step.mjs`, STORE, ...a]);
const ctx = (extra = {}) => ({ projectId: P, threadId: T, ...extra });
const list = (token = "read") => rpc(tok(token), "bridget.lineage.read", ctx({ action: "list" }));
const code = (r) => r.error?.code ?? r.error?.tag ?? r.transport_error ?? null;

const maxSeq = () => Number(sql("select coalesce(max(sequence),0) from orchestration_events"));
const delta = (since) => {
  const out = {};
  for (const line of sql(`select event_type||'|'||stream_id||'|'||count(*) from orchestration_events where sequence > ${since} group by event_type, stream_id`).split("\n").filter(Boolean)) {
    const [type, thread, n] = line.split("|");
    (out[type] ??= []).push([thread, Number(n)]);
  }
  const total = (type) => (out[type] ?? []).reduce((a, [, n]) => a + n, 0);
  return { by: out, total, all: Object.values(out).flat().reduce((a, [, n]) => a + n, 0), threads: (type) => new Set((out[type] ?? []).map(([t]) => t)) };
};
const avail = (threadId) => sql(`select json_extract(payload_json,'$.bridgetLineage.available') from orchestration_v2_projection_threads where thread_id='${threadId}'`);
const marker = (threadId) => sql(`select json_extract(payload_json,'$.bridgetTaskRef.generation')||'|'||json_extract(payload_json,'$.bridgetTaskRef.seq') from orchestration_v2_projection_threads where thread_id='${threadId}'`);
const rootState = () => sql(`select json_extract(payload_json,'$.bridgetLineage.generation')||'|'||json_extract(payload_json,'$.bridgetLineage.seq')||'|'||json_extract(payload_json,'$.bridgetLineage.available') from orchestration_v2_projection_threads where thread_id='${T}'`);
const virtualCount = () => Number(sql("select count(*) from orchestration_v2_projection_threads where thread_id like 'thread:bridget-task:%'"));
const virtualAvail = (flag) => Number(sql(`select count(*) from orchestration_v2_projection_threads where thread_id like 'thread:bridget-task:%' and json_extract(payload_json,'$.bridgetLineage.available')=${flag}`));
const storeSeq = () => JSON.parse(readFileSync(STORE, "utf8")).seq;
const backup = (name) => { copyFileSync(STORE, `${C}/${name}`); chmodSync(`${C}/${name}`, 0o600); };
const restore = (name) => { copyFileSync(`${C}/${name}`, STORE); chmodSync(STORE, 0o600); };

let pass = 0, fail = 0, finding = 0;
const check = (id, title, ok, detail) => {
  if (ok) pass++; else fail++;
  console.log(`${ok ? "PASS" : "FAIL"} ${id.padEnd(6)} ${title} => ${detail}`);
};
const known = (id, title, ok, detail) => {
  // oracle de sûreté : un échec est un DÉFAUT à signaler (compté à part), pas une erreur d'outil
  if (ok) pass++; else finding++;
  console.log(`${ok ? "PASS" : "FINDING"} ${id.padEnd(6)} ${title} => ${detail}`);
};
const fmt = (d) => JSON.stringify(Object.fromEntries(Object.entries(d.by).map(([k, v]) => [k, v.reduce((a, [, n]) => a + n, 0)])));

const groups = {
  tick() {
    list(); // normalise : tout est synchronisé au seq courant
    let s = maxSeq();
    step("journal");
    list();
    let d = delta(s);
    check("T1", "journal 1/4 (fait de tâche modifié) : T1 + racine seulement", d.total("subagent.updated") === 1 && d.total("thread.created") === 0 && d.total("thread.metadata-updated") === 2 && d.threads("thread.metadata-updated").has(VT(1)) && d.threads("thread.metadata-updated").has(T) && d.all === 3, `${fmt(d)} fils_metadata=${[...d.threads("thread.metadata-updated")].map((x) => x.slice(-4)).join(",")}`);
    check("T2", "tâches inchangées T2/T3/T4 : 0 événement", ![VT(2), VT(3), VT(4)].some((t) => Object.values(d.by).flat().some(([th]) => th === t)), "aucun événement sur …0002/…0003/…0004");
    s = maxSeq();
    step("tick");
    list();
    d = delta(s);
    check("T3", "tick journal seul (seq+1, aucun fait) : racine seule", d.all === 1 && d.total("thread.metadata-updated") === 1 && d.threads("thread.metadata-updated").has(T), fmt(d));
    const [g, sq, av] = rootState().split("|");
    check("T4", "filigrane racine = seq du magasin ; marqueur d'un enfant inchangé ≤ racine", Number(sq) === storeSeq() && av === "1" && Number(marker(VT(4)).split("|")[1]) < Number(sq), `racine ${g.slice(-3)}|${sq}|${av} enfant4 ${marker(VT(4))}`);
  },
  seed() {
    list();
    let s = maxSeq();
    step("seed", "130");
    const r = list();
    let d = delta(s);
    check("S1", "seed 130 (134 tâches, 2 pages) : 130 créations + 130 subagent + racine", r.value?.status === "ok" && d.total("thread.created") === 130 && d.total("subagent.updated") === 130 && d.total("thread.metadata-updated") === 1, `${fmt(d)} fils_virtuels=${virtualCount()}`);
    s = maxSeq();
    step("journal");
    list();
    d = delta(s);
    check("S2", "journal 1/134 : 0 réécriture des 133 autres, ≤2 événements pour la tâche modifiée + racine 1", d.total("thread.created") === 0 && d.total("subagent.updated") === 1 && d.total("thread.metadata-updated") === 2 && d.all === 3, fmt(d));
    s = maxSeq();
    step("tick");
    list();
    d = delta(s);
    check("S3", "tick journal seul 1/134 : racine seule (1 événement, pas ~270)", d.all === 1 && d.threads("thread.metadata-updated").has(T), fmt(d));
  },
  cursor() {
    list();
    let s = maxSeq();
    const before = { v: virtualCount(), root: rootState() };
    step("cursor-bump", "3");
    const r = list();
    let d = delta(s);
    check("C1", "mutation entre pages x3 : snapshot_changed, AUCUNE réconciliation partielle (nombre de fils inchangé)", code(r) === "snapshot_changed" && virtualCount() === before.v, `code=${code(r)} fils=${virtualCount()}/${before.v} delta=${fmt(d)} racine=${rootState()}`);
    check("C2", "échec de snapshot : 0 événement de projection hors dégradation (aucun thread.created / subagent.updated)", d.total("thread.created") === 0 && d.total("subagent.updated") === 0, fmt(d));
    // la racine a pu être marquée indisponible par l'échec (transport/format) ; un snapshot complet la rétablit
    s = maxSeq();
    step("cursor-bump", "1");
    const ok = list();
    d = delta(s);
    check("C3", "une mutation seule : 2e tentative réussit, snapshot complet, 0 création/subagent", ok.value?.status === "ok" && d.total("thread.created") === 0 && d.total("subagent.updated") === 0, `${fmt(d)} racine=${rootState()}`);
  },
  reads() {
    list();
    const s = maxSeq();
    const a = rpc(tok("read"), "bridget.lineage.read", ctx({ action: "list" }), ["--repeat", "100"]);
    const b = rpc(tok("read"), "bridget.lineage.read", ctx({ action: "show", taskId: U(3) }), ["--repeat", "100"]);
    const d = delta(s);
    check("R1", "100 list + 100 show sur snapshot inchangé : 0 événement", a.ok === 100 && b.ok === 100 && d.all === 0, `list_ok=${a.ok} show_ok=${b.ok} delta=${fmt(d)}`);
  },
  generation() {
    list();
    const s = maxSeq();
    step("generation");
    const n = virtualCount();
    list();
    const d = delta(s);
    const genMarkers = Number(sql(`select count(*) from orchestration_v2_projection_threads where thread_id like 'thread:bridget-task:%' and json_extract(payload_json,'$.bridgetTaskRef.generation')='${U(999)}'`));
    check("G1", "nouvelle génération, faits inchangés : marqueurs de tous les fils réécrits, 0 subagent.updated, 0 création", d.total("subagent.updated") === 0 && d.total("thread.created") === 0 && genMarkers === n && d.total("thread.metadata-updated") === n + 1, `${fmt(d)} fils=${n} marqueurs_gen999=${genMarkers} racine=${rootState().replace(/0000000000/g, "")}`);
  },
  global() {
    list();
    backup("store.json.r3global");
    const sq0 = storeSeq();
    const n = virtualCount();
    let s = maxSeq();
    step("store-corrupt");
    const r = list();
    let d = delta(s);
    check("F1", "panne GLOBALE (magasin illisible) : refus fermé, racine et fils indisponibles", ["invalid_output", "store_unavailable", "unavailable"].includes(code(r)) && avail(T) === "0" && virtualAvail(0) === n, `code=${code(r)} racine_available=${avail(T)} fils_indisponibles=${virtualAvail(0)}/${n} delta=${fmt(d)}`);
    restore("store.json.r3global");
    s = maxSeq();
    const ok = list();
    d = delta(s);
    const [, sq, av] = rootState().split("|");
    check("F2", "reprise COMPLÈTE au même seq : tout redevient available:true, 0 création, 0 subagent, aucun doublon", ok.value?.status === "ok" && av === "1" && Number(sq) === sq0 && virtualAvail(1) === n && d.total("thread.created") === 0 && d.total("subagent.updated") === 0, `${fmt(d)} seq=${sq}/${sq0} fils_disponibles=${virtualAvail(1)}/${n} fils=${virtualCount()}`);
  },
  unseed() {
    list();
    const s = maxSeq();
    step("unseed");
    list();
    const d = delta(s);
    const hist = Number(sql(`select count(*) from orchestration_v2_projection_threads where thread_id like 'thread:bridget-task:%' and json_extract(payload_json,'$.bridgetLineage.available')=0`));
    const obs = sql(`select min(json_extract(payload_json,'$.bridgetLineage.seq'))||'..'||max(json_extract(payload_json,'$.bridgetLineage.seq')) from orchestration_v2_projection_threads where thread_id like 'thread:bridget-task:%' and json_extract(payload_json,'$.bridgetLineage.available')=0`);
    const mk = sql(`select min(json_extract(payload_json,'$.bridgetTaskRef.seq'))||'..'||max(json_extract(payload_json,'$.bridgetTaskRef.seq')) from orchestration_v2_projection_threads where thread_id like 'thread:bridget-task:%' and json_extract(payload_json,'$.bridgetLineage.available')=0`);
    check("U1", "unseed 130 : historique voulu conservé (pas de prune), 130 fils available:false, 4 available:true", hist === 130 && virtualAvail(1) === 4 && virtualCount() === 134 && d.total("subagent.updated") === 0, `${fmt(d)} historiques=${hist} disponibles=${virtualAvail(1)} fils=${virtualCount()} observé_seq=${obs} marqueur_seq=${mk}`);
  },
  local() {
    list();
    const s = maxSeq();
    const j4 = rpc(tok("read"), "bridget.lineage.read", ctx({ action: "journal", taskId: U(4) }));
    const off = rpc(tok("read"), "bridget.lineage.read", ctx({ action: "show", taskId: U(3), offset: 99999 }));
    const js = rpc(tok("read"), "bridget.lineage.journal", ctx({ taskId: U(4) }), ["--take", "1"]);
    const d = delta(s);
    check("L1", "journal_unavailable (T4, read + flux) : message local, racine et T1/T2/T3 inchangés", code(j4) === "journal_unavailable" && js.error?.code === "journal_unavailable" && d.all === 0 && avail(T) === "1" && [1, 2, 3].every((n) => avail(VT(n)) === "1"), `read=${code(j4)} flux=${js.error?.code} delta=${fmt(d)} racine=${avail(T)} T1..T3=${[1, 2, 3].map((n) => avail(VT(n))).join("")}`);
    check("L2", "result_offset_invalid : racine inchangée, 0 événement", code(off) === "result_offset_invalid" && d.all === 0 && avail(T) === "1", `code=${code(off)} delta=${fmt(d)}`);
    const bad = rpc(tok("read"), "bridget.lineage.read", ctx({ action: "show", taskId: U(3), offset: 99999 }));
    check("L3", "refus d'arguments (envelope_mismatch sur cancel) : racine inchangée", (() => { const id = uuid(); rpc(tok("operate"), "bridget.lineage.cancel", ctx({ taskId: U(4), requestId: id })); const r = rpc(tok("operate"), "bridget.lineage.cancel", ctx({ taskId: U(3), requestId: id })); return code(r) === "envelope_mismatch" && avail(T) === "1"; })(), `racine=${avail(T)}`);
    void bad;
  },
  known() {
    list();
    backup("store.json.r3known");
    step("remove-task", U(3));
    const s = maxSeq();
    const r = rpc(tok("read"), "bridget.lineage.read", ctx({ action: "show", taskId: U(3) }));
    const d = delta(s);
    check("K1", "task_unavailable sur enfant CONNU de ce root : ce fil seul est invalidé", code(r) === "task_unavailable" && avail(VT(3)) === "0" && [1, 2, 4].every((n) => avail(VT(n)) === "1") && avail(T) === "1", `code=${code(r)} T3=${avail(VT(3))} T1,T2,T4=${[1, 2, 4].map((n) => avail(VT(n))).join("")} racine=${avail(T)} delta=${fmt(d)}`);
    restore("store.json.r3known");
    const ok = list();
    check("K2", "reprise : le fil redevient disponible au prochain snapshot complet", ok.value?.status === "ok" && avail(VT(3)) === "1", `T3=${avail(VT(3))}`);
  },
  foreign() {
    list();
    backup("store.json.r3foreign");
    step("foreign-root", `${C}/proj2`);
    const bind = rpc(tok("operate"), "bridget.lineage.read", { projectId: P2, threadId: T2, action: "list" });
    restore("store.json.r3foreign");
    const xThread = VT(5000);
    check("X0", "préparation : fil virtuel X lié à la racine ÉTRANGÈRE (root2)", bind.value?.status === "ok" && avail(xThread) === "1" && sql(`select json_extract(payload_json,'$.bridgetTaskRef.rootThreadId') from orchestration_v2_projection_threads where thread_id='${xThread}'`) === T2, `bind=${bind.value?.status ?? code(bind)} X_available=${avail(xThread)}`);
    list(); // rétablit root1 au cas où
    const s = maxSeq();
    const rootBefore = rootState();
    const r1 = rpc(tok("read"), "bridget.lineage.read", ctx({ action: "show", taskId: U(5000) }));
    const r2 = rpc(tok("operate"), "bridget.lineage.cancel", ctx({ taskId: U(5000), requestId: uuid() }));
    const r3 = rpc(tok("read"), "bridget.lineage.read", ctx({ action: "show", taskId: uuid() }));
    const d = delta(s);
    check("X1", "UUID d'une tâche d'un AUTRE root (show/cancel/inconnu) : task_unavailable, root1 et root2 intacts, 0 événement", code(r1) === "task_unavailable" && code(r2) === "task_unavailable" && code(r3) === "task_unavailable" && d.all === 0 && rootState() === rootBefore && avail(xThread) === "1" && avail(T2) !== "0", `show=${code(r1)} cancel=${code(r2)} inconnu=${code(r3)} delta=${fmt(d)} X_available=${avail(xThread)} root1_inchange=${rootState() === rootBefore}`);
  },
  forged() {
    list();
    const rootBefore = rootState();
    const s = maxSeq();
    // contexte forgé : projectId d'un AUTRE projet avec le fil réel de root1 (refus project_mismatch côté T3, avant le daemon)
    const r = rpc(tok("read"), "bridget.lineage.read", { projectId: P2, threadId: T, action: "list" });
    const d = delta(s);
    known("Z1", "contexte forgé (mauvais projectId + fil réel) : refusé ET sans effet sur la disponibilité du fil réel", code(r) === "project_mismatch" && avail(T) === "1" && d.all === 0, `code=${code(r)} racine_available=${avail(T)} (avant ${rootBefore.split("|")[2]}) fils_indisponibles=${virtualAvail(0)} delta=${fmt(d)}`);
    list(); // rétablit
  },
};

const want = process.argv[2] ?? "all";
const order = ["tick", "seed", "cursor", "reads", "generation", "global", "unseed", "local", "known", "foreign", "forged"];
for (const g of order) if (want === "all" || want === g) { console.log(`# groupe ${g}`); groups[g](); }
console.log(`TOTAL pass=${pass} fail=${fail} finding=${finding}`);
process.exit(fail === 0 ? 0 : 1);
