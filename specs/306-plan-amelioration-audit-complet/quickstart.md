# Guide de validation rapide (quickstart) : Spec 306

Toutes les commandes Cargo passent obligatoirement par `python3 scripts/build.py cargo ...`.

## Prérequis
- Rust toolchain verrouillée (`rust-toolchain.toml`)
- Go 1.27+ (`deploy/caddy-security`)
- Python 3 + `ruff`
- `osv-scanner`
- JDK + `./android/gradlew` pour les tests unitaires Android

## Vérifications par vague

### Vague 1 (Sécurité et hygiène)
```bash
cd /Users/moi/Nextcloud/10.Scripts/64.bridget
osv-scanner scan source --recursive .
(cd deploy/caddy-security && go build -mod=readonly -trimpath -o /tmp/caddy-test . && rm -f /tmp/caddy-test)
python3 scripts/check-docs.py
cargo fmt --all -- --check
python3 -m unittest scripts/tests/test_install.py scripts/tests/test_sentinel.py
```

### Vague 2 (Performance et complexité)
```bash
python3 scripts/build.py cargo test -p bridget-core --locked
python3 scripts/build.py cargo test -p bridget-daemon --locked -- registry mobile::store mobile::notifications native_delegation
```

### Vague 3 (`unsafe`, tests et CI)
```bash
! rg -n "invariant non établi par lecture locale" crates/
python3 -m unittest scripts/tests/test_build_cleanup.py scripts/tests/test_updates.py scripts/tests/test_install.py scripts/tests/test_sentinel.py scripts/tests/test_publish_android.py scripts/tests/test_linux_abi.py
ruff check scripts
bash -n scripts/*.sh
python3 scripts/build.py cargo clippy --workspace --all-targets --locked -- -D warnings
```

### Vague 4 (Maintenabilité Rust et SQLite)
```bash
python3 scripts/build.py cargo test --workspace --locked -- store_schema mobile::http
```

### Vague 5 (Android)
```bash
(cd android && ./gradlew testDebugUnitTest)
```

### Vague 6 (Fonctionnalités)
```bash
python3 scripts/build.py cargo test -p bridget-daemon --locked -- doctor backup restore dry_run completions reaper mcp_schema
```
