"""Recette172 : vrais dépôts Git, transport simulé seulement, aucune dépendance T3."""
import importlib.util
import io
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "scripts/prepare-project.py"
spec = importlib.util.spec_from_file_location("prepare_project", MODULE)
prep = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prep)

class PreparationTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="spec172-")
        self.base = Path(self.tmp.name).resolve()
        self.home = self.base / "remote-home"
        self.home.mkdir(mode=0o700)
        self.source = self.base / "source espace ' et $"
        self.source.mkdir()
        prep.git(self.source, "init")
        (self.source / "code.txt").write_text("première version\n")
        self.commit()
        self.registry = self.home / "registry.json"
        self.authority = dict(worker_id="worker-test", provisioning_revision=1,
                              credential_fingerprint="a" * 64)
        self.config = dict(machines=dict(test=dict(self.authority, enabled=True, mappings={})))
        prep.atomic_json(self.registry, self.config)
        self.request = dict(machine_id="test", origin_host="source", source_root=str(self.source),
                            cwd=str(self.source), commit=self.sha, authority=self.authority)
        self.bundle = self.base / "source.bundle"
        prep.export_bundle(self.source, self.sha, self.bundle)

    def tearDown(self):
        self.tmp.cleanup()

    def commit(self):
        prep.git(self.source, "add", "code.txt")
        prep.git(self.source, "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                 "commit", "-m", "test: fixture")
        self.sha = prep.git(self.source, "rev-parse", "HEAD").stdout.decode().strip()

    def inspect(self):
        with patch.object(Path, "home", return_value=self.home):
            return prep.remote_inspect(self.request, self.registry)

    def receive(self, stream=None):
        with patch.object(Path, "home", return_value=self.home):
            with self.bundle.open("rb") as bundle:
                return prep.remote_receive(self.request, self.registry, stream or bundle)

    def test_new_unpublished_project_and_replay(self):
        before = sorted(str(p.relative_to(self.source)) for p in self.source.rglob("*"))
        planned = self.inspect()
        self.assertTrue(planned["needs_bundle"])
        result = self.receive()
        root = Path(result["mapping"]["remote_root"])
        self.assertEqual(prep.git(root, "rev-parse", "HEAD").stdout.decode().strip(), self.sha)
        self.assertEqual((root / "code.txt").read_text(), "première version\n")
        self.assertEqual(result, self.receive(io.BytesIO(b"not a bundle")))
        self.assertEqual(before, sorted(str(p.relative_to(self.source)) for p in self.source.rglob("*")))
        self.assertEqual(prep.git(self.source, "config", "--get", "remote.origin.url", check=False).returncode, 1)

    def test_new_commit_import_preserves_current_checkout(self):
        first = self.receive()
        root = Path(first["mapping"]["remote_root"])
        old = self.sha
        (root / "local.txt").write_text("travail distant conservé")
        (self.source / "code.txt").write_text("version deux\n")
        self.commit()
        self.request["commit"] = self.sha
        self.bundle.unlink()
        # L'export utilise un dépôt temporaire distinct.
        import shutil
        shutil.rmtree(self.base / "export.git")
        prep.export_bundle(self.source, self.sha, self.bundle, old)
        self.assertTrue(self.inspect()["needs_bundle"])
        self.receive()
        self.assertTrue(prep.commit_present(root, self.sha))
        self.assertEqual(prep.git(root, "rev-parse", "HEAD").stdout.decode().strip(), old)
        self.assertEqual((root / "local.txt").read_text(), "travail distant conservé")

    def test_retry_after_remote_checkout_before_registry_publication(self):
        with patch.object(prep, "publish_mapping", side_effect=prep.Refused("simulated")):
            with self.assertRaisesRegex(prep.Refused, "simulated"):
                self.receive()
        self.assertEqual(json.loads(self.registry.read_text())["machines"]["test"]["mappings"], {})
        self.assertFalse(self.inspect()["needs_bundle"])
        result = self.receive(io.BytesIO())
        self.assertIn(result["project_id"], json.loads(self.registry.read_text())["machines"]["test"]["mappings"])

    def test_foreign_directory_is_not_overwritten(self):
        root = Path(self.inspect()["mapping"]["remote_root"])
        root.mkdir(parents=True)
        (root / "foreign.txt").write_text("conserver")
        with self.assertRaisesRegex(prep.Refused, "destination_occupied"):
            self.receive()
        self.assertEqual((root / "foreign.txt").read_text(), "conserver")

    def test_symlink_destination_is_refused(self):
        root = Path(self.inspect()["mapping"]["remote_root"])
        root.parent.mkdir(parents=True)
        root.symlink_to(self.source, target_is_directory=True)
        with self.assertRaisesRegex(prep.Refused, "symlink"):
            self.receive()

    def test_machine_and_explicit_project_are_strict(self):
        self.request["authority"] = dict(self.authority, worker_id="other")
        with self.assertRaisesRegex(prep.Refused, "machine_mismatch"):
            self.receive()
        self.request["authority"] = self.authority
        self.request["project_id"] = "not-declared"
        with self.assertRaisesRegex(prep.Refused, "project_mapping_unknown"):
            self.receive()

    def test_registry_compare_and_swap(self):
        old = prep.private_read(self.registry)
        self.registry.write_text("{}")
        with self.assertRaisesRegex(prep.Refused, "registry_changed"):
            prep.atomic_json(self.registry, self.config, old)
        self.assertEqual(self.registry.read_text(), "{}")

    def test_worker_registry_can_differ_from_tunnel_namespace(self):
        real = self.home / ".config/bridget/agents.json"
        real.parent.mkdir(parents=True, mode=0o700)
        prep.atomic_json(real, self.config)
        request = dict(self.request, registry=str(self.home / "tunnel/agents.json"))
        with patch.object(Path, "home", return_value=self.home):
            self.assertEqual(prep.remote_registry(request), real)

    def prepare_via_federation(self):
        local_home = self.base / "local-home"
        local_home.mkdir(mode=0o700)
        namespace = local_home / "namespace"
        namespace.mkdir(mode=0o700)
        registry = namespace / "agents.json"
        prep.atomic_json(registry, self.config)
        remote_ns = self.home / "namespace"
        remote_ns.mkdir(mode=0o700)
        prep.atomic_json(remote_ns / "agents.json", self.config)
        installation = local_home / ".local/share/bridget-federation/test"
        installation.mkdir(mode=0o700, parents=True)
        runner = installation / "runner.sh"
        service = installation / "service"
        runner.write_text("fixture runner")
        service.write_text("fixture service")
        config = dict(version="1", label="test", root=str(namespace),
                      remote_root=str(remote_ns), host="test", user="moi", port="22",
                      identity=str(local_home / "key"), known_hosts=str(local_home / "hosts"))
        raw = "".join(k + "=" + v + "\n" for k, v in config.items()).encode()
        (installation / "config").write_bytes(raw)
        (installation / "config").chmod(0o600)
        receipt = dict(config=str(installation / "config"), config_sha256=prep.digest(raw),
                       runner=str(runner), runner_sha256=prep.digest(runner.read_bytes()),
                       service=str(service), service_sha256=prep.digest(service.read_bytes()))
        receipt_path = installation / "receipt"
        receipt_path.write_text("".join(k + "=" + v + "\n" for k, v in receipt.items()))
        receipt_path.chmod(0o600)
        bin_dir = self.base / "bin"
        bin_dir.mkdir()
        ssh = bin_dir / "ssh"
        ssh.write_text("#!/usr/bin/env python3\nimport os,sys\nenv=dict(os.environ,HOME=" + repr(str(self.home)) +
                       ")\nos.execve('/bin/sh',['sh','-c',sys.argv[-1]],env)\n")
        ssh.chmod(0o700)
        request = dict(self.request, registry=str(registry))
        with patch.dict(os.environ, HOME=str(local_home), PATH=str(bin_dir) + ":" + os.environ["PATH"]):
            result = prep.prepare(request)
            self.assertFalse(result["needs_bundle"])
            self.assertEqual(prep.prepare(dict(self.request, registry=str(registry))), result)
        local = json.loads(registry.read_text())["machines"]["test"]["mappings"]
        remote = json.loads((remote_ns / "agents.json").read_text())["machines"]["test"]["mappings"]
        return result, local, remote

    def test_divergent_history_exports_only_attested_common_base(self):
        (self.source / "history.bin").write_bytes(os.urandom(2 * 1024 * 1024))
        prep.git(self.source, "add", "history.bin")
        self.commit()
        common = self.sha
        remote = self.home / "known-project"
        prep.git(self.home, "clone", str(self.source), str(remote))
        (remote / "remote.txt").write_text("travail distant conserve")
        prep.git(remote, "add", "remote.txt")
        prep.git(remote, "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                 "commit", "-m", "test: divergence distante")
        remote_head = prep.git(remote, "rev-parse", "HEAD").stdout.decode().strip()
        prep.git(self.source, "fetch", "--no-write-fetch-head", str(remote), "HEAD")
        (self.source / "code.txt").write_text("branche source divergente\n")
        self.commit()
        self.request["commit"] = self.sha
        self.config["machines"]["test"]["mappings"]["known"] = dict(
            source_host="source", source_root=str(self.source), remote_root=str(remote),
            expected_git_identity=str(self.source), mapping_revision=1)
        exported = []
        original = prep.export_bundle
        def capture(source, commit, destination, have_commit=None):
            original(source, commit, destination, have_commit)
            data = destination.read_bytes()
            exported.append((have_commit, data.split(b"\n\n", 1)[0], len(data)))
        with patch.object(prep, "export_bundle", side_effect=capture):
            self.prepare_via_federation()
        self.assertEqual(len(exported), 1)
        base, header, size = exported[0]
        self.assertEqual(base, common, "la tete divergente ne doit pas imposer un export complet")
        self.assertIn(("-" + common + " ").encode(), header)
        self.assertLess(size, 32 * 1024)
        self.assertTrue(prep.commit_present(remote, self.sha))
        self.assertEqual(prep.git(remote, "rev-parse", "HEAD").stdout.decode().strip(), remote_head)
        self.assertEqual((remote / "remote.txt").read_text(), "travail distant conserve")

    def test_bundle_base_must_exist_on_destination(self):
        self.receive()
        (self.source / "code.txt").write_text("commit absent de la destination\n")
        self.commit()
        self.request.update(commit=self.sha, bundle_base=self.sha)
        with self.assertRaisesRegex(prep.Refused, "bundle_base_missing"):
            self.inspect()
        self.request["bundle_base"] = "--all"
        with self.assertRaisesRegex(prep.Refused, "bundle_base_invalid"):
            self.inspect()

    def test_preparation_timeouts_identify_operation(self):
        timeout = subprocess.TimeoutExpired(["private-argument"], prep.TIMEOUT)
        with patch.object(subprocess, "run", side_effect=timeout):
            for phase in ("git", "bundle_base", "bundle_export", "bundle_verify", "bundle_import"):
                with self.subTest(phase=phase):
                    with self.assertRaises(prep.Refused) as error:
                        prep.git(self.source, "rev-parse", "HEAD", phase=phase)
                    self.assertEqual(str(error.exception), "project_preparation_" + phase + "_timeout")
            link = dict(remote_root=str(self.home), known_hosts="/private-hosts",
                        identity="/private-key", port="22", user="moi", host="test")
            for operation in ("inspect", "prepare"):
                with self.subTest(operation=operation):
                    with self.assertRaises(prep.Refused) as error:
                        prep.ssh(link, self.request, operation)
                    self.assertEqual(str(error.exception), "project_preparation_ssh_" + operation + "_timeout")

    def test_master_to_remote_without_t3_or_project_mapping(self):
        result, local, remote = self.prepare_via_federation()
        self.assertEqual(local, remote)
        self.assertIn(result["project_id"], local)

    def mapped_linked_worktree(self):
        remote = self.home / "known-project"
        prep.git(self.home, "clone", str(self.source), str(remote))
        mapping = dict(source_host="source", source_root=str(self.source),
                       remote_root=str(remote), expected_git_identity=str(self.source),
                       mapping_revision=1)
        self.config["machines"]["test"]["mappings"]["known"] = mapping
        checkout = self.source / ".worktrees/feature espace ' et $"
        prep.git(self.source, "worktree", "add", "--detach", str(checkout), self.sha)
        (checkout / "code.txt").write_text("commit du worktree\n")
        prep.git(checkout, "add", "code.txt")
        prep.git(checkout, "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                 "commit", "-m", "test: linked checkout")
        commit = prep.git(checkout, "rev-parse", "HEAD").stdout.decode().strip()
        self.request.update(source_root=str(checkout), cwd=str(checkout), commit=commit)
        return checkout, remote, remote / checkout.relative_to(self.source)

    def test_parent_root_prepares_requested_linked_worktree(self):
        checkout, remote, target = self.mapped_linked_worktree()
        expected = self.request["commit"]
        self.request["source_root"] = str(self.source)
        result, local, distant = self.prepare_via_federation()
        self.assertEqual(result["project_id"], "known")
        self.assertEqual(local, distant)
        self.assertEqual(len(local), 1)
        self.assertEqual(local["known"]["source_root"], str(self.source))
        self.assertEqual(prep.git(target, "rev-parse", "HEAD").stdout.decode().strip(), expected)
        self.assertEqual(prep.git(checkout, "rev-parse", "HEAD").stdout.decode().strip(), expected)
        self.assertEqual(prep.git(remote, "rev-parse", "HEAD").stdout.decode().strip(), self.sha)
        self.assertEqual(prep.git(self.source, "rev-parse", "HEAD").stdout.decode().strip(), self.sha)

    def test_parent_root_does_not_adopt_nested_independent_repository(self):
        nested = self.source / "independent"
        nested.mkdir()
        prep.git(nested, "init")
        self.request["cwd"] = str(nested)
        with patch.object(prep, "ssh", side_effect=AssertionError("transport interdit")):
            with self.assertRaisesRegex(prep.Refused, "git_root_outside_parent"):
                self.prepare_via_federation()

    def test_parent_subdirectory_does_not_authorize_ancestor_checkout(self):
        checkout, _, _ = self.mapped_linked_worktree()
        subdirectory = checkout / "scoped-subdirectory"
        subdirectory.mkdir()
        self.request.update(source_root=str(subdirectory), cwd=str(subdirectory))
        with patch.object(prep, "ssh", side_effect=AssertionError("transport interdit")):
            with self.assertRaisesRegex(prep.Refused, "git_root_outside_parent"):
                self.prepare_via_federation()

    def test_parent_root_does_not_authorize_external_linked_worktree(self):
        self.mapped_linked_worktree()
        external = self.base / "external-checkout"
        prep.git(self.source, "worktree", "add", "--detach", str(external), self.request["commit"])
        self.request.update(source_root=str(self.source), cwd=str(external))
        with patch.object(prep, "ssh", side_effect=AssertionError("transport interdit")):
            with self.assertRaisesRegex(prep.Refused, "cwd_outside_parent_project"):
                self.prepare_via_federation()

    def test_linked_worktree_preparation_preserves_main_and_replays(self):
        checkout, remote, target = self.mapped_linked_worktree()
        (remote / "local.txt").write_text("travail principal conserve")
        (checkout / "uncommitted.txt").write_text("ne pas transferer")
        result, local, distant = self.prepare_via_federation()
        self.assertEqual(result["project_id"], "known")
        self.assertEqual(local, distant)
        self.assertEqual(len(local), 1, "pas de mapping qui chevauche le projet")
        self.assertEqual(prep.git(target, "rev-parse", "HEAD").stdout.decode().strip(),
                         self.request["commit"])
        self.assertEqual((target / "code.txt").read_text(), "commit du worktree\n")
        self.assertFalse((target / "uncommitted.txt").exists())
        self.assertEqual((checkout / "uncommitted.txt").read_text(), "ne pas transferer")
        self.assertEqual(prep.git(remote, "rev-parse", "HEAD").stdout.decode().strip(), self.sha)
        self.assertEqual(prep.git(self.source, "rev-parse", "HEAD").stdout.decode().strip(), self.sha)
        self.assertEqual((remote / "local.txt").read_text(), "travail principal conserve")
        common = prep.git(target, "rev-parse", "--path-format=absolute",
                          "--git-common-dir").stdout.decode().strip()
        self.assertEqual(Path(common), remote / ".git")
        self.assertEqual(prep.git(target, "symbolic-ref", "-q", "HEAD", check=False).returncode, 1)
        (target / "local.txt").write_text("travail du checkout conserve")
        # Une nouvelle inspection/reception ne remet jamais les fichiers a zero.
        namespace = Path(local["known"]["remote_root"]).parent / "namespace/agents.json"
        request = dict(self.request, authority=self.authority,
                       project_id="known", linked_checkout=str(checkout.relative_to(self.source)))
        with patch.object(Path, "home", return_value=self.home):
            prep.remote_receive(request, namespace, io.BytesIO())
        self.assertEqual((target / "local.txt").read_text(), "travail du checkout conserve")

    def test_linked_worktree_preparation_refuses_foreign_destination(self):
        _, remote, target = self.mapped_linked_worktree()
        target.mkdir(parents=True)
        (target / "foreign.txt").write_text("conserver")
        with self.assertRaisesRegex(prep.Refused, "destination_occupied"):
            self.prepare_via_federation()
        self.assertEqual((target / "foreign.txt").read_text(), "conserver")
        self.assertEqual(prep.git(remote, "rev-parse", "HEAD").stdout.decode().strip(), self.sha)

    def test_linked_worktree_preparation_refuses_symlink_destination(self):
        _, _, target = self.mapped_linked_worktree()
        target.parent.mkdir(parents=True)
        target.symlink_to(self.source, target_is_directory=True)
        with self.assertRaisesRegex(prep.Refused, "symlink"):
            self.prepare_via_federation()
        self.assertTrue(target.is_symlink())

    def test_linked_worktree_preparation_disables_destination_hooks(self):
        _, remote, _ = self.mapped_linked_worktree()
        marker = self.base / "destination-hook-marker"
        hook = remote / ".git/hooks/post-checkout"
        hook.write_text("#!/bin/sh\nprintf ran > " + shlex.quote(str(marker)) + "\n")
        hook.chmod(0o700)
        self.prepare_via_federation()
        self.assertFalse(marker.exists())
        # Controle positif sur le meme depot : le hook est executable.
        subprocess.run(["git", "-C", str(remote), "-c",
                        "core.hooksPath=" + str(hook.parent), "worktree", "add",
                        "--detach", str(self.base / "control-checkout"), self.sha],
                       check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.assertEqual(marker.read_text(), "ran")

    def test_linked_worktree_partial_checkout_is_not_replayed(self):
        checkout, remote, target = self.mapped_linked_worktree()
        prep.atomic_json(self.registry, self.config)
        prep.git(remote, "fetch", str(checkout), self.request["commit"])
        self.request["linked_checkout"] = str(checkout.relative_to(self.source))
        original = prep.git
        def interrupted(root, *arguments, **kwargs):
            if arguments[:2] == ("worktree", "add"):
                original(root, *arguments[:2], "--no-checkout", *arguments[2:], **kwargs)
                raise subprocess.TimeoutExpired("git", prep.TIMEOUT)
            return original(root, *arguments, **kwargs)
        with patch.object(prep, "git", side_effect=interrupted):
            with self.assertRaises(subprocess.TimeoutExpired):
                self.receive()
        self.assertTrue((target / ".git").is_file())
        self.assertFalse((target / "code.txt").exists())
        with self.assertRaisesRegex(prep.Refused, "checkout_incomplete"):
            self.receive(io.BytesIO())
        # Meme un ancien residu sans recu n'est pas pris pour un checkout fini.
        receipts = list((self.registry.parent / "linked-checkouts").glob("*.json"))
        self.assertEqual(len(receipts), 1)
        receipts[0].unlink()
        with self.assertRaisesRegex(prep.Refused, "checkout_incomplete"):
            self.receive(io.BytesIO())
        self.assertFalse((target / "code.txt").exists())
        self.assertEqual(prep.git(remote, "rev-parse", "HEAD").stdout.decode().strip(), self.sha)

    def test_linked_worktree_existing_commit_is_left_for_worker_validation(self):
        checkout, _, target = self.mapped_linked_worktree()
        _, local, _ = self.prepare_via_federation()
        prep.git(target, "checkout", "--detach", self.sha)
        request = dict(self.request, project_id="known",
                       linked_checkout=str(checkout.relative_to(self.source)))
        registry = self.home / "namespace/agents.json"
        for receipt in (registry.parent / "linked-checkouts").glob("*.json"):
            receipt.unlink()
        with patch.object(Path, "home", return_value=self.home):
            prep.remote_receive(request, registry, io.BytesIO())
        self.assertEqual(prep.git(target, "rev-parse", "HEAD").stdout.decode().strip(), self.sha)
        self.assertEqual(len(local), 1)

    def test_remote_commit_format_is_checked_before_preparation(self):
        self.request["commit"] = "--invalid"
        with self.assertRaisesRegex(prep.Refused, "expected_base_commit_invalid"):
            self.receive()
        self.assertFalse((self.registry.parent / "linked-checkouts").exists())

    def test_linked_checkout_clean_failure_can_be_replayed(self):
        checkout, remote, target = self.mapped_linked_worktree()
        prep.atomic_json(self.registry, self.config)
        prep.git(remote, "fetch", str(checkout), self.request["commit"])
        self.request["linked_checkout"] = str(checkout.relative_to(self.source))
        original = prep.git
        def failed(root, *arguments, **kwargs):
            if arguments[:2] == ("worktree", "add"):
                raise prep.Refused("project_preparation_git_failed")
            return original(root, *arguments, **kwargs)
        with patch.object(prep, "git", side_effect=failed):
            with self.assertRaisesRegex(prep.Refused, "git_failed"):
                self.receive()
        self.assertFalse(target.exists())
        self.assertEqual(list((self.registry.parent / "linked-checkouts").glob("*.json")), [])
        self.receive(io.BytesIO())
        self.assertEqual(prep.git(target, "rev-parse", "HEAD").stdout.decode().strip(), self.request["commit"])

    def test_linked_checkout_missing_directory_with_registration_stays_incomplete(self):
        import shutil
        checkout, remote, target = self.mapped_linked_worktree()
        prep.atomic_json(self.registry, self.config)
        prep.git(remote, "fetch", str(checkout), self.request["commit"])
        self.request["linked_checkout"] = str(checkout.relative_to(self.source))
        original = prep.git
        def failed(root, *arguments, **kwargs):
            if arguments[:2] == ("worktree", "add"):
                original(root, *arguments[:2], "--no-checkout", *arguments[2:], **kwargs)
                shutil.rmtree(target)
                raise prep.Refused("project_preparation_git_failed")
            return original(root, *arguments, **kwargs)
        with patch.object(prep, "git", side_effect=failed):
            with self.assertRaisesRegex(prep.Refused, "git_failed"):
                self.receive()
        self.assertFalse(target.exists())
        self.assertEqual(len(list((self.registry.parent / "linked-checkouts").glob("*.json"))), 1)
        with self.assertRaisesRegex(prep.Refused, "checkout_incomplete"):
            self.receive(io.BytesIO())

    def test_linked_checkout_timeout_without_directory_remains_unconfirmed(self):
        checkout, remote, target = self.mapped_linked_worktree()
        prep.atomic_json(self.registry, self.config)
        prep.git(remote, "fetch", str(checkout), self.request["commit"])
        self.request["linked_checkout"] = str(checkout.relative_to(self.source))
        original = subprocess.run
        def expired(command, *args, **kwargs):
            if "worktree" in command and command[command.index("worktree") + 1] == "add":
                raise subprocess.TimeoutExpired(command, prep.TIMEOUT)
            return original(command, *args, **kwargs)
        with patch.object(subprocess, "run", side_effect=expired):
            with self.assertRaisesRegex(prep.Refused, "project_preparation_git_timeout"):
                self.receive()
        self.assertFalse(target.exists())
        self.assertEqual(len(list((self.registry.parent / "linked-checkouts").glob("*.json"))), 1)
        with self.assertRaisesRegex(prep.Refused, "checkout_incomplete"):
            self.receive(io.BytesIO())

    def test_nested_repository_is_not_a_linked_worktree(self):
        checkout, _, _ = self.mapped_linked_worktree()
        nested = self.source / "independent"
        nested.mkdir()
        prep.git(nested, "init")
        prep.git(nested, "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                 "commit", "--allow-empty", "-m", "test: independent repository")
        commit = prep.git(nested, "rev-parse", "HEAD").stdout.decode().strip()
        self.request.update(source_root=str(nested), cwd=str(nested), commit=commit)
        with self.assertRaisesRegex(prep.Refused, "git_identity_mismatch"):
            self.prepare_via_federation()
        self.assertFalse((self.home / "known-project/independent").exists())

    def test_new_project_preparation_never_runs_source_hooks(self):
        marker = self.base / "hook-marker.txt"
        script = "#!/bin/sh\nprintf '%s\\n' \"$0\" >> " + shlex.quote(str(marker)) + "\n"
        for name in ("post-checkout", "post-commit"):
            hook = self.source / ".git/hooks" / name
            hook.write_text(script)
            hook.chmod(0o700)
        empty_config = self.base / "empty.gitconfig"
        empty_config.write_text("")
        with patch.dict(os.environ, GIT_CONFIG_GLOBAL=str(empty_config), GIT_CONFIG_NOSYSTEM="1"):
            hooked = self.base / "hooked"
            hooked.mkdir()
            bundle_path = hooked / "source.bundle"
            prep.export_bundle(self.source, self.sha, bundle_path)
            self.assertTrue(self.inspect()["needs_bundle"])
            with bundle_path.open("rb") as stream:
                result = self.receive(stream)
            root = Path(result["mapping"]["remote_root"])
            self.assertEqual(prep.git(root, "rev-parse", "HEAD").stdout.decode().strip(), self.sha)
            self.assertEqual((root / "code.txt").read_text(), "première version\n")
            self.assertFalse(marker.exists())
            subprocess.run(["git", "-C", str(self.source), "-c", "user.name=Test",
                            "-c", "user.email=test@example.invalid", "commit", "-q",
                            "--allow-empty", "-m", "test: contrôle"], check=True)
            self.assertTrue(marker.exists())

if __name__ == "__main__":
    unittest.main()
