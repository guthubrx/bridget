"""Empêcher la distribution d'un binaire incompatible avec la cible commune."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    "linux_abi", Path(__file__).resolve().parents[1] / "check-linux-abi.py")
abi = importlib.util.module_from_spec(spec)
spec.loader.exec_module(abi)


class LinuxAbi(unittest.TestCase):
    def test_debian12_and_older_symbols_are_compatible(self):
        self.assertEqual(abi.qualify_versions(
            "Name: GLIBC_2.2.5\nName: GLIBC_2.34\nName: GLIBC_2.36"),
            {"status": "compatible", "required_glibc": "2.36", "maximum_glibc": "2.36"})

    def test_newer_symbol_cannot_hide_behind_an_older_one(self):
        with self.assertRaisesRegex(ValueError, "linux_abi_too_recent:required=2.39"):
            abi.qualify_versions("Name: GLIBC_2.36\nName: GLIBC_2.39\nName: GLIBC_2.38")

    def test_no_observation_is_not_a_success(self):
        with self.assertRaisesRegex(ValueError, "linux_abi_unattested"):
            abi.qualify_versions("No version information found in this file.")


if __name__ == "__main__":
    unittest.main()
