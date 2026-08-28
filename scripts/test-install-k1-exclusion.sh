#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fixture_root="$(mktemp -d -t maicie-install-exclusion.XXXXXX)"
lock_holder_pid=""
lock_release="${fixture_root}/install-lock.release"

cleanup() {
  if [[ -n "$lock_holder_pid" ]] && kill -0 "$lock_holder_pid" 2>/dev/null; then
    : >"$lock_release"
    wait "$lock_holder_pid" || true
  fi
  rm -rf "$fixture_root"
}
trap cleanup EXIT

fixture_repo="${fixture_root}/repo"
fixture_home="${fixture_root}/home"
candidate="${fixture_repo}/target/release/maicie"
installed="${fixture_home}/.local/bin/maicie"
installed_before_lock="${fixture_root}/maicie.before-lock"
lock_path="${fixture_home}/.local/bin/maicie.install.lock"
lock_ready="${fixture_root}/install-lock.ready"

mkdir -p \
  "${fixture_repo}/scripts" \
  "${fixture_repo}/target/release" \
  "${fixture_home}/.cargo/bin" \
  "${fixture_home}/.local/bin" \
  "${fixture_home}/.config/maicie"
cp "${root_dir}/scripts/install-k1.sh" "${fixture_repo}/scripts/install-k1.sh"

for tool in cargo rustc; do
  printf '#!/usr/bin/env bash\nexit 0\n' >"${fixture_home}/.cargo/bin/${tool}"
done
printf '#!/usr/bin/env bash\nexit 0\n' >"${fixture_repo}/target/release/bridget"
cat >"$candidate" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "preflight" ]]; then
  printf '%s\n' '{"kind":"schema_preflight","state":"compatible","database_schema":16,"binary_schema":16,"write_schema_compatible":true,"bootstrap_required":false}'
  exit 0
fi
exit 99
EOF
cat >"$installed" <<'EOF'
#!/usr/bin/env bash
echo ancien
EOF
cat >"${fixture_home}/.config/maicie/config.json" <<EOF
{"version":1,"database_path":"${fixture_root}/maicie.sqlite3"}
EOF
chmod 0755 \
  "${fixture_repo}/scripts/install-k1.sh" \
  "${fixture_home}/.cargo/bin/cargo" \
  "${fixture_home}/.cargo/bin/rustc" \
  "${fixture_repo}/target/release/bridget" \
  "$candidate" \
  "$installed"
cp "$installed" "$installed_before_lock"

python3 - "$lock_path" "$lock_ready" "$lock_release" <<'PY' &
import fcntl
import os
import sys
import time

lock_path, ready_path, release_path = sys.argv[1:]
flags = os.O_RDWR | os.O_CREAT
flags |= getattr(os, "O_CLOEXEC", 0)
flags |= getattr(os, "O_NOFOLLOW", 0)
lock_fd = os.open(lock_path, flags, 0o600)
fcntl.flock(lock_fd, fcntl.LOCK_EX)
with open(ready_path, "x", encoding="utf-8") as ready:
    ready.write("LOCK_HELD=yes\n")
while not os.path.exists(release_path):
    time.sleep(0.02)
PY
lock_holder_pid=$!
for _ in {1..250}; do
  [[ -e "$lock_ready" ]] && break
  kill -0 "$lock_holder_pid" 2>/dev/null || break
  sleep 0.02
done
grep -qx 'LOCK_HELD=yes' "$lock_ready"

set +e
locked_output="$({
  HOME="$fixture_home" \
  "${fixture_repo}/scripts/install-k1.sh" --force --skip-services --skip-verify
} 2>&1)"
locked_code=$?
set -e
if ! cmp -s "$installed_before_lock" "$installed"; then
  echo "l'installateur a remplacé Maicie pendant la tenue du verrou partagé" >&2
  exit 1
fi
if [[ "$locked_code" != "1" ]]; then
  echo "l'installateur n'a pas refusé le verrou partagé (code=$locked_code)" >&2
  exit 1
fi
grep -q 'republication ou migration Maicie déjà en cours' <<<"$locked_output"

: >"$lock_release"
wait "$lock_holder_pid"
lock_holder_pid=""

set +e
released_output="$({
  HOME="$fixture_home" \
  "${fixture_repo}/scripts/install-k1.sh" --force --skip-services --skip-verify
} 2>&1)"
released_code=$?
set -e
if [[ "$released_code" != "0" ]]; then
  echo "l'installateur n'a pas réussi après libération du verrou (code=$released_code): $released_output" >&2
  exit 1
fi
if ! cmp -s "$candidate" "$installed"; then
  echo "l'installateur n'a pas publié le candidat après libération du verrou" >&2
  exit 1
fi

echo "exclusion installation Maicie: absence sous verrou et succès après libération vérifiés"
