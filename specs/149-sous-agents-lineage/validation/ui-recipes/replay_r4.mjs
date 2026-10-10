#!/usr/bin/env node
// Recette 149 r4 - rejeux du correctif F1 (garde de racine centralisée dans BridgetLineage.unavailable) contre le VRAI serveur T3 de recette.
// Couches RÉELLES : serveur T3 WT, RPC WebSocket scopés, BridgetLineage, Orchestrator, projection SQL privée.
// Couche SIMULÉE nommée : bridget_fixture.mjs (daemon, magasin JSON privé). Données synthétiques [recette149].
// Mesure : événements d'orchestration (table orchestration_events) et disponibilité des fils virtuels (SQL lecture seule).
// Usage : RECIPE149_CACHE=... RECIPE149_T3_PORT=... node replay_r3.mjs <groupe|all>
//   groupes : forgedread forgedroot forgedtask genuine
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


// instantané complet de l'état de disponibilité : racine, hôte étranger, fils virtuels, séquence d'événements
const flags = () => `${rootState()}|virt1=${virtualAvail(1)}|virt0=${virtualAvail(0)}|n=${virtualCount()}|root2=${avail(T2)}`;
const noEffect = (since, before) => delta(since).all === 0 && flags() === before;

const groups = {
  forgedread() {
    list();
    const before = flags();
    const s = maxSeq();
    const forged = { projectId: P2, threadId: T };
    const cases = {
      list: rpc(tok("read"), "bridget.lineage.read", { ...forged, action: "list" }),
      show: rpc(tok("read"), "bridget.lineage.read", { ...forged, action: "show", taskId: U(3) }),
      journalRead: rpc(tok("read"), "bridget.lineage.read", { ...forged, action: "journal", taskId: U(3) }),
      watch: rpc(tok("read"), "bridget.lineage.watch", forged, ["--take", "1"]),
      journalStream: rpc(tok("read"), "bridget.lineage.journal", { ...forged, taskId: U(3) }, ["--take", "1"]),
      cancelOperate: rpc(tok("operate"), "bridget.lineage.cancel", { ...forged, taskId: U(1), requestId: uuid() }),
    };
    for (const [name, r] of Object.entries(cases)) {
      const c = code(r) ?? r.error?.code ?? r.items?.[0]?.code;
      known(`ZR-${name}`, `contexte forgé (projectId étranger + fil réel), ${name} : refus project_mismatch, aucun effet`, c === "project_mismatch" && !r.value && !(r.items?.length), `code=${c}`);
    }
    const d = delta(s);
    known("ZR-all", "6 appels forgés (scope read + operate) : 0 événement, drapeaux de disponibilité identiques", noEffect(s, before), `delta=${fmt(d)} flags_avant=${before} flags_après=${flags()}`);
    const ok = list();
    check("ZR-ok", "contexte VRAI juste après : list réussit, racine toujours disponible, même état", ok.value?.status === "ok" && flags() === before, `flags=${flags()}`);
  },
  forgedroot() {
    list();
    const before = flags();
    const s = maxSeq();
    const unknown = uuid();
    const calls = {
      "racine inconnue": [{ projectId: P, threadId: unknown }, unknown],
      "racine virtuelle (fil enfant utilisé comme racine)": [{ projectId: P, threadId: VT(1) }, VT(1)],
    };
    for (const [name, [c0]] of Object.entries(calls)) {
      const results = [
        rpc(tok("read"), "bridget.lineage.read", { ...c0, action: "list" }),
        rpc(tok("read"), "bridget.lineage.read", { ...c0, action: "show", taskId: U(1) }),
        rpc(tok("read"), "bridget.lineage.watch", c0, ["--take", "1"]),
        rpc(tok("read"), "bridget.lineage.journal", { ...c0, taskId: U(1) }, ["--take", "1"]),
      ];
      const codes = results.map((r) => code(r) ?? r.items?.[0]?.code ?? r.error?.code);
      known(`ZT-${name.split(" ")[1]}`, `${name} : refusé (binding_unavailable) sur list/show/watch/journal, 0 mutation`, codes.every((c) => c === "binding_unavailable") && noEffect(s, before), `codes=${codes.join(",")} delta=${fmt(delta(s))} flags=${flags() === before ? "intacts" : flags()}`);
    }
    list();
  },
  forgedtask() {
    list();
    backup("store.json.r4task");
    step("remove-task", U(3));
    const before = flags();
    const s = maxSeq();
    // tâche absente du magasin + contexte projet forgé : le refus project_mismatch précède tout contact daemon
    const forged = rpc(tok("read"), "bridget.lineage.read", { projectId: P2, threadId: T, action: "show", taskId: U(3) });
    known("ZK-1", "tâche CONNUE retirée du magasin + projet forgé : project_mismatch, ce fil enfant n'est PAS invalidé", code(forged) === "project_mismatch" && noEffect(s, before), `code=${code(forged)} T3=${avail(VT(3))} delta=${fmt(delta(s))}`);
    // autre root (UUID d'une tâche rattachée à un autre root) et UUID inconnu, contexte vrai : task_unavailable sans mutation
    const unknown = rpc(tok("read"), "bridget.lineage.read", { projectId: P, threadId: T, action: "show", taskId: uuid() });
    known("ZK-2", "UUID de tâche inconnu, contexte vrai : task_unavailable, 0 mutation", code(unknown) === "task_unavailable" && noEffect(s, before), `code=${code(unknown)} delta=${fmt(delta(s))}`);
    // contexte vrai + tâche connue de ce root retirée : SEUL ce fil est invalidé (autorité réelle conservée)
    const known3 = rpc(tok("read"), "bridget.lineage.read", ctx({ action: "show", taskId: U(3) }));
    check("ZK-3", "contexte VRAI + tâche connue retirée : task_unavailable, ce fil seul passe indisponible, racine intacte", code(known3) === "task_unavailable" && avail(VT(3)) === "0" && avail(T) === "1" && [1, 2, 4].every((n) => avail(VT(n)) === "1"), `T3=${avail(VT(3))} racine=${avail(T)}`);
    restore("store.json.r4task");
    const ok = list();
    check("ZK-4", "reprise complète : T3 redevient disponible", ok.value?.status === "ok" && avail(VT(3)) === "1", `T3=${avail(VT(3))}`);
  },
  genuine() {
    list();
    const base = flags();
    const recover = (id, name) => {
      const s = maxSeq();
      const ok = list();
      const [, sq, av] = rootState().split("|");
      // le filigrane racine égale le seq COURANT du magasin (l'étape hang incrémente le seq du magasin, pas la racine)
      check(id, `${name} : reprise COMPLÈTE au seq courant du magasin, racine redevient disponible, aucun fil en double`, ok.value?.status === "ok" && av === "1" && Number(sq) === storeSeq() && virtualCount() === Number(base.match(/n=(\d+)/)[1]), `filigrane=${sq} seq_magasin=${storeSeq()} racine=${av} ${fmt(delta(s))}`);
    };
    // 1. project_mismatch NATIF : contexte vrai pour T3, mais le daemon refuse (autre root dans le magasin)
    backup("store.json.r4gen");
    step("foreign-root", `${C}/proj2`);
    let s = maxSeq();
    let r = list();
    check("ZG-1", "VRAI contexte + project_mismatch NATIF (daemon, autre root) : la racine réelle passe indisponible", code(r) === "project_mismatch" && avail(T) === "0", `code=${code(r)} racine=${avail(T)} ${fmt(delta(s))}`);
    restore("store.json.r4gen");
    recover("ZG-1r", "project_mismatch natif");
    // 2. invalid_output : sortie du daemon illisible
    step("store-corrupt");
    s = maxSeq();
    r = list();
    check("ZG-2", "VRAI contexte + invalid_output (magasin illisible) : racine indisponible", ["invalid_output", "store_unavailable"].includes(code(r)) && avail(T) === "0", `code=${code(r)} racine=${avail(T)}`);
    restore("store.json.r4gen");
    recover("ZG-2r", "invalid_output");
    // 3. timeout : daemon plus lent que le délai du lecteur
    step("hang", "8000");
    s = maxSeq();
    r = list();
    check("ZG-3", "VRAI contexte + timeout (daemon lent 8 s > 6 s) : racine indisponible", code(r) === "timeout" && avail(T) === "0", `code=${code(r)} racine=${avail(T)}`);
    step("hang", "0");
    recover("ZG-3r", "timeout");
    // 4. journal_unavailable local reste sans effet (garde de portée inchangée)
    s = maxSeq();
    const j = rpc(tok("read"), "bridget.lineage.read", ctx({ action: "journal", taskId: U(4) }));
    check("ZG-4", "VRAI contexte + journal_unavailable : local, racine intacte (portée non élargie)", code(j) === "journal_unavailable" && avail(T) === "1" && delta(s).all === 0, `code=${code(j)}`);
  },
  deleted() {
    // binding_unavailable avec contexte vrai : fil hôte (même projet, non virtuel) supprimé après avoir été lié
    const T5 = uuid();
    const mk = rpc(tok("operate"), "orchestration.dispatchCommand", { type: "thread.create", commandId: uuid(), threadId: T5, projectId: P, title: "Fil hôte 5 recette 149", createdBy: "user", creationSource: "web", modelSelection: { instanceId: "codex", model: "gpt-5" }, runtimeMode: "full-access", interactionMode: "default", branch: null, worktreePath: null });
    const first = rpc(tok("read"), "bridget.lineage.read", { projectId: P, threadId: T5, action: "list" });
    const av1 = avail(T5);
    const del = rpc(tok("operate"), "orchestration.dispatchCommand", { type: "thread.delete", commandId: uuid(), threadId: T5 });
    const s = maxSeq();
    const after = rpc(tok("read"), "bridget.lineage.read", { projectId: P, threadId: T5, action: "list" });
    check("ZD-1", "OBSERVATION (limite fixture : le fil T5 n'est jamais lié, 1re lecture non ok) - fil hôte supprimé : binding_unavailable, racine principale intacte", code(after) === "binding_unavailable" && avail(T) === "1", `création=${JSON.stringify(mk).slice(0, 60)} liaison=${first.value?.status ?? code(first)} T5_avant=${av1} suppression=${JSON.stringify(del).slice(0, 60)} après=${code(after)} T5_après=${avail(T5)} racine_principale=${avail(T)} ${fmt(delta(s))}`);
  },
};

const want = process.argv[2] ?? "all";
const order = ["forgedread", "forgedroot", "forgedtask", "genuine", "deleted"];
for (const g of order) if (want === "all" || want === g) { console.log(`# groupe ${g}`); groups[g](); }
console.log(`TOTAL pass=${pass} fail=${fail} finding=${finding}`);
process.exit(fail === 0 && finding === 0 ? 0 : 1);
