# Guide de vérification rapide - Spec 307

```bash
# 1. Scripts Python et shell
ruff check scripts
for f in scripts/*.sh; do bash -n "$f"; done
python3 -m unittest discover -s scripts/tests -p 'test_build_cleanup.py' -v
python3 -m unittest discover -s scripts/tests -p 'test_updates.py' -v
bash scripts/tests/federation_089_test.sh
bash scripts/tests/federation_095_test.sh

# 2. Rust (lint + tests ciblés)
make lint
python3 scripts/build.py cargo test -p bridget-transport --lib journal::
mkdir -p /tmp/bt && chmod 700 /tmp/bt
TMPDIR=/tmp/bt python3 scripts/build.py cargo test -p bridget-daemon --lib purge_expired_auto_admission
TMPDIR=/tmp/bt python3 scripts/build.py cargo test -p bridget-daemon --lib t3code::
TMPDIR=/tmp/bt python3 scripts/build.py cargo test -p bridget-daemon --lib cmd_who
rm -rf /tmp/bt

# 3. Mesures réelles post-activation
python3 scripts/build.py clean --dry-run
time /Users/moi/.local/bin/bridget who
```
