"""Session 243 : première vue d'un projet neuf vers une autre machine."""
import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "prepare-project.py"
spec = importlib.util.spec_from_file_location("prepare_project", SCRIPT)
prep = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prep)


class PremiereVue(unittest.TestCase):
    def test_projet_neuf_distant_est_prepare_avant_la_vue(self):
        request = {"machine_id": "cartae", "origin_host": "monordinateur"}
        self.assertTrue(prep.first_view_prepares_project(True, "auto-x", {"mappings": {}}, request))

    def test_projet_connu_reste_en_lecture(self):
        request = {"machine_id": "cartae", "origin_host": "monordinateur"}
        selected = {"mappings": {"auto-x": {}}}
        self.assertFalse(prep.first_view_prepares_project(True, "auto-x", selected, request))

    def test_meme_machine_et_worktree_inchanges(self):
        local = {"machine_id": "monordinateur", "origin_host": "monordinateur"}
        self.assertFalse(prep.first_view_prepares_project(True, "auto-x", {"mappings": {}}, local))
        remote = {"machine_id": "cartae", "origin_host": "monordinateur"}
        self.assertFalse(prep.first_view_prepares_project(False, "auto-x", {"mappings": {}}, remote))


if __name__ == "__main__":
    unittest.main()
