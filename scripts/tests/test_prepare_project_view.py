"""Session 243 : première vue d'un projet neuf vers une autre machine."""
import importlib.util
import unittest
from unittest.mock import patch
import subprocess
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "prepare-project.py"
spec = importlib.util.spec_from_file_location("prepare_project", SCRIPT)
prep = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prep)


class PremiereVue(unittest.TestCase):
    def test_transport_refuse_conserve_operation_et_exit_sans_sortie_privee(self):
        link = dict(remote_root='/target', known_hosts='/known', identity='/identity',
                    port='22', user='fixture', host='fixture.invalid')
        result = subprocess.CompletedProcess([], 255, b'private output', b'private stderr')
        with patch.object(prep.subprocess, 'run', return_value=result):
            with self.assertRaises(prep.Refused) as refused:
                prep.ssh(link, {}, 'inspect')
        self.assertEqual(str(refused.exception), 'project_preparation_process_failed')
        self.assertEqual(refused.exception.detail, dict(operation='inspect', exit_code=255))
        self.assertNotIn('private', repr(refused.exception.detail))

    def test_sortie_trop_grande_est_distinguee_du_transport(self):
        link = dict(remote_root='/target', known_hosts='/known', identity='/identity',
                    port='22', user='fixture', host='fixture.invalid')
        result = subprocess.CompletedProcess([], 0, b'x' * 33, b'private stderr')
        with patch.object(prep, 'MAX_JSON', 32), patch.object(prep.subprocess, 'run', return_value=result):
            with self.assertRaisesRegex(prep.Refused, '^project_preparation_output_limit$'):
                prep.ssh(link, {}, 'prepare')

    def test_projet_neuf_distant_est_prepare_avant_la_vue(self):
        request = {"machine_id": "cartae", "origin_host": "monordinateur"}
        self.assertTrue(prep.first_view_prepares_project(True, "auto-x", {"mappings": {}}, request))

    def test_projet_connu_reste_en_lecture(self):
        request = {"machine_id": "cartae", "origin_host": "monordinateur"}
        selected = {"mappings": {"auto-x": {}}}
        self.assertFalse(prep.first_view_prepares_project(True, "auto-x", selected, request))

    def test_meme_machine_et_worker_distinct_prepare_la_vue_neuve(self):
        local = {"machine_id": "monordinateur", "origin_host": "monordinateur"}
        self.assertTrue(prep.first_view_prepares_project(True, "auto-x", {"mappings": {}}, local))
        remote = {"machine_id": "cartae", "origin_host": "monordinateur"}
        self.assertFalse(prep.first_view_prepares_project(False, "auto-x", {"mappings": {}}, remote))


if __name__ == "__main__":
    unittest.main()
