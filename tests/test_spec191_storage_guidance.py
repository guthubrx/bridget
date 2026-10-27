"""Les erreurs de stockage de préparation restent fermées et traversent main."""
import errno
import importlib.util
import io
import json
from pathlib import Path
import unittest
from unittest.mock import patch

spec=importlib.util.spec_from_file_location("prep191",Path(__file__).resolve().parents[1]/"scripts/prepare-project.py")
prep=importlib.util.module_from_spec(spec)
spec.loader.exec_module(prep)

class StorageGuidanceTests(unittest.TestCase):
    def invoke(self,error):
        output=io.StringIO()
        with patch.object(prep.sys,"argv",["prepare.py"]), patch.object(prep.sys,"stdin",io.TextIOWrapper(io.BytesIO(b"{}"))), patch.object(prep,"prepare",side_effect=error), patch.object(prep.sys,"stdout",output):
            prep.main()
        return json.loads(output.getvalue())

    def test_errno_disk_full_reaches_closed_result_without_private_text(self):
        answer=self.invoke(OSError(errno.ENOSPC,"private191 secret/path"))
        self.assertEqual(answer,{"ok":False,"error":"project_preparation_disk_full"})

    def test_errno_disk_quota_is_not_a_provider_account_quota(self):
        self.assertEqual(self.invoke(OSError(errno.EDQUOT,"secret191"))["error"],"project_preparation_disk_quota")

    def test_unknown_oserror_and_text_only_disk_claim_remain_unknown(self):
        self.assertEqual(self.invoke(OSError("No space left on device secret191"))["error"],"project_preparation_failed")
        self.assertEqual(self.invoke(OSError(errno.EACCES,"No space left on device secret191"))["error"],"project_preparation_failed")

    def test_existing_refusal_is_not_changed(self):
        self.assertEqual(self.invoke(prep.Refused("project_preparation_busy"))["error"],"project_preparation_busy")

if __name__=="__main__":unittest.main()
