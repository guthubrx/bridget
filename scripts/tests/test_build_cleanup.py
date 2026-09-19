"""Recette sans daemon, fournisseur ou données réelles : temporaires uniquement."""
import contextlib
import fcntl
import importlib.util
import io
import json
import os
from pathlib import Path
import select
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "build.py"
spec = importlib.util.spec_from_file_location("bridget_build", SCRIPT)
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)


class CleanupTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="bridget-cleanup-test-")
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name).resolve() / "repo"
        self.repo.mkdir()
        self.cache = self.repo.parent / "cache"
        self.cache.mkdir()
        for name, value in (("worktrees", [self.repo]), ("staging_parent", self.cache),
                            ("owner_id", "test-owner"), ("opened_paths", [])):
            self.enterContext(patch.object(build, name, return_value=value))
        self.enterContext(contextlib.redirect_stderr(io.StringIO()))

    def profile(self, root=None, age=0, size=128):
        path = (root or self.repo / "target") / "debug"
        path.mkdir(parents=True)
        (path / ".cargo-lock").touch()
        (path / "bridget").write_bytes(b"executable conserve")
        for name in build.CACHE_DIRS:
            folder = path / name
            folder.mkdir()
            (folder / "generated").write_bytes(b"x" * size)
            os.utime(folder / "generated", (time.time() - age,) * 2)
            os.utime(folder, (time.time() - age,) * 2)
        return path

    def clean(self, **kwargs):
        return build.prune(self.repo, kwargs.pop("max_bytes", 0),
                           kwargs.pop("max_age", 0), **kwargs)

    def test_cache_seul_supprime_executable_sources_et_sauvegardes_conserves(self):
        path = self.profile()
        (self.repo / "source.rs").write_text("source")
        backup = self.repo / "target" / "ssh-probe"
        backup.mkdir()
        (backup / "preuve").write_text("preuve")
        self.assertEqual(self.clean(), 512)
        self.assertTrue((path / "bridget").is_file())
        self.assertTrue((path / ".cargo-lock").is_file())
        self.assertTrue((backup / "preuve").is_file())
        self.assertTrue((self.repo / "source.rs").is_file())
        self.assertFalse((path / "deps").exists())

    def test_cache_recent_sous_budget_conserve(self):
        path = self.profile()
        self.assertEqual(self.clean(max_bytes=1024, max_age=86400), 0)
        self.assertTrue((path / "deps").exists())

    def test_age_declenche_meme_sous_budget(self):
        self.profile(age=90000)
        self.assertEqual(self.clean(max_bytes=1024, max_age=86400), 512)

    def test_budget_elimine_le_plus_ancien_en_premier(self):
        old = self.profile(age=500)
        other = self.repo.parent / "other"
        other.mkdir()
        fresh = self.profile(other / "target")
        with patch.object(build, "worktrees", return_value=[self.repo, other]):
            self.assertEqual(self.clean(max_bytes=512, max_age=86400), 512)
        self.assertFalse((old / "deps").exists())
        self.assertTrue((fresh / "deps").exists())

    def test_dry_run_ne_supprime_rien(self):
        path = self.profile()
        self.assertEqual(self.clean(dry_run=True), 512)
        self.assertTrue((path / "deps").exists())

    def test_verrou_cargo_occupe_protege(self):
        path = self.profile()
        with (path / ".cargo-lock").open("r+") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            self.assertEqual(self.clean(), 0)
        self.assertTrue((path / "deps").exists())

    def test_processus_utilisant_profil_protege_tout_le_cache(self):
        path = self.profile()
        with patch.object(build, "opened_paths", return_value=[str(path / "bridget")]):
            self.assertEqual(self.clean(), 0)
        self.assertTrue((path / "deps").exists())

    def test_observation_indisponible_interdit_suppression(self):
        path = self.profile()
        with patch.object(build, "opened_paths", side_effect=RuntimeError("indisponible")):
            with self.assertRaises(RuntimeError):
                self.clean()
        self.assertTrue((path / "deps").exists())

    def test_root_target_symbolique_refuse(self):
        outside = self.repo.parent / "outside"
        path = self.profile(outside)
        (self.repo / "target").symlink_to(outside, target_is_directory=True)
        self.assertEqual(self.clean(), 0)
        self.assertTrue((path / "deps").exists())

    def test_cache_symbolique_refuse_sans_suivre(self):
        path = self.profile()
        external = self.repo.parent / "external"
        (path / "deps").rename(external)
        (path / "deps").symlink_to(external, target_is_directory=True)
        self.assertEqual(self.clean(), 0)
        self.assertTrue((external / "generated").exists())

    def test_lien_interne_ne_supprime_pas_sa_cible(self):
        path = self.profile()
        external = self.repo.parent / "precieux"
        external.mkdir()
        (external / "document").write_text("conserver")
        (path / "deps" / "external").symlink_to(external, target_is_directory=True)
        self.clean()
        self.assertEqual((external / "document").read_text(), "conserver")

    def test_cache_change_depuis_scan_est_conserve(self):
        path = self.profile()
        with patch.object(build, "measure", side_effect=[(512, 0), (600, 1)]):
            self.assertEqual(self.clean(), 0)
        self.assertTrue((path / "deps").exists())

    def test_staging_non_marque_ou_etranger_ignore(self):
        stage = self.cache / "bridget-build-test"
        path = self.profile(stage)
        self.assertEqual(self.clean(), 0)
        (stage / build.OWNER).write_text(json.dumps({"version": 1, "repo": "autre"}))
        self.assertEqual(self.clean(), 0)
        self.assertTrue((path / "deps").exists())

    def staging(self):
        stage = self.cache / "bridget-build-test"
        profile = self.profile(stage)
        with patch.dict(os.environ, {"CARGO_TARGET_DIR": str(stage)}):
            build.register_staging(self.repo, ["build", "--release"])
        profile.rename(stage / "release")
        return stage

    def test_installation_identique_purge_intermediaires_pas_binaire(self):
        stage = self.staging()
        binary = self.repo / "installed"
        shutil.copyfile(stage / "release/bridget", binary)
        self.assertEqual(build.installed(self.repo, stage, binary, False), 512)
        self.assertFalse((stage / "release/deps").exists())
        self.assertTrue((stage / "release/bridget").is_file())
        self.assertTrue(binary.is_file())

    def test_installation_differente_ou_dans_staging_refusee(self):
        stage = self.staging()
        binary = self.repo / "installed"
        binary.write_bytes(b"pas la version compilee")
        for candidate in [binary, stage / "release/bridget"]:
            with self.assertRaises(ValueError):
                build.installed(self.repo, stage, candidate, False)
        self.assertTrue((stage / "release/deps").exists())

    def test_installation_de_staging_actif_ne_supprime_rien(self):
        stage = self.staging()
        binary = self.repo / "installed"
        shutil.copyfile(stage / "release/bridget", binary)
        with (stage / "release/.cargo-lock").open("r+") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            self.assertEqual(build.installed(self.repo, stage, binary, False), 0)

    def test_seuils_invalides_refuses(self):
        for value in ["nan", "inf", "-1", "non"]:
            with patch.dict(os.environ, {"BRIDGET_BUILD_CACHE_GIB": value}):
                with self.assertRaises(ValueError):
                    build.limits()


class ProcessTests(unittest.TestCase):
    def test_cli_succes_nettoie_automatiquement_dans_un_depot_isole(self):
        with tempfile.TemporaryDirectory(prefix="bridget-success-test-") as root:
            root = Path(root).resolve()
            (root / "scripts").mkdir()
            shutil.copyfile(SCRIPT, root / "scripts/build.py")
            profile = root / "target/debug"
            (profile / "deps").mkdir(parents=True)
            (profile / ".cargo-lock").touch()
            (profile / "deps/generated").write_text("intermediaire")
            (profile / "bridget").write_text("binaire")
            cargo = root / "cargo-fixture"
            cargo.write_text("#!/bin/sh\nexit 0\n")
            cargo.chmod(0o700)
            result = subprocess.run([sys.executable, str(root / "scripts/build.py"),
                "cargo", "build"], cwd=root, capture_output=True, text=True,
                env={**os.environ, "BRIDGET_CARGO": str(cargo),
                     "BRIDGET_BUILD_CACHE_GIB": "0"}, timeout=40)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse((profile / "deps").exists(), result.stderr)
            self.assertEqual((profile / "bridget").read_text(), "binaire")

    def test_recettes_make_raccordees_et_installation_syntaxiquement_valide(self):
        repo = SCRIPT.parent.parent
        for target in ["build", "release", "test", "clean", "clean-builds", "install"]:
            result = subprocess.run(["make", "-n", target], cwd=repo,
                                    capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("python3 scripts/build.py", result.stdout)
            syntax = subprocess.run(["sh", "-n"], input=result.stdout,
                                    capture_output=True, text=True, timeout=10)
            self.assertEqual(syntax.returncode, 0, syntax.stderr)

    def test_lsof_voit_un_fichier_reel_ouvert_par_un_autre_processus(self):
        if not shutil.which("lsof"):
            self.skipTest("lsof indisponible : le nettoyeur refusera aussi")
        with tempfile.TemporaryDirectory(prefix="bridget-open-test-") as root:
            path = Path(root).resolve() / "opened"
            path.write_text("fixture")
            child = subprocess.Popen([sys.executable, "-c",
                "import sys; f=open(sys.argv[1]); print('ready',flush=True); sys.stdin.readline()",
                str(path)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
            try:
                self.assertEqual(child.stdout.readline().strip(), "ready")
                self.assertIn(str(path), build.opened_paths())
            finally:
                child.communicate("done\n", timeout=10)

    def test_cli_preserve_arguments_et_code_echec_sans_nettoyage(self):
        with tempfile.TemporaryDirectory(prefix="bridget-cli-test-") as root:
            root = Path(root).resolve()
            (root / "scripts").mkdir()
            shutil.copyfile(SCRIPT, root / "scripts/build.py")
            cargo = root / "cargo-fixture"
            cargo.write_text("#!/bin/sh\nprintf '%s\\n' \"$@\"\nexit 7\n")
            cargo.chmod(0o700)
            result = subprocess.run([sys.executable, str(root / "scripts/build.py"),
                "cargo", "test", "--", "--test-threads=1"], capture_output=True, text=True,
                env={**os.environ, "BRIDGET_CARGO": str(cargo)}, timeout=10)
            self.assertEqual(result.returncode, 7)
            self.assertEqual(result.stdout.splitlines(), ["test", "--", "--test-threads=1"])
            self.assertNotIn("Nettoyage", result.stderr)

    def test_verrou_est_effectivement_respecte_par_cargo_installe(self):
        cargo = shutil.which("cargo") or str(Path.home() / ".cargo/bin/cargo")
        if not Path(cargo).is_file():
            self.skipTest("Cargo absent")
        with tempfile.TemporaryDirectory(prefix="bridget-cargo-lock-test-") as root:
            root = Path(root).resolve()
            (root / "src").mkdir()
            (root / "src/main.rs").write_text("fn main() {}\n")
            (root / "Cargo.toml").write_text(
                '[package]\nname="lock-fixture"\nversion="0.1.0"\nedition="2021"\n')
            profile = root / "target/debug"
            profile.mkdir(parents=True)
            with (profile / ".cargo-lock").open("w+") as lock:
                fcntl.flock(lock, fcntl.LOCK_EX)
                child = subprocess.Popen([cargo, "build", "--offline", "--target-dir", str(root / "target")],
                    cwd=root, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                try:
                    ready, _, _ = select.select([child.stderr], [], [], 20)
                    self.assertTrue(ready, "Cargo doit annoncer son attente de verrou")
                    line = child.stderr.readline()
                    self.assertIn("Blocking waiting for file lock", line)
                    self.assertIsNone(child.poll())
                    self.assertFalse((profile / "lock-fixture").exists())
                finally:
                    fcntl.flock(lock, fcntl.LOCK_UN)
                    output, errors = child.communicate(timeout=60)
                self.assertEqual(child.returncode, 0, output + errors)
                self.assertTrue((profile / "lock-fixture").is_file())


if __name__ == "__main__":
    unittest.main()
