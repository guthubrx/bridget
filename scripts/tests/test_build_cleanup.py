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


def inventaire_reel(*_args):
    # Les avertissements lsof d'autres montages ne font pas partie de ce scénario.
    return build.parse_open_paths(subprocess.run(build.LSOF_OPEN_FILES, capture_output=True,
                                                 timeout=30).stdout)


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
            if "observation des processus incomplète" in result.stderr:
                self.skipTest("observation lsof incomplète sur cet hôte : nettoyage refusé")
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
                try:
                    opened = build.opened_paths((path.parent,))
                except RuntimeError as error:
                    self.skipTest(f"observation lsof indisponible sur cet hôte : {error}")
                self.assertIn(str(path), opened)
            finally:
                child.communicate("done\n", timeout=10)

    def test_inventaire_lsof_garde_chemins_avec_espaces_et_fichiers_supprimes(self):
        stdout = (b"n/tmp/a b\nn/tmp/a b/fichier ouvert (copie).txt\n"
                  b"n/tmp/a b/supprime.txt (deleted)\nnpipe\n")
        self.assertEqual(build.parse_open_paths(stdout), [
            Path("/tmp/a b"), Path("/tmp/a b/fichier ouvert (copie).txt"),
            Path("/tmp/a b/supprime.txt")])

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


class WorktreePruneTests(unittest.TestCase):
    """Session 116 : retrait automatique des worktrees fusionnés (constitution XVI.3)."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="bridget-worktree-test-")
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name).resolve() / "repo"
        self.repo.mkdir()
        self.enterContext(contextlib.redirect_stderr(io.StringIO()))
        self.old = str(int(time.time()) - 3 * 24 * 3600)
        self.git("init", "-q", "-b", "main")
        self.commit(self.repo, "base", self.old)

    def git(self, *arguments, cwd=None, date=None):
        env = dict(os.environ, GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@example.invalid",
                   GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@example.invalid")
        if date:
            env.update(GIT_AUTHOR_DATE=f"{date} +0000", GIT_COMMITTER_DATE=f"{date} +0000")
        subprocess.run(["git", "-C", str(cwd or self.repo), *arguments], check=True,
                       capture_output=True, env=env)

    def commit(self, where, name, date):
        (where / name).write_text(name)
        self.git("add", name, cwd=where)
        self.git("commit", "-q", "-m", name, cwd=where, date=date)

    def worktree(self, name, date, merge=True):
        path = self.repo / ".worktrees" / name
        self.git("worktree", "add", "-q", str(path), "-b", f"b-{name}", "main")
        self.commit(path, name, date)
        if merge:
            self.git("merge", "-q", "--ff-only", f"b-{name}")
        return path

    def test_seul_un_worktree_fusionne_propre_ancien_et_inoccupe_est_retire(self):
        fusionne = self.worktree("fusionne", self.old)
        sale = self.worktree("sale", self.old)
        (sale / "non-suivi.txt").write_text("brouillon")
        recent = self.worktree("recent", str(int(time.time())))
        non_fusionne = self.worktree("non-fusionne", self.old, merge=False)
        occupe = self.worktree("occupe", self.old)
        with patch.object(build, "busy_paths", return_value=[occupe / "sous-dossier"]):
            retires = build.prune_merged_worktrees(self.repo)
        self.assertEqual(retires, [fusionne])
        self.assertFalse(fusionne.exists())
        for garde in (sale, recent, non_fusionne, occupe):
            self.assertTrue(garde.exists(), garde.name)
        branches = subprocess.run(["git", "-C", str(self.repo), "branch", "--list", "b-fusionne"],
                                  capture_output=True, text=True, check=True).stdout
        self.assertIn("b-fusionne", branches, "la branche est conservée : rien n'est perdu")

    def test_repertoires_courants_inconnus_ne_retirent_rien(self):
        fusionne = self.worktree("fusionne", self.old)
        with patch.object(build, "busy_paths", return_value=None):
            self.assertEqual(build.prune_merged_worktrees(self.repo), [])
        self.assertTrue(fusionne.exists())

    def test_simulation_ne_retire_rien(self):
        fusionne = self.worktree("fusionne", self.old)
        with patch.object(build, "busy_paths", return_value=[]):
            self.assertEqual(build.prune_merged_worktrees(self.repo, dry_run=True), [fusionne])
        self.assertTrue(fusionne.exists())

    def test_lance_depuis_un_worktree_vise_les_worktrees_du_depot_principal(self):
        fusionne = self.worktree("fusionne", self.old)
        courant = self.worktree("courant", str(int(time.time())), merge=False)
        with patch.object(build, "busy_paths", return_value=[]):
            self.assertEqual(build.prune_merged_worktrees(courant), [fusionne])

    def test_fichier_ouvert_dans_un_worktree_bloque_son_retrait(self):
        fusionne = self.worktree("fusionne", self.old)
        ouvert = fusionne / "dossier avec espaces" / "fichier ouvert (copie).txt"
        with patch.object(build, "busy_paths", return_value=[ouvert]):
            self.assertEqual(build.prune_merged_worktrees(self.repo), [])
        self.assertTrue(fusionne.exists())

    def test_processus_tenant_un_fichier_depuis_un_autre_cwd_bloque_puis_libere(self):
        if not shutil.which("lsof"):
            self.skipTest("lsof indisponible : le nettoyeur refusera aussi")
        path = self.repo / ".worktrees" / "tenu"
        self.git("worktree", "add", "-q", str(path), "-b", "b-tenu", "main")
        self.commit(path, "fichier ouvert (copie).txt", self.old)
        self.git("merge", "-q", "--ff-only", "b-tenu")
        child = subprocess.Popen([sys.executable, "-c",
            "import sys; f=open(sys.argv[1]); print('ready',flush=True); sys.stdin.readline()",
            str(path / "fichier ouvert (copie).txt")], cwd=self.temp.name,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        try:
            self.assertEqual(child.stdout.readline().strip(), "ready")
            with patch.object(build, "busy_paths", side_effect=inventaire_reel):
                self.assertEqual(build.prune_merged_worktrees(self.repo), [])
            self.assertTrue(path.exists())
        finally:
            child.communicate("done\n", timeout=10)
        with patch.object(build, "busy_paths", side_effect=inventaire_reel):
            self.assertEqual(build.prune_merged_worktrees(self.repo), [path])
        self.assertFalse(path.exists())


    def test_ignored_outputs_are_preserved_until_archived(self):
        worktree = self.worktree("ignored-output", self.old)
        ignored = self.repo / ".git/info/exclude"
        ignored.parent.mkdir(parents=True, exist_ok=True)
        with ignored.open("a") as stream:
            stream.write("\nlocal-evidence/\n")
        evidence = worktree / "local-evidence"
        evidence.mkdir()
        (evidence / "receipt.json").write_text('{"kept": true}')
        with patch.object(build, "busy_paths", return_value=[]):
            self.assertEqual(build.prune_merged_worktrees(self.repo), [])
        self.assertEqual((evidence / "receipt.json").read_text(), '{"kept": true}')

    def test_incomplete_process_inventory_preserves_worktree(self):
        real_run = build.observed_run
        for returncode, stderr in ((1, b""), (0, b"WARNING: inaccessible filesystem")):
            with self.subTest(returncode=returncode, stderr=stderr):
                worktree = self.worktree(f"incomplete-lsof-{returncode}", self.old)
                def incomplete(command, *args, **kwargs):
                    if any("lsof" in part for part in command):
                        return subprocess.CompletedProcess(command, returncode, b"p42\nn/tmp\n", stderr)
                    return real_run(command, *args, **kwargs)
                with patch.object(build, "observed_run", side_effect=incomplete):
                    self.assertEqual(build.prune_merged_worktrees(self.repo), [])
                self.assertTrue(worktree.exists())

    def test_locked_worktree_est_protege_du_retrait(self):
        fusionne = self.worktree("verrouille", self.old)
        self.git("worktree", "lock", str(fusionne))
        with patch.object(build, "busy_paths", return_value=[]):
            self.assertEqual(build.prune_merged_worktrees(self.repo), [])
        self.assertTrue(fusionne.exists())

    def test_inventaire_motifs_fermes_par_worktree(self):
        self.worktree("fusionne", self.old)
        sale = self.worktree("sale", self.old)
        (sale / "brouillon.txt").write_text("brouillon")
        self.worktree("recent", str(int(time.time())))
        self.worktree("non-fusionne", self.old, merge=False)
        self.worktree("verrouille", self.old)
        self.git("worktree", "lock", str(self.repo / ".worktrees" / "verrouille"))
        parent = self.worktree("parent", self.old)
        self.git("worktree", "add", "-q", str(parent / ".worktrees" / "enfant"),
                 "-b", "b-enfant", "main")
        with patch.object(build, "busy_paths", return_value=[]):
            report = build.inventory(self.repo, "main", time.time(), Path("/"))
        motifs = {Path(row["path"]).name: row["motifs"] for row in report["worktrees"]}
        self.assertEqual(motifs["repo"], ["principal"])
        self.assertEqual(motifs["fusionne"], ["candidat_a_examiner"])
        self.assertEqual(motifs["sale"], ["dirty_ignored"])
        self.assertEqual(motifs["recent"], ["recent"])
        self.assertEqual(motifs["non-fusionne"], ["non_integre"])
        self.assertEqual(motifs["verrouille"], ["locked"])
        self.assertIn("nested", motifs["parent"])
        self.assertIn("dirty_ignored", motifs["parent"])
        self.assertEqual(motifs["enfant"], ["nested"])

    def test_capacite_seuils_et_comptes_separes(self):
        self.assertEqual([build.capacity_state(n) for n in (0, 95, 96, 127, 128)],
                         ["normal", "normal", "warning", "warning", "limit"])
        self.worktree("un", self.old)
        self.worktree("deux", self.old)
        self.git("worktree", "add", "-q", str(self.repo.parent / "hors"), "-b", "b-hors", "main")
        with patch.object(build, "busy_paths", return_value=[]):
            report = build.inventory(self.repo, "main", time.time(), Path("/"))
        capacity = report["capacity"]
        self.assertEqual((capacity["scoped_count"], capacity["git_total"], capacity["state"]),
                         (2, 4, "normal"))
        motifs = {Path(row["path"]).name: row["motifs"] for row in report["worktrees"]}
        self.assertEqual(motifs["hors"], ["hors_perimetre"])

    def test_base_absente_rend_les_motifs_inconnus_sans_candidat(self):
        self.worktree("fusionne", self.old)
        with patch.object(build, "busy_paths", return_value=[]):
            report = build.inventory(self.repo, "absente", time.time(), Path("/"))
        self.assertEqual([row["motifs"] for row in report["worktrees"][1:]],
                         [["lecture_inconnue"]])

    def test_worktree_de_lancement_est_protege_et_observation_inconnue_declaree(self):
        fusionne = self.worktree("fusionne", self.old)
        with patch.object(build, "busy_paths", return_value=[]):
            report = build.inventory(self.repo, "main", time.time(), fusionne / "sous")
        self.assertEqual(report["worktrees"][1]["motifs"], ["processus_actif"])
        with patch.object(build, "busy_paths", return_value=None):
            report = build.inventory(self.repo, "main", time.time(), Path("/"))
        self.assertEqual(report["processus"], "inconnu")
        self.assertEqual(report["worktrees"][1]["motifs"], ["lecture_inconnue"])

    def test_registre_illisible_sort_en_echec_sans_compte(self):
        plain = Path(self.temp.name).resolve() / "pas-git"
        plain.mkdir()
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            code = build.report_worktrees(plain, "main", True)
        self.assertEqual(code, 1)
        self.assertEqual(json.loads(output.getvalue()),
                         {"version": 1, "erreur": "inventaire_git_indisponible"})

    def test_processus_reel_dans_worktree_est_observe_ou_declare_indisponible(self):
        path = self.worktree("tenu", self.old)
        child = subprocess.Popen([sys.executable, "-c",
            "import sys; f=open(sys.argv[1]); print('ready',flush=True); sys.stdin.readline()",
            str(path / "tenu")], cwd=self.temp.name, stdin=subprocess.PIPE,
            stdout=subprocess.PIPE, text=True)
        try:
            self.assertEqual(child.stdout.readline().strip(), "ready")
            report = build.inventory(self.repo, "main", time.time(), Path("/"))
            if report["processus"] == "inconnu":
                self.skipTest("lsof incomplet pendant cet inventaire : processus non observables")
        finally:
            child.communicate("done\n", timeout=10)
        motifs = {Path(row["path"]).name: row["motifs"] for row in report["worktrees"]}
        self.assertIn("processus_actif", motifs["tenu"])

    def test_commande_worktrees_json_stable_et_lecture_seule(self):
        self.worktree("fusionne", self.old)
        listing = ["git", "-C", str(self.repo), "worktree", "list", "--porcelain"]
        before = subprocess.run(listing, capture_output=True, text=True, check=True).stdout
        command = [sys.executable, str(SCRIPT), "worktrees", "--repo", str(self.repo)]
        result = subprocess.run([*command, "--json"], capture_output=True, text=True,
                                cwd=self.temp.name, timeout=60)
        self.assertEqual(result.returncode, 0, result.stderr)
        data = json.loads(result.stdout)
        self.assertEqual((data["version"], data["capacity"]["scoped_count"]), (1, 1))
        self.assertIn("ne garantit ni égalité", data["capacity"]["limite"])
        self.assertIn(data["processus"], ("complet", "inconnu"))
        human = subprocess.run(command, capture_output=True, text=True,
                               cwd=self.temp.name, timeout=60)
        self.assertIn("Capacité : 1/128", human.stdout)
        self.assertIn("ne garantit ni égalité", human.stdout)
        self.assertEqual(subprocess.run(listing, capture_output=True, text=True,
                                        check=True).stdout, before)
        self.assertTrue((self.repo / ".worktrees" / "fusionne").exists())

    def test_perimetre_du_checkout_demande_et_base_explicitement_choisie(self):
        parent = self.worktree("parent", self.old)
        self.git("worktree", "add", "-q", str(parent / ".worktrees" / "enfant"),
                 "-b", "b-enfant", "main")
        with patch.object(build, "busy_paths", return_value=[]):
            report = build.inventory(parent, "main", time.time(), Path("/"))
        self.assertEqual(report["capacity"]["scope"], str(parent / ".worktrees"))
        self.assertEqual(report["capacity"]["scoped_count"], 1)
        self.assertEqual(report["capacity"]["git_total"], 3)
        for row in report["worktrees"]:
            self.assertTrue(row["recovery"])
        self.git("branch", "archive", "main~1")
        with patch.object(build, "busy_paths", return_value=[]):
            report = build.inventory(self.repo, "archive", time.time(), Path("/"))
        reasons = {Path(row["path"]).name: row["motifs"] for row in report["worktrees"]}
        self.assertIn("non_integre", reasons["parent"])

    def test_chemin_absent_et_lien_symbolique_ne_sont_pas_candidats(self):
        worktree = self.worktree("disparu", self.old)
        saved = worktree.with_name("sauvegarde")
        worktree.rename(saved)
        with patch.object(build, "busy_paths", return_value=[]):
            report = build.inventory(self.repo, "main", time.time(), Path("/"))
        self.assertIn("chemin_absent", report["worktrees"][1]["motifs"])
        worktree.symlink_to(saved, target_is_directory=True)
        with patch.object(build, "busy_paths", return_value=[]):
            report = build.inventory(self.repo, "main", time.time(), Path("/"))
            self.assertEqual(build.prune_merged_worktrees(self.repo), [])
        self.assertIn("lien_symbolique", report["worktrees"][1]["motifs"])
        self.assertTrue(saved.is_dir())

    def test_erreurs_outils_sont_inconnues_et_commande_ne_purge_rien(self):
        for error in (FileNotFoundError("outil absent"), subprocess.TimeoutExpired("outil", 15)):
            with self.subTest(error=type(error).__name__):
                with patch.object(build.subprocess, "Popen", side_effect=error):
                    self.assertIsNone(build.busy_paths())
                    with self.assertRaises(RuntimeError):
                        build.worktree_entries(self.repo)
        self.worktree("fusionne", self.old)
        with patch.object(sys, "argv", [str(SCRIPT), "worktrees", "--repo", str(self.repo), "--json"]), \
                patch.object(build, "busy_paths", return_value=[]), \
                patch.object(build, "prune") as caches, \
                patch.object(build, "prune_merged_worktrees") as worktrees, \
                contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(build.main(), 0)
        caches.assert_not_called()
        worktrees.assert_not_called()


    def test_nettoyage_sans_secondaire_ne_lance_pas_lsof(self):
        with patch.object(build, "busy_paths") as observe:
            self.assertEqual(build.prune_merged_worktrees(self.repo), [])
        observe.assert_not_called()

    def test_prefixe_systeme_lie_ne_bloque_pas_un_chemin_interne_sain(self):
        worktree = self.worktree("prefixe", self.old)
        alias = self.repo.parent / "alias"
        alias.symlink_to(self.repo, target_is_directory=True)
        entries = build.worktree_entries(self.repo)
        entries[1] = entries[1]._replace(path=alias / ".worktrees" / worktree.name)
        with patch.object(build, "worktree_entries", return_value=entries), \
                patch.object(build, "busy_paths", return_value=[]):
            report = build.inventory(self.repo, "main", time.time(), Path("/"))
        self.assertEqual(report["worktrees"][1]["motifs"], ["candidat_a_examiner"])



    def test_budget_epuise_conserve_toutes_les_entrees_non_examinees(self):
        for name in ("reste-a", "reste-b"):
            self.worktree(name, self.old)
        budget = build.ObservationBudget()
        def expire(*_args):
            budget.deadline = time.monotonic() - 1
            return []
        with patch.object(build, "busy_paths", side_effect=expire):
            report = build.inventory(self.repo, "main", time.time(), Path("/"), budget)
        self.assertEqual(report["capacity"]["scoped_count"], 2)
        self.assertTrue(all("lecture_inconnue" in row["motifs"] for row in report["worktrees"][1:]))
        self.assertEqual(budget.git_calls, 1)
        self.assertIn("deadline", report["observation"]["incomplete"])

    def test_faits_git_identiques_sont_lus_une_fois_par_passage(self):
        common = subprocess.check_output(["git", "-C", str(self.repo), "rev-parse", "HEAD"], text=True).strip()
        for name in ("partage-a", "partage-b", "partage-c"):
            path = self.worktree(name, self.old)
            subprocess.run(["git", "-C", str(path), "reset", "--hard", common],
                           check=True, capture_output=True)
        budget = build.ObservationBudget()
        with patch.object(build, "busy_paths", return_value=[]):
            report = build.inventory(self.repo, "main", time.time(), Path("/"), budget)
        self.assertEqual(budget.git_calls, 7)
        self.assertTrue(all(row["motifs"] == ["candidat_a_examiner"] for row in report["worktrees"][1:]))

    def test_registre_au_dela_du_plafond_est_inconnu(self):
        self.worktree("secondaire", self.old)
        with patch.object(build, "MAX_WORKTREE_ENTRIES", 1), self.assertRaises(RuntimeError):
            build.worktree_entries(self.repo)


class Observation176Tests(unittest.TestCase):
    def test_seul_verrou_propre_est_exclu_du_cache(self):
        lock = Path("/tmp/profile/.cargo-lock")
        data = (f"p{os.getpid()}\nn{lock}\nn/tmp/profile/other\n"
                f"p{os.getpid()+1}\nn{lock}\n").encode()
        self.assertEqual(build.parse_open_paths(data, (lock,)),
                         [Path("/tmp/profile/other"), lock])

    def test_fuse_sans_avertissement_et_annotation_reelle_bloquent(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            dev = f"{os.major(root.stat().st_dev)}:{os.minor(root.stat().st_dev)}"
            mounts = {Path("/"): [(dev, Path("/"), "ext4")],
                      Path("/portal"): [("0:999", Path("/"), "fuse.portal")]}
            for suffix in ("", " (-e /portal)"):
                data = f"p1\nf1\ntREG\nn/portal/file{suffix}\n".encode()
                with self.subTest(suffix=suffix), patch.object(build.sys, "platform", "linux"), \
                        patch.object(build, "mount_snapshot", return_value=(b"stable", mounts)), \
                        patch.object(build, "observed_run", return_value=subprocess.CompletedProcess([], 0, data, b"")):
                    self.assertIsNone(build.busy_paths((root,)))

    def test_portee_echappee_et_prefixe_exempte_restent_inconnus(self):
        with patch.object(build, "observed_run") as run:
            for root in (Path("/tmp/é"), Path("/tmp/a\nb"), Path("/tmp/a^b")):
                self.assertIsNone(build.busy_paths((root,)))
            run.assert_not_called()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            dev = f"{os.major(root.stat().st_dev)}:{os.minor(root.stat().st_dev)}"
            prefix = Path(str(root)[:-2])
            mounts = {Path("/"): [(dev, Path("/"), "ext4")],
                      prefix: [("0:999", Path("/"), "fuse.portal")]}
            warning = (f"lsof: WARNING: can't stat() fuse.portal file system {prefix}\n"
                       "      Output information may be incomplete.\n").encode()
            with self.assertRaises(ValueError):
                build.foreign_mount_exemptions(warning, (root,), mounts)

    def test_capture_bornee_arrete_la_sortie_sans_la_parser(self):
        budget = build.ObservationBudget(seconds=2)
        result = build.observed_run([sys.executable, "-c", "import os; os.write(1,b'x'*1000000)"],
                                    budget, limit=4096)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b"")
        self.assertIn("output_limit", budget.failures)

    def test_budget_global_arrete_un_enfant_et_bloque_le_suivant(self):
        budget = build.ObservationBudget(seconds=0.1)
        start = time.monotonic()
        result = build.observed_run([sys.executable, "-c", "import time; time.sleep(10)"], budget)
        self.assertNotEqual(result.returncode, 0)
        self.assertLess(time.monotonic()-start, 2)
        with patch.object(build.subprocess, "Popen") as spawn:
            result = build.observed_run([sys.executable, "-c", "pass"], budget)
        spawn.assert_not_called()
        self.assertIn("deadline", budget.failures)

    def test_erreur_d_acces_dans_stdout_interdit_absence(self):
        self.assertFalse(build.process_output_complete(b"p1\nf1\ntREG\nn/proc/1/fd (opendir: Permission denied)\n"))
        self.assertTrue(build.process_output_complete(b"p1\nf1\ntREG\nn/tmp/fichier avec espaces\n"))

    def test_index_imbrication_conserve_ancetres_et_descendants(self):
        a,b,c=Path("/w/a"),Path("/w/a/b"),Path("/w/c")
        self.assertEqual(build.nested_worktrees([a,b,c]),{a,b})


    def test_fichier_disparu_reste_un_chemin_occupe(self):
        data = b"p1\nf1\ntREG\nn/tmp/w/f (stat: No such file or directory)\n"
        self.assertTrue(build.process_output_complete(data))
        self.assertEqual(build.parse_open_paths(data), [Path("/tmp/w/f")])

    def test_montage_exempte_et_format_inconnu_bloquent(self):
        for data in (b"p1\nf1\ntUNKN\nn/portal/file (-e /portal)\n",
                     b"p1\nf1\ntREG\nn/portal/file\n",
                     b"p1\nf1\ntREG\nn/elsewhere/file (-e /portal)\n",
                     b"p1\nf1\ntREG\nnrelative-unknown\n",
                     b"p1\nf1\ntNEWTYPE\nn/tmp/file\n"):
            with self.subTest(data=data):
                self.assertFalse(build.process_output_complete(data, ("/portal",)))

    def test_objets_noyau_connus_ne_sont_pas_des_fichiers_disque(self):
        for kind, name in (("FIFO", "pipe"), ("REG", "net"), ("REG", "mnt"), ("NEXUS", ""), ("CHAN", "channel"),
                           ("REG", "anon_inode:[io_uring] (stat: No such file or directory)")):
            self.assertTrue(build.process_output_complete(
                f"p1\nf1\nt{kind}\nn{name}\n".encode()))
        self.assertFalse(build.process_output_complete(b"p1\nf1\ntREG\nnother_inode:[unknown]\n"))

    def test_mountinfo_decode_espaces_et_garde_couches(self):
        raw = (b"1 0 8:1 / / rw - ext4 /dev/x rw\n"
               b"2 1 0:2 / /foreign\\040mount rw - tmpfs tmpfs rw\n"
               b"3 2 0:3 / /foreign\\040mount rw - tmpfs tmpfs rw\n")
        with patch("builtins.open", return_value=io.BytesIO(raw)):
            data, mounts = build.mount_snapshot()
        self.assertEqual(data, raw)
        self.assertEqual(len(mounts[Path("/foreign mount")]), 2)

    def test_alias_et_montage_interne_refusent_observation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            dev = f"{os.major(root.stat().st_dev)}:{os.minor(root.stat().st_dev)}"
            mounts = {Path("/"): [(dev, Path("/"), "ext4")],
                      Path("/foreign"): [("0:999", Path("/"), "fuse.portal"), ("0:998", Path("/"), "tmpfs")]}
            build.validate_mount_scopes((root,), mounts)
            for other, layers in ((Path("/alias"), [(dev, root, "ext4")]),
                                  (root/"inside", [("0:997", Path("/"), "tmpfs")]),
                                  (root, [(dev, root, "ext4"), (dev, root, "ext4")])):
                with self.subTest(other=other), self.assertRaises(ValueError):
                    build.validate_mount_scopes((root,), {**mounts, other: layers})

    def test_diagnostic_montage_ferme_exige_seconde_lecture_complete(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            dev = f"{os.major(root.stat().st_dev)}:{os.minor(root.stat().st_dev)}"
            mounts = {Path("/"): [(dev, Path("/"), "ext4")], Path("/portal"): [("0:999", Path("/"), "fuse.portal")]}
            warning = b"lsof: WARNING: can't stat() fuse.portal file system /portal\n      Output information may be incomplete.\n"
            good = f"p1\nf1\ntREG\nn{root}/held\n".encode()
            first = subprocess.CompletedProcess([], 0, b"", warning)
            second = subprocess.CompletedProcess([], 0, good, b"")
            with patch.object(build.sys, "platform", "linux"), \
                    patch.object(build, "mount_snapshot", return_value=(b"stable", mounts)), \
                    patch.object(build, "observed_run", side_effect=[first, second]) as run:
                budget = build.ObservationBudget()
                self.assertEqual(build.busy_paths((root,), budget), [root/"held"])
                self.assertEqual(budget.exempt_mounts, 1)
                self.assertEqual(run.call_args.args[0][-2:], ["-e", "/portal"])
            with patch.object(build.sys, "platform", "linux"), \
                    patch.object(build, "mount_snapshot", side_effect=[(b"before", mounts), (b"after", mounts)]), \
                    patch.object(build, "observed_run", return_value=second):
                self.assertIsNone(build.busy_paths((root,)))
            for diagnostic in (b"other warning", warning.replace(b"/portal", str(root).encode())):
                with self.subTest(diagnostic=diagnostic), self.assertRaises(ValueError):
                    build.foreign_mount_exemptions(diagnostic, (root,), mounts)

    def test_echec_lecture_privilegiee_ne_replie_pas(self):
        with patch.object(build.sys, "platform", "linux"), \
                patch.object(build, "mount_snapshot", return_value=(b"stable", {})), \
                patch.object(build, "observed_run", return_value=subprocess.CompletedProcess([], 1, b"", b"denied")) as run:
            self.assertIsNone(build.busy_paths())
            self.assertEqual(run.call_count, 1)

    def test_stderr_borne_comme_stdout(self):
        budget = build.ObservationBudget(seconds=2)
        result = build.observed_run([sys.executable, "-c", "import os; os.write(2,b'x'*1000000)"],
                                    budget, limit=4096)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stderr, b"")
        self.assertIn("output_limit", budget.failures)


if __name__ == "__main__":
    unittest.main()
