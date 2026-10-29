import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("bridget_update", Path(__file__).parents[1] / "update.py")
update = importlib.util.module_from_spec(spec)
spec.loader.exec_module(update)


class Updates179Tests(unittest.TestCase):
    def metadata(self, release="0.6.0"):
        name = f"bridget-{release}-linux-x86_64.tar.gz"
        return {"draft": False, "prerelease": False, "tag_name": "v" + release, "assets": [
            {"name": name, "size": 100, "digest": "sha256:" + "a" * 64,
             "browser_download_url": update.BASE + "v" + release + "/" + name}]}

    def test_release_selection_is_numeric_and_old_release_does_not_need_new_assets(self):
        metadata = self.metadata("0.4.1")
        metadata["assets"] = []
        self.assertIsNone(update.candidate(metadata, "0.5.0", "linux-x86_64"))
        self.assertEqual(update.candidate(self.metadata("0.10.0"), "0.9.99", "linux-x86_64")["version"], "0.10.0")
        for value in ["0.6", "0.6.0-dev", "01.2.3", "1.2.3.4", "r10e", "1000.0.0"]:
            with self.assertRaises(update.Refused):
                update.version(value)

    def test_only_exact_official_platform_asset_with_digest_is_selected(self):
        for field, bad in [("draft", True), ("prerelease", True), ("tag_name", "main")]:
            value = self.metadata()
            value[field] = bad
            with self.assertRaises(update.Refused):
                update.candidate(value, "0.5.0", "linux-x86_64")
        for field, bad in [("digest", None), ("size", 0), ("size", True), ("browser_download_url", "https://evil.test/package")]:
            value = self.metadata()
            value["assets"][0][field] = bad
            with self.assertRaises(update.Refused):
                update.candidate(value, "0.5.0", "linux-x86_64")
        value = self.metadata()
        value["assets"] *= 2
        with self.assertRaises(update.Refused):
            update.candidate(value, "0.5.0", "linux-x86_64")

    def test_redirects_preserve_https_and_allow_only_github_asset_hosts(self):
        self.assertTrue(update.allowed_url(update.BASE + "v0.6.0/a"))
        self.assertTrue(update.allowed_url("https://release-assets.githubusercontent.com/a?token=x"))
        for url in ["http://github.com/a", "https://github.com.evil/a", "https://github.com:444/a", "https://user@github.com/a", "https://example.org/a", "file:///tmp/a"]:
            self.assertFalse(update.allowed_url(url))

    def test_tar_extracts_only_the_known_regular_member_and_rejects_links_duplicates(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            archive = root / "archive.tar.gz"
            member = "bridget-0.6.0-linux-x86_64/bridget"
            for kind in ["regular", "link", "duplicate", "missing"]:
                with tarfile.open(archive, "w:gz") as package:
                    bad = tarfile.TarInfo("../../outside")
                    bad.size = 7
                    package.addfile(bad, io.BytesIO(b"ignored"))
                    if kind != "missing":
                        entry = tarfile.TarInfo(member)
                        entry.size = 3
                        if kind == "link":
                            entry.type = tarfile.SYMTYPE
                            entry.linkname = "/etc/passwd"
                            entry.size = 0
                        package.addfile(entry, io.BytesIO(b"bin"))
                        if kind == "duplicate":
                            package.addfile(entry, io.BytesIO(b"bin"))
                destination = root / kind
                if kind == "regular":
                    update.extract_binary(archive, member, destination)
                    self.assertEqual(destination.read_bytes(), b"bin")
                else:
                    with self.assertRaises(update.Refused):
                        update.extract_binary(archive, member, destination)
                self.assertFalse((root.parent / "outside").exists())

    def test_private_state_is_atomic_and_refuses_symlink_and_world_readable_file(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "state"
            update.atomic_json(path, {"automatic": False})
            self.assertEqual(update.private_json(path)["automatic"], False)
            path.chmod(0o644)
            with self.assertRaises(update.Refused):
                update.private_json(path)
            path.unlink()
            path.symlink_to(Path(temp) / "absent")
            with self.assertRaises(update.Refused):
                update.private_json(path)

    def fixture(self, root):
        home = root / "home"
        namespace = home / "state"
        namespace.mkdir(parents=True, mode=0o700)
        install = home / ".local/lib/bridget-communication"
        old = install / "old/bin/bridget"
        old.parent.mkdir(parents=True)
        old.write_text("#!/bin/sh\nif [ \"$1\" = clients ]; then cat >/dev/null; echo '{\"valid\":true}'; else echo 'bridget 0.5.0'; fi\n")
        old.chmod(0o700)
        link = home / ".local/bin/bridget"
        link.parent.mkdir(parents=True)
        link.symlink_to(old)
        update.atomic_json(namespace / "update-host.json", {"host": "fixture", "services": ["core"]})
        with patch.object(update.platform, "system", return_value="Linux"), patch.object(update.platform, "machine", return_value="x86_64"):
            updater = update.Updater(home, namespace)
        c = update.candidate(self.metadata(), "0.5.0", "linux-x86_64")
        new = install / "v0.6.0-auto/bin/bridget"
        new.parent.mkdir(parents=True)
        new.write_text("#!/bin/sh\nif [ \"$1:$2\" = clients:validate-report ]; then cat >/dev/null; echo '{\"valid\":true}'; elif [ \"$1\" = clients ]; then echo '{\"version\":1,\"verdict\":\"compatible\"}'; else echo 'bridget 0.6.0'; fi\n")
        new.chmod(0o700)
        digest = update.hashlib.sha256(new.read_bytes()).hexdigest()
        update.atomic_json(new.parent.parent / "receipt.json", {"version": "0.6.0", "sha256": digest})
        updater.save(candidate=c, checked_at=int(update.time.time()))
        return updater, old, new

    def test209_client_refusal_happens_before_prepare_restart_and_switch(self):
        for verdict in ["client_update_required", "client_check_unavailable"]:
            with self.subTest(verdict=verdict), tempfile.TemporaryDirectory() as temp:
                updater, old, new = self.fixture(Path(temp))
                report = {"version": 1, "verdict": verdict}
                new.write_text("#!/bin/sh\nif [ \"$1\" = clients ]; then echo '" + json.dumps(report) +
                               "'; exit 1; else echo 'bridget 0.6.0'; fi\n")
                update.atomic_json(new.parent.parent / "receipt.json",
                                   {"version": "0.6.0", "sha256": update.file_digest(new)})
                with patch.object(update, "rpc", return_value={"status":"ok","automatic":True}) as rpc, patch.object(update, "restart") as restart:
                    updater.step()
                self.assertFalse(any(call.args[1] == "prepare" for call in rpc.call_args_list))
                restart.assert_not_called()
                self.assertEqual(updater.link.resolve(), old)
                self.assertEqual(updater.state["stage"], verdict)
                self.assertEqual(updater.state["client_compatibility"], report)

    def test209_candidate_check_failure_has_no_private_output_or_activation(self):
        with tempfile.TemporaryDirectory() as temp:
            updater, old, new = self.fixture(Path(temp))
            new.write_text("#!/bin/sh\nif [ \"$1\" = clients ]; then echo PRIVATE-OUTPUT; exit 7; else echo 'bridget 0.6.0'; fi\n")
            update.atomic_json(new.parent.parent / "receipt.json",
                               {"version":"0.6.0","sha256":update.file_digest(new)})
            with patch.object(update, "rpc", return_value={"status":"ok","automatic":True}) as rpc, patch.object(update, "restart") as restart:
                updater.step()
            self.assertEqual(updater.state["stage"], "client_check_unavailable")
            self.assertNotIn("PRIVATE-OUTPUT", json.dumps(updater.state))
            self.assertFalse(any(call.args[1] == "prepare" for call in rpc.call_args_list))
            restart.assert_not_called()
            self.assertEqual(updater.link.resolve(), old)

    def test209_unconfirmed_probe_stop_is_not_automatically_repeated(self):
        with tempfile.TemporaryDirectory() as temp:
            updater, old, new = self.fixture(Path(temp))
            report={"version":1,"verdict":"client_check_unavailable","clients":[{"state":"probe_stop_unconfirmed"}]}
            with patch.object(update,"client_check",return_value=report) as check, patch.object(update,"rpc",return_value={"status":"ok","automatic":True}) as rpc, patch.object(update,"restart") as restart:
                updater.step()
                updater.step()
            self.assertEqual(check.call_count,1)
            self.assertEqual(updater.state["client_probe_block_generation"],0)
            restart.assert_not_called()
            self.assertFalse(any(call.args[1]=="prepare" for call in rpc.call_args_list))
            self.assertEqual(updater.link.resolve(),old)

    def test209_both_candidate_streams_are_bounded_without_raw_output(self):
        with tempfile.TemporaryDirectory() as temp:
            p=Path(temp)/"probe"
            for stream in ["stdout","stderr"]:
                body = "import os\nfor i in range(1024): os.write(" + ("1" if stream=="stdout" else "2") + ", b'x'*4096)\n"
                p.write_text("#!/usr/bin/env python3\n"+body)
                p.chmod(0o700)
                if stream=="stdout":
                    with self.assertRaisesRegex(update.Refused,"update_client_check_unavailable"):
                        update.bounded_client_command([str(p)])
                else:
                    code, raw=update.bounded_client_command([str(p)])
                    self.assertEqual((code,raw),(0,b""))

    def test_disabled_never_downloads_or_prepares(self):
        with tempfile.TemporaryDirectory() as temp:
            updater, old, _ = self.fixture(Path(temp))
            with patch.object(update, "rpc", return_value={"status": "ok", "automatic": False}) as rpc, patch.object(update, "download") as download:
                updater.step()
            download.assert_not_called()
            self.assertFalse(any(call.args[1] == "prepare" for call in rpc.call_args_list))
            self.assertEqual(updater.link.resolve(), old)

    def test_active_mission_defers_and_does_not_restart_services(self):
        with tempfile.TemporaryDirectory() as temp:
            updater, old, _ = self.fixture(Path(temp))
            def rpc(binary, action, *args):
                if action == "prepare":
                    raise update.Refused("update_missions_active_or_unconfirmed")
                return {"status": "ok", "automatic": True}
            with patch.object(update, "rpc", side_effect=rpc), patch.object(update, "restart") as restart:
                updater.step()
            restart.assert_not_called()
            self.assertEqual(updater.link.resolve(), old)
            self.assertEqual(updater.state["stage"], "waiting_idle")

    def test_health_failure_restores_link_and_does_not_retry_same_failed_version(self):
        with tempfile.TemporaryDirectory() as temp:
            updater, old, new = self.fixture(Path(temp))
            def rpc(binary, action, *args):
                return {"status": "ok", "automatic": True, "expires_at": update.time.time() + 180}
            with patch.object(update, "rpc", side_effect=rpc), patch.object(update, "restart") as restart, patch.object(update, "healthy", side_effect=[False, True]):
                with self.assertRaisesRegex(update.Refused, "update_health_failed"):
                    updater.step()
                self.assertEqual(restart.call_count, 2)
                self.assertEqual(updater.link.resolve(), old)
                self.assertEqual(updater.state["failed_version"], "0.6.0")
                restart.reset_mock()
                updater.step()
                restart.assert_not_called()
            self.assertTrue(new.exists())

    def test_success_releases_barrier_after_health_and_cleanup_failure_never_rolls_back(self):
        with tempfile.TemporaryDirectory() as temp:
            updater, _, new = self.fixture(Path(temp))
            events = []
            def rpc(binary, action, *args):
                events.append(action)
                return {"status": "ok", "automatic": True, "expires_at": update.time.time() + 180}
            with patch.object(update, "rpc", side_effect=rpc), patch.object(update, "restart") as restart, patch.object(update, "healthy", return_value=True), patch.object(updater, "cleanup", side_effect=OSError("busy")):
                updater.step()
            restart.assert_called_once()
            self.assertEqual(updater.link.resolve(), new)
            self.assertIsNone(updater.state["activation"])
            self.assertIn("finish", events)
            self.assertEqual(updater.state["cleanup"], "inventory_unavailable")

    def test_lost_finish_reply_does_not_rollback_after_barrier_may_have_reopened(self):
        with tempfile.TemporaryDirectory() as temp:
            updater, _, new = self.fixture(Path(temp))
            def rpc(binary, action, *args):
                if action == "finish":
                    raise update.Refused("update_daemon_unavailable")
                return {"status": "ok", "automatic": True, "expires_at": update.time.time() + 180}
            with patch.object(update, "rpc", side_effect=rpc), patch.object(update, "restart") as restart, patch.object(update, "healthy", return_value=True):
                with self.assertRaisesRegex(update.Refused, "update_daemon_unavailable"):
                    updater.step()
            restart.assert_called_once()
            self.assertEqual(updater.link.resolve(), new)
            self.assertIsNone(updater.state["activation"])

    def test_explicit_install_works_with_auto_off_but_only_for_selected_version(self):
        with tempfile.TemporaryDirectory() as temp:
            updater, old, new = self.fixture(Path(temp))
            policy={"status":"ok","automatic":False,"install_target":"0.7.0","expires_at":update.time.time()+180}
            with patch.object(update,"rpc",side_effect=lambda *a:policy), patch.object(update,"restart") as restart, patch.object(update,"healthy",return_value=True), patch.object(updater,"cleanup"):
                updater.step()
                restart.assert_not_called()
                self.assertEqual(updater.link.resolve(),old)
                policy["install_target"]="0.6.0"
                updater.step()
                restart.assert_called_once()
                self.assertEqual(updater.link.resolve(),new)

    def test_current_file_with_old_loaded_service_waits_for_the_same_gate(self):
        with tempfile.TemporaryDirectory() as temp:
            updater, old, new = self.fixture(Path(temp))
            update.switch_link(updater.link, new)
            updater.save(candidate=None, rollback=str(old))
            def rpc(binary, action, *args):
                if action == "prepare":
                    raise update.Refused("update_missions_active_or_unconfirmed")
                return {"status": "ok", "automatic": True}
            with patch.object(update, "rpc", side_effect=rpc), patch.object(update, "healthy", return_value=False), patch.object(update, "restart") as restart:
                updater.step()
            restart.assert_not_called()
            self.assertEqual(updater.state["stage"], "waiting_idle")
            self.assertEqual(updater.link.resolve(), new)

    def test_disabling_during_prepare_prevents_switch(self):
        with tempfile.TemporaryDirectory() as temp:
            updater, old, _ = self.fixture(Path(temp))
            state={"automatic":True}
            def rpc(binary,action,*args):
                if action=="prepare":state["automatic"]=False
                return {"status":"ok","automatic":state["automatic"],"expires_at":update.time.time()+180}
            with patch.object(update,"rpc",side_effect=rpc),patch.object(update,"restart") as restart:
                updater.step()
            restart.assert_not_called()
            self.assertEqual(updater.link.resolve(),old)


if __name__ == "__main__":
    unittest.main()
