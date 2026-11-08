"""Les reçus natifs, pas le récit du modèle, déterminent le résultat."""
import importlib.util
import json
from pathlib import Path
import sqlite3
import tempfile
import unittest
from unittest.mock import patch

spec=importlib.util.spec_from_file_location("sentinel244",Path(__file__).resolve().parents[1]/"sentinel.py")
sentinel=importlib.util.module_from_spec(spec)
spec.loader.exec_module(sentinel)

class Sentinel244(unittest.TestCase):
    def setUp(self):
        self.db=sqlite3.connect(":memory:")
        self.db.executescript("CREATE TABLE native_delegations(request_id TEXT,payload TEXT); CREATE TABLE auto_admission_receipts(owner TEXT,request_id TEXT,state TEXT,refusal_code TEXT);")
        self.case=dict(request_id="sentinel-test",machine="sys",engine="codex",strategy="commit_view",status="not_started")
        self.task=dict(task_id="task",owner="parent",request=dict(machine_id="sys",workspace=dict(strategy="commit_view")),
                       definition=dict(protocol="codex_app_server"),state="result_available",result=sentinel.TOKEN,acknowledged_at=None,guarded=True,cleanup_done=False)

    def write(self):
        self.db.execute("DELETE FROM native_delegations")
        self.db.execute("INSERT INTO native_delegations VALUES(?,?)",(self.case["request_id"],json.dumps(self.task)))

    def test_model_result_without_ack_or_closure_is_not_a_pass(self):
        self.write()
        self.assertEqual(sentinel.observe(self.db,self.case,"parent")["status"],"pending")
        self.task.update(acknowledged_at=123,guarded=False,cleanup_done=True)
        self.write()
        self.assertEqual(sentinel.observe(self.db,self.case,"parent")["status"],"passed")

    def test_guarded_is_admission_policy_not_current_open_guard(self):
        self.task.update(acknowledged_at=123,guarded=True,guard_released=True,cleanup_done=True)
        self.write()
        self.assertEqual(sentinel.observe(self.db,self.case,"parent")["status"],"passed")
        self.task["guard_released"]=False;self.write()
        self.assertEqual(sentinel.observe(self.db,self.case,"parent")["status"],"pending")

    def test_wrong_owner_machine_or_workspace_never_passes(self):
        self.task.update(acknowledged_at=123,guarded=False,cleanup_done=True)
        self.write()
        self.assertEqual(sentinel.observe(self.db,self.case,"foreign")["status"],"not_started")
        for change in [dict(machine_id="other"),dict(workspace=dict(strategy="shared"))]:
            self.task["request"].update(change);self.write()
            self.assertEqual(sentinel.observe(self.db,self.case,"parent")["code"],"sentinel_contract_changed")

    def test_capacity_is_deferred_and_unknown_admission_is_not_absent(self):
        self.db.execute("INSERT INTO auto_admission_receipts VALUES(?,?,?,?)",("parent","sentinel-test","refused","resource_saturated"))
        self.assertEqual(sentinel.observe(self.db,self.case,"parent")["status"],"deferred")
        self.db.execute("UPDATE auto_admission_receipts SET state='pending'")
        self.assertEqual(sentinel.observe(self.db,self.case,"parent"),self.case)

    def test_communication_requires_reply_thread_ack_and_notification(self):
        self.db.executescript("CREATE TABLE tracked_requests(id TEXT,sender TEXT,target TEXT,state TEXT,reply_summary TEXT); CREATE TABLE thread_operations(actor_id TEXT,operation_id TEXT,action TEXT,thread_id TEXT); CREATE TABLE discussion_threads(thread_id TEXT,creator_id TEXT,closed_at INTEGER); CREATE TABLE discussion_members(thread_id TEXT,agent_id TEXT,acked_seq INTEGER); CREATE TABLE discussion_entries(thread_id TEXT,seq INTEGER,author_id TEXT,body TEXT);")
        self.case.update(communication=True,direct_id="direct",thread_create="create")
        self.task.update(child="child",acknowledged_at=123,guarded=False,cleanup_done=True,delivery_acked=True)
        self.write()
        self.assertEqual(sentinel.observe(self.db,self.case,"parent")["code"],"sentinel_communication_unconfirmed")
        summary=json.dumps(dict(from_="child",summary=sentinel.COMMUNICATION_TOKEN)).replace('"from_"','"from"')
        self.db.execute("INSERT INTO tracked_requests VALUES('direct','parent','child','answered',?)",(summary,))
        self.db.execute("INSERT INTO thread_operations VALUES('parent','create','create','thread')")
        self.db.execute("INSERT INTO discussion_threads VALUES('thread','parent',123)")
        self.db.execute("INSERT INTO discussion_members VALUES('thread','child',0)")
        self.db.execute("INSERT INTO discussion_entries VALUES('thread',1,'parent',?)",(sentinel.COMMUNICATION_TOKEN,))
        self.assertEqual(sentinel.observe(self.db,self.case,"parent")["status"],"failed")
        self.db.execute("UPDATE discussion_members SET acked_seq=1")
        self.assertEqual(sentinel.observe(self.db,self.case,"parent")["status"],"passed")
        self.task["delivery_acked"]=False;self.write()
        self.assertFalse(sentinel.observe(self.db,self.case,"parent")["communication_evidence"]["result_notification"])

    def test_matrix_deduplicates_models_and_keeps_all_strategies(self):
        rows=sentinel.cases([dict(machine_id="a",engine="codex"),dict(machine_id="a",engine="codex"),dict(machine_id="b",engine="claude")],"run")
        self.assertEqual(len(rows),6)
        self.assertEqual(len({r["request_id"] for r in rows}),6)
        self.assertEqual({(r["fixture"],r["strategy"]) for r in rows},{("new","commit_view"),("existing","worktree"),("existing","commit_view")})

    def test_private_atomic_state_roundtrip_and_shared_file_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/"run.json"
            sentinel.save(path,dict(request_id="stable",phase="send_submitted"))
            self.assertEqual(sentinel.private(path)["request_id"],"stable")
            path.chmod(0o644)
            with self.assertRaisesRegex(ValueError,"private_file"):
                sentinel.private(path)

    def test_command_output_is_drained_bounded_and_secrets_masked(self):
        result=sentinel.captured(["/usr/bin/python3","-c","import sys;sys.stdout.write('x'*1048576);sys.stderr.write('password=secret-value '+('e'*1048576))"],stdout_limit=4096)
        self.assertEqual(result.returncode,74)
        self.assertEqual(len(result.stdout),4096)
        result=sentinel.captured(["/usr/bin/python3","-c","import sys;sys.stderr.write('password=secret-value '+('e'*1048576))"])
        self.assertLessEqual(len(result.stderr),512)
        self.assertNotIn("secret-value",result.stderr)

    def test_alert_unknown_resolves_same_envelope_without_new_notification(self):
        with tempfile.TemporaryDirectory() as directory:
            state=Path(directory)
            incident=dict(target="maintainer",id="stable-id",issued_at=123,scope="stable-scope",body="incident",reason="mandat",state="unknown")
            incidents={"key":incident}
            with patch.object(sentinel,"call",return_value=dict(exit=0,stdout="OK: accepted id=stable-id",stderr="")) as call:
                sentinel.resolve_alert("bridget",state,incidents,"key")
                args=call.call_args.args[1]
                self.assertIn("stable-id",args);self.assertIn("123",args)
                self.assertEqual(incident["state"],"confirmed")
            self.assertEqual(sentinel.private(state/"sentinel-incidents.json")["key"]["id"],"stable-id")

    def test_repair_starts_only_a_failed_owned_component_with_no_pid(self):
        class Result:
            returncode=0
            stdout="LoadState=loaded\nActiveState=failed\nMainPID=0\n"
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(sentinel,"captured",return_value=Result()) as call:
                rows=sentinel.repair_components(Path(directory),dict(repair_units=["bridget-core.service"]))
                self.assertEqual(rows["bridget-core.service"]["action"],"start_requested")
                self.assertEqual(call.call_count,2)
            Result.stdout="LoadState=loaded\nActiveState=active\nMainPID=123\n"
            with patch.object(sentinel,"captured",return_value=Result()) as call:
                sentinel.repair_components(Path(directory),dict(repair_units=["bridget-core.service"]))
                self.assertEqual(call.call_count,1)
            with self.assertRaisesRegex(ValueError,"component_not_owned"):
                sentinel.repair_components(Path(directory),dict(repair_units=["foreign.service"]))

if __name__=="__main__":
    unittest.main()
