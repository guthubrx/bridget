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

    def test_master_to_remote_without_t3_or_project_mapping(self):
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
        self.assertEqual(local, remote)
        self.assertIn(result["project_id"], local)

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
