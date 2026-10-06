"""Recette isolée du lookup ROOT du wrapper, sans moteur de heartbeat ni réseau."""
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import types
import unittest
from unittest.mock import Mock, patch

sys.dont_write_bytecode = True
WRAPPER = Path(os.environ.get("PSYCHOLOGIE_LOOKUP_WRAPPER",
    "/Users/moi/Documents/opus2D/orchestration/psychologie-heartbeat/heartbeat.py"))
spec = importlib.util.spec_from_file_location("psychologie_lookup_fixture", WRAPPER)
wrapper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(wrapper)


class LegacyPsychologieLookupTests(unittest.TestCase):
    def exercise(self, *, present=True, project_root=""):
        with tempfile.TemporaryDirectory(prefix="legacy-psychologie-lookup-") as private:
            rd = Path(private)
            run = {"run_id": "fixture", "policies": {},
                   "domain_context": {"project_root": "/legacy/domain/context"}}
            if project_root:
                run["project_root"] = project_root
            (rd / "run.json").write_text(json.dumps(run))
            (rd / "tasks").mkdir()
            before = {str(p.relative_to(rd)): p.read_bytes() for p in rd.rglob("*") if p.is_file()}
            root = {"agent_id": wrapper.ROOT_UUID}
            outside = {"agent_id": "unrelated-project-agent"}

            def agents(**kwargs):
                # Nouveau contrat canonique : sans contexte, aucune suggestion.
                return ([root, outside] if present else [outside]) if kwargs.get("global_scope") else []

            def resolve(target, directory):
                if target != wrapper.ROOT_UUID:
                    raise AssertionError("lookup d'une cible non mandatée")
                found = next((a for a in directory if a["agent_id"] == target), None)
                if found is None:
                    raise SystemExit("unknown agent")
                return found

            def load(path, default):
                return json.loads(path.read_text()) if path.is_file() else default

            canon = types.SimpleNamespace(load_json=load, bridget_agents=Mock(side_effect=agents),
                resolve_bridget_target=Mock(side_effect=resolve), write_json=Mock(),
                send_bridget_message=Mock())
            looked_up = []

            def notification(args, events, **kwargs):
                self.assertTrue(args.dry_run)
                self.assertEqual(events, [])
                self.assertNotIn("project_root", kwargs["run"] if not project_root else {})
                looked_up.append(kwargs["role_resolver"](rd, "orchestrator"))
                looked_up.append(kwargs["role_resolver"](rd, "root"))
                # Aucun transport déclenché. On teste seulement le callback réel.

            canon.notify_mission_events = Mock(side_effect=notification)
            with patch.object(wrapper, "evaluer", return_value=({"closed": False}, [])):
                result = wrapper.passage(rd, canon, maintenant=1700000000, dry_run=True)
            canon.bridget_agents.assert_called_once_with(global_scope=True, project_root=project_root)
            self.assertEqual([call.args[0] for call in canon.resolve_bridget_target.call_args_list],
                             [wrapper.ROOT_UUID, wrapper.ROOT_UUID])
            canon.write_json.assert_not_called()
            canon.send_bridget_message.assert_not_called()
            self.assertEqual(result["sent"], 0)
            after = {str(p.relative_to(rd)): p.read_bytes() for p in rd.rglob("*") if p.is_file()}
            self.assertEqual(before, after)
            return looked_up

    def test_legacy_explicit_root_lookup_and_dry_run_unchanged(self):
        recipients = self.exercise()
        self.assertEqual([r["agent_id"] for r in recipients], [wrapper.ROOT_UUID] * 2)

    def test_missing_root_never_selects_unrelated_agent(self):
        self.assertEqual(self.exercise(present=False), [None, None])

    def test_explicit_run_project_passed_without_domain_promotion(self):
        recipients = self.exercise(project_root="/declared/project")
        self.assertEqual(recipients[0]["agent_id"], wrapper.ROOT_UUID)


if __name__ == "__main__":
    unittest.main()
