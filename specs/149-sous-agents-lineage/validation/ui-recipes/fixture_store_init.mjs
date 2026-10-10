#!/usr/bin/env node
// Recette UI 149 — magasin fixture initial, données synthétiques NOMMÉES.
// Ces tâches ne viennent d'aucun modèle natif : elles prouvent le rendu, les
// scopes et les refus du chemin T3, jamais SC001 end-to-end ni T037.
// Usage : node fixture_store_init.mjs <store.json> <project_root_abs>
import { writeFileSync, mkdirSync, chmodSync } from "node:fs";
import { dirname } from "node:path";

const [, , storePath, projectRoot] = process.argv;
if (!storePath || !projectRoot || !storePath.startsWith("/") || !projectRoot.startsWith("/")) {
  process.stderr.write("usage: fixture_store_init.mjs <store.json absolu> <project_root absolu>\n");
  process.exit(2);
}
if (!storePath.includes("bridget149-ui")) {
  process.stderr.write("refus : le magasin doit vivre sous /Users/moi/.cache/bridget149-ui.*\n");
  process.exit(2);
}

// UUID déterministes de recette (préfixe 149 — jamais des identifiants réels)
const U = (n) => `00000000-0000-4000-8000-${String(n).padStart(12, "0")}`;
const ROOT_THREAD = U(101); // fil T3 attesté qui demande la projection
const NOW = Math.floor(Date.now() / 1000);

const store = {
  _fixture: "recette-ui-149 données synthétiques nommées — pas des tâches natives réelles",
  project_root: projectRoot,
  generation: U(900),
  seq: 1,
  root_owner_agent_id: U(901),
  shared_threads: [
    { thread_id: ROOT_THREAD, title: "Fil hôte recette 149", last_activity_at: NOW },
  ],
  shared_seq: 1,
  tasks: [
    {
      task_id: U(1), parent_task_id: null, parent_agent_id: U(901), child_agent_id: U(911),
      child_instance_id: null, created_at: NOW, updated_at: NOW, started_at: NOW, completed_at: null,
      agent_type: "claude", execution_protocol: "claude_stream_json", model: "glm-5.3-flash",
      effort: null, cwd: projectRoot, posture: "development",
      title: "[recette149] racine déléguée — donnée synthétique",
      status: "working", error: null, journal_available: true,
    },
    {
      task_id: U(2), parent_task_id: U(1), parent_agent_id: U(911), child_agent_id: U(912),
      child_instance_id: null, created_at: NOW + 1, updated_at: NOW + 1, started_at: NOW + 1, completed_at: null,
      agent_type: "claude", execution_protocol: "claude_stream_json", model: "glm-5.3-flash",
      effort: null, cwd: projectRoot, posture: "development",
      title: "[recette149] enfant imbriqué — donnée synthétique",
      status: "working", error: null, journal_available: true,
    },
    {
      task_id: U(3), parent_task_id: null, parent_agent_id: U(901), child_agent_id: U(913),
      child_instance_id: null, created_at: NOW + 2, updated_at: NOW + 3, started_at: NOW + 2, completed_at: NOW + 3,
      agent_type: "claude", execution_protocol: "claude_stream_json", model: "glm-5.3-flash",
      effort: "low", cwd: projectRoot, posture: "discovery",
      title: "[recette149] racine terminale avec résultat",
      status: "result_available", error: null, journal_available: true,
    },
    {
      task_id: U(4), parent_task_id: null, parent_agent_id: U(901), child_agent_id: U(914),
      child_instance_id: null, created_at: NOW + 4, updated_at: NOW + 4, started_at: NOW + 4, completed_at: NOW + 4,
      agent_type: "claude", execution_protocol: "claude_stream_json", model: "glm-5.3-flash",
      effort: null, cwd: projectRoot, posture: "development",
      title: "[recette149] racine échouée",
      status: "failed", error: "provider_permission_denied", journal_available: false,
    },
  ],
  results: {
    [U(3)]: "Résultat de recette 149 (donnée synthétique nommée) : écriture allowed/write-ok.md simulée.",
  },
  journals: {
    [U(1)]: [
      { v: 1, seq: 1, ts: "2026-10-10T12:00:00Z", session_id: "fixture-s149-t1", event: "mission_started", payload: { model: "glm-5.3-flash", synthétique: true } },
      { v: 1, seq: 2, ts: "2026-10-10T12:00:05Z", session_id: "fixture-s149-t1", event: "tool_use", message_id: "m1", payload: { tool: "Write", path: "allowed/write-ok.md" } },
    ],
    [U(2)]: [
      { v: 1, seq: 1, ts: "2026-10-10T12:00:10Z", session_id: "fixture-s149-t2", event: "mission_started", payload: { synthétique: true } },
    ],
    [U(3)]: [
      { v: 1, seq: 1, ts: "2026-10-10T12:01:00Z", session_id: "fixture-s149-t3", event: "mission_started", payload: { synthétique: true } },
      { v: 1, seq: 2, ts: "2026-10-10T12:01:10Z", session_id: "fixture-s149-t3", event: "result_published", payload: { bytes: 74 } },
    ],
  },
  journal_next_seq: { [U(1)]: 2, [U(2)]: 1, [U(3)]: 2 },
  gaps: {},
  cancel_receipts: {},
};

mkdirSync(dirname(storePath), { recursive: true, mode: 0o700 });
writeFileSync(storePath, JSON.stringify(store, null, 2), { mode: 0o600 });
chmodSync(dirname(storePath), 0o700);
process.stdout.write(`magasin fixture écrit : ${storePath} (root=${projectRoot}, thread=${ROOT_THREAD})\n`);
process.stdout.write(`ROOT_THREAD_ID=${ROOT_THREAD}\n`);
