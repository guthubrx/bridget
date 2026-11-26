#!/usr/bin/env python3
"""Tests unitaires pour scripts/deploy-fleet.py (sans SSH ni modification d'hôte)."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock


SCRIPT_PATH = Path(__file__).resolve().parent.parent / "deploy-fleet.py"
SPEC = importlib.util.spec_from_file_location("deploy_fleet", SCRIPT_PATH)
assert SPEC and SPEC.loader
deploy_fleet = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(deploy_fleet)


class DeployFleetTests(unittest.TestCase):
    def test_discover_enabled_machines_includes_macbook_and_excludes_disabled(self):
        agents = {
            "machines": {
                "cartae": {"enabled": True},
                "sysle2": {"enabled": True},
                "ksstore": {"enabled": True},
                "monordinateur": {"enabled": True},
                "macbook-m1": {"enabled": True},
                "retired-box": {"enabled": False},
            }
        }
        discovered = deploy_fleet.discover_enabled_machines(agents)
        self.assertEqual(
            discovered,
            ["cartae", "ksstore", "macbook-m1", "monordinateur", "sysle2"],
        )

    def test_filter_safe_services_never_includes_t3(self):
        raw = ["t3", "com.bridget.t3", "com.bridget.worker.monordinateur", "core"]
        self.assertEqual(
            deploy_fleet.filter_safe_services(raw),
            ["com.bridget.worker.monordinateur", "core"],
        )
        self.assertEqual(
            deploy_fleet.map_systemd_services("cartae", ["core", "worker", "mobile", "t3"]),
            ["bridget-core.service", "bridget-worker-cartae.service", "bridget-mobile.service"],
        )

    def test_rollback_compatible_checks_subset_of_required_formats(self):
        base = {"workspace.worktree.v1", "delegation.model_options.v1"}
        extended = base | {"auto_admission.metadata_v2_512k.v1"}
        self.assertTrue(deploy_fleet.rollback_compatible(extended, base))
        self.assertTrue(deploy_fleet.rollback_compatible(extended, extended))
        self.assertFalse(deploy_fleet.rollback_compatible(base, extended))
        self.assertFalse(deploy_fleet.rollback_compatible(set(), base))

    def test_plan_rollout_order_puts_master_and_canary_first(self):
        machines = ["cartae", "ksstore", "macbook-m1", "monordinateur", "sysle2"]
        canary, rest = deploy_fleet.plan_rollout_order(machines, "cartae", "sysle2")
        self.assertEqual(canary, ["cartae", "sysle2"])
        self.assertEqual(rest, ["ksstore", "macbook-m1", "monordinateur"])

    def test_canary_failure_aborts_fleet_propagation_and_reports_unreachable(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            agents_file = tmp_dir / "agents.json"
            agents_file.write_text(
                json.dumps(
                    {
                        "machines": {
                            "cartae": {"enabled": True},
                            "sysle2": {"enabled": True},
                            "ksstore": {"enabled": True},
                            "macbook-m1": {"enabled": True},
                        }
                    }
                ),
                encoding="utf-8",
            )
            linux_bin = tmp_dir / "bridget-linux"
            linux_bin.write_bytes(b"fake-linux-elf")
            macos_bin = tmp_dir / "bridget-macos"
            macos_bin.write_bytes(b"fake-macos-macho")

            probed: list[str] = []

            def fake_probe(runner):
                probed.append(runner.machine_id)
                if runner.machine_id == "macbook-m1":
                    return {"reachable": False, "error": "ssh_timeout"}
                return {
                    "reachable": True,
                    "os": "Linux",
                    "arch": "x86_64",
                    "home": "/home/moi",
                    "active_binary": "/home/moi/.local/lib/bridget-communication/356/bin/bridget",
                    "update_host": {"services": ["core", "worker"] if runner.machine_id == "cartae" else ["worker"]},
                }

            def fake_deploy(runner, **kwargs):
                if runner.machine_id == "sysle2":
                    raise RuntimeError("canary_health_failed")
                return {
                    "machine_id": runner.machine_id,
                    "os": "Linux",
                    "arch": "x86_64",
                    "services": ["bridget-core.service"],
                    "status": "activated",
                }

            with mock.patch.object(deploy_fleet, "probe_host", side_effect=fake_probe), mock.patch.object(
                deploy_fleet, "deploy_one_host", side_effect=fake_deploy
            ):
                rc = deploy_fleet.main(
                    [
                        "--build-name",
                        "357-test-r1",
                        "--linux-bin",
                        str(linux_bin),
                        "--macos-bin",
                        str(macos_bin),
                        "--agents-json",
                        str(agents_file),
                        "--json",
                    ]
                )
            self.assertEqual(rc, 1)
            # Le canari ayant échoué sur sysle2, ksstore et macbook-m1 ne doivent pas être sondés/touchés.
            self.assertEqual(probed, ["cartae", "sysle2"])


if __name__ == "__main__":
    unittest.main()
