import importlib.util
import argparse
import contextlib
import io
import json
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "agent_loop.py"
SPEC = importlib.util.spec_from_file_location("agent_loop", SCRIPT)
agent_loop = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(agent_loop)


def write_json(path: Path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


class ResolveAgentTargetTests(unittest.TestCase):
    def test_strict_alias_is_resolved_before_tmux(self):
        agents = [
            {"pane": "%12", "addr": "agents:1.10", "type": "codex", "name": "agent10", "pane_name": "agent10"}
        ]
        with patch.object(agent_loop, "bridge_discover", return_value=agents), patch.object(agent_loop, "tmux_display_pane_id") as tmux_display:
            self.assertEqual(agent_loop.resolve_agent_target("agent10"), "%12")
            tmux_display.assert_not_called()

    def test_non_exact_unknown_target_does_not_call_tmux_fuzzy(self):
        with patch.object(agent_loop, "bridge_discover", return_value=[]), patch.object(agent_loop, "tmux_display_pane_id") as tmux_display:
            with self.assertRaises(SystemExit) as raised:
                agent_loop.resolve_agent_target("thunderlens")
            self.assertIn("agent target not found", str(raised.exception))
            tmux_display.assert_not_called()

    def test_exact_tmux_pane_id_is_allowed_after_alias_miss(self):
        with patch.object(agent_loop, "bridge_discover", return_value=[]), patch.object(agent_loop, "tmux_display_pane_id", return_value="%9") as tmux_display:
            self.assertEqual(agent_loop.resolve_agent_target("%9"), "%9")
            tmux_display.assert_called_once_with("%9")


class BridgetBackendTests(unittest.TestCase):
    AGENT_ID = "87515984-f549-49ae-ab9c-a6c47dada0e5"

    def agent(self, **overrides):
        data = {
            "agent_id": self.AGENT_ID,
            "display_name": "coordinateur",
            "agent_type": "codex",
            "transport": "t3code",
            "state": "connected",
            "domain": "69.opus2D",
            "model": None,
        }
        data.update(overrides)
        return data

    def make_run(self, root: Path) -> Path:
        rd = root / "run"
        write_json(
            rd / "run.json",
            {
                "schema_version": "agent-loop-run-v1",
                "run_id": "run",
                "policies": {"allowed_backends": ["existing_bridget"]},
            },
        )
        return rd

    def test_resolve_bridget_target_accepts_unique_uuid_prefix(self):
        resolved = agent_loop.resolve_bridget_target(self.AGENT_ID[:8], [self.agent()])
        self.assertEqual(resolved["agent_id"], self.AGENT_ID)

    def test_resolve_bridget_target_refuses_stopped_or_ambiguous_agents(self):
        with self.assertRaisesRegex(SystemExit, "not connected"):
            agent_loop.resolve_bridget_target(self.AGENT_ID, [self.agent(state="stopped")])
        with self.assertRaisesRegex(SystemExit, "ambiguous"):
            agent_loop.resolve_bridget_target(
                "agent non identifié",
                [
                    self.agent(display_name="Agent non identifié"),
                    self.agent(
                        agent_id="030f9205-f823-4bff-9529-c501367aaa0c",
                        display_name="Agent non identifié",
                    ),
                ],
            )

    def test_background_environment_removes_inherited_t3_identity(self):
        inherited = {
            "TMUX_PANE": "%99",
            "BRIDGET_AGENT_ID": "borrowed",
            "BRIDGET_AGENT_ID_FILE": "/tmp/identity",
            "BRIDGET_AGENT_INSTANCE_ID": "instance",
            "BRIDGET_DELEGATED_PID": "123",
            "BRIDGET_DELEGATED_REF": "ref",
            "PATH": "/usr/bin",
        }
        with patch.dict(agent_loop.os.environ, inherited, clear=True):
            env = agent_loop.bridget_background_environment()
        self.assertEqual(env["PATH"], "/usr/bin")
        for key in inherited:
            if key != "PATH":
                self.assertNotIn(key, env)

    def test_attach_and_lookup_t3_orchestrator_never_query_tmux(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            args = argparse.Namespace(
                root=raw,
                run_id="run",
                role="orchestrator",
                target=self.AGENT_ID,
                backend="existing_bridget",
                session_id="",
                jsonl_path="",
                notes="",
            )
            with patch.object(agent_loop, "resolve_bridget_target", return_value=self.agent()), \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_attach_agent(args), 0)
            with patch.object(agent_loop, "bridget_agents_by_id", return_value={self.AGENT_ID: self.agent()}), \
                    patch.object(agent_loop, "agent_by_pane", side_effect=AssertionError("tmux queried")):
                session = agent_loop.live_session_for_role(rd, "orchestrator")
            self.assertEqual(session["agent_id"], self.AGENT_ID)
            self.assertEqual(session["backend"], "existing_bridget")

    def test_send_bridget_message_uses_uuid_without_sender_identity(self):
        with patch.dict(agent_loop.os.environ, {"BRIDGET_AGENT_ID": "borrowed", "TMUX_PANE": "%1"}), \
                patch.object(agent_loop.subprocess, "run") as run:
            agent_loop.send_bridget_message(self.AGENT_ID, "rappel")
        cmd = run.call_args.args[0]
        env = run.call_args.kwargs["env"]
        self.assertEqual(cmd[:6], [str(agent_loop.BRIDGET), "send", "--to", self.AGENT_ID, "--hops", "4"])
        self.assertEqual(cmd[-2:], ["--", "rappel"])
        self.assertIn("--id", cmd)
        self.assertIn("--issuer-scope", cmd)
        self.assertNotIn("--from", cmd)
        self.assertNotIn("BRIDGET_AGENT_ID", env)
        self.assertNotIn("TMUX_PANE", env)

    def test_dispatch_existing_bridget_records_uuid_and_sends_once(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            write_json(
                rd / "tasks" / "review.json",
                {
                    "task_id": "review",
                    "status": "pending",
                    "attempts": 0,
                    "max_attempts": 1,
                    "backend": "existing_bridget",
                    "delegate": True,
                    "agent_target": self.AGENT_ID,
                    "assigned_agent": self.AGENT_ID,
                    "objective": "Relire",
                    "acceptance": "Verdict",
                    "reply_channel": "file",
                },
            )
            args = argparse.Namespace(
                root=raw,
                run_id="run",
                task_id="review",
                dry_run=False,
                force=False,
                spawn_command="",
                runner_command="",
            )
            with patch.object(agent_loop, "resolve_bridget_target", return_value=self.agent()), \
                    patch.object(agent_loop, "send_bridget_message") as send, \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 0)
            send.assert_called_once()
            self.assertEqual(send.call_args.args[0], self.AGENT_ID)
            stored = agent_loop.load_task(rd, "review")
            self.assertEqual(stored["status"], "dispatched")
            self.assertEqual(stored["dispatch_backend"], "existing_bridget")
            self.assertEqual(stored["resolved_target"], self.AGENT_ID)


class BridgeBackgroundIdentityTests(unittest.TestCase):
    def test_dispatch_and_heartbeat_have_distinct_declared_subjects(self):
        with patch.dict(agent_loop.os.environ, {"TMUX_PANE": "%99", "BRIDGE_FROM": "wrong"}):
            dispatch = agent_loop.bridge_background_environment("dispatch")
            heartbeat = agent_loop.bridge_background_environment("heartbeat")

        self.assertEqual(dispatch["BRIDGE_BACKGROUND_SUBJECT"], "agent-loop-dispatch")
        self.assertEqual(heartbeat["BRIDGE_BACKGROUND_SUBJECT"], "agent-loop-heartbeat")
        self.assertEqual(dispatch["BRIDGE_BACKGROUND_ROLE"], "agent-loop")
        self.assertEqual(heartbeat["BRIDGE_BACKGROUND_ROLE"], "agent-loop")
        self.assertEqual(dispatch["BRIDGE_FROM"], "agent-loop-dispatch")
        self.assertEqual(heartbeat["BRIDGE_FROM"], "agent-loop-heartbeat")
        self.assertNotIn("TMUX_PANE", dispatch)

    def test_unknown_bridge_emission_is_refused(self):
        with self.assertRaises(ValueError):
            agent_loop.bridge_background_environment("unknown")


class CanonicalBootContextTests(unittest.TestCase):
    def write_registry(self, root: Path, entries):
        path = root / "boot-role-bindings.json"
        write_json(path, {"boot_id": "boot-0123456789abcdef0123456789abcdef", "entries": entries})
        return path

    def entry(self, **overrides):
        entry = {
            "agent_name": "agent0",
            "pane_id": "%365",
            "cli": "codex",
            "role_status": "confirmed",
            "boot_cycle": "boot-0123456789abcdef0123456789abcdef",
            "invocation_id": "boot-0123456789abcdef0123456789abcdef:agent0:%365",
        }
        entry.update(overrides)
        return entry

    def test_resolves_exact_registry_context_without_inherited_environment(self):
        with tempfile.TemporaryDirectory() as raw:
            path = self.write_registry(Path(raw), [self.entry()])
            with patch.object(agent_loop, "BOOT_ROLE_BINDINGS_PATH", path), \
                    patch.dict(agent_loop.os.environ, {}, clear=True):
                self.assertEqual(
                    agent_loop.canonical_boot_context(agent="agent0", pane="%365", cli="codex"),
                    {
                        "boot_cycle": "boot-0123456789abcdef0123456789abcdef",
                        "invocation_id": "boot-0123456789abcdef0123456789abcdef:agent0:pane365",
                        "source_ref": str(path.resolve()),
                    },
                )

    def test_refuses_ambiguous_or_contradictory_registry_context(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            ambiguous = self.write_registry(root, [self.entry(), self.entry()])
            with patch.object(agent_loop, "BOOT_ROLE_BINDINGS_PATH", ambiguous), \
                    patch.dict(agent_loop.os.environ, {}, clear=True), \
                    self.assertRaisesRegex(RuntimeError, "ambigu"):
                agent_loop.canonical_boot_context(agent="agent0", pane="%365", cli="codex")
            contradictory = self.write_registry(root, [self.entry(invocation_id="forged")])
            with patch.object(agent_loop, "BOOT_ROLE_BINDINGS_PATH", contradictory), \
                    patch.dict(agent_loop.os.environ, {}, clear=True), \
                    self.assertRaisesRegex(RuntimeError, "contradictoire"):
                agent_loop.canonical_boot_context(agent="agent0", pane="%365", cli="codex")

    def test_refuses_inherited_boot_id_that_disagrees_with_registry(self):
        with tempfile.TemporaryDirectory() as raw:
            path = self.write_registry(Path(raw), [self.entry()])
            with patch.object(agent_loop, "BOOT_ROLE_BINDINGS_PATH", path), \
                    patch.dict(agent_loop.os.environ, {"MAICOMPANY_BOOT_ID": "boot-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}, clear=True), \
                    self.assertRaisesRegex(RuntimeError, "MAICOMPANY_BOOT_ID contradictoire"):
                agent_loop.canonical_boot_context(agent="agent0", pane="%365", cli="codex")

    def test_uses_target_invocation_when_dispatcher_inherits_another_agent_invocation(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            target = self.entry(
                agent_name="agent2",
                pane_id="%367",
                cli="claude",
                invocation_id="boot-0123456789abcdef0123456789abcdef:agent2:%367",
            )
            path = self.write_registry(root, [target])
            with patch.object(agent_loop, "BOOT_ROLE_BINDINGS_PATH", path), \
                    patch.dict(agent_loop.os.environ, {
                        "MAICOMPANY_BOOT_ID": "boot-0123456789abcdef0123456789abcdef",
                        "MAICOMPANY_INVOCATION_ID": "boot-0123456789abcdef0123456789abcdef:agent0:%365",
                    }, clear=True):
                context = agent_loop.canonical_boot_context(agent="agent2", pane="%367", cli="claude")
            self.assertEqual(
                context["invocation_id"],
                "boot-0123456789abcdef0123456789abcdef:agent2:pane367",
            )


class WritableRootsDispatchTests(unittest.TestCase):
    def make_run(self, root: Path, provider: str = "codex_exec", **task_overrides):
        rd = root / "run"
        worktree = root / "worktree"
        output = worktree / "outputs"
        worktree.mkdir()
        output.mkdir()
        write_json(
            rd / "run.json",
            {"run_id": "run", "policies": {"default_agent_provider": provider, "allowed_backends": ["llm_process", "local"]}},
        )
        task = {
            "task_id": "write-task", "status": "pending", "attempts": 0, "max_attempts": 1,
            "backend": "llm_process", "delegate": True, "objective": "Ecrire", "acceptance": "preuve",
            "reply_channel": "file", "agent_provider": provider, "agent_model": "",
            "agent_command_template": "", "cwd": str(worktree), "output_paths": [str(output)],
        }
        task.update(task_overrides)
        write_json(rd / "tasks" / "write-task.json", task)
        return rd, worktree, output

    def test_codex_exec_uses_run_dir_cwd_and_adds_external_worktree_root(self):
        with tempfile.TemporaryDirectory() as raw:
            rd, worktree, output = self.make_run(Path(raw))
            task = agent_loop.load_task(rd, "write-task")
            task["cwd"] = str(rd)
            with patch.object(agent_loop, "shutil_which", return_value="/usr/bin/codex"):
                command = agent_loop.render_agent_command(
                    "codex_exec", "", rd / "prompt.txt", rd / "final.txt", rd / "stdout", rd / "stderr", "",
                    *agent_loop.writable_roots_for_llm(rd, task, "codex_exec"),
                )
            self.assertIn(f"-C {shlex.quote(str(rd.resolve()))}", command)
            self.assertIn(f"--add-dir {shlex.quote(str(output.resolve()))}", command)

    def test_unavailable_writable_roots_refuse_before_worker_launch(self):
        with tempfile.TemporaryDirectory() as raw:
            rd, _, _ = self.make_run(Path(raw), provider="claude_print")
            args = argparse.Namespace(root=raw, run_id="run", task_id="write-task", dry_run=False, force=False, spawn_command="", runner_command="")
            with patch.object(agent_loop.subprocess, "Popen") as popen, contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 2)
            popen.assert_not_called()
            task = agent_loop.load_task(rd, "write-task")
            self.assertEqual(task["status"], "blocked")
            self.assertEqual(task["blocked_reason"], "writable_root_unavailable")
            self.assertNotIn("pid", task)
            self.assertFalse((rd / "results" / "write-task.result.json").exists())

    def test_local_task_keeps_running_with_external_output_paths(self):
        with tempfile.TemporaryDirectory() as raw:
            rd, _, output = self.make_run(Path(raw), backend="local")
            task = agent_loop.load_task(rd, "write-task")
            task["delegate"] = False
            task["runner_command"] = f"touch {shlex.quote(str(output / 'ok.txt'))}"
            write_json(rd / "tasks" / "write-task.json", task)
            args = argparse.Namespace(root=raw, run_id="run", task_id="write-task", dry_run=False, force=False, spawn_command="", runner_command="")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 0)
            self.assertTrue((output / "ok.txt").exists())


class ExhaustedTaskIsolationTests(unittest.TestCase):
    def test_exhausted_blocked_task_does_not_invalidate_its_successor(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = Path(raw) / "run"
            write_json(rd / "run.json", {"run_id": "run", "policies": {}})
            write_json(
                rd / "tasks" / "obsolete.json",
                {
                    "task_id": "obsolete", "status": "blocked", "attempts": 1,
                    "max_attempts": 1, "backend": "existing_tmux", "delegate": True,
                    "objective": "Ancienne revue", "acceptance": "Supersedee",
                },
            )
            write_json(
                rd / "tasks" / "successor.json",
                {
                    "task_id": "successor", "status": "pending", "attempts": 0,
                    "max_attempts": 1, "backend": "existing_tmux", "delegate": True,
                    "objective": "Revue fraiche", "acceptance": "Dispatchable",
                },
            )
            self.assertEqual(agent_loop.collect_errors(rd), [])

    def test_exhausted_pending_task_remains_a_plan_error(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = Path(raw) / "run"
            write_json(rd / "run.json", {"run_id": "run", "policies": {}})
            write_json(
                rd / "tasks" / "stuck.json",
                {
                    "task_id": "stuck", "status": "pending", "attempts": 1,
                    "max_attempts": 1, "backend": "existing_tmux", "delegate": True,
                    "objective": "A relancer", "acceptance": "Impossible sans force",
                },
            )
            self.assertEqual(agent_loop.collect_errors(rd), ["stuck: max_attempts reached"])


class DispatchRecipientGuardTests(unittest.TestCase):
    def test_unauthorized_assigned_agent_is_refused_before_bridge_send(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            rd = root / "run"
            write_json(
                rd / "run.json",
                {
                    "schema_version": "agent-loop-run-v1",
                    "run_id": "run",
                    "policies": {"allowed_backends": ["existing_tmux"]},
                },
            )
            write_json(
                rd / "tasks" / "bad-recipient.json",
                {
                    "task_id": "bad-recipient",
                    "status": "pending",
                    "created_at": "2026-07-08T00:00:00Z",
                    "updated_at": "2026-07-08T00:00:00Z",
                    "objective": "Verifier le refus pre-transport",
                    "acceptance": "La tache est bloquee sans bridge send",
                    "backend": "existing_tmux",
                    "delegate": True,
                    "agent_target": "agent8",
                    "assigned_agent": "agent8",
                    "authorized_agents": ["agent10"],
                },
            )
            args = argparse.Namespace(
                root=str(root),
                run_id="run",
                task_id="bad-recipient",
                dry_run=False,
                force=False,
                spawn_command="",
                runner_command="",
            )
            with patch.object(agent_loop, "resolve_agent_target") as resolve, patch.object(agent_loop.subprocess, "run") as subprocess_run:
                with contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(agent_loop.cmd_dispatch(args), 2)
                resolve.assert_not_called()
                subprocess_run.assert_not_called()
            task = json.loads((rd / "tasks" / "bad-recipient.json").read_text(encoding="utf-8"))
            self.assertEqual(task["status"], "blocked")
            self.assertEqual(task["blocked_reason"], "invalid_recipient")


class ExistingTmuxBindingDispatchTests(unittest.TestCase):
    def make_run(self, root: Path, **task_overrides):
        rd = root / "run"
        write_json(
            rd / "run.json",
            {
                "schema_version": "agent-loop-run-v1",
                "run_id": "run",
                "policies": {"allowed_backends": ["existing_tmux"]},
            },
        )
        task = {
            "schema_version": "agent-loop-task-v1",
            "task_id": "bound-task",
            "status": "pending",
            "created_at": "2026-07-18T00:00:00Z",
            "updated_at": "2026-07-18T00:00:00Z",
            "objective": "Refuser tout transport hors binding actif",
            "acceptance": "La cible est liee avant le transport",
            "backend": "existing_tmux",
            "delegate": True,
            "agent_target": "agent4",
            "assigned_agent": "agent4",
            "authorized_agents": ["agent4"],
            "reply_channel": "file",
        }
        task.update(task_overrides)
        write_json(rd / "tasks" / "bound-task.json", task)
        return rd, argparse.Namespace(
            root=str(root), run_id="run", task_id="bound-task", dry_run=False,
            force=False, spawn_command="", runner_command="",
        )

    def write_bindings(self, root: Path, entries):
        path = root / "boot-role-bindings.json"
        write_json(
            path,
            {
                "schema_version": "mai-boot-role-bindings-v2",
                "boot_id": "boot-0123456789abcdef0123456789abcdef",
                "entries": entries,
            },
        )
        return path

    def entry(self, **overrides):
        entry = {
            "agent_name": "agent4",
            "pane_id": "%369",
            "cli": "claude",
            "role_status": "confirmed",
            "boot_cycle": "boot-0123456789abcdef0123456789abcdef",
            "invocation_id": "boot-0123456789abcdef0123456789abcdef:agent4:%369",
        }
        entry.update(overrides)
        return entry

    def test_missing_binding_blocks_before_target_resolution_or_transport(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            rd, args = self.make_run(root)
            missing = root / "missing-boot-role-bindings.json"
            with patch.object(agent_loop, "BOOT_ROLE_BINDINGS_PATH", missing), \
                    patch.object(agent_loop, "resolve_agent_target", return_value="%369") as resolve, \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 2)
            resolve.assert_not_called()
            subprocess_run.assert_not_called()
            task = json.loads((rd / "tasks" / "bound-task.json").read_text(encoding="utf-8"))
            self.assertEqual(task["status"], "blocked")
            self.assertEqual(task["blocked_reason"], "target_out_of_binding")

    def test_target_absent_from_active_bindings_blocks_before_transport(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            rd, args = self.make_run(root)
            bindings = self.write_bindings(root, [self.entry(agent_name="agent5", pane_id="%370", invocation_id="boot-0123456789abcdef0123456789abcdef:agent5:%370")])
            with patch.object(agent_loop, "BOOT_ROLE_BINDINGS_PATH", bindings), \
                    patch.object(agent_loop, "resolve_agent_target", return_value="%369") as resolve, \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 2)
            resolve.assert_not_called()
            subprocess_run.assert_not_called()
            task = json.loads((rd / "tasks" / "bound-task.json").read_text(encoding="utf-8"))
            self.assertEqual(task["status"], "blocked")
            self.assertEqual(task["blocked_reason"], "target_out_of_binding")

    def test_ambiguous_active_binding_blocks_before_transport(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            rd, args = self.make_run(root)
            bindings = self.write_bindings(
                root,
                [
                    self.entry(),
                    self.entry(
                        pane_id="%370",
                        invocation_id="boot-0123456789abcdef0123456789abcdef:agent4:%370",
                    ),
                ],
            )
            with patch.object(agent_loop, "BOOT_ROLE_BINDINGS_PATH", bindings), \
                    patch.object(agent_loop, "resolve_agent_target", return_value="%369") as resolve, \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 2)
            resolve.assert_not_called()
            subprocess_run.assert_not_called()
            task = json.loads((rd / "tasks" / "bound-task.json").read_text(encoding="utf-8"))
            self.assertEqual(task["status"], "blocked")
            self.assertEqual(task["blocked_reason"], "target_ambiguous")

    def test_valid_binding_is_dispatched_once(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            rd, args = self.make_run(root)
            bindings = self.write_bindings(root, [self.entry()])
            with patch.object(agent_loop, "BOOT_ROLE_BINDINGS_PATH", bindings), \
                    patch.object(agent_loop, "resolve_agent_target", return_value="%369"), \
                    patch.object(agent_loop, "bridge_model_info", return_value={}), \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 0)
            subprocess_run.assert_called_once()
            task = json.loads((rd / "tasks" / "bound-task.json").read_text(encoding="utf-8"))
            self.assertEqual(task["status"], "dispatched")


class ExistingTmuxAcknowledgementTests(unittest.TestCase):
    def make_run(self, tmp: Path) -> tuple[Path, argparse.Namespace]:
        rd = tmp / "run"
        write_json(
            rd / "run.json",
            {
                "schema_version": "agent-loop-run-v1",
                "run_id": "run",
                "policies": {"allowed_backends": ["existing_tmux"], "max_concurrent_tasks": 1},
            },
        )
        write_json(
            rd / "tasks" / "ack-task.json",
            {
                "task_id": "ack-task",
                "status": "pending",
                "created_at": "2026-07-11T00:00:00Z",
                "updated_at": "2026-07-11T00:00:00Z",
                "objective": "Prouver la prise en charge",
                "acceptance": "ACK puis resultat terminal",
                "backend": "existing_tmux",
                "delegate": True,
                "agent_target": "agent4",
                "assigned_agent": "agent4",
                "authorized_agents": ["agent4"],
                "reply_channel": "file",
                "acknowledged_at": "2026-07-10T00:00:00Z",
                "acknowledged_by": "old-worker",
                "started_at": "2026-07-10T00:00:00Z",
            },
        )
        args = argparse.Namespace(
            root=str(tmp), run_id="run", task_id="ack-task", dry_run=False,
            force=False, spawn_command="", runner_command="",
        )
        return rd, args

    def test_dispatch_is_not_running_until_worker_acknowledges(self):
        with tempfile.TemporaryDirectory() as raw:
            rd, args = self.make_run(Path(raw))
            with patch.object(agent_loop, "active_existing_tmux_binding", return_value=(True, "", {"agent_name": "agent4", "pane_id": "%7", "source_ref": "test"})), \
                    patch.object(agent_loop, "resolve_agent_target", return_value="%7"), \
                    patch.object(agent_loop, "bridge_model_info", return_value={}), \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 0)
            sent_message = subprocess_run.call_args.args[0][-1]
            self.assertIn("TU ES LE WORKER CIBLE, pas un relais", sent_message)
            self.assertIn(" ack --root ", sent_message)
            task = json.loads((rd / "tasks" / "ack-task.json").read_text(encoding="utf-8"))
            self.assertEqual(task["status"], "dispatched")
            self.assertEqual(task["acknowledged_at"], "")
            self.assertEqual(task["acknowledged_by"], "")
            self.assertEqual(task["started_at"], "")
            candidates, reason = agent_loop.scheduler_candidates(rd, 1)
            self.assertEqual(candidates, [])
            self.assertIn("max_concurrent_tasks reached", reason)

            ack_args = argparse.Namespace(root=str(Path(raw)), run_id="run", task_id="ack-task", worker="agent4")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_ack(ack_args), 0)
            task = json.loads((rd / "tasks" / "ack-task.json").read_text(encoding="utf-8"))
            self.assertEqual(task["status"], "running")
            self.assertEqual(task["acknowledged_by"], "agent4")
            events = [json.loads(line) for line in (rd / "events.jsonl").read_text(encoding="utf-8").splitlines()]
            self.assertIn("task.acknowledged", [event["event"] for event in events])

    def test_dispatch_passes_background_identity_to_bridge(self):
        with tempfile.TemporaryDirectory() as raw:
            _, args = self.make_run(Path(raw))
            with patch.object(agent_loop, "active_existing_tmux_binding", return_value=(True, "", {"agent_name": "agent4", "pane_id": "%7", "source_ref": "test"})), \
                    patch.object(agent_loop, "resolve_agent_target", return_value="%7"), \
                    patch.object(agent_loop, "bridge_model_info", return_value={}), \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 0)
            env = subprocess_run.call_args.kwargs["env"]
            self.assertEqual(env["BRIDGE_BACKGROUND_SUBJECT"], "agent-loop-dispatch")
            self.assertEqual(env["BRIDGE_BACKGROUND_ROLE"], "agent-loop")

    def test_heartbeat_passes_background_identity_to_bridge(self):
        session = {"pane": "%7", "agent_name": "agent4"}
        with patch.object(agent_loop.subprocess, "run") as subprocess_run:
            agent_loop.send_orchestrator_message(Path("/tmp/run"), session, "heartbeat")
        sent = subprocess_run.call_args.args[0]
        env = subprocess_run.call_args.kwargs["env"]
        self.assertEqual(sent[0], str(agent_loop.BRIDGE))
        self.assertEqual(sent[3], "%7")
        self.assertEqual(env["BRIDGE_BACKGROUND_SUBJECT"], "agent-loop-heartbeat")
        self.assertEqual(env["BRIDGE_BACKGROUND_ROLE"], "agent-loop")

    def test_heartbeat_distinguishes_missing_ack_from_missing_result(self):
        with tempfile.TemporaryDirectory() as raw:
            rd, args = self.make_run(Path(raw))
            with patch.object(agent_loop, "active_existing_tmux_binding", return_value=(True, "", {"agent_name": "agent4", "pane_id": "%7", "source_ref": "test"})), \
                    patch.object(agent_loop, "resolve_agent_target", return_value="%7"), \
                    patch.object(agent_loop, "bridge_model_info", return_value={}), \
                    patch.object(agent_loop.subprocess, "run"), \
                    contextlib.redirect_stdout(io.StringIO()):
                agent_loop.cmd_dispatch(args)
            with patch.object(agent_loop, "age_seconds", return_value=600), \
                    patch.object(agent_loop, "agent_by_pane", return_value={"%7": {}}):
                alert = agent_loop.running_task_alerts(rd, 60)[0]
            self.assertEqual(alert["event"], "task.dispatched_no_ack")
            self.assertEqual(alert["status"], "dispatched")

    def test_transport_failure_cannot_leave_false_dispatched_state(self):
        with tempfile.TemporaryDirectory() as raw:
            rd, args = self.make_run(Path(raw))
            failure = agent_loop.subprocess.CalledProcessError(17, ["bridge.sh", "send"])
            with patch.object(agent_loop, "active_existing_tmux_binding", return_value=(True, "", {"agent_name": "agent4", "pane_id": "%7", "source_ref": "test"})), \
                    patch.object(agent_loop, "resolve_agent_target", return_value="%7"), \
                    patch.object(agent_loop, "bridge_model_info", return_value={}), \
                    patch.object(agent_loop.subprocess, "run", side_effect=failure), \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 2)
            task = json.loads((rd / "tasks" / "ack-task.json").read_text(encoding="utf-8"))
            self.assertEqual(task["status"], "blocked")
            self.assertEqual(task["blocked_reason"], "dispatch_transport_failed")

    def test_ack_before_dispatch_is_rejected(self):
        with tempfile.TemporaryDirectory() as raw:
            self.make_run(Path(raw))
            args = argparse.Namespace(root=raw, run_id="run", task_id="ack-task", worker="agent4")
            with self.assertRaises(SystemExit) as raised:
                agent_loop.cmd_ack(args)
            self.assertIn("expected 'dispatched'", str(raised.exception))


class RoleRecoveryDispatchGateTests(unittest.TestCase):
    def test_runtime_project_exposes_canonical_policy_and_worker_role(self):
        project = agent_loop.ROLE_RECOVERY_PROJECT_ROOT
        policy = (project / "AGENTS.md").read_text(encoding="utf-8")
        roles = json.loads(
            (project / "40-registry" / "agents" / "roles.json").read_text(encoding="utf-8")
        )

        self.assertEqual(
            policy.count("<!-- ROLE-RECOVERY:START policy=role-recovery-policy-v1 -->"),
            1,
        )
        self.assertEqual(policy.count("<!-- ROLE-RECOVERY:END -->"), 1)
        self.assertIn("worker", roles["roles"])

    def make_project(self, tmp: Path) -> Path:
        project = tmp / "project"
        write_json(
            project / "40-registry" / "agents" / "roles.json",
            {"schema_version": "agent-roles-v1", "roles": {"worker": {"mandat": "Tester"}}},
        )
        contracts = project / "specs" / "101-role-prewarm-recovery" / "contracts"
        write_json(
            contracts / "mai-task-provenance-verdict-v1.schema.json",
            {"$id": "https://maicompany.local/contracts/mai-task-provenance-verdict-v1.schema.json"},
        )
        write_json(
            contracts / "mai-role-recovery-assertion-v1.schema.json",
            {"$id": "https://maicompany.local/contracts/mai-role-recovery-assertion-v1.schema.json"},
        )
        (project / "AGENTS.md").write_text(
            "<!-- ROLE-RECOVERY:START policy=role-recovery-policy-v1 -->\n"
            "<!-- ROLE-RECOVERY:END -->\n",
            encoding="utf-8",
        )
        return project

    def make_run(self, tmp: Path, **task_overrides) -> tuple[Path, dict]:
        rd = tmp / "coordination" / "run"
        write_json(
            rd / "run.json",
            {
                "schema_version": "agent-loop-run-v1",
                "run_id": "run",
                "policies": {"role_recovery_enforced": True},
            },
        )
        task = {
            "schema_version": "agent-loop-task-v1",
            "task_id": "role-task",
            "status": "dispatched",
            "backend": "existing_tmux",
            "agent_target": "agent4",
            "assigned_agent": "agent4",
            "agent_role": "worker",
            "context": "fresh",
            "reply_channel": "file",
            "delegate": True,
            "objective": "Tester la récupération de rôle",
            "acceptance": "Verdict prouvé",
        }
        task.update(task_overrides)
        write_json(rd / "tasks" / "role-task.json", task)
        return rd, task

    def test_valid_task_produces_confirmed_ephemeral_assertion(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, task = self.make_run(tmp)
            verdict = agent_loop.evaluate_role_recovery_dispatch(
                rd, task, current_agent="agent4", cli_family="claude", project_root=project,
                coordination_root=tmp / "coordination",
            )
            self.assertTrue(verdict["allowed"])
            self.assertEqual(verdict["assertion"]["verdict"], "confirmed")
            self.assertEqual(verdict["assertion"]["cli_family"], "claude")
            self.assertEqual(verdict["assertion"]["role"], "worker")
            self.assertFalse(verdict["assertion"]["raw_pane_text_used"])

    def test_continuation_without_snapshot_is_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, task = self.make_run(tmp, context="continuation")
            verdict = agent_loop.evaluate_role_recovery_dispatch(
                rd, task, current_agent="agent4", cli_family="claude", project_root=project,
                coordination_root=tmp / "coordination",
            )
            self.assertFalse(verdict["allowed"])
            self.assertIn("resume_snapshot_missing", verdict["reason_codes"])

    def test_valid_continuation_reuses_task_snapshot(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, task = self.make_run(
                tmp, context="continuation", resume_from="snapshots/role-task-checkpoint.json"
            )
            write_json(
                rd / "snapshots" / "role-task-checkpoint.json",
                {"schema_version": "task-snapshot-v1", "task_id": "role-task"},
            )
            verdict = agent_loop.evaluate_role_recovery_dispatch(
                rd, task, current_agent="agent4", cli_family="claude", project_root=project,
                coordination_root=tmp / "coordination",
            )
            self.assertTrue(verdict["allowed"])
            self.assertEqual(verdict["assertion"]["context"], "continuation")

    def test_missing_agent_role_and_terminal_run_are_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, task = self.make_run(tmp, assigned_agent="", agent_role="")
            run = json.loads((rd / "run.json").read_text(encoding="utf-8"))
            run["status"] = "completed"
            write_json(rd / "run.json", run)
            verdict = agent_loop.evaluate_role_recovery_dispatch(
                rd, task, current_agent="agent4", cli_family="codex", project_root=project,
                coordination_root=tmp / "coordination",
            )
            self.assertFalse(verdict["allowed"])
            self.assertIn("assigned_agent_missing", verdict["reason_codes"])
            self.assertIn("role_source_missing", verdict["reason_codes"])
            self.assertIn("run_terminal", verdict["reason_codes"])

    def test_mismatch_and_fixture_path_are_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, task = self.make_run(tmp, assigned_agent="agent9")
            fixture_rd = tmp / "fixtures" / "run"
            fixture_rd.mkdir(parents=True)
            (fixture_rd / "run.json").write_text((rd / "run.json").read_text(), encoding="utf-8")
            (fixture_rd / "tasks").mkdir()
            write_json(fixture_rd / "tasks" / "role-task.json", task)
            verdict = agent_loop.evaluate_role_recovery_dispatch(
                fixture_rd, task, current_agent="agent4", cli_family="codex", project_root=project,
                coordination_root=tmp,
            )
            self.assertFalse(verdict["allowed"])
            self.assertIn("assigned_agent_mismatch", verdict["reason_codes"])
            self.assertIn("test_or_document_path", verdict["reason_codes"])

    def test_path_outside_coordination_is_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, task = self.make_run(tmp)
            verdict = agent_loop.evaluate_role_recovery_dispatch(
                rd, task, current_agent="agent4", cli_family="codex", project_root=project,
                coordination_root=tmp / "another-coordination",
            )
            self.assertFalse(verdict["allowed"])
            self.assertIn("path_outside_coordination", verdict["reason_codes"])

    def test_two_active_tasks_for_same_agent_are_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, task = self.make_run(tmp)
            sibling = dict(task, task_id="other-task")
            write_json(rd / "tasks" / "other-task.json", sibling)
            verdict = agent_loop.evaluate_role_recovery_dispatch(
                rd, task, current_agent="agent4", cli_family="codex", project_root=project,
                coordination_root=tmp / "coordination",
            )
            self.assertFalse(verdict["allowed"])
            self.assertIn("ambiguous_active_tasks", verdict["reason_codes"])

    def test_cmd_dispatch_invalid_gate_never_resolves_or_transports(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, task = self.make_run(tmp, status="pending", assigned_agent="agent9")
            args = argparse.Namespace(
                root=str(tmp / "coordination"), run_id="run", task_id="role-task",
                dry_run=False, force=False, spawn_command="", runner_command="",
            )
            with patch.object(agent_loop, "ROLE_RECOVERY_PROJECT_ROOT", project), \
                    patch.object(agent_loop, "resolve_agent_target") as resolve, \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 2)
            resolve.assert_not_called()
            subprocess_run.assert_not_called()
            stored = json.loads((rd / "tasks" / "role-task.json").read_text(encoding="utf-8"))
            self.assertEqual(stored["status"], "blocked")
            self.assertEqual(stored["blocked_reason"], "role_recovery_gate_failed")

    def test_cmd_dispatch_valid_gate_transports_once_and_records_confirmation(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, _ = self.make_run(tmp, status="pending")
            args = argparse.Namespace(
                root=str(tmp / "coordination"), run_id="run", task_id="role-task",
                dry_run=False, force=False, spawn_command="", runner_command="",
            )
            with patch.object(agent_loop, "ROLE_RECOVERY_PROJECT_ROOT", project), \
                    patch.object(agent_loop, "ROLE_RECOVERY_COORDINATION_ROOT", tmp / "coordination"), \
                    patch.object(agent_loop, "resolve_agent_target", return_value="agent4") as resolve, \
                    patch.object(agent_loop, "bridge_model_info", return_value={}), \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 0)
            resolve.assert_called_once_with("agent4")
            subprocess_run.assert_called_once()
            stored = json.loads((rd / "tasks" / "role-task.json").read_text(encoding="utf-8"))
            self.assertEqual(stored["status"], "dispatched")
            self.assertEqual(stored["role_recovery_gate"], "confirmed")

    def test_cmd_dispatch_resolved_agent_mismatch_never_transports(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, _ = self.make_run(tmp, status="pending")
            args = argparse.Namespace(
                root=str(tmp / "coordination"), run_id="run", task_id="role-task",
                dry_run=False, force=False, spawn_command="", runner_command="",
            )
            with patch.object(agent_loop, "ROLE_RECOVERY_PROJECT_ROOT", project), \
                    patch.object(agent_loop, "ROLE_RECOVERY_COORDINATION_ROOT", tmp / "coordination"), \
                    patch.object(agent_loop, "active_existing_tmux_binding", return_value=(True, "", {"agent_name": "agent4", "pane_id": "%7", "source_ref": "test"})), \
                    patch.object(agent_loop, "resolve_agent_target", return_value="%7"), \
                    patch.object(agent_loop, "bridge_model_info", return_value={"agent_name": "agent9"}), \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 2)
            subprocess_run.assert_not_called()
            stored = json.loads((rd / "tasks" / "role-task.json").read_text(encoding="utf-8"))
            self.assertEqual(stored["blocked_reason"], "role_recovery_resolved_recipient_mismatch")

    def test_cmd_dispatch_dry_run_refusal_does_not_mutate_task(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, original = self.make_run(tmp, status="pending", assigned_agent="agent9")
            before = (rd / "tasks" / "role-task.json").read_text(encoding="utf-8")
            args = argparse.Namespace(
                root=str(tmp / "coordination"), run_id="run", task_id="role-task",
                dry_run=True, force=False, spawn_command="", runner_command="",
            )
            with patch.object(agent_loop, "ROLE_RECOVERY_PROJECT_ROOT", project), \
                    patch.object(agent_loop, "resolve_agent_target") as resolve, \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 2)
            resolve.assert_not_called()
            self.assertEqual((rd / "tasks" / "role-task.json").read_text(encoding="utf-8"), before)

    def test_cmd_dispatch_dry_run_valid_worker_never_resolves_or_transports(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, _ = self.make_run(tmp, status="pending", agent_role="worker")
            before = (rd / "tasks" / "role-task.json").read_text(encoding="utf-8")
            args = argparse.Namespace(
                root=str(tmp / "coordination"), run_id="run", task_id="role-task",
                dry_run=True, force=False, spawn_command="", runner_command="",
            )
            stdout = io.StringIO()
            with patch.object(agent_loop, "ROLE_RECOVERY_PROJECT_ROOT", project), \
                    patch.object(agent_loop, "ROLE_RECOVERY_COORDINATION_ROOT", tmp / "coordination"), \
                    patch.object(agent_loop, "resolve_agent_target") as resolve, \
                    patch.object(agent_loop, "bridge_model_info") as model_info, \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 0)
            resolve.assert_not_called()
            model_info.assert_not_called()
            subprocess_run.assert_not_called()
            self.assertIn("role_recovery_gate: confirmed", stdout.getvalue())
            self.assertEqual((rd / "tasks" / "role-task.json").read_text(encoding="utf-8"), before)

    def test_cmd_dispatch_dry_run_missing_policy_is_refused_without_transport(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            (project / "AGENTS.md").write_text("# Sans politique 101\n", encoding="utf-8")
            rd, _ = self.make_run(tmp, status="pending", agent_role="worker")
            args = argparse.Namespace(
                root=str(tmp / "coordination"), run_id="run", task_id="role-task",
                dry_run=True, force=False, spawn_command="", runner_command="",
            )
            with patch.object(agent_loop, "ROLE_RECOVERY_PROJECT_ROOT", project), \
                    patch.object(agent_loop, "ROLE_RECOVERY_COORDINATION_ROOT", tmp / "coordination"), \
                    patch.object(agent_loop, "resolve_agent_target") as resolve, \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 2)
            resolve.assert_not_called()
            subprocess_run.assert_not_called()

    def test_cmd_dispatch_dry_run_unknown_role_is_refused_without_transport(self):
        with tempfile.TemporaryDirectory() as raw:
            tmp = Path(raw)
            project = self.make_project(tmp)
            rd, _ = self.make_run(tmp, status="pending", agent_role="invented")
            args = argparse.Namespace(
                root=str(tmp / "coordination"), run_id="run", task_id="role-task",
                dry_run=True, force=False, spawn_command="", runner_command="",
            )
            with patch.object(agent_loop, "ROLE_RECOVERY_PROJECT_ROOT", project), \
                    patch.object(agent_loop, "ROLE_RECOVERY_COORDINATION_ROOT", tmp / "coordination"), \
                    patch.object(agent_loop, "resolve_agent_target") as resolve, \
                    patch.object(agent_loop.subprocess, "run") as subprocess_run, \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 2)
            resolve.assert_not_called()
            subprocess_run.assert_not_called()

    def test_add_task_writes_role_recovery_metadata(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            rd = root / "run"
            write_json(rd / "run.json", {"schema_version": "agent-loop-run-v1", "run_id": "run"})
            args = argparse.Namespace(
                root=raw, run_id="run", task_id="created", objective="Tester",
                acceptance="Preuve", delegate="true", backend="existing_tmux",
                agent_target="agent4", agent_role="reviewer", context="continuation",
                resume_from="snapshots/created.json", depends_on="", parallelizable="false",
                priority=100, conflicts_with="", parent_task_id="", spawn_reason="",
                max_attempts=1, reply_channel="file", runner_command="", timeout_sec=0,
                keep_alive="false", agent_provider="codex_exec", agent_model="",
                agent_command_template="", input_paths="", output_paths="", forbidden_paths="",
            )
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_add_task(args), 0)
            stored = json.loads((rd / "tasks" / "created.json").read_text(encoding="utf-8"))
            self.assertEqual(stored["assigned_agent"], "agent4")
            self.assertEqual(stored["agent_role"], "reviewer")
            self.assertEqual(stored["context"], "continuation")
            self.assertEqual(stored["resume_from"], "snapshots/created.json")


class HeartbeatBackoffTests(unittest.TestCase):
    def make_run(self, tmp: Path) -> Path:
        rd = tmp / "run"
        write_json(
            rd / "run.json",
            {
                "schema_version": "agent-loop-run-v1",
                "run_id": "unit",
                "policies": {"running_task_notice_sec": 60},
            },
        )
        return rd

    def test_ready_alert_signature_ignores_age_bucket(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            task = {
                "task_id": "ready-task",
                "status": "pending",
                "created_at": "2026-07-08T00:00:00Z",
                "updated_at": "2026-07-08T00:00:00Z",
                "backend": "existing_tmux",
                "agent_target": "agent10",
                "delegate": True,
            }
            with patch.object(agent_loop, "age_seconds", return_value=120):
                alerts = agent_loop.ready_task_alerts(rd, [task], 60)
            self.assertEqual(alerts[0]["event"], "task.ready_waiting")
            self.assertEqual(alerts[0]["age_bucket"], 2)
            first = agent_loop.notify_signature(alerts)
            with patch.object(agent_loop, "age_seconds", return_value=180):
                later = agent_loop.notify_signature(agent_loop.ready_task_alerts(rd, [task], 60))
            self.assertEqual(first, later)

    def test_ready_work_is_filtered_per_agent_not_globally(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            write_json(
                rd / "tasks" / "agent-a-running.json",
                {
                    "task_id": "agent-a-running",
                    "status": "running",
                    "assigned_agent": "agent-a",
                },
            )
            ready = [
                {"task_id": "agent-a-next", "status": "pending", "assigned_agent": "agent-a"},
                {"task_id": "agent-b-next", "status": "pending", "assigned_agent": "agent-b"},
                {"task_id": "local-next", "status": "pending"},
            ]

            filtered = agent_loop.ready_tasks_for_idle_agents(rd, ready)

            self.assertEqual(
                [task["task_id"] for task in filtered],
                ["agent-b-next", "local-next"],
            )

            run = json.loads((rd / "run.json").read_text(encoding="utf-8"))
            run["policies"]["max_concurrent_tasks"] = 1
            write_json(rd / "run.json", run)
            self.assertEqual(agent_loop.ready_tasks_for_idle_agents(rd, ready), [])

    def test_collected_review_alerts_until_arbitrated(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            write_json(
                rd / "tasks" / "review-task.json",
                {
                    "task_id": "review-task",
                    "status": "review",
                    "created_at": "2026-07-08T00:00:00Z",
                    "updated_at": "2026-07-08T00:05:00Z",
                    "collected_at": "2026-07-08T00:05:00Z",
                    "result_path": str(rd / "results" / "review-task.result.json"),
                },
            )
            with patch.object(agent_loop, "age_seconds", return_value=360):
                alerts = agent_loop.collected_task_alerts(rd, 60)
            self.assertEqual(len(alerts), 1)
            self.assertEqual(alerts[0]["event"], "task.collected_needs_arbitration")
            self.assertEqual(alerts[0]["age_bucket"], 6)

    def test_orchestrator_missing_writes_decision_request(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            path = agent_loop.write_orchestrator_missing_decision_request(
                rd,
                "orchestrator",
                ready_count=2,
                collected_count=1,
                running_alert_count=0,
            )
            data = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(data["schema_version"], "decision-request-v1")
            self.assertEqual(data["status"], "open")
            self.assertEqual(data["level"], "decision_requise")
            self.assertIn("ready=2", data["why_escalated"])


class MissionControlV2Tests(unittest.TestCase):
    def make_run(self, root: Path) -> Path:
        rd = root / "run"
        write_json(
            rd / "run.json",
            {
                "schema_version": "agent-loop-run-v1",
                "run_id": "run",
                "status": "open",
                "mission_control_enabled_at": "2026-10-05T08:00:00Z",
                "policies": {
                    "allowed_backends": ["existing_bridget"],
                    "ack_notice_sec": 120,
                    "progress_notice_sec": 300,
                    "reminder_interval_sec": 120,
                    "escalation_after_reminders": 2,
                },
            },
        )
        return rd

    def task_args(self, root: str) -> argparse.Namespace:
        return argparse.Namespace(
            root=root, run_id="run", task_id="mission", objective="Corriger",
            acceptance="preuve testée", delegate="true", backend="existing_bridget",
            agent_target="worker-uuid", agent_role="worker", context="fresh",
            resume_from="", depends_on="", parallelizable="false", priority=100,
            conflicts_with="", parent_task_id="", spawn_reason="", max_attempts=2,
            reply_channel="file", runner_command="", timeout_sec=0, keep_alive="false",
            agent_provider="", agent_model="", agent_command_template="", cwd="",
            input_paths="", output_paths="", forbidden_paths="", owner="",
            due_at="", expected_result="", work_kind="code",
        )

    def test_add_task_records_owner_due_date_and_expected_result(self):
        with tempfile.TemporaryDirectory() as raw:
            self.make_run(Path(raw))
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_add_task(self.task_args(raw)), 0)
            task = agent_loop.load_task(Path(raw) / "run", "mission")
            self.assertEqual(task["assigned_agent"], "worker-uuid")
            self.assertEqual(task["expected_result"], "preuve testée")
            self.assertEqual(task["work_kind"], "code")
            self.assertTrue(task["due_at"].endswith("Z"))

    def test_ack_is_initial_verified_progress(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            write_json(rd / "tasks" / "mission.json", {
                "task_id": "mission", "status": "dispatched", "agent_target": "worker-uuid"
            })
            args = argparse.Namespace(root=raw, run_id="run", task_id="mission", worker="worker-uuid")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_ack(args), 0)
            task = agent_loop.load_task(rd, "mission")
            self.assertEqual(task["last_progress_kind"], "worker_ack")
            self.assertEqual(task["last_progress_ref"], "ack:worker-uuid")
            self.assertEqual(task["last_progress_at"], task["acknowledged_at"])

    def test_progress_requires_a_typed_reference(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            source = Path(raw) / "source.py"
            source.write_text("before", encoding="utf-8")
            baseline = agent_loop.progress_snapshot({"input_paths": [str(source)]})
            write_json(rd / "tasks" / "mission.json", {
                "task_id": "mission", "status": "running", "agent_target": "worker-uuid",
                "input_paths": [str(source)], "progress_baseline": baseline,
            })
            bad = argparse.Namespace(
                root=raw, run_id="run", task_id="mission", worker="worker-uuid",
                kind="source_changed", evidence_ref="je vais le faire",
            )
            with self.assertRaisesRegex(SystemExit, "evidence-ref"):
                agent_loop.cmd_progress(bad)
            good = argparse.Namespace(
                root=raw, run_id="run", task_id="mission", worker="worker-uuid",
                kind="source_changed", evidence_ref=f"file:{source}",
            )
            with self.assertRaisesRegex(SystemExit, "no verified change"):
                agent_loop.cmd_progress(good)
            source.write_text("after", encoding="utf-8")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_progress(good), 0)
            task = agent_loop.load_task(rd, "mission")
            self.assertEqual(task["last_progress_kind"], "source_changed")
            self.assertEqual(task["last_progress_ref"], f"file:{source}")
            before = (rd / "tasks" / "mission.json").read_bytes()
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_progress(good), 0)
            self.assertEqual((rd / "tasks" / "mission.json").read_bytes(), before)

    def test_issue_state_advances_once_per_due_interval_then_stops(self):
        event = {
            "event": "task.running_no_progress", "task_id": "mission",
            "status": "running", "evidence_marker": "ack:one", "routing": "worker",
        }
        first = agent_loop.advance_issue_state(event, None, interval_elapsed=True, escalation_after=2)
        self.assertEqual(first["reminder_count"], 1)
        self.assertEqual(first["audience"], "worker")
        quiet = agent_loop.advance_issue_state(event, first, interval_elapsed=False, escalation_after=2)
        self.assertIsNone(quiet)
        second = agent_loop.advance_issue_state(event, first, interval_elapsed=True, escalation_after=2)
        self.assertEqual(second["audience"], "coordinator")
        third = agent_loop.advance_issue_state(event, second, interval_elapsed=True, escalation_after=2)
        self.assertEqual(third["audience"], "escalation")
        self.assertIsNone(agent_loop.advance_issue_state(event, third, interval_elapsed=True, escalation_after=2))

    def test_disposition_preserves_terminal_verdict(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            write_json(rd / "tasks" / "mission.json", {
                "task_id": "mission", "status": "review", "collected_at": "2026-10-05T08:01:00Z"
            })
            write_json(rd / "tasks" / "correction.json", {
                "task_id": "correction", "status": "running", "assigned_agent": "worker-uuid"
            })
            args = argparse.Namespace(
                root=raw, run_id="run", task_id="mission", decision="correction_assigned",
                reason_code="correction_required", owner="worker-uuid",
                next_check="", successor_task_id="correction",
            )
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_disposition(args), 0)
            task = agent_loop.load_task(rd, "mission")
            self.assertEqual(task["status"], "review")
            self.assertEqual(task["disposition"]["decision"], "correction_assigned")

    def test_close_run_refuses_open_work_then_closes_clean_run(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            write_json(rd / "tasks" / "mission.json", {"task_id": "mission", "status": "running"})
            args = argparse.Namespace(root=raw, run_id="run", reason="Terminé")
            with self.assertRaisesRegex(SystemExit, "open obligations"):
                agent_loop.cmd_close_run(args)
            write_json(rd / "tasks" / "mission.json", {
                "task_id": "mission", "status": "pass", "collected_at": "2026-10-05T08:01:00Z",
                "disposition": {"decision": "accepted"}, "dispositioned_at": "2026-10-05T08:02:00Z",
            })
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_close_run(args), 0)
            self.assertEqual(agent_loop.load_json(rd / "run.json")["status"], "closed")


class MissionHeartbeatIntegrationTests(unittest.TestCase):
    make_run = MissionControlV2Tests.make_run
    def make_task(self, rd, task_id="mission", **overrides):
        task = {
            "task_id": task_id, "status": "dispatched", "backend": "existing_bridget",
            "delegate": True, "agent_target": "worker-uuid", "assigned_agent": "worker-uuid",
            "resolved_target": "worker-uuid", "objective": "PRIVATE_PROMPT",
            "acceptance": "OPAQUE_REASONING", "dispatched_at": "2026-10-05T08:00:00Z",
            "created_at": "2026-10-05T08:00:00Z", "updated_at": "2026-10-05T08:00:00Z",
        }
        task.update(overrides)
        write_json(rd / "tasks" / f"{task_id}.json", task)
        return task

    def args(self, root, **overrides):
        result = argparse.Namespace(root=str(root), run_id="run", dry_run=False,
                                    orchestrator_role="", force=False, verbose=True)
        for key, value in overrides.items():
            setattr(result, key, value)
        return result

    def role_session(self, rd, role):
        return {"backend": "existing_bridget", "agent_id": role + "-uuid", "status": "live"}

    def tick(self, root, send, *, age=120, role=None, dry_run=False, live=True):
        with patch.object(agent_loop, "cmd_refresh_sessions", return_value=0), \
                patch.object(agent_loop, "bridget_agents_by_id", return_value={"worker-uuid": {"state": "connected"}} if live else {}), \
                patch.object(agent_loop, "live_session_for_role", side_effect=role or self.role_session), \
                patch.object(agent_loop, "age_seconds", side_effect=age if callable(age) else None, return_value=age), \
                patch.object(agent_loop, "send_orchestrator_message", send), \
                contextlib.redirect_stdout(io.StringIO()):
            return agent_loop.cmd_heartbeat(self.args(root, dry_run=dry_run))

    def test_ack_threshold_then_silence_then_coordinator_then_root(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd)
            send = Mock()
            self.tick(raw, send, age=119)
            send.assert_not_called()
            self.tick(raw, send, age=120)
            self.assertEqual(send.call_args.args[1]["agent_id"], "worker-uuid")
            self.tick(raw, send, age=lambda value: 120 if value == "2026-10-05T08:00:00Z" else 0)
            self.assertEqual(send.call_count, 1)
            self.tick(raw, send, age=120)
            self.assertEqual(send.call_args.args[1]["agent_id"], "orchestrator-uuid")
            self.tick(raw, send, age=120)
            self.assertEqual(send.call_args.args[1]["agent_id"], "root-uuid")
            self.tick(raw, send, age=999)
            self.assertEqual(send.call_count, 3)

    def test_progress_threshold_299_and_300(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="running", last_progress_at="2026-10-05T08:02:00Z")
            send = Mock()
            self.tick(raw, send, age=299)
            send.assert_not_called()
            self.tick(raw, send, age=300)
            self.assertIn("task.running_no_progress", send.call_args.args[2])

    def test_two_issues_for_same_recipient_produce_one_private_digest(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, task_id="one")
            self.make_task(rd, task_id="two")
            send = Mock()
            self.tick(raw, send)
            send.assert_called_once()
            message = send.call_args.args[2]
            self.assertIn("actions: 2", message)
            self.assertNotIn("PRIVATE_PROMPT", message)
            self.assertNotIn("OPAQUE_REASONING", message)

    def test_disconnected_worker_escalates_immediately(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd)
            send = Mock()
            self.tick(raw, send, live=False)
            self.assertEqual(send.call_args.args[1]["agent_id"], "orchestrator-uuid")
            self.assertIn("recipient_unreachable", (rd / "events.jsonl").read_text())

    def test_failed_transport_escalates_to_coordinator_in_same_tick(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd)
            send = Mock(side_effect=[agent_loop.subprocess.CalledProcessError(1, ["bridget"]), None])
            self.tick(raw, send)
            self.assertEqual(send.call_count, 2)
            self.assertEqual(send.call_args.args[1]["agent_id"], "orchestrator-uuid")

    def test_missing_all_recipients_writes_one_durable_decision(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd)
            send = Mock()
            self.tick(raw, send, live=False, role=lambda rd, role: None)
            send.assert_not_called()
            self.assertEqual(len(list((rd / "decisions").glob("*.json"))), 1)
            before = (rd / "events.jsonl").read_bytes()
            self.tick(raw, send, age=999, live=False, role=lambda rd, role: None)
            self.assertEqual((rd / "events.jsonl").read_bytes(), before)

    def test_failed_initial_coordinator_delivery_reaches_root_in_same_tick(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="review", collected_at="2026-10-05T08:00:01Z")
            send = Mock(side_effect=[agent_loop.subprocess.CalledProcessError(1, ["bridget"]), None])
            self.tick(raw, send)
            self.assertEqual(send.call_count, 2)
            self.assertEqual(send.call_args.args[1]["agent_id"], "root-uuid")

    def test_transport_failure_diagnostics_keep_only_type_and_numeric_code(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd)
            failure = agent_loop.subprocess.CalledProcessError(7, ["PRIVATE_PROMPT"], stderr="OPAQUE_REASONING")
            self.tick(raw, Mock(side_effect=[failure, None]))
            events = (rd / "events.jsonl").read_text()
            self.assertIn('"returncode": 7', events)
            self.assertNotIn("PRIVATE_PROMPT", events)
            self.assertNotIn("OPAQUE_REASONING", events)

    def test_dry_run_has_no_file_or_transport_side_effect(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd)
            before = {str(path): path.read_bytes() for path in rd.rglob("*") if path.is_file()}
            send = Mock()
            self.tick(raw, send, dry_run=True)
            send.assert_not_called()
            self.assertEqual(before, {str(path): path.read_bytes() for path in rd.rglob("*") if path.is_file()})

    def test_healthy_workers_do_not_wake_coordinator(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="running", last_progress_at=agent_loop.now())
            send = Mock()
            self.tick(raw, send, age=10)
            send.assert_not_called()

    def test_new_terminal_result_uses_stable_arbitration_issue(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="running")
            write_json(agent_loop.result_path(rd, "mission"), {"task_id": "mission", "status": "review"})
            send = Mock()
            self.tick(raw, send)
            first = agent_loop.load_json(rd / "heartbeat-state.json")["issues"]
            self.assertIn("task.collected_needs_arbitration:mission", first)
            self.tick(raw, send, age=0)
            self.assertEqual(send.call_count, 1)

    def test_execution_pause_leaves_code_ready(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            run = agent_loop.load_json(rd / "run.json")
            run["policies"]["paused_work_kinds"] = ["execution_authorization"]
            write_json(rd / "run.json", run)
            self.make_task(rd, "execute", status="pending", work_kind="execution_authorization")
            self.make_task(rd, "develop", status="pending", work_kind="code")
            self.assertEqual([task["task_id"] for task in agent_loop.ready_tasks(rd)], ["develop"])

    def test_migration_preserves_history_and_is_idempotent(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            run = agent_loop.load_json(rd / "run.json")
            del run["mission_control_enabled_at"]
            write_json(rd / "run.json", run)
            self.make_task(rd, "old", status="review", attempts=7, collected_at="2026-10-04T08:00:00Z")
            original = (rd / "tasks" / "old.json").read_bytes()
            args = argparse.Namespace(root=raw, run_id="run")
            with contextlib.redirect_stdout(io.StringIO()):
                agent_loop.cmd_migrate_run(args)
                migrated = (rd / "run.json").read_bytes()
                agent_loop.cmd_migrate_run(args)
            self.assertEqual(original, (rd / "tasks" / "old.json").read_bytes())
            self.assertEqual(migrated, (rd / "run.json").read_bytes())
            self.assertFalse(agent_loop.task_requires_disposition(agent_loop.load_json(rd / "run.json"), agent_loop.load_task(rd, "old")))
            with self.assertRaisesRegex(SystemExit, "open obligations"):
                agent_loop.cmd_close_run(argparse.Namespace(root=raw, run_id="run", reason="fait"))

    def test_issue_keys_separate_two_anomalies_on_same_task(self):
        first = {"event": "task.timeout", "task_id": "mission"}
        second = {"event": "task.worker_lost", "task_id": "mission"}
        self.assertNotEqual(agent_loop.issue_key(first), agent_loop.issue_key(second))

    def test_timeout_is_not_reset_by_progress(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="running", timeout_sec=300,
                           last_progress_at="2026-10-05T08:04:00Z")
            with patch.object(agent_loop, "age_seconds", side_effect=lambda value: 600 if value == "2026-10-05T08:00:00Z" else 10), \
                    patch.object(agent_loop, "bridget_agents_by_id", return_value={"worker-uuid": {"state": "connected"}}):
                alert = agent_loop.running_task_alerts(rd)[0]
            self.assertEqual(alert["event"], "task.timeout")

    def test_typed_intention_without_artifact_is_rejected(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="running")
            for kind, reference in [("task_assigned", "task:missing"), ("result_checked", "result:missing"),
                                    ("blocker_reported", "blocker:missing"), ("dependency_handoff", "handoff:missing")]:
                with self.subTest(kind=kind), self.assertRaises(SystemExit):
                    agent_loop.cmd_progress(argparse.Namespace(root=raw, run_id="run", task_id="mission",
                        worker="worker-uuid", kind=kind, evidence_ref=reference))

    def test_terminal_verdict_cannot_be_changed_by_resolve_task(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="review")
            with self.assertRaisesRegex(SystemExit, "immutable"):
                agent_loop.cmd_resolve_task(argparse.Namespace(root=raw, run_id="run", task_id="mission",
                                                               status="pass", reason="faire passer"))
            self.assertEqual(agent_loop.load_task(rd, "mission")["status"], "review")

    def test_escalation_delivery_recovers_when_root_reconnects(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd)
            send = Mock()
            self.tick(raw, send, live=False, role=lambda rd, role: None)
            self.tick(raw, send, live=False)
            send.assert_called_once()
            self.assertEqual(send.call_args.args[1]["agent_id"], "root-uuid")
            self.tick(raw, send, live=False)
            self.assertEqual(send.call_count, 1)

    def test_outbox_freezes_replay_after_crash_following_delivery(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd)
            original_write = agent_loop.write_json

            def fail_after_send(path, data):
                if path == rd / "heartbeat-state.json" and data.get("issues"):
                    raise OSError("simulated process crash")
                return original_write(path, data)

            send = Mock()
            with patch.object(agent_loop, "write_json", side_effect=fail_after_send), self.assertRaises(OSError):
                self.tick(raw, send)
            first_message = send.call_args.args[2]
            first_replay = dict(send.call_args.kwargs["replay"])
            self.tick(raw, send, age=900)
            self.assertEqual(send.call_args.args[2], first_message)
            self.assertEqual(send.call_args.kwargs["replay"], first_replay)
            self.assertEqual(send.call_count, 1)

    def test_stale_task_update_preserves_newer_result_fields(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="running")
            stale = agent_loop.load_task(rd, "mission")
            newer = agent_loop.load_task(rd, "mission")
            agent_loop.update_task(rd, newer, status="review", result_path="durable-result")
            agent_loop.update_task(rd, stale, last_progress_kind="source_changed")
            actual = agent_loop.load_task(rd, "mission")
            self.assertEqual(actual["status"], "review")
            self.assertEqual(actual["result_path"], "durable-result")

    def test_stale_ack_cannot_revive_terminal_task(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd)
            stale = agent_loop.load_task(rd, "mission")
            agent_loop.update_task(rd, agent_loop.load_task(rd, "mission"), status="review")
            with self.assertRaisesRegex(SystemExit, "changed concurrently"):
                agent_loop.update_task(rd, stale, expected_statuses={"dispatched"}, status="running")
            self.assertEqual(agent_loop.load_task(rd, "mission")["status"], "review")

    def test_progress_replay_is_rejected_after_another_proof_and_stale_read(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="running")
            task = agent_loop.load_task(rd, "mission")
            stale = dict(task)
            agent_loop.update_task(rd, task, last_progress_hash="first-proof", last_progress_at="first-time")
            agent_loop.update_task(rd, task, last_progress_hash="second-proof", last_progress_at="second-time")
            before = (rd / "tasks" / "mission.json").read_bytes()
            changed = agent_loop.update_task(rd, stale, last_progress_hash="first-proof", last_progress_at="fake-later-time")
            self.assertFalse(changed)
            self.assertEqual(before, (rd / "tasks" / "mission.json").read_bytes())

    def test_progress_command_rejects_nonconsecutive_result_replay(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="running")
            for tid in ("one", "two"):
                write_json(agent_loop.result_path(rd, tid), {"task_id": tid, "status": "review"})
            args = argparse.Namespace(root=raw, run_id="run", task_id="mission", worker="worker-uuid", kind="result_checked", evidence_ref="result:one")
            with contextlib.redirect_stdout(io.StringIO()):
                agent_loop.cmd_progress(args)
                args.evidence_ref = "result:two"
                agent_loop.cmd_progress(args)
                before = (rd / "tasks" / "mission.json").read_bytes()
                args.evidence_ref = "result:one"
                agent_loop.cmd_progress(args)
            self.assertEqual(before, (rd / "tasks" / "mission.json").read_bytes())

    def test_ambiguous_role_lookup_does_not_write_during_dry_run(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            run = agent_loop.load_json(rd / "run.json")
            run["policies"].update(orchestrator_selection="error", max_orchestrators=1)
            write_json(rd / "run.json", run)
            for tid in ("one", "two"):
                write_json(rd / "sessions" / f"{tid}.json", {"backend": "existing_bridget", "role": "orchestrator", "status": "live", "agent_id": tid})
            before = {str(p): p.read_bytes() for p in rd.rglob("*") if p.is_file()}
            with patch.object(agent_loop, "bridget_agents_by_id", return_value={tid: {"state": "connected"} for tid in ("one", "two")}):
                self.assertIsNone(agent_loop.live_session_for_role(rd, "orchestrator"))
            self.assertEqual(before, {str(p): p.read_bytes() for p in rd.rglob("*") if p.is_file()})

    def test_migration_snapshot_does_not_overwrite_concurrent_result(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            run = agent_loop.load_json(rd / "run.json")
            del run["mission_control_enabled_at"]
            write_json(rd / "run.json", run)
            self.make_task(rd, status="running")
            def finish_during_snapshot(task):
                agent_loop.update_task(rd, agent_loop.load_task(rd, "mission"), status="review", result_path="durable-result")
                return {"declared-file": "snapshot"}
            with patch.object(agent_loop, "progress_snapshot", side_effect=finish_during_snapshot), contextlib.redirect_stdout(io.StringIO()):
                agent_loop.cmd_migrate_run(argparse.Namespace(root=raw, run_id="run"))
            actual = agent_loop.load_task(rd, "mission")
            self.assertEqual(actual["status"], "review")
            self.assertEqual(actual["result_path"], "durable-result")

    def test_same_agent_as_worker_and_coordinator_receives_one_digest(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd)
            self.make_task(rd, "pending", status="pending", assigned_agent="other", agent_target="other")
            send = Mock()
            self.tick(raw, send, role=lambda rd, role: {"backend": "existing_bridget", "agent_id": "worker-uuid"})
            send.assert_called_once()
            self.assertIn("attribue, arbitre ou relance", send.call_args.args[2])
            self.assertIn("ack ou progress", send.call_args.args[2])

    def test_closed_run_skips_refresh_collection_and_delivery(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            run = agent_loop.load_json(rd / "run.json")
            run["status"] = "closed"
            write_json(rd / "run.json", run)
            with patch.object(agent_loop, "collect_task_results") as collect:
                send = Mock()
                self.tick(raw, send)
                collect.assert_not_called()
                send.assert_not_called()

    def test_two_subprocess_heartbeats_read_persisted_state_without_duplicate(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd)
            fake = Path(raw) / "bridget-test"
            log = Path(raw) / "deliveries.jsonl"
            fake.write_text(
                '#!/usr/bin/env python3\nimport sys,json\n'
                'if sys.argv[1] == "agents":\n'
                ' print(json.dumps([{"agent_id":"worker-uuid","state":"connected"}]))\n'
                'elif sys.argv[1] == "send":\n'
                f' with open({str(log)!r}, "a") as handle: handle.write(json.dumps(sys.argv[2:])+"\\n")\n',
                encoding="utf-8",
            )
            fake.chmod(0o700)
            environment = dict(agent_loop.os.environ, AGENT_LOOP_BRIDGET_BIN=str(fake))
            command = [sys.executable, str(SCRIPT), "heartbeat", "--root", raw, "--run-id", "run"]
            for _ in range(3):
                result = subprocess.run(command, env=environment, capture_output=True, text=True, timeout=10)
                self.assertEqual(result.returncode, 0, result.stderr)
            deliveries = [json.loads(line) for line in log.read_text().splitlines()]
            self.assertEqual(len(deliveries), 1)
            self.assertIn("--id", deliveries[0])
            self.assertIn("--issued-at", deliveries[0])
            self.assertIn("--issuer-scope", deliveries[0])

    def test_relaunch_resets_previous_ack_progress_and_disposition(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="review", last_progress_at="old", acknowledged_at="old",
                           dispositioned_at="old", disposition={"decision": "accepted"})
            with contextlib.redirect_stdout(io.StringIO()):
                agent_loop.cmd_relaunch(argparse.Namespace(root=raw, run_id="run", task_id="mission",
                                                           reason="correction", agent_target="", assigned_agent=""))
            source = agent_loop.load_task(rd, "mission")
            successor = agent_loop.load_task(rd, source["successor_task_id"])
            self.assertEqual(source["status"], "review")
            self.assertEqual(successor["acknowledged_at"], "")
            self.assertEqual(successor["last_progress_at"], "")
            self.assertEqual(successor["disposition"], {})

    def test_missing_successor_cannot_fake_assigned_disposition(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            self.make_task(rd, status="review")
            with self.assertRaisesRegex(SystemExit, "successor"):
                agent_loop.cmd_disposition(argparse.Namespace(root=raw, run_id="run", task_id="mission",
                    decision="next_task_assigned", reason_code="next_work_assigned", owner="worker-uuid",
                    next_check="", successor_task_id="missing"))


class HeartbeatRoleAliasTests(unittest.TestCase):
    def test_orchestrator_and_coordinateur_are_the_only_aliases(self):
        expected = frozenset({"orchestrator", "coordinateur"})
        self.assertEqual(agent_loop.role_alias_set("orchestrator"), expected)
        self.assertEqual(agent_loop.role_alias_set("coordinateur"), expected)
        self.assertEqual(agent_loop.role_alias_set("worker"), frozenset({"worker"}))

    def test_default_orchestrator_finds_live_coordinateur(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = Path(raw) / "run"
            write_json(rd / "run.json", {"run_id": "run"})
            write_json(
                rd / "sessions" / "coordinateur.json",
                {"role": "coordinateur", "status": "live", "pane": "%365", "started_at": "2026-07-15T00:00:00Z"},
            )
            with patch.object(agent_loop, "agent_by_pane", return_value={"%365": {}}), \
                    patch.object(agent_loop, "session_is_live", return_value=True):
                session = agent_loop.live_session_for_role(rd, "orchestrator")
            self.assertIsNotNone(session)
            self.assertEqual(session["role"], "coordinateur")

    def test_real_absence_stays_absent(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = Path(raw) / "run"
            write_json(rd / "run.json", {"run_id": "run"})
            write_json(
                rd / "sessions" / "worker.json",
                {"role": "worker", "status": "live", "pane": "%370", "started_at": "2026-07-15T00:00:00Z"},
            )
            with patch.object(agent_loop, "agent_by_pane", return_value={"%370": {}}), \
                    patch.object(agent_loop, "session_is_live", return_value=True):
                self.assertIsNone(agent_loop.live_session_for_role(rd, "orchestrator"))


class GatePreflightTests(unittest.TestCase):
    def make_run(self, root: Path, *, gate: str = "refuse_if_no_passing_review") -> Path:
        rd = root / "run"
        write_json(
            rd / "run.json",
            {"run_id": "run", "policies": {"allowed_backends": ["existing_tmux"]}},
        )
        write_json(
            rd / "tasks" / "code-task.json",
            {
                "task_id": "code-task",
                "status": "pending",
                "attempts": 0,
                "max_attempts": 1,
                "backend": "existing_tmux",
                "delegate": True,
                "agent_target": "agent4",
                "objective": "Tâche code protégée",
                "acceptance": "Le préflight doit être appliqué",
                "gate_preflight": gate,
                "gate_preflight_review_task_id": "review-task",
            },
        )
        return rd

    def test_missing_review_excludes_task_from_ready_queue(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            task = agent_loop.load_task(rd, "code-task")

            self.assertEqual(
                agent_loop.gate_preflight_verdict(rd, task),
                (False, "gate_preflight_no_passing_review"),
            )
            self.assertEqual(agent_loop.ready_tasks(rd), [])

    def test_passing_review_admits_task_to_ready_queue(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            write_json(rd / "results" / "review-task.result.json", {"status": "pass"})
            task = agent_loop.load_task(rd, "code-task")

            self.assertEqual(agent_loop.gate_preflight_verdict(rd, task), (True, ""))
            self.assertEqual([item["task_id"] for item in agent_loop.ready_tasks(rd)], ["code-task"])

    def test_dispatch_refuses_without_review_before_resolving_target(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            rd = self.make_run(root)
            args = argparse.Namespace(
                root=str(root), run_id="run", task_id="code-task", dry_run=False,
                force=False, spawn_command="", runner_command="",
            )

            with patch.object(agent_loop, "resolve_agent_target") as resolve, \
                    contextlib.redirect_stdout(io.StringIO()), \
                    contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(agent_loop.cmd_dispatch(args), 2)
            resolve.assert_not_called()
            stored = agent_loop.load_task(rd, "code-task")
            self.assertEqual(stored["status"], "blocked")
            self.assertEqual(stored["blocked_reason"], "gate_preflight_no_passing_review")
            self.assertIn("task.dispatch_refused", (rd / "events.jsonl").read_text(encoding="utf-8"))

    def test_bootstrap_and_ungated_tasks_remain_admissible(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            bootstrap_rd = self.make_run(root / "bootstrap", gate="bootstrap_self_test")
            bootstrap = agent_loop.load_task(bootstrap_rd, "code-task")
            self.assertEqual(agent_loop.gate_preflight_verdict(bootstrap_rd, bootstrap), (True, ""))

            ungated_rd = self.make_run(root / "ungated", gate="")
            ungated = agent_loop.load_task(ungated_rd, "code-task")
            self.assertEqual(agent_loop.gate_preflight_verdict(ungated_rd, ungated), (True, ""))

    def test_non_passing_review_remains_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = self.make_run(Path(raw))
            write_json(
                rd / "results" / "review-task.result.json",
                {"status": "review", "business_verdict": "NO_GO_PLAN"},
            )
            task = agent_loop.load_task(rd, "code-task")
            self.assertEqual(
                agent_loop.gate_preflight_verdict(rd, task),
                (False, "gate_preflight_no_passing_review"),
            )


class ReconciliationTests(unittest.TestCase):
    def test_result_identity_is_deterministic_and_attempt_bound(self):
        task = {"task_id": "t", "attempts": 1}
        result = {"status": "pass", "message": "ok"}
        first = agent_loop.result_identity(task, result)
        self.assertEqual(first, agent_loop.result_identity(task, result))
        self.assertNotEqual(first[0], agent_loop.result_identity({**task, "attempts": 2}, result)[0])

    def test_reconcile_blocks_result_when_input_changed_after_dispatch(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = Path(raw) / "run"
            source = Path(raw) / "spec.md"
            source.write_text("v1\n", encoding="utf-8")
            write_json(rd / "run.json", {"run_id": "run"})
            task = {"task_id": "review", "status": "running", "input_paths": [str(source)]}
            task["input_sha256"] = agent_loop.input_snapshot(task)
            source.write_text("v2\n", encoding="utf-8")
            write_json(rd / "tasks" / "review.json", task)
            write_json(rd / "results" / "review.result.json", {"status": "pass"})
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_reconcile(argparse.Namespace(root=raw, run_id="run")), 0)
            result = agent_loop.load_json(rd / "results" / "review.result.json")
            self.assertEqual(result["validation_error"], "input_changed_after_dispatch")
            self.assertEqual(agent_loop.load_json(rd / "tasks" / "review.json")["status"], "blocked")

    def test_reconcile_collects_result_and_rejects_invalid_status(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = Path(raw) / "run"
            write_json(rd / "run.json", {"run_id": "run"})
            write_json(rd / "tasks" / "done.json", {"task_id": "done", "status": "running"})
            write_json(rd / "results" / "done.result.json", {"status": "pass"})
            write_json(rd / "tasks" / "bad.json", {"task_id": "bad", "status": "running"})
            write_json(rd / "results" / "bad.result.json", {"status": "invented"})
            args = argparse.Namespace(root=raw, run_id="run")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_reconcile(args), 0)
            self.assertEqual(agent_loop.load_json(rd / "tasks" / "done.json")["status"], "pass")
            self.assertEqual(agent_loop.load_json(rd / "tasks" / "bad.json")["status"], "blocked")
            self.assertEqual(agent_loop.load_json(rd / "results" / "bad.result.json")["validation_error"], "invalid_or_missing_status")
            receipts = list((rd / "receipts").glob("*.json"))
            self.assertEqual(len(receipts), 2)
            self.assertTrue(all(agent_loop.load_json(path)["state"] == "open" for path in receipts))

    def test_reconcile_records_active_without_result_without_fabricating_completion(self):
        with tempfile.TemporaryDirectory() as raw:
            rd = Path(raw) / "run"
            write_json(rd / "run.json", {"run_id": "run"})
            write_json(rd / "tasks" / "orphan.json", {"task_id": "orphan", "status": "running"})
            args = argparse.Namespace(root=raw, run_id="run")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(agent_loop.cmd_reconcile(args), 0)
            self.assertEqual(agent_loop.load_json(rd / "tasks" / "orphan.json")["status"], "running")
            events = (rd / "events.jsonl").read_text(encoding="utf-8")
            self.assertIn("task.reconciliation_needed", events)


if __name__ == "__main__":
    unittest.main()
