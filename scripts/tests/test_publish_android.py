import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


MODULE = Path(__file__).resolve().parents[1] / "publish-android.py"
SPEC = importlib.util.spec_from_file_location("publish_android", MODULE)
publication = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(publication)


class AndroidPublicationTest(unittest.TestCase):
    def test_version_contract(self):
        self.assertEqual(publication.version_code("0.7.1"), 7001)
        for version in ("v0.7.1", "0.7", "0.7.1-beta", "1.1000.0"):
            with self.assertRaises(ValueError):
                publication.version_code(version)

    def test_release_and_certificate_must_match(self):
        with tempfile.TemporaryDirectory() as directory:
            apk = Path(directory) / "candidate.apk"
            apk.write_bytes(b"candidate")
            old = ("app.cartae.bridget", 7000, "0.7.0", {publication.CERTIFICATE})
            good = ("app.cartae.bridget", 7001, "0.7.1", {publication.CERTIFICATE})
            for candidate in (old, (*good[:3], {"b" * 64}),
                              (good[0], 7000, good[2], good[3]),
                              ("another.package", *good[1:])):
                with patch.object(publication, "inspect", side_effect=[candidate, old]):
                    with self.assertRaises(ValueError):
                        publication.verify(apk, apk, "0.7.1", Path(directory))
            with patch.object(publication, "inspect", side_effect=[good, old]):
                self.assertEqual(publication.verify(
                    apk, apk, "0.7.1", Path(directory))["version_code"], 7001)

    def test_failed_signature_verification_is_never_accepted(self):
        badging = "package: name='app.cartae.bridget' versionCode='7001' versionName='0.7.1'"
        with patch.object(publication, "run", side_effect=[
                badging, subprocess.CalledProcessError(1, ["apksigner"])]):
            with self.assertRaises(subprocess.CalledProcessError):
                publication.inspect(Path("candidate.apk"), Path("tools"))


    def test_untrusted_reference_does_not_authorize_another_certificate(self):
        with tempfile.TemporaryDirectory() as directory:
            apk = Path(directory) / "candidate.apk"
            apk.write_bytes(b"candidate")
            forged = ("app.cartae.bridget", 7001, "0.7.1", {"b" * 64})
            with patch.object(publication, "inspect", return_value=forged):
                with self.assertRaises(ValueError):
                    publication.verify(apk, apk, "0.7.1", Path(directory))

    def test_missing_signer_is_refused(self):
        badging = "package: name='app.cartae.bridget' versionCode='7001' versionName='0.7.1'"
        with patch.object(publication, "run", side_effect=[badging, "Verifies"]):
            with self.assertRaises(ValueError):
                publication.inspect(Path("candidate.apk"), Path("tools"))

    def test_public_partial_upload_is_not_success(self):
        import json
        with tempfile.TemporaryDirectory() as directory:
            checksum = Path(directory) / "checksum"
            checksum.write_text("expected")
            result = {"sha256": "a" * 64, "size": 42}
            assets = [{"name": "bridget-android.apk", "digest": "sha256:" + "a" * 64,
                       "size": 42}]
            with patch.object(publication, "run", return_value=json.dumps(
                    {"tag_name": "v0.7.1", "assets": assets})):
                with self.assertRaises(ValueError):
                    publication.verify_publication("0.7.1", result, checksum)

    def test_draft_assets_can_be_checked_without_claiming_publication(self):
        import json
        with tempfile.TemporaryDirectory() as directory:
            checksum = Path(directory) / "checksum"
            checksum.write_text("expected")
            result = {"sha256": "a" * 64, "size": 42}
            release = {"tag_name": "v0.7.3", "draft": True, "assets": [
                {"name": "bridget-android.apk", "digest": "sha256:" + result["sha256"], "size": 42},
                {"name": "bridget-android.apk.sha256",
                 "digest": "sha256:" + publication.hashlib.sha256(checksum.read_bytes()).hexdigest(),
                 "size": checksum.stat().st_size}]}
            with patch.object(publication, "run", side_effect=[
                    subprocess.CalledProcessError(1, ["gh"]), json.dumps([release])]):
                self.assertFalse(publication.verify_publication("0.7.3", result, checksum))


if __name__ == "__main__":
    unittest.main()
