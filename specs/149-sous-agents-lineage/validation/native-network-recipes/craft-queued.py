#!/usr/bin/python3
"""SIMULATION (r3) - rejoue, sur la base PRIVÉE d'une fixture dont le daemon est ARRÊTÉ, la transaction exacte d'admission.

`DelegationStore::insert_projected` (crates/bridget-daemon/src/delegation_lineage.rs:149) fait, en UNE transaction :
  1. INSERT d'UNE ligne dans native_delegations (task_id, owner_instance, request_id, canonical, payload) ;
  2. UPDATE native_delegation_projection_meta SET seq = seq + 1.
`handle_locked_with_project` n'écrit rien d'autre avant le tick (native_delegation.rs:452-504). Un arrêt brutal entre cette transaction et
le tick laisse donc exactement : une ligne `queued`, sans enfant, sans exécution, sans lien de flotte.

Usage : craft-queued.py <bridget.db> <task_id_source> <nouveau_request_id>
La ligne source est une ligne réelle produite par le daemon ; on en clone l'enveloppe (propriétaire, parent, droits, définition figée)
avec de NOUVEAUX identifiants (task_id, child, mission) et l'état `queued`. Rien d'autre n'est modifié. Sortie : JSON sur stdout.
"""
import json
import sqlite3
import sys
import time
import uuid

db, source, new_rid = sys.argv[1:4]
connection = sqlite3.connect(db, timeout=10)
connection.execute("PRAGMA busy_timeout=10000")
row = connection.execute("SELECT owner_instance, request_id, canonical, payload FROM native_delegations WHERE task_id=?", (source,)).fetchone()
if row is None:
    print(json.dumps({"error": "source_introuvable"}))
    sys.exit(2)
owner_instance, old_rid, canonical, payload = row
task = json.loads(payload)
now = int(time.time())
task.update({
    "task_id": str(uuid.uuid4()), "child": str(uuid.uuid4()), "mission": str(uuid.uuid4()),
    "state": "queued", "child_instance": None, "result": None, "error": None, "result_sent": False,
    "cleanup_done": False, "failure_sent": False, "mission_deadline_at": None,
    "created_at": now, "updated_at": now, "started_at": None, "completed_at": None,
})
text = json.dumps(task, separators=(",", ":"), ensure_ascii=False)
old_b, new_b = old_rid.encode(), new_rid.encode()
if canonical.count(old_b) != 1:
    print(json.dumps({"error": "canonical_ambigu", "count": canonical.count(old_b)}))
    sys.exit(3)
new_canonical = canonical.replace(old_b, new_b)
# le request_id vit aussi dans payload.request : même remplacement exact (une occurrence)
if text.count(old_rid) < 1:
    print(json.dumps({"error": "request_id_absent_du_payload"}))
    sys.exit(4)
text = text.replace(f'"request_id":"{old_rid}"', f'"request_id":"{new_rid}"')
with connection:
    connection.execute("INSERT INTO native_delegations(task_id, owner_instance, request_id, canonical, payload) VALUES(?,?,?,?,?)",
                       (task["task_id"], owner_instance, new_rid, new_canonical, text))
    connection.execute("UPDATE native_delegation_projection_meta SET seq = seq + 1 WHERE singleton = 1")
print(json.dumps({"task_id": task["task_id"], "child": task["child"], "mission": task["mission"], "owner_instance": owner_instance,
                  "parent_task_id": task["parent_task_id"], "state": task["state"]}))
