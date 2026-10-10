#!/usr/bin/env node
// Recette UI 149 — contrôleurs de scénario : mutations nommées du magasin fixture.
// Chaque étape est explicite et identifiée "recette149" ; aucune ne simule un
// modèle natif réel (T037 hors périmètre). Usage :
//   node scenario_step.mjs <store.json> <étape> [args...]
// Étapes : journal | nested-status | result | cancelled | generation | gap |
//          journal-off | store-corrupt | reset
import { readFileSync, writeFileSync, renameSync } from "node:fs";

const [, , storePath, step, ...rest] = process.argv;
if (!storePath?.startsWith("/") || !step) {
  process.stderr.write("usage: scenario_step.mjs <store.json> <étape> [args...]\n");
  process.exit(2);
}
const U = (n) => `00000000-0000-4000-8000-${String(n).padStart(12, "0")}`;
const T1 = U(1), T2 = U(2), T3 = U(3), T4 = U(4);

const store = JSON.parse(readFileSync(storePath, "utf8"));
const now = Math.floor(Date.now() / 1000);
const notes = [];

function bump() { store.seq += 1; }

function journalAppend(taskId, event, payload) {
  const list = store.journals[taskId] ?? (store.journals[taskId] = []);
  const seq = (store.journal_next_seq[taskId] ?? 0) + 1;
  list.push({ v: 1, seq, ts: new Date().toISOString(), session_id: `fixture-s149-${taskId.slice(-4)}`, event, payload });
  store.journal_next_seq[taskId] = seq;
  notes.push(`journal ${taskId.slice(-4)} +seq${seq} ${event}`);
}

switch (step) {
  case "journal": {
    // progression live de l'enfant racine T1
    journalAppend(T1, "tool_use", { tool: rest[0] ?? "Write", path: rest[1] ?? "allowed/write-ok.md" });
    const t1 = store.tasks.find((t) => t.task_id === T1);
    t1.updated_at = Math.max(now, t1.updated_at + 1); // le fait de tâche change à coup sûr
    bump();
    break;
  }
  case "tick": {
    // journal seul : une ligne de journal et seq+1, AUCUN fait de tâche modifié (r3)
    journalAppend(rest[0] ?? T1, "tool_use", { tool: "Read", path: "tick-only" });
    bump();
    break;
  }
  case "remove-task": {
    // retire une tâche du magasin sans resynchroniser T3 (task_unavailable côté lecture) (r3)
    const id = rest[0];
    const before = store.tasks.length;
    store.tasks = store.tasks.filter((t) => t.task_id !== id);
    bump();
    notes.push(`tâche ${id?.slice(-4)} retirée (${before} -> ${store.tasks.length})`);
    break;
  }
  case "foreign-root": {
    // le magasin devient celui d'un AUTRE root (proj2) avec une seule tâche X ; sert à lier un fil virtuel
    // à la racine étrangère. À restaurer ensuite depuis la copie du magasin (cp). (r3)
    const foreignRoot = rest[0];
    if (!foreignRoot?.startsWith("/")) { process.stderr.write("foreign-root exige un chemin absolu\n"); process.exit(2); }
    store.project_root = foreignRoot;
    store.tasks = [{
      task_id: U(5000), parent_task_id: null, parent_agent_id: store.root_owner_agent_id, child_agent_id: U(5001),
      child_instance_id: null, created_at: now, updated_at: now, started_at: now, completed_at: null,
      agent_type: "claude", execution_protocol: "claude_stream_json", model: "glm-5.3-flash", effort: null,
      cwd: foreignRoot, posture: "development", title: "[recette149] tâche du root ÉTRANGER", status: "working", error: null, journal_available: false,
    }];
    bump();
    notes.push("magasin = root étranger, 1 tâche X (…5000)");
    break;
  }
  case "title": {
    // renomme une tâche (titre long pour la recette de mise en page) (r3)
    const t = store.tasks.find((x) => x.task_id === (rest[0] ?? T2));
    t.title = rest.slice(1).join(" ");
    t.updated_at = Math.max(now, t.updated_at + 1);
    bump();
    break;
  }
  case "nested-status": {
    // l'enfant imbriqué T2 change d'état visible (waiting_for_children)
    const t2 = store.tasks.find((t) => t.task_id === T2);
    t2.status = rest[0] ?? "waiting_for_children";
    t2.updated_at = now;
    journalAppend(T2, "status", { status: t2.status });
    bump();
    break;
  }
  case "result": {
    // la racine T1 publie son résultat
    const t1 = store.tasks.find((t) => t.task_id === T1);
    t1.status = "result_available";
    t1.completed_at = now;
    t1.updated_at = now;
    store.results[T1] = "Résultat racine T1 (recette149, synthétique) : 2 écritures autorisées, 1 refus Bash.";
    journalAppend(T1, "result_published", { bytes: 60 });
    bump();
    break;
  }
  case "cancelled": {
    // achève l'annulation commencée par lineage cancel (cancelling → cancelled)
    const taskId = rest[0] ?? T2;
    const t = store.tasks.find((x) => x.task_id === taskId);
    if (!t) { process.stderr.write("tâche inconnue\n"); process.exit(2); }
    t.status = "cancelled";
    t.completed_at = now;
    t.updated_at = now;
    journalAppend(taskId, "cancelled", { by: "recette149" });
    bump();
    break;
  }
  case "generation": {
    // nouvelle génération du magasin → resync exigé côté lecteur
    store.generation = U(999);
    bump();
    notes.push("génération changée → resync");
    break;
  }
  case "gap": {
    // lacune explicite : les seq 3..5 ont été perdues (purge processus)
    store.gaps[T1] = { from_seq: 3, to_seq: 5, reason: "recette149 : segments purgés avec le processus" };
    // saute volontairement journal_next_seq à 6 sans fournir 3..5
    store.journal_next_seq[T1] = Math.max(store.journal_next_seq[T1] ?? 0, 6);
    journalAppend(T1, "after_gap", { note: "reprise après lacune" });
    bump();
    break;
  }
  case "journal-off": {
    // indisponibilité journal (cleanup a supprimé les segments)
    const taskId = rest[0] ?? T2;
    const t = store.tasks.find((x) => x.task_id === taskId);
    t.journal_available = false;
    delete store.journals[taskId];
    store.journal_next_seq[taskId] = 0;
    bump();
    notes.push(`journal indisponible ${taskId.slice(-4)}`);
    break;
  }
  case "seed": {
    // N tâches racines terminales synthétiques (pagination > 100 : S149-27)
    const n = Number(rest[0] ?? 130);
    for (let i = 1; i <= n; i++) {
      store.tasks.push({
        task_id: U(2000 + i), parent_task_id: null, parent_agent_id: U(901), child_agent_id: U(3000 + i),
        child_instance_id: null, created_at: now + 10 + i, updated_at: now + 10 + i, started_at: now + 10 + i, completed_at: now + 10 + i,
        agent_type: "claude", execution_protocol: "claude_stream_json", model: "glm-5.3-flash", effort: null,
        cwd: store.project_root, posture: "discovery", title: `[recette149] seed ${i}`,
        status: "cancelled", error: null, journal_available: false,
      });
    }
    bump();
    notes.push(`${n} tâches seed ajoutées (total ${store.tasks.length})`);
    break;
  }
  case "unseed": {
    store.tasks = store.tasks.filter((t) => Number(t.task_id.slice(-12)) < 2000);
    bump();
    notes.push(`seed retiré (total ${store.tasks.length})`);
    break;
  }
  case "cursor-bump": {
    // les N prochains appels list avec curseur subissent une mutation entre pages (snapshot_changed)
    store.bump_on_cursor = Number(rest[0] ?? 1);
    notes.push(`bump_on_cursor=${store.bump_on_cursor}`);
    break;
  }
  case "hang": {
    // daemon lent : chaque appel dort N ms (0 = normal) ; prouve le timeout du lecteur
    store.hang_ms = Number(rest[0] ?? 0);
    bump();
    notes.push(`hang_ms=${store.hang_ms}`);
    break;
  }
  case "store-corrupt": {
    // indisponibilité technique du magasin (exit 3 côté CLI) — réversible
    writeFileSync(storePath + ".broken", "not json {", { mode: 0o600 });
    renameSync(storePath + ".broken", storePath);
    notes.push("magasin corrompu volontairement (store_unavailable attendu)");
    process.stdout.write("store corrompu — relancez reset pour restaurer\n");
    process.exit(0);
  }
  case "reset": {
    // réécrit la première page du magasin depuis l'init (root inchangé)
    process.stderr.write("reset : réexécutez fixture_store_init.mjs\n");
    process.exit(2);
  }
  default:
    process.stderr.write(`étape inconnue : ${step}\n`);
    process.exit(2);
}

const tmp = storePath + ".tmp";
writeFileSync(tmp, JSON.stringify(store, null, 2), { mode: 0o600 });
renameSync(tmp, storePath);
process.stdout.write(`étape ${step} appliquée (seq=${store.seq})${notes.length ? " : " + notes.join("; ") : ""}\n`);
