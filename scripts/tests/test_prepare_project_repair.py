"""Réparer une copie manquante d’une autorisation, sans en inventer."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("prep244", Path(__file__).resolve().parents[1] / "prepare-project.py")
prep = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prep)

class Repair244(unittest.TestCase):
    def fixture(self):
        old = dict(source_host="mac", source_root="/project", remote_root="/target", expected_git_identity="url", mapping_revision=1)
        extra = dict(source_root="/attachments", source_uid=501, source_mode=448, access="inherit", remote_root="/approved/attachments", remote_uid=1000, remote_mode=448)
        selected = dict(enabled=True, worker_id="worker", provisioning_revision=1, credential_fingerprint="f", mappings={"old":old, "approved":dict(old, additional_root=extra)})
        request = dict(machine_id="target", origin_host="mac", authority={k:selected[k] for k in ["worker_id","provisioning_revision","credential_fingerprint"]}, additional_root={k:extra[k] for k in ["source_root","source_uid","source_mode","access"]})
        return old, extra, selected, request

    def test_missing_root_is_copied_once_without_changing_other_fields(self):
        old, extra, selected, request = self.fixture()
        repaired = prep.repaired_mapping(selected, request, "old", old)
        self.assertEqual(repaired, dict(old, additional_root=extra, mapping_revision=2))
        self.assertEqual(old, selected["mappings"]["old"])
        self.assertTrue(prep.is_additional_root_repair(old, repaired))
        self.assertEqual(prep.repaired_mapping(selected, request, "old", repaired), repaired)

    def test_no_unknown_or_foreign_source_is_granted(self):
        old, extra, selected, request = self.fixture()
        for change in [dict(source_uid=502),dict(source_root="/other"),dict(access="write")]:
            other = dict(request, additional_root=dict(request["additional_root"], **change))
            self.assertEqual(prep.repaired_mapping(selected, other, "old", old), old)
        self.assertEqual(prep.repaired_mapping(selected, dict(request, origin_host="foreign"), "old", old), old)

    def test_existing_root_and_unrelated_changes_are_not_repaired(self):
        old, extra, selected, request = self.fixture()
        current = dict(old, additional_root=dict(extra, source_root="/different"))
        self.assertEqual(prep.repaired_mapping(selected, request, "old", current), current)
        repaired = dict(old, additional_root=extra, mapping_revision=2)
        self.assertFalse(prep.is_additional_root_repair(old, dict(repaired, remote_root="/elsewhere")))
        self.assertFalse(prep.is_additional_root_repair(old, dict(repaired, mapping_revision=3)))

    def test_two_approved_destinations_are_ambiguous(self):
        old, extra, selected, request = self.fixture()
        selected["mappings"]["other"] = dict(old, additional_root=dict(extra, remote_root="/other/attachments"))
        with self.assertRaisesRegex(Exception, "project_preparation_additional_root_changed"):
            prep.repaired_mapping(selected, request, "old", old)
        self.assertNotIn("additional_root", old)

    def test_publication_backs_up_and_checks_authority_under_existing_lock(self):
        old, extra, selected, request = self.fixture()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory).resolve() / "agents.json"
            raw = json.dumps(dict(machines={"target":selected})).encode()
            path.write_bytes(raw);path.chmod(0o600)
            repaired = prep.repaired_mapping(selected, request, "old", old)
            with prep.locked(path.parent / "project-preparation.lock"):
                prep.publish_mapping(path, request, "old", repaired)
            saved = json.loads(path.read_text())
            self.assertEqual(saved["machines"]["target"]["mappings"]["old"], repaired)
            backups = list(path.parent.glob("agents.json.avant-244-*"))
            self.assertEqual(len(backups), 1);self.assertEqual(backups[0].read_bytes(), raw)
            with prep.locked(path.parent / "project-preparation.lock"):
                prep.publish_mapping(path, request, "old", repaired)
            self.assertEqual(len(list(path.parent.glob("agents.json.avant-244-*"))), 1)

if __name__ == "__main__":
    unittest.main()
