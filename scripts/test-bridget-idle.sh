#!/usr/bin/env bash
# Harnais bridget-idle : partition + bornes de tour + backlog Git local + contrôles positifs.
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
idle="${root_dir}/scripts/bridget-idle.py"
system_python="/usr/bin/python3"
fixture_root="$(mktemp -d -t bridget-idle-test.XXXXXX)"
cleanup() { rm -rf "$fixture_root"; }
trap cleanup EXIT

[[ -f "$idle" ]] || { echo "absent: $idle" >&2; exit 1; }

"$system_python" - "$idle" "$fixture_root" <<'PY'
import hashlib, importlib.util, json, pathlib, sys, os, sqlite3, subprocess, time

idle_path = pathlib.Path(sys.argv[1])
fixture = pathlib.Path(sys.argv[2])
spec = importlib.util.spec_from_file_location("bridget_idle", idle_path)
mod = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(mod)

agents = [
    {"name": "alice", "state": "connected", "domain": "bridget", "last_seen_secs": 10},
    {"name": "bob", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 5},
    {"name": "cursor10-like", "state": "connected", "domain": "cursor10-lot", "last_seen_secs": 3},
    {"name": "cursorbridget-like", "state": "busy", "domain": "bridget", "last_seen_secs": 1},
    {"name": "relec6-like", "state": "busy", "domain": "relec6-lot", "last_seen_secs": 2},
    {"name": "bridget", "state": "connected", "domain": "bridget", "last_seen_secs": 0},
    # La mission ne suffit pas : la borne du dernier tour tranche l'activité.
    {"name": "healthy-consumer", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "frozen-consumer", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "freshly-finished", "state": "connected", "domain": "bridget", "transport": "acp", "last_seen_secs": 2},
    {"name": "resumed-consumer", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "corrupt-journal", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "missing-journal", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "busy-without-journal", "state": "busy", "domain": "bridget", "transport": "ssh-unix", "last_seen_secs": 2},
    {"name": "contradictory-consumer", "state": "busy", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "ssh-open-only", "state": "connected", "domain": "bridget", "transport": "ssh-unix", "last_seen_secs": 2},
    {"name": "acp-anomaly-live", "state": "connected", "domain": "bridget", "transport": "acp", "last_seen_secs": 2},
    {"name": "ambiguous-error", "state": "connected", "domain": "bridget", "transport": "acp", "last_seen_secs": 2},
    {"name": "timeout-sans-kind", "state": "connected", "domain": "bridget", "transport": "codex_app_server", "last_seen_secs": 2},
    {"name": "unsafe-stop", "state": "connected", "domain": "bridget", "transport": "acp", "last_seen_secs": 2},
    {"name": "unsafe-error", "state": "connected", "domain": "bridget", "transport": "acp", "last_seen_secs": 2},
    {"name": "codex-pasted", "agent_type": "codex", "host": "fixture-host", "state": "connected", "domain": "bridget", "transport": "tmux", "location": "codex-pasted:1.1", "last_seen_secs": 2},
    {"name": "codex-visible", "agent_type": "codex", "host": "fixture-host", "state": "connected", "domain": "bridget", "transport": "tmux", "location": "codex-visible:1.1", "last_seen_secs": 2},
    {"name": "codex-consumed", "agent_type": "codex", "host": "fixture-host", "state": "connected", "domain": "bridget", "transport": "tmux", "location": "codex-consumed:1.1", "last_seen_secs": 2},
    {"name": "claude-enqueued", "agent_type": "claude", "host": "fixture-host", "state": "connected", "domain": "bridget", "transport": "tmux", "location": "claude-enqueued:1.1", "last_seen_secs": 2},
    {"name": "claude-consumed", "agent_type": "claude", "host": "fixture-host", "state": "connected", "domain": "bridget", "transport": "tmux", "location": "claude-consumed:1.1", "last_seen_secs": 2},
    {"name": "remote-pasted", "agent_type": "codex", "host": "autre-hote", "state": "connected", "domain": "bridget", "transport": "tmux", "location": "remote-pasted:1.1", "last_seen_secs": 2},
    {"name": "cartae0", "agent_type": "codex", "host": "fixture-host", "state": "connected", "domain": "bridget", "transport": "tmux", "location": "cartae0:1.1", "last_seen_secs": 2},
    {"name": "rc7", "agent_type": "codex", "host": "fixture-host", "state": "connected", "domain": "bridget", "transport": "tmux", "location": "rc7:1.1", "last_seen_secs": 2},
]
occupied = {
    "bob",
    "healthy-consumer",
    "frozen-consumer",
    "freshly-finished",
    "resumed-consumer",
    "corrupt-journal",
    "missing-journal",
    "busy-without-journal",
    "contradictory-consumer",
    "ssh-open-only",
    "acp-anomaly-live",
    "ambiguous-error",
    "timeout-sans-kind",
    "unsafe-stop",
    "unsafe-error",
    "codex-pasted",
    "codex-consumed",
    "claude-enqueued",
    "claude-consumed",
    "remote-pasted",
}
exclude = {"bridget", "fable", "poucave", "sol", "maicie"}
daemon = {a["name"] for a in agents}

journal_root = fixture / "journals"

def write_journal(agent, events, *, invalid_line=False):
    directory = journal_root / agent
    directory.mkdir(parents=True)
    path = directory / "2026-08-25.jsonl"
    with path.open("w", encoding="utf-8") as stream:
        for event in events:
            stream.write(json.dumps(event, ensure_ascii=False) + "\n")
        if invalid_line:
            stream.write('{"v":1,"seq":999,"event":"turn_start"\n')

def boundary(seq, event, message_id, payload=None):
    return {
        "v": 1,
        "seq": seq,
        "ts": f"2026-08-25T10:00:{seq:02d}Z",
        "session_id": "session-fixture",
        "event": event,
        "message_id": message_id,
        "payload": payload or {},
    }

write_journal("bob", [boundary(1, "turn_start", "bob-tour")])
write_journal(
    "healthy-consumer",
    [
        boundary(1, "turn_start", "healthy-tour"),
        boundary(2, "provider_request", "healthy-tour", {"state": "pending"}),
    ],
)
write_journal(
    "frozen-consumer",
    [
        boundary(1, "turn_start", "frozen-tour"),
        boundary(
            2,
            "error",
            "frozen-tour",
            {"reason": "échéance fournisseur dépassée", "terminal_kind": "turn_failed"},
        ),
    ],
)
write_journal(
    "freshly-finished",
    [
        boundary(1, "turn_start", "fresh-tour"),
        boundary(2, "turn_end", "fresh-tour", {"stop_reason": "inconnu"}),
    ],
)
write_journal(
    "resumed-consumer",
    [
        boundary(1, "turn_start", "old-tour"),
        boundary(2, "turn_end", "old-tour", {"stop_reason": "end_turn"}),
        boundary(3, "turn_start", "new-tour"),
        # Un terminal tardif de l'ancien message ne ferme pas le nouveau tour.
        boundary(4, "error", "old-tour", {"reason": "retard ancien"}),
    ],
)
write_journal(
    "corrupt-journal",
    [boundary(1, "turn_start", "corrupt-tour")],
    invalid_line=True,
)
write_journal(
    "contradictory-consumer",
    [
        boundary(1, "turn_start", "contradictory-tour"),
        boundary(2, "turn_end", "contradictory-tour", {"stop_reason": "end_turn"}),
    ],
)
write_journal("ssh-open-only", [boundary(1, "turn_start", "ssh-tour")])
write_journal(
    "acp-anomaly-live",
    [
        boundary(1, "turn_start", "message-live"),
        boundary(2, "error", "message-live", {"reason": "notification ACP inconnue: vendor/future"}),
        boundary(3, "update", "message-live", {"kind": "text", "content": "le tour continue"}),
    ],
)
write_journal(
    "ambiguous-error",
    [
        boundary(1, "turn_start", "message-ambiguous"),
        boundary(2, "error", "message-ambiguous", {"reason": "ancien terminal ou anomalie"}),
    ],
)
# Tour tué par échéance SANS terminal_kind (journaux pré-bornes / binaire non
# relancé) : doit quand même sortir des OCCUPES — sinon la ronde ment.
write_journal(
    "timeout-sans-kind",
    [
        boundary(1, "turn_start", "timeout-tour"),
        boundary(
            2,
            "error",
            "timeout-tour",
            {"reason": "échéance Codex dépassée"},
        ),
    ],
)
unsafe_detail = "provider\x1b[2J\refface\u202ele diagnostic"
if not all(control in unsafe_detail for control in ("\x1b", "\r", "\u202e")):
    raise SystemExit("CONTROLE POSITIF RATE: les contrôles dangereux manquent à la fixture")
write_journal(
    "unsafe-stop",
    [
        boundary(1, "turn_start", "unsafe-stop-tour"),
        boundary(2, "turn_end", "unsafe-stop-tour", {"stop_reason": unsafe_detail}),
    ],
)
write_journal(
    "unsafe-error",
    [
        boundary(1, "turn_start", "unsafe-error-tour"),
        boundary(
            2,
            "error",
            "unsafe-error-tour",
            {"reason": unsafe_detail, "terminal_kind": "turn_failed"},
        ),
    ],
)
write_journal(
    "codex-pasted",
    [
        {
            **boundary(1, "turn_start", "mcp-49971-6a900824-a6"),
            "ts": "2026-08-27T09:49:24Z",
            "payload": {"body": "[Pasted Content 4925 chars]"},
        }
    ],
)
write_journal(
    "codex-consumed",
    [
        {
            **boundary(1, "turn_start", "mcp-codex-consumed"),
            "ts": "2026-08-27T09:49:24Z",
            "payload": {"body": "mandat codex sain"},
        }
    ],
)
write_journal(
    "codex-visible",
    [
        {
            **boundary(1, "turn_start", "mcp-codex-visible"),
            "ts": "2026-08-27T09:49:24Z",
            "payload": {"body": "mandat visible en texte normal"},
        }
    ],
)
write_journal(
    "claude-enqueued",
    [
        {
            **boundary(1, "turn_start", "mcp-claude-enqueued"),
            "ts": "2026-08-27T10:07:09Z",
            "payload": {"body": "mandat Claude seulement en file"},
        }
    ],
)
write_journal(
    "claude-consumed",
    [
        {
            **boundary(1, "turn_start", "mcp-claude-consumed"),
            "ts": "2026-08-27T10:07:09Z",
            "payload": {"body": "mandat Claude pris"},
        }
    ],
)
write_journal(
    "cartae0",
    [
        {
            **boundary(1, "turn_start", "mcp-cartae0-steering"),
            "ts": "2026-08-27T10:04:24Z",
            "payload": {"body": "remise pendant un tour Codex déjà actif"},
        }
    ],
)
write_journal(
    "rc7",
    [
        {
            **boundary(1, "turn_start", "mcp-rc7-idle"),
            "ts": "2026-08-27T10:40:17Z",
            "payload": {"body": "mandat remis après la fin du tour"},
        }
    ],
)
write_journal(
    "remote-pasted",
    [
        {
            **boundary(1, "turn_start", "mcp-remote-pasted"),
            "ts": "2026-08-27T09:49:24Z",
            "payload": {"body": "mandat distant"},
        }
    ],
)
for unsafe_agent, detail_key in (("unsafe-stop", "stop_reason"), ("unsafe-error", "reason")):
    path = journal_root / unsafe_agent / "2026-08-25.jsonl"
    terminal = json.loads(path.read_text(encoding="utf-8").splitlines()[-1])
    if terminal["payload"][detail_key] != unsafe_detail:
        raise SystemExit(f"CONTROLE POSITIF RATE: détail dangereux absent de {unsafe_agent}")
print("controle_positif_details_dangereux_presents: OK")

turns = mod.read_turn_observations(journal_root, daemon)
if turns["healthy-consumer"]["state"] != "open":
    raise SystemExit(f"tour sain non ouvert: {turns['healthy-consumer']}")
if turns["frozen-consumer"]["state"] != "ended":
    raise SystemExit(f"fin sans reprise non détectée: {turns['frozen-consumer']}")
if turns["resumed-consumer"]["state"] != "open":
    raise SystemExit(f"reprise non prioritaire: {turns['resumed-consumer']}")
if turns["acp-anomaly-live"]["state"] != "open":
    raise SystemExit(f"une error ACP non terminale a fermé le tour: {turns['acp-anomaly-live']}")
if turns["ambiguous-error"]["state"] != "unknown" or turns["ambiguous-error"].get("reason") != "error-terminalite-non-attestee":
    raise SystemExit(f"une ancienne error ambiguë a produit une certitude: {turns['ambiguous-error']}")
if turns["timeout-sans-kind"]["state"] != "ended":
    raise SystemExit(
        f"échéance sans terminal_kind doit quand même fermer le tour: {turns['timeout-sans-kind']}"
    )
for unsafe_agent, terminal_kind in (("unsafe-stop", "turn_completed"), ("unsafe-error", "turn_failed")):
    if turns[unsafe_agent].get("state") != "ended" or turns[unsafe_agent].get("terminal_kind") != terminal_kind:
        raise SystemExit(f"code terminal fermé absent pour {unsafe_agent}: {turns[unsafe_agent]}")
if turns["corrupt-journal"]["state"] != "unknown" or not turns["corrupt-journal"]["reason"].startswith("journal-json-invalide:"):
    raise SystemExit(f"journal corrompu conclu à tort: {turns['corrupt-journal']}")
if turns["missing-journal"]["state"] != "unknown" or "absent" not in turns["missing-journal"]["reason"]:
    raise SystemExit(f"journal absent conclu à tort: {turns['missing-journal']}")
print("lecture_bornes_tour_et_incertitudes: OK")

# Prise réelle : le journal atteste l'injection, les traces natives attestent
# (ou n'attestent pas) la consommation. Le contenu sentinelle ne doit jamais
# apparaître dans la projection opérateur.
trace_root = fixture / "intake-traces"
trace_root.mkdir()
secret_content = "SECRET-CONTENU-NE-DOIT-PAS-ETRE-PROJETE"


def write_trace(name, records):
    path = trace_root / f"{name}.jsonl"
    with path.open("w", encoding="utf-8") as stream:
        for record in records:
            stream.write(json.dumps(record, ensure_ascii=False) + "\n")
    return path


codex_pasted_trace = write_trace(
    "codex-pasted",
    [
        {
            "timestamp": "2026-08-25T13:13:16.390Z",
            "type": "session_meta",
            "payload": {
                "id": "session-pasted",
                "cwd": "/fixture/codex-pasted",
                "thread_source": "user",
            },
        },
        {
            "timestamp": "2026-08-27T09:48:59.841Z",
            "type": "event_msg",
            "payload": {"type": "task_complete", "turn_id": "ancien-tour"},
        },
        {
            "timestamp": "2026-08-27T09:48:59.900Z",
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "assistant",
                "content": [{"type": "output_text", "text": secret_content}],
            },
        },
    ],
)
codex_consumed_trace = write_trace(
    "codex-consumed",
    [
        {
            "timestamp": "2026-08-25T13:13:16.390Z",
            "type": "session_meta",
            "payload": {
                "id": "session-consumed",
                "cwd": "/fixture/codex-consumed",
                "thread_source": "user",
            },
        },
        {
            "timestamp": "2026-08-27T10:01:24.444Z",
            "type": "event_msg",
            "payload": {"type": "task_started", "turn_id": "tour-consumed"},
        },
        {
            "timestamp": "2026-08-27T10:01:24.857Z",
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "user",
                "content": [
                    {
                        "type": "input_text",
                        "text": (
                            "💬 bridget → codex-consumed "
                            "(reply=no, id=mcp-codex-consumed)\n"
                            f"{secret_content}"
                        ),
                    }
                ],
            },
        },
    ],
)
claude_enqueued_trace = write_trace(
    "claude-enqueued",
    [
        {"type": "mode", "sessionId": "claude-enqueued"},
        {
            "timestamp": "2026-08-27T10:07:10.100Z",
            "type": "user",
            "sessionId": "claude-enqueued",
            "message": {
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "content": (
                            "💬 bridget → claude-enqueued "
                            "(reply=no, id=mcp-claude-enqueued)"
                        ),
                    }
                ],
            },
        },
        {
            "timestamp": "2026-08-27T10:07:10.670Z",
            "type": "queue-operation",
            "operation": "enqueue",
            "sessionId": "claude-enqueued",
            "content": (
                "💬 bridget → claude-enqueued "
                "(reply=no, id=mcp-claude-enqueued)"
            ),
        },
    ],
)
cartae0_steering_trace = write_trace(
    "cartae0-steering",
    [
        {
            "timestamp": "2026-08-27T07:58:08Z",
            "type": "session_meta",
            "payload": {
                "id": "session-cartae0",
                "cwd": "/fixture/cartae0",
                "thread_source": "user",
            },
        },
        {
            "timestamp": "2026-08-27T10:03:53.760Z",
            "type": "event_msg",
            "payload": {"type": "task_started", "turn_id": "tour-actif"},
        },
    ],
)
rc7_idle_trace = write_trace(
    "rc7-idle",
    [
        {
            "timestamp": "2026-08-27T09:00:00Z",
            "type": "session_meta",
            "payload": {
                "id": "session-rc7",
                "cwd": "/fixture/rc7",
                "thread_source": "user",
            },
        },
        {
            "timestamp": "2026-08-27T10:39:00Z",
            "type": "event_msg",
            "payload": {"type": "task_started", "turn_id": "tour-termine"},
        },
        {
            "timestamp": "2026-08-27T10:39:46.868Z",
            "type": "event_msg",
            "payload": {"type": "task_complete", "turn_id": "tour-termine"},
        },
    ],
)
claude_consumed_trace = write_trace(
    "claude-consumed",
    [
        {"type": "mode", "sessionId": "claude-consumed"},
        {
            "timestamp": "2026-08-27T10:07:10.670Z",
            "type": "queue-operation",
            "operation": "enqueue",
            "sessionId": "claude-consumed",
            "content": (
                "💬 bridget → claude-consumed "
                "(reply=no, id=mcp-claude-consumed)"
            ),
        },
        {
            "timestamp": "2026-08-27T10:08:00.083Z",
            "type": "queue-operation",
            "operation": "remove",
            "sessionId": "claude-consumed",
            "content": (
                "💬 bridget → claude-consumed "
                "(reply=no, id=mcp-claude-consumed)\n"
                f"{secret_content}"
            ),
        },
    ],
)
claude_direct_trace = write_trace(
    "claude-direct",
    [
        {"type": "mode", "sessionId": "claude-direct"},
        {
            "timestamp": "2026-08-27T10:07:11.553Z",
            "type": "user",
            "sessionId": "claude-direct",
            "message": {
                "role": "user",
                "content": (
                    "💬 bridget → claude-direct "
                    "(reply=no, id=mcp-claude-direct)"
                ),
            },
        },
    ],
)


def codex_correlation_trace(name, first_line, body=""):
    return write_trace(
        name,
        [
            {
                "timestamp": "2026-08-27T10:00:00Z",
                "type": "session_meta",
                "payload": {
                    "id": f"session-{name}",
                    "cwd": f"/fixture/{name}",
                    "thread_source": "user",
                },
            },
            {
                "timestamp": "2026-08-27T10:01:01Z",
                "type": "response_item",
                "payload": {
                    "type": "message",
                    "role": "user",
                    "content": [
                        {
                            "type": "input_text",
                            "text": first_line + (f"\n{body}" if body else ""),
                        }
                    ],
                },
            },
        ],
    )


codex_exact_trace = codex_correlation_trace(
    "codex-id-exact",
    "💬 bridget → codex-id-exact (reply=no, id=mcp-abc)",
)
codex_prefix_trace = codex_correlation_trace(
    "codex-id-prefix",
    "💬 bridget → codex-id-prefix (reply=no, id=mcp-abc999)",
)
codex_mention_trace = codex_correlation_trace(
    "codex-id-mention",
    "💬 bridget → codex-id-mention (reply=no, id=mcp-autre)",
    "Le diagnostic mentionne id=mcp-abc sans remettre ce mandat.",
)

trace_paths = {
    "codex-pasted": codex_pasted_trace,
    "codex-visible": codex_pasted_trace,
    "codex-consumed": codex_consumed_trace,
    "claude-enqueued": claude_enqueued_trace,
    "claude-consumed": claude_consumed_trace,
    "remote-pasted": codex_consumed_trace,
    "cartae0": cartae0_steering_trace,
    "rc7": rc7_idle_trace,
}
specimen_now = int(mod._epoch_from_iso8601("2026-08-27T10:01:25Z"))
specimen = mod.read_intake_observations(
    agents,
    {"codex-pasted", "codex-visible", "codex-consumed"},
    turns,
    now=specimen_now,
    intake_after_secs=60,
    local_host="fixture-host",
    codex_trace_root=trace_root,
    claude_trace_root=trace_root,
    tmux_bin="tmux-inutilise",
    trace_paths=trace_paths,
)
if specimen["codex-pasted"]["state"] != "MANDAT_NON_SOUMIS":
    raise SystemExit(f"specimen collé non détecté: {specimen['codex-pasted']}")
if specimen["codex-pasted"]["age_secs"] != 721:
    raise SystemExit(f"âge du specimen 720,857 s perdu: {specimen['codex-pasted']}")
if specimen["codex-pasted"]["records_read"] != 3 or specimen["codex-pasted"]["matching_acceptances"] != 0:
    raise SystemExit(f"instrument muet ou acceptation inventée: {specimen['codex-pasted']}")
print("prise_specimen_720s_non_soumise: OK")
if specimen["codex-visible"]["state"] != "MANDAT_NON_SOUMIS":
    raise SystemExit(f"contenu sans marqueur visuel non détecté: {specimen['codex-visible']}")
print("prise_sans_marqueur_visuel_non_soumise: OK")
if specimen["codex-consumed"]["state"] != "PRISE_ACCEPTEE":
    raise SystemExit(f"prise Codex saine accusée à tort: {specimen['codex-consumed']}")
if specimen["codex-consumed"]["matching_acceptances"] != 1:
    raise SystemExit(f"cardinal Codex sain inattendu: {specimen['codex-consumed']}")
print("prise_saine_interdit_faux_positif: OK")

correlation_names = {"codex-id-exact", "codex-id-prefix", "codex-id-mention"}
correlation_agents = [
    {
        "name": name,
        "agent_type": "codex",
        "host": "fixture-host",
        "state": "connected",
        "domain": "bridget",
        "transport": "tmux",
        "location": f"{name}:1.1",
    }
    for name in correlation_names
]
correlation_turns = {
    name: {
        "state": "open",
        "message_id": "mcp-abc",
        "ts": "2026-08-27T10:01:00Z",
    }
    for name in correlation_names
}
correlation = mod.read_intake_observations(
    correlation_agents,
    correlation_names,
    correlation_turns,
    now=int(mod._epoch_from_iso8601("2026-08-27T10:03:00Z")),
    intake_after_secs=60,
    local_host="fixture-host",
    codex_trace_root=trace_root,
    claude_trace_root=trace_root,
    tmux_bin="tmux-inutilise",
    trace_paths={
        "codex-id-exact": codex_exact_trace,
        "codex-id-prefix": codex_prefix_trace,
        "codex-id-mention": codex_mention_trace,
    },
)
if (
    correlation["codex-id-exact"]["state"] != "PRISE_ACCEPTEE"
    or correlation["codex-id-exact"]["matching_acceptances"] != 1
    or correlation["codex-id-exact"]["acceptance_records_read"] != 1
):
    raise SystemExit(f"identifiant canonique exact non reconnu: {correlation['codex-id-exact']}")
print("correlation_identifiant_exact_positive: OK")
if (
    correlation["codex-id-prefix"]["state"] != "MANDAT_NON_SOUMIS"
    or correlation["codex-id-prefix"]["matching_acceptances"] != 0
    or correlation["codex-id-prefix"]["acceptance_records_read"] != 1
    or correlation["codex-id-prefix"]["records_read"] != 2
):
    raise SystemExit(f"collision de préfixe prise pour une acceptation: {correlation['codex-id-prefix']}")
print("correlation_surensemble_negative: OK")
if (
    correlation["codex-id-mention"]["state"] != "MANDAT_NON_SOUMIS"
    or correlation["codex-id-mention"]["matching_acceptances"] != 0
    or correlation["codex-id-mention"]["acceptance_records_read"] != 1
    or correlation["codex-id-mention"]["records_read"] != 2
):
    raise SystemExit(f"mention non corrélée prise pour une acceptation: {correlation['codex-id-mention']}")
print("correlation_mention_non_correlee_negative: OK")

intake_now = int(mod._epoch_from_iso8601("2026-08-27T10:44:20Z"))
intakes = mod.read_intake_observations(
    agents,
    daemon,
    turns,
    now=intake_now,
    intake_after_secs=60,
    local_host="fixture-host",
    codex_trace_root=trace_root,
    claude_trace_root=trace_root,
    tmux_bin="tmux-inutilise",
    trace_paths=trace_paths,
)
if intakes["claude-enqueued"]["state"] != "REMISE_PENDANT_TOUR_ACTIF":
    raise SystemExit(f"enqueue Claude non nommé comme steering: {intakes['claude-enqueued']}")
if intakes["claude-consumed"]["state"] != "PRISE_ACCEPTEE":
    raise SystemExit(f"remove Claude non reconnu: {intakes['claude-consumed']}")
direct = mod.read_native_intake_trace(
    claude_direct_trace,
    provider="claude",
    message_id="mcp-claude-direct",
    injected_epoch=mod._epoch_from_iso8601("2026-08-27T10:07:09Z"),
)
if direct["state"] != "PRISE_ACCEPTEE" or direct["matching_acceptances"] != 1:
    raise SystemExit(f"entrée Claude directe non reconnue: {direct}")
print("prise_claude_enqueue_remove_et_direct_discrimines: OK")

if intakes["cartae0"]["state"] != "REMISE_PENDANT_TOUR_ACTIF":
    raise SystemExit(f"steering réel cartae0 accusé comme mandat non soumis: {intakes['cartae0']}")
if intakes["cartae0"]["client_state_at_injection"] != "active":
    raise SystemExit(f"tour actif cartae0 non attesté: {intakes['cartae0']}")
if intakes["rc7"]["state"] != "MANDAT_NON_SOUMIS":
    raise SystemExit(f"contrôle idle rc7 masqué par la correction steering: {intakes['rc7']}")
if intakes["rc7"]["client_state_at_injection"] != "idle":
    raise SystemExit(f"repos rc7 non attesté: {intakes['rc7']}")
print("steering_cartae0_nomme_et_controle_idle_rc7_conserve: OK")

for unavailable in ("remote-pasted",):
    if intakes[unavailable]["state"] != "PRISE_INOBSERVABLE":
        raise SystemExit(f"source indisponible déclarée saine pour {unavailable}: {intakes[unavailable]}")
    if intakes[unavailable]["records_read"] != 0:
        raise SystemExit(f"cardinal indisponible non nul pour {unavailable}: {intakes[unavailable]}")
empty_trace = write_trace("empty", [])
empty = mod.read_native_intake_trace(
    empty_trace,
    provider="codex",
    message_id="mcp-vide",
    injected_epoch=0,
)
if empty["state"] != "PRISE_INOBSERVABLE" or empty["reason"] != "trace-vide":
    raise SystemExit(f"trace vide déclarée saine: {empty}")
print("prise_inobservable_cardinal_non_muet: OK")

valid_meta = json.dumps(
    {
        "timestamp": "2026-08-27T10:00:00Z",
        "type": "session_meta",
        "payload": {"thread_source": "user"},
    }
)
invalid_trace = trace_root / "invalid.jsonl"
invalid_trace.write_text(valid_meta + "\n{json-invalide}\n", encoding="utf-8")
partial_trace = trace_root / "partial.jsonl"
partial_trace.write_text(valid_meta + "\n{\"type\":\"event_msg\"}", encoding="utf-8")
unknown_trace = write_trace(
    "unknown-format",
    [{"timestamp": "2026-08-27T10:00:00Z", "type": "format-inconnu"}],
)
for path, reason_prefix, cardinal in (
    (invalid_trace, "trace-json-invalide:", 1),
    (partial_trace, "trace-ligne-partielle:", 1),
    (unknown_trace, "trace-format-inconnu", 1),
):
    unavailable = mod.read_native_intake_trace(
        path,
        provider="codex",
        message_id="mcp-inexistant",
        injected_epoch=0,
    )
    if (
        unavailable["state"] != "PRISE_INOBSERVABLE"
        or not unavailable["reason"].startswith(reason_prefix)
        or unavailable["records_read"] != cardinal
    ):
        raise SystemExit(f"source native invalide déclarée saine: {unavailable}")
print("prise_inobservable_sources_invalides: OK")

public_intakes = [
    mod._public_intake_observation(name, observation)
    for name, observation in intakes.items()
]
if secret_content in json.dumps(public_intakes, ensure_ascii=False):
    raise SystemExit("le contenu conversationnel a fuité dans la projection de prise")
print("contenu_trace_non_projete: OK")

# Découverte Codex par l'arbre de processus : présence positive avant les cas
# d'absence/ambiguïté. Le faux /proc ne lance aucun processus.
fake_proc = fixture / "proc"
for pid in (700, 701):
    (fake_proc / str(pid) / "task" / str(pid)).mkdir(parents=True)
    (fake_proc / str(pid) / "fd").mkdir()
(fake_proc / "700" / "task" / "700" / "children").write_text("701\n", encoding="ascii")
(fake_proc / "701" / "task" / "701" / "children").write_text("\n", encoding="ascii")
codex_discovery_trace = trace_root / "rollout-fixture.jsonl"
codex_discovery_trace.write_bytes(codex_consumed_trace.read_bytes())
(fake_proc / "701" / "fd" / "9").symlink_to(codex_discovery_trace)
codex_subagent_trace = trace_root / "rollout-subagent.jsonl"
codex_subagent_trace.write_text(
    json.dumps(
        {
            "timestamp": "2026-08-27T10:01:24.444Z",
            "type": "session_meta",
            "payload": {
                "id": "subagent",
                "cwd": "/fixture/codex-consumed",
                "thread_source": "subagent",
            },
        }
    )
    + "\n",
    encoding="utf-8",
)
(fake_proc / "701" / "fd" / "10").symlink_to(codex_subagent_trace)
discovered, discovery_error = mod.discover_codex_trace(700, trace_root, proc_root=fake_proc)
if discovery_error or discovered != codex_discovery_trace:
    raise SystemExit(f"trace Codex active non découverte: path={discovered} error={discovery_error}")
print("decouverte_trace_codex_par_processus: OK")

# Une disparition concurrente est tolérable ; une permission refusée ne l'est
# pas, sinon une seconde trace peut disparaître d'une source dite observable.
descriptor_directory = fake_proc / "701" / "fd"
descriptor_directory.chmod(0)
try:
    denied_path, denied_error = mod.discover_codex_trace(
        700, trace_root, proc_root=fake_proc
    )
finally:
    descriptor_directory.chmod(0o700)
if denied_path is not None or not (denied_error or "").startswith(
    "descripteurs-codex-inaccessibles:"
):
    raise SystemExit(
        "permission des descripteurs ignorée: "
        f"path={denied_path} error={denied_error}"
    )
print("decouverte_codex_permission_refusee_indisponible: OK")

codex_partial_role_trace = trace_root / "rollout-role-partial.jsonl"
codex_partial_role_trace.write_text(
    json.dumps(
        {
            "timestamp": "2026-08-27T10:01:24.444Z",
            "type": "session_meta",
            "payload": {
                "id": "partial",
                "cwd": "/fixture/codex-consumed",
                "thread_source": "user",
            },
        }
    ),
    encoding="utf-8",
)
partial_descriptor = fake_proc / "701" / "fd" / "11"
partial_descriptor.symlink_to(codex_partial_role_trace)
partial_path, partial_error = mod.discover_codex_trace(
    700, trace_root, proc_root=fake_proc
)
if partial_path is not None or partial_error != "traces-codex-role-inconnu:1":
    raise SystemExit(
        "première ligne partielle ignorée par la découverte: "
        f"path={partial_path} error={partial_error}"
    )
print("decouverte_codex_ligne_partielle_fermee: OK")

fake_tmux = fixture / "tmux-fixture"
fake_tmux.write_text(
    "#!/bin/sh\n"
    "case \"$5\" in\n"
    "  '#{pane_pid}') printf '700\\n' ;;\n"
    "  '#{pane_current_path}') printf '/fixture/codex-consumed\\n' ;;\n"
    "  *) exit 2 ;;\n"
    "esac\n",
    encoding="utf-8",
)
fake_tmux.chmod(0o700)
partial_chain = mod.read_intake_observations(
    [
        {
            "name": "codex-partial",
            "agent_type": "codex",
            "host": "fixture-host",
            "state": "connected",
            "domain": "bridget",
            "transport": "tmux",
            "location": "codex-partial:1.1",
        }
    ],
    {"codex-partial"},
    {
        "codex-partial": {
            "state": "open",
            "message_id": "mcp-partial",
            "ts": "2026-08-27T10:01:00Z",
        }
    },
    now=int(mod._epoch_from_iso8601("2026-08-27T10:03:00Z")),
    intake_after_secs=60,
    local_host="fixture-host",
    codex_trace_root=trace_root,
    claude_trace_root=trace_root,
    tmux_bin=str(fake_tmux),
    proc_root=fake_proc,
)
partial_observation = partial_chain.get("codex-partial")
if (
    not isinstance(partial_observation, dict)
    or partial_observation.get("state") != "PRISE_INOBSERVABLE"
    or partial_observation.get("source_state") != "unavailable"
    or partial_observation.get("reason") != "traces-codex-role-inconnu:1"
    or partial_observation.get("records_read") != 0
    or partial_observation.get("matching_acceptances") != 0
):
    raise SystemExit(f"ligne partielle disparue de la chaîne complète: {partial_observation}")
print("chaine_prise_codex_ligne_partielle_inobservable: OK")
partial_descriptor.unlink()

codex_unknown_trace = trace_root / "rollout-role-unknown.jsonl"
codex_unknown_trace.write_text(
    json.dumps(
        {
            "timestamp": "2026-08-27T10:01:24.444Z",
            "type": "session_meta",
            "payload": {"id": "unknown", "cwd": "/fixture/codex-consumed"},
        }
    )
    + "\n",
    encoding="utf-8",
)
unknown_descriptor = fake_proc / "701" / "fd" / "11"
unknown_descriptor.symlink_to(codex_unknown_trace)
unknown_path, unknown_error = mod.discover_codex_trace(
    700, trace_root, proc_root=fake_proc
)
if unknown_path is not None or unknown_error != "traces-codex-role-inconnu:1":
    raise SystemExit(
        f"trace Codex de rôle inconnu ignorée à tort: path={unknown_path} error={unknown_error}"
    )
unknown_descriptor.unlink()

for pid in (702,):
    (fake_proc / str(pid) / "task" / str(pid)).mkdir(parents=True)
    (fake_proc / str(pid) / "fd").mkdir()
(fake_proc / "702" / "task" / "702" / "children").write_text("\n", encoding="ascii")
(fake_proc / "702" / "fd" / "9").symlink_to(codex_subagent_trace)
subagent_only, subagent_error = mod.discover_codex_trace(
    702, trace_root, proc_root=fake_proc
)
if subagent_only is not None or not (subagent_error or "").startswith(
    "trace-codex-principale-absente:"
):
    raise SystemExit(
        f"trace de sous-agent prise pour la principale: path={subagent_only} error={subagent_error}"
    )

codex_second_primary = trace_root / "rollout-second-primary.jsonl"
codex_second_primary.write_text(
    json.dumps(
        {
            "timestamp": "2026-08-27T10:01:24.444Z",
            "type": "session_meta",
            "payload": {
                "id": "second-primary",
                "cwd": "/fixture/codex-consumed",
                "thread_source": "user",
            },
        }
    )
    + "\n",
    encoding="utf-8",
)
(fake_proc / "701" / "fd" / "11").symlink_to(codex_second_primary)
ambiguous, ambiguous_error = mod.discover_codex_trace(
    700, trace_root, proc_root=fake_proc
)
if ambiguous is not None or ambiguous_error != "traces-codex-principales-ambigues:2":
    raise SystemExit(
        f"deux traces Codex principales conclues à tort: path={ambiguous} error={ambiguous_error}"
    )
print("decouverte_codex_refuse_sous_agent_et_ambiguite: OK")

fake_claude_root = fixture / "claude-projects"
fake_claude_cwd = pathlib.Path("/fixture/claude-project")
fake_claude_project = fake_claude_root / str(fake_claude_cwd).replace("/", "-")
fake_claude_project.mkdir(parents=True)
fake_claude_trace = fake_claude_project / "session.jsonl"
fake_claude_trace.write_text(
    json.dumps(
        {
            "timestamp": "2026-08-27T10:07:11.553Z",
            "type": "user",
            "sessionId": "session",
            "cwd": str(fake_claude_cwd),
            "message": {"role": "user", "content": "fixture"},
        }
    )
    + "\n",
    encoding="utf-8",
)
discovered, discovery_error = mod.discover_claude_trace(fake_claude_cwd, fake_claude_root)
if discovery_error or discovered != fake_claude_trace:
    raise SystemExit(f"trace Claude active non découverte: path={discovered} error={discovery_error}")
fake_claude_unknown = fake_claude_project / "unknown.jsonl"
fake_claude_unknown.write_text("{json-invalide}\n", encoding="utf-8")
unknown_claude, unknown_claude_error = mod.discover_claude_trace(
    fake_claude_cwd, fake_claude_root
)
if (
    unknown_claude is not None
    or unknown_claude_error != "traces-claude-indeterminables:1"
):
    raise SystemExit(
        f"trace Claude indéterminable ignorée à tort: path={unknown_claude} error={unknown_claude_error}"
    )
fake_claude_unknown.unlink()
print("decouverte_trace_claude_par_projet: OK")

# --- Contrôle positif omission (ancienne logique) ---
legacy = mod.classify_legacy(agents, occupied, exclude=exclude, silent_after_secs=1800)
ok_legacy, detail_legacy = mod.partition_oracle(legacy, daemon)
if ok_legacy:
    raise SystemExit(f"CONTROLE POSITIF RATE: l'ancienne logique aurait dû casser l'oracle ({detail_legacy})")
print(f"controle_positif_ancien_rouge: OK ({detail_legacy})")

legacy_covered = (
    {n for n, _ in legacy["libres"]}
    | {n for n, _ in legacy["muets"]}
    | set(legacy["occupes"])
    | {n for n, _ in legacy.get("bloques", [])}
    | {n for n, _ in legacy["indetermines"]}
)
for ghost in ("cursor10-like", "cursorbridget-like", "relec6-like"):
    if ghost in legacy_covered:
        raise SystemExit(f"CONTROLE POSITIF RATE: {ghost} visible dans l'ancienne logique")
print("controle_positif_trois_fantomes_omis: OK")

# Le backlog ne décide plus : ouvert >1 h reste sain, terminal frais reste fini.
backlog = {
    "healthy-consumer": 65 * 60,
    "frozen-consumer": 44 * 60,
    "freshly-finished": 1,
    "resumed-consumer": 65 * 60,
    "corrupt-journal": 44 * 60,
    "missing-journal": 44 * 60,
    "busy-without-journal": 44 * 60,
    "contradictory-consumer": 44 * 60,
    "ssh-open-only": 65 * 60,
    "acp-anomaly-live": 65 * 60,
    "ambiguous-error": 44 * 60,
    "timeout-sans-kind": 44 * 60,
    "unsafe-stop": 44 * 60,
    "unsafe-error": 44 * 60,
    "codex-pasted": 44 * 60,
    "codex-visible": 44 * 60,
    "codex-consumed": 44 * 60,
    "claude-enqueued": 44 * 60,
    "claude-consumed": 44 * 60,
    "remote-pasted": 44 * 60,
    "cartae0": 44 * 60,
    "rc7": 44 * 60,
}
fixed = mod.classify(
    agents,
    occupied,
    exclude=exclude,
    silent_after_secs=1800,
    backlog_ages=backlog,
    turn_observations=turns,
    intake_observations=intakes,
    intake_after_secs=60,
)
ok_fixed, detail_fixed = mod.partition_oracle(fixed, daemon)
if not ok_fixed:
    raise SystemExit(f"correctif ROUGE sur oracle: {detail_fixed}")
print(f"partition_correctif: OK ({detail_fixed})")

bloques = {item["name"]: item for item in fixed["bloques"]}
if "frozen-consumer" not in bloques:
    raise SystemExit(f"figé absente de BLOQUES: {bloques}")
if "timeout-sans-kind" not in bloques:
    raise SystemExit(f"échéance sans kind absente de BLOQUES: {bloques}")
if "timeout-sans-kind" in fixed["occupes"]:
    raise SystemExit("échéance sans kind ne doit pas rester OCCUPE")
if "healthy-consumer" in bloques:
    raise SystemExit(f"consommateur sain à tort BLOQUE: {bloques}")
if "freshly-finished" not in bloques:
    raise SystemExit(f"terminal frais masqué par l'âge de remise: {bloques}")
if "healthy-consumer" not in fixed["occupes"]:
    raise SystemExit(f"sain devrait rester OCCUPE: {fixed['occupes']}")
if "frozen-consumer" in fixed["occupes"]:
    raise SystemExit("figé ne doit pas rester OCCUPE une fois BLOQUE")
if "resumed-consumer" not in fixed["occupes"]:
    raise SystemExit(f"tour repris devrait rester OCCUPE: {fixed['occupes']}")
if "busy-without-journal" not in fixed["occupes"]:
    raise SystemExit(f"état busy positif perdu: {fixed['occupes']}")
if "acp-anomaly-live" not in fixed["occupes"]:
    raise SystemExit(f"anomalie ACP non terminale sortie des OCCUPES: {fixed['occupes']}")
if "codex-consumed" not in fixed["occupes"] or "claude-consumed" not in fixed["occupes"]:
    raise SystemExit(f"prises attestées absentes des OCCUPES: {fixed['occupes']}")
non_soumis = {item["name"]: item for item in fixed["mandats_non_soumis"]}
if set(non_soumis) != {"codex-pasted", "codex-visible", "rc7"}:
    raise SystemExit(f"MANDAT_NON_SOUMIS inattendus: {non_soumis}")
if "codex-visible" in occupied:
    raise SystemExit("le contrôle sans greffe locale appartient encore à Maicie")
if "codex-visible" not in non_soumis:
    raise SystemExit("une copie Maicie locale vide masque le mandat non soumis")
print("prise_independante_de_la_copie_maicie_locale: OK")
if non_soumis["codex-pasted"]["records_read"] != 3:
    raise SystemExit(f"cardinal du specimen perdu: {non_soumis['codex-pasted']}")
inobservables = {item["name"]: item for item in fixed["prises_inobservables"]}
if not {"ssh-open-only", "remote-pasted"}.issubset(inobservables):
    raise SystemExit(f"PRISE_INOBSERVABLE incomplète: {inobservables}")
if any(inobservables[name]["records_read"] != 0 for name in ("ssh-open-only", "remote-pasted")):
    raise SystemExit(f"source muette rendue avec cardinal non nul: {inobservables}")
print("categories_prise_partitionnees_deux_sens: OK")
if set(inobservables) & set(bloques):
    raise SystemExit(
        f"prise inobservable confondue avec un blocage: inobservables={inobservables} bloques={bloques}"
    )
for name in ("ssh-open-only", "remote-pasted"):
    if (
        inobservables[name]["state"] != "PRISE_INOBSERVABLE"
        or inobservables[name]["source_state"] != "unavailable"
        or inobservables[name]["records_read"] != 0
    ):
        raise SystemExit(f"zéro sur zéro examiné présenté comme succès: {inobservables[name]}")
print("prise_inobservable_distincte_de_tout_bloque: OK")
steering = {
    item["name"]: item for item in fixed["remises_pendant_tour_actif"]
}
if set(steering) != {"cartae0", "claude-enqueued"}:
    raise SystemExit(f"remises pendant tour actif inattendues: {steering}")
if "cartae0" in non_soumis or "rc7" in steering:
    raise SystemExit(
        f"populations idle et steering confondues: steering={steering} non_soumis={non_soumis}"
    )
print("populations_idle_et_steering_disjointes: OK")
ind_fixed = {name: reason for name, reason in fixed["indetermines"]}
for uncertain in ("corrupt-journal", "missing-journal", "ambiguous-error"):
    if uncertain not in ind_fixed or "activite-tour=" not in ind_fixed[uncertain]:
        raise SystemExit(f"incertitude de journal non nommée pour {uncertain}: {ind_fixed}")
if "contradictory-consumer" not in ind_fixed or "contradiction-state-busy" not in ind_fixed["contradictory-consumer"]:
    raise SystemExit(f"contradiction de sources conclue à tort: {ind_fixed}")
if "ssh-open-only" in ind_fixed:
    raise SystemExit(f"prise inobservable noyée dans INDETERMINES: {ind_fixed}")
if bloques["frozen-consumer"]["condition"] != "dernier-tour-termine-sans-reprise":
    raise SystemExit(f"condition terminale absente: {bloques['frozen-consumer']}")
for unsafe_agent in ("unsafe-stop", "unsafe-error"):
    if unsafe_agent not in bloques:
        raise SystemExit(f"univers terminal incomplet, {unsafe_agent} absent: {bloques}")
print("controle_positif_bloques_deux_sens: OK")

# Sans backlog, la borne terminale reste concluante : l'âge n'est qu'un contexte.
no_backlog = mod.classify(
    agents,
    occupied,
    exclude=exclude,
    silent_after_secs=1800,
    backlog_ages={},
    turn_observations=turns,
    intake_observations=intakes,
    intake_after_secs=60,
)
if "frozen-consumer" not in {item["name"] for item in no_backlog["bloques"]}:
    raise SystemExit("sans backlog, la fin sans reprise doit rester visible")
print("borne_terminale_independante_du_backlog: OK")

ind = {name: reason for name, reason in fixed["indetermines"]}
if "cursorbridget-like" not in ind or "busy-sans-mission-greffe" not in ind["cursorbridget-like"]:
    raise SystemExit(f"cursorbridget-like mal classé: {ind}")
libres = {n for n, _ in fixed["libres"]}
if "cursor10-like" not in libres:
    raise SystemExit(f"cursor10-like devrait être LIBRE: libres={libres} ind={ind}")
if "alice" not in libres or "bob" not in fixed["occupes"]:
    raise SystemExit(f"branches saines cassées: libres={libres} occupes={fixed['occupes']}")
if "bridget" not in ind:
    raise SystemExit("bridget (hors-perimetre) doit apparaître en INDETERMINES")
print("semantique_correctif: OK")

# Le contrôle positif porte sur la présence d'un tour ouvert, pas un seuil.
if fixed["occupes"].count("healthy-consumer") != 1:
    raise SystemExit(f"tour ouvert long non conservé exactement une fois: {fixed['occupes']}")
print("tour_ouvert_65min_reste_occupe: OK")

# Lecture Maicie sur copie : sidecars source intacts
db = fixture / "maicie.sqlite3"
conn = sqlite3.connect(db)
conn.execute("PRAGMA journal_mode=WAL")
conn.execute("CREATE TABLE objectives(id TEXT PRIMARY KEY, state TEXT NOT NULL)")
conn.execute("CREATE TABLE delegations(id TEXT PRIMARY KEY, objective_id TEXT NOT NULL, payload_json TEXT NOT NULL)")
conn.execute("INSERT INTO objectives VALUES ('o1', 'en_coordination')")
conn.execute("INSERT INTO delegations VALUES ('d1', 'o1', '{\"participant\":\"bob\"}')")
conn.commit()
assert (fixture / "maicie.sqlite3-wal").exists()
wal_before = os.stat(db.with_name(db.name + "-wal")).st_mtime_ns
shm_before = os.stat(db.with_name(db.name + "-shm")).st_mtime_ns
config = fixture / "config.json"
config.write_text(json.dumps({"database_path": str(db)}), encoding="utf-8")
occ, err = mod.read_occupied_from_maicie_copy(str(config))
if err or occ != {"bob"}:
    raise SystemExit(f"lecture copie Maicie: occ={occ} err={err}")
wal_after = os.stat(db.with_name(db.name + "-wal")).st_mtime_ns
shm_after = os.stat(db.with_name(db.name + "-shm")).st_mtime_ns
if (wal_before, shm_before) != (wal_after, shm_after):
    raise SystemExit("lecture idle a touché les sidecars de production (copie obligatoire)")
print("maicie_copie_sidecars_intacts: OK")

conn.execute("INSERT INTO delegations VALUES ('d2', 'o1', '{\"participant\":42}')")
conn.commit()
invalid_occ, invalid_err = mod.read_occupied_from_maicie_copy(str(config))
if invalid_occ is not None or invalid_err != "2 délégations reçues/1 valide":
    raise SystemExit(
        "participant actif non textuel filtré silencieusement: "
        f"occ={invalid_occ} err={invalid_err}"
    )
print("maicie_participant_invalide_ferme_la_source: OK")

# Bridget DB copie : requête backlog + sidecars intacts
bdb = fixture / "bridget.db"
bconn = sqlite3.connect(bdb)
bconn.execute("PRAGMA journal_mode=WAL")
bconn.execute("CREATE TABLE ledger(id TEXT PRIMARY KEY, ts INTEGER NOT NULL, sender TEXT, target TEXT NOT NULL, body TEXT, conversation_key TEXT)")
bconn.execute(
    "CREATE TABLE send_deliveries(delivery_id TEXT PRIMARY KEY, issuer_scope TEXT, operation_kind TEXT, "
    "idempotency_key TEXT, recipient_instance_id TEXT, delivery_generation INTEGER, phase TEXT, expires_at INTEGER, message_bytes BLOB)"
)
now = 1_000_000
bconn.execute("INSERT INTO ledger VALUES ('m-fresh', ?, 'a', 'healthy-consumer', 'x', 'c')", (now - 300,))
bconn.execute("INSERT INTO ledger VALUES ('m-old', ?, 'a', 'frozen-consumer', 'x', 'c')", (now - 44 * 60,))
bconn.execute(
    "INSERT INTO send_deliveries VALUES ('d1','s','send','m-fresh','r',1,'dispatching',?,NULL)",
    (now + 600000,),
)
bconn.execute(
    "INSERT INTO send_deliveries VALUES ('d2','s','send','m-old','r',1,'dispatching',?,NULL)",
    (now + 600000,),
)
# Orphelin sans ledger : ne doit PAS inventer un âge
bconn.execute(
    "INSERT INTO send_deliveries VALUES ('d3','s','send','orphan-key','r',1,'dispatching',?,NULL)",
    (now + 600000,),
)
bconn.commit()
assert (fixture / "bridget.db-wal").exists()
b_wal_before = os.stat(str(bdb) + "-wal").st_mtime_ns
ages, berr = mod.read_backlog_ages_from_bridget_copy(str(bdb), now=now)
if berr:
    raise SystemExit(berr)
if ages.get("healthy-consumer") != 300 or ages.get("frozen-consumer") != 44 * 60:
    raise SystemExit(f"backlog ages inattendus: {ages}")
if "orphan" in str(ages):
    raise SystemExit(f"orphelin sans ledger a fuité: {ages}")
b_wal_after = os.stat(str(bdb) + "-wal").st_mtime_ns
if b_wal_before != b_wal_after:
    raise SystemExit("lecture backlog a touché le WAL source")
print("backlog_copie_ledger_join_seule: OK")

# Dépôt réel jetable : présence avant absence, conflit, base périmée et zéro
# mutation de l'object store observé.
git_now = 2_000_000_000
git_env = os.environ.copy()
git_env.update(
    {
        "GIT_AUTHOR_NAME": "Test User",
        "GIT_AUTHOR_EMAIL": "test@example.invalid",
        "GIT_COMMITTER_NAME": "Test User",
        "GIT_COMMITTER_EMAIL": "test@example.invalid",
    }
)


def git(cwd, *arguments, timestamp=None, check=True):
    env = git_env.copy()
    if timestamp is not None:
        stamp = f"@{timestamp} +0000"
        env["GIT_AUTHOR_DATE"] = stamp
        env["GIT_COMMITTER_DATE"] = stamp
    completed = subprocess.run(
        ["git", *arguments],
        cwd=cwd,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if check and completed.returncode:
        raise SystemExit(
            f"git {' '.join(arguments)}: {(completed.stderr or completed.stdout).strip()}"
        )
    return completed.stdout.strip(), completed.returncode


def commit_file(repository, relative, content, message, timestamp):
    target = repository / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")
    git(repository, "add", relative)
    git(repository, "commit", "-m", message, timestamp=timestamp)


remote = fixture / "origin.git"
repository = fixture / "repository"
git(fixture, "init", "--bare", str(remote))
git(fixture, "init", "-b", "main", str(repository))
commit_file(repository, "shared.txt", "base\n", "base", git_now - 8 * 3600)
git(repository, "remote", "add", "origin", str(remote))
git(repository, "push", "-u", "origin", "main")

git(repository, "switch", "-c", "merged-lot")
commit_file(repository, "merged.txt", "livré puis fusionné\n", "merged", git_now - 6 * 3600)
git(repository, "push", "-u", "origin", "merged-lot")
git(repository, "switch", "main")
git(repository, "merge", "--ff-only", "merged-lot")
git(repository, "push", "origin", "main")

git(repository, "switch", "-c", "pending-lot")
commit_file(repository, "pending.txt", "corps exact du lot en attente\n", "pending", git_now - 2 * 3600)
git(repository, "push", "-u", "origin", "pending-lot")
pending_head, _ = git(repository, "rev-parse", "HEAD")
git(repository, "switch", "main")

git(repository, "switch", "-c", "conflict-lot")
commit_file(repository, "shared.txt", "branche\n", "conflict branch", git_now - 3 * 3600)
git(repository, "push", "-u", "origin", "conflict-lot")
git(repository, "switch", "main")
commit_file(repository, "shared.txt", "main\n", "conflict main", git_now - 3600)
git(repository, "push", "origin", "main")

git(repository, "switch", "--orphan", "stale-lot")
git(repository, "rm", "-rf", "--ignore-unmatch", ".")
commit_file(repository, "stale.txt", "histoire étrangère\n", "stale", git_now - 4 * 3600)
git(repository, "push", "-u", "origin", "stale-lot")
git(repository, "switch", "main")

# La branche existe sur le remote mais pas dans les refs locales : l'analyse
# sans fetch doit l'ignorer et l'annoncer comme limite.
git(fixture, "--git-dir", str(remote), "update-ref", "refs/heads/unseen-lot", pending_head)


def git_inventory(repo):
    refs, _ = git(repo, "show-ref")
    git_root = repo / ".git"
    git_files = sorted(
        (
            str(path.relative_to(git_root)),
            path.stat().st_size,
            path.stat().st_mtime_ns,
            hashlib.sha256(path.read_bytes()).hexdigest(),
        )
        for path in git_root.rglob("*")
        if path.is_file()
    )
    worktree_files = sorted(
        (
            str(path.relative_to(repo)),
            path.stat().st_size,
            path.stat().st_mtime_ns,
            hashlib.sha256(path.read_bytes()).hexdigest(),
        )
        for path in repo.rglob("*")
        if path.is_file() and git_root not in path.parents
    )
    return refs, git_files, worktree_files


before = git_inventory(repository)
branch_backlog, branch_error = mod.read_branch_backlog(
    str(repository), now=git_now, timeout_secs=5.0
)
if branch_error or branch_backlog is None:
    raise SystemExit(f"branche_non_fusionnee_presente: analyse absente: {branch_error}")
by_ref = {item["ref"]: item for item in branch_backlog["lots"]}

# Oracle de PRÉSENCE d'abord : la tête exacte et l'âge doivent être lus.
pending = by_ref.get("origin/pending-lot")
if pending is None or pending["head"] != pending_head or pending["age_secs"] != 2 * 3600:
    raise SystemExit(f"branche_non_fusionnee_presente: attendu pending exact, reçu {pending}")
if pending["textual_merge"] != "sans conflit textuel":
    raise SystemExit(f"branche_non_fusionnee_presente: état inattendu {pending}")
print("branche_non_fusionnee_presente: OK (origin/pending-lot, tete et age exacts)")

# Oracle d'ABSENCE seulement après la présence : une implémentation morte qui
# renverrait toujours [] ne peut donc pas passer.
if "origin/merged-lot" in by_ref:
    raise SystemExit(
        "branche_fusionnee_absente_apres_presence: origin/merged-lot est encore visible"
    )
print("branche_fusionnee_absente_apres_presence: OK")

conflict = by_ref.get("origin/conflict-lot")
if conflict is None or conflict["textual_merge"] != "en conflit" or conflict["blocking"] != "en conflit":
    raise SystemExit(f"branche_en_conflit_visible: reçu {conflict}")
print("branche_en_conflit_visible: OK")

stale = by_ref.get("origin/stale-lot")
if (
    stale is None
    or stale["base_state"] != "perimee"
    or stale["textual_merge"] != "sans conflit textuel"
    or stale["blocking"] != "base perimee, a rebaser"
):
    raise SystemExit(f"branche_base_perimee_visible: reçu {stale}")
print("branche_base_perimee_visible: OK")

if "origin/unseen-lot" in by_ref:
    raise SystemExit("refs_locales_sans_fetch: une ref distante non récupérée a été inventée")
if branch_backlog["refs_scope"] != "refs locales sans fetch":
    raise SystemExit(f"refs_locales_sans_fetch: limite absente {branch_backlog}")
print("refs_locales_sans_fetch: OK")

after = git_inventory(repository)
if before != after:
    raise SystemExit("depot_git_lecture_seule: refs, fichiers Git ou worktree modifiés")
print("depot_git_lecture_seule: OK")

# --- Base datée : un verdict d'ascendance rendu contre une ref locale périmée
# n'est pas recevable. Reproduit le faux « lot non mergé » d'un lot déjà
# intégré côté remote, que la ref locale ignore faute de fetch.
if branch_backlog["main_freshness"] != "a_jour":
    raise SystemExit(
        f"base_datee_verdict_recevable: base fraîche attendue, reçu {branch_backlog['main_freshness']}"
    )
if branch_backlog["remote_main_head"] != branch_backlog["main_head"]:
    raise SystemExit("base_datee_verdict_recevable: tête distante != locale sur base fraîche")
fresh_render = mod.format_branch_backlog(branch_backlog, branch_error=None)
if "LOTS LIVRES NON MERGES :" not in fresh_render:
    raise SystemExit(f"base_datee_verdict_recevable: affirmation attendue sur base fraîche\n{fresh_render}")
print("base_datee_verdict_recevable: OK (base à jour, verdict affirmé)")

# Le remote avance seul : le dépôt local n'est pas fetché, exactement comme en production.
git(fixture, "--git-dir", str(remote), "update-ref", "refs/heads/main", pending_head)
stale_backlog, stale_error = mod.read_branch_backlog(
    str(repository), now=git_now, timeout_secs=5.0
)
if stale_error or stale_backlog is None:
    raise SystemExit(f"base_perimee_verdict_non_recevable: analyse absente: {stale_error}")
if stale_backlog["main_freshness"] != "perimee":
    raise SystemExit(
        f"base_perimee_verdict_non_recevable: périmée attendue, reçu {stale_backlog['main_freshness']}"
    )
if stale_backlog["remote_main_head"] != pending_head:
    raise SystemExit("base_perimee_verdict_non_recevable: tête distante non lue")
stale_render = mod.format_branch_backlog(stale_backlog, branch_error=None)
# Assertion métier : plus aucune affirmation « non mergé » sur une base morte…
if "LOTS LIVRES NON MERGES :" in stale_render:
    raise SystemExit(
        f"base_perimee_verdict_non_recevable: verdict encore affirmé sur base périmée\n{stale_render}"
    )
if "base perimee" not in stale_render or "LOTS CANDIDATS" not in stale_render:
    raise SystemExit(f"base_perimee_verdict_non_recevable: qualification absente\n{stale_render}")
# …et pas de faux négatif : les lots restent visibles, seule l'affirmation tombe.
if "origin/pending-lot" not in stale_render:
    raise SystemExit(f"base_perimee_verdict_non_recevable: lot masqué (faux négatif)\n{stale_render}")
print("base_perimee_verdict_non_recevable: OK (qualifié, rien de masqué)")

# Le dépôt observé reste inerte malgré la lecture réseau `ls-remote`.
if git_inventory(repository) != before:
    raise SystemExit("base_perimee_lecture_seule: dépôt muté par la mesure de fraîcheur")
print("base_perimee_lecture_seule: OK")
git(fixture, "--git-dir", str(remote), "update-ref", "refs/heads/main", branch_backlog["main_head"])

# Un Git lent est borné globalement et rend une indisponibilité, jamais [].
slow_git = fixture / "git-slow"
slow_git.write_text("#!/bin/sh\nexec sleep 10\n", encoding="utf-8")
slow_git.chmod(0o700)
started = time.monotonic()
slow_backlog, slow_error = mod.read_branch_backlog(
    str(repository), now=git_now, timeout_secs=0.05, git_bin=str(slow_git)
)
elapsed = time.monotonic() - started
if slow_backlog is not None or not slow_error or "delai Git" not in slow_error:
    raise SystemExit(
        f"backlog_branches_timeout_indisponible: backlog={slow_backlog} error={slow_error}"
    )
if elapsed >= 1.0:
    raise SystemExit(f"backlog_branches_timeout_indisponible: borne dépassée ({elapsed:.3f}s)")
print("backlog_branches_timeout_indisponible: OK")

bad_backlog, bad_error = mod.read_branch_backlog(
    str(fixture / "absent"), now=git_now, timeout_secs=1.0
)
if bad_backlog is not None or not bad_error:
    raise SystemExit(
        f"backlog_branches_depot_invalide_indisponible: backlog={bad_backlog} error={bad_error}"
    )
print("backlog_branches_depot_invalide_indisponible: OK")

agents_path = fixture / "agents.json"
invalid_agents_path = fixture / "agents-invalides.json"
occupied_path = fixture / "occupied.json"
backlog_path = fixture / "backlog.json"
intake_traces_path = fixture / "intake-traces.json"
agents_path.write_text(json.dumps(agents), encoding="utf-8")
invalid_agents_path.write_text(
    json.dumps([agents[0], "entree-invalide", {"state": "connected"}]),
    encoding="utf-8",
)
occupied_path.write_text(json.dumps(sorted(occupied)), encoding="utf-8")
backlog_path.write_text(json.dumps(backlog), encoding="utf-8")
intake_traces_path.write_text(
    json.dumps({name: str(path) for name, path in trace_paths.items()}),
    encoding="utf-8",
)
print(f"FIXTURES {agents_path} {occupied_path} {backlog_path} {journal_root} {intake_traces_path}")
PY

agents_json="${fixture_root}/agents.json"
occupied_json="${fixture_root}/occupied.json"
backlog_json="${fixture_root}/backlog.json"
intake_traces_json="${fixture_root}/intake-traces.json"
journal_root="${fixture_root}/journals"
git_repo="${fixture_root}/repository"
out="$("$system_python" "$idle" --agents-json "$agents_json" --occupied-json "$occupied_json" --backlog-json "$backlog_json" --intake-traces-json "$intake_traces_json" --local-host fixture-host --journal-root "$journal_root" --git-repo "$git_repo" --now 2000000000)"
grep -q 'BLOQUES' <<<"$out"
grep -q 'frozen-consumer' <<<"$out"
grep -q 'dernier-tour-termine-sans-reprise' <<<"$out"
grep -q 'cursor10-like' <<<"$out"
grep -q 'MANDATS NON SOUMIS' <<<"$out"
grep -q 'codex-pasted' <<<"$out"
grep -q 'PRISES INOBSERVABLES' <<<"$out"
grep -q 'REMISES PENDANT TOUR ACTIF' <<<"$out"
grep -q 'records-lus=' <<<"$out"
grep -q 'BACKLOG BRANCHES INDISPONIBLE (greffe sans etat exploitable)' <<<"$out"
grep -q 'origin/pending-lot' <<<"$out"
if grep -q 'origin/merged-lot' <<<"$out"; then
  echo 'branche_fusionnee_absente_cli: origin/merged-lot visible' >&2
  exit 1
fi
grep -q 'refs locales sans fetch' <<<"$out"
grep -q 'age du commit de tete uniquement' <<<"$out"
echo 'cli_texte_branches_et_limites: OK'
maicie_invalid_json="$("$system_python" "$idle" --agents-json "$agents_json" --config "${fixture_root}/config.json" --backlog-json "$backlog_json" --intake-traces-json "$intake_traces_json" --local-host fixture-host --journal-root "$journal_root" --git-repo "$git_repo" --now 2000000000 --json)"
"$system_python" - "$maicie_invalid_json" <<'PY'
import json, sys

data = json.loads(sys.argv[1])
assert data["maicie"] == {
    "state": "unavailable",
    "reason": "2 délégations reçues/1 valide",
}
assert data["libres"] == []
assert data["muets"] == []
assert any(
    item["name"] == "alice"
    and item["reason"] == "missions-inobservables:maicie-indisponible"
    for item in data["indetermines"]
)
print("maicie_invalide_ferme_la_partition_complete: OK")
PY
set +e
invalid_agents_out="$("$system_python" "$idle" --agents-json "${fixture_root}/agents-invalides.json" --occupied-json "$occupied_json" --backlog-json "$backlog_json" --intake-traces-json "$intake_traces_json" --local-host fixture-host --journal-root "$journal_root" --git-repo "$git_repo" --now 2000000000 2>&1)"
invalid_agents_rc=$?
set -e
if [[ "$invalid_agents_rc" -eq 0 ]]; then
  echo "annuaire_invalide_refuse: sortie nulle: ${invalid_agents_out}" >&2
  exit 1
fi
grep -q 'annuaire Bridget indisponible: 3 recus/1 valides' <<<"$invalid_agents_out"
echo 'annuaire_invalide_refuse: OK (3 recus/1 valides)'
"$system_python" - "$out" <<'PY'
import sys

rendered = sys.argv[1]
for control in ("\x1b", "\r", "\u202e"):
    if control in rendered:
        raise SystemExit(f"contrôle terminal brut dans stdout: U+{ord(control):04X}")
for escaped in (r"\u001b", r"\u000d", r"\u202e"):
    if escaped not in rendered:
        raise SystemExit(f"détail neutralisé absent du rendu texte: {escaped}")
print("rendu_texte_details_inertes: OK")
PY
slow_out="$("$system_python" "$idle" --agents-json "$agents_json" --occupied-json "$occupied_json" --backlog-json "$backlog_json" --intake-traces-json "$intake_traces_json" --local-host fixture-host --journal-root "$journal_root" --git-repo "$git_repo" --git-bin "${fixture_root}/git-slow" --git-timeout-secs 0.05 --now 2000000000)"
grep -q 'BACKLOG BRANCHES INDISPONIBLE (delai Git depasse)' <<<"$slow_out"
if grep -q 'origin/pending-lot' <<<"$slow_out"; then
  echo 'backlog_branches_timeout_cli: liste partielle visible' >&2
  exit 1
fi
echo 'cli_timeout_sans_liste_partielle: OK'
json_out="$("$system_python" "$idle" --agents-json "$agents_json" --occupied-json "$occupied_json" --backlog-json "$backlog_json" --intake-traces-json "$intake_traces_json" --local-host fixture-host --journal-root "$journal_root" --git-repo "$git_repo" --now 2000000000 --json)"
"$system_python" - "$json_out" <<'PY'
import json, sys

raw = sys.argv[1]
data = json.loads(raw)
assert data["daemon_count"] == 28
assert any(item["name"] == "frozen-consumer" and item["condition"] == "dernier-tour-termine-sans-reprise" for item in data["bloques"])
assert any(item["name"] == "timeout-sans-kind" and item["condition"] == "dernier-tour-termine-sans-reprise" for item in data["bloques"])
assert "timeout-sans-kind" not in data["occupes"]
assert not any(item["name"] == "healthy-consumer" for item in data["bloques"])
assert "healthy-consumer" in data["occupes"]
assert "acp-anomaly-live" in data["occupes"]
assert any(item["name"] == "missing-journal" and "journal-absent" in item["reason"] for item in data["indetermines"])
assert not any(item["name"] == "ssh-open-only" for item in data["indetermines"])
assert any(item["name"] == "ambiguous-error" and "error-terminalite-non-attestee" in item["reason"] for item in data["indetermines"])
assert {item["name"] for item in data["mandats_non_soumis"]} == {
    "codex-pasted",
    "codex-visible",
    "rc7",
}
assert {item["name"] for item in data["remises_pendant_tour_actif"]} == {
    "cartae0",
    "claude-enqueued",
}
assert {"ssh-open-only", "remote-pasted"}.issubset(
    {item["name"] for item in data["prises_inobservables"]}
)
assert "codex-consumed" in data["occupes"]
assert "claude-consumed" in data["occupes"]
assert data["intake"]["eligible_count"] == 8
assert data["intake"]["available_count"] == 7
assert data["intake"]["records_read"] > 0
assert "SECRET-CONTENU-NE-DOIT-PAS-ETRE-PROJETE" not in raw
unsafe_detail = "provider\x1b[2J\refface\u202ele diagnostic"
unsafe = {item["name"]: item for item in data["bloques"] if item["name"].startswith("unsafe-")}
assert unsafe["unsafe-stop"]["terminal_kind"] == "turn_completed"
assert unsafe["unsafe-error"]["terminal_kind"] == "turn_failed"
assert unsafe["unsafe-stop"]["terminal_reason"] == unsafe_detail
assert unsafe["unsafe-error"]["terminal_reason"] == unsafe_detail
for control in ("\x1b", "\r", "\u202e"):
    if control in raw:
        raise SystemExit(f"contrôle brut dans le JSON sérialisé: U+{ord(control):04X}")
b = data["branch_backlog"]
assert b["state"] == "partial"
assert b["reason"] == "greffe sans etat exploitable"
assert any(x["ref"] == "origin/pending-lot" and x["age_secs"] == 7200 for x in b["lots"])
assert not any(x["ref"] == "origin/merged-lot" for x in b["lots"])
print("json_diagnostic_valide_et_inerte_et_backlog_branches: OK")
PY

echo "test-bridget-idle: checks OK (partition + bornes de tour + backlog Git + incertitudes + copies + cli)"
