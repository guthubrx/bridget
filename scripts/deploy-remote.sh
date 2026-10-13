#!/usr/bin/env bash
# Client-only, préfixe neuf. Aucun service, symlink global, profil ou skill.
set -euo pipefail
umask 077
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
# shellcheck source=federate-ssh.sh
source "$SCRIPT_DIR/federate-ssh.sh"

usage() {
  printf '%s\n' \
    'Usage: deploy-remote.sh --label NAME --host HOST --user USER --identity FILE --known-hosts FILE' \
    '       --source DIR --remote-prefix DIR --remote-cargo PATH [--expect-commit SHA40] [--port 22] [--dry-run]' \
    'Client-only exclusivement. Préfixe distant neuf, parent déjà préparé.' \
    'Seuls les fichiers suivis Git sont transférés, modifications suivies incluses, sans les non-suivis.' \
    'Cargo/Rust et leur cache doivent être disponibles : build --locked --offline.' \
    'Aucune installation Rust automatique, aucun profil/skill/service/binaire global modifié.' \
    'dry-run ne contacte pas SSH/rsync et ne crée aucun fichier. Ancienne syntaxe daemon retirée.'
}

host= user= port=22 label= identity= known_hosts= source_dir= remote_prefix= remote_cargo= expect_commit= dry_run=false
while (( $# )); do
  case "$1" in
    --help) usage; exit 0 ;;
    --dry-run) dry_run=true; shift; continue ;;
    --host|--user|--port|--label|--identity|--known-hosts|--source|--remote-prefix|--remote-cargo|--expect-commit)
      (( $# >= 2 )) || federation_fail "valeur absente pour $1" ;;
    *) federation_fail "option inconnue (client-only) : $1" ;;
  esac
  case "$1" in
    --host) host=$2 ;; --user) user=$2 ;; --port) port=$2 ;; --label) label=$2 ;;
    --identity) identity=$2 ;; --known-hosts) known_hosts=$2 ;; --source) source_dir=$2 ;;
    --remote-prefix) remote_prefix=$2 ;; --remote-cargo) remote_cargo=$2 ;;
    --expect-commit) expect_commit=$2; [[ "$expect_commit" =~ ^[0-9a-f]{40}$ ]] || federation_fail "commit attendu invalide (40 caractères hexadécimaux minuscules)" ;;
  esac
  shift 2
done
federation_connection
federation_path "$source_dir"
federation_ancestors "$source_dir"
[[ -f "$source_dir/Cargo.toml" && -f "$source_dir/Cargo.lock" ]] || federation_fail "source Cargo explicite incomplète"
federation_state_path "$remote_prefix"
federation_path "$remote_cargo"
if $dry_run; then
  printf 'dry-run [%s] : client-only %s -> %s:%s ; cargo=%s ; préflight distant non exécuté\n' "$label" "$source_dir" "$target" "$remote_prefix" "$remote_cargo"
  exit 0
fi
[[ $(git -C "$source_dir" rev-parse --show-toplevel) == "$source_dir" ]] || federation_fail "--source doit être la racine du dépôt Git"
build_id=$(git -C "$source_dir" rev-parse HEAD)
if [[ -n "$expect_commit" ]]; then
  [[ "$build_id" == "$expect_commit" ]] || federation_fail "source différente du commit attendu"
  source_status=$(git -C "$source_dir" status --porcelain --untracked-files=all)
  [[ -z "$source_status" ]] || federation_fail "source stricte non propre"
fi
# Même représentation que build_identity.rs : ne pas provoquer un faux écart
# entre SHA complet distant et SHA 12 caractères du même code côté maître.
build_id=${build_id:0:12}
source_status=$(git -C "$source_dir" status --porcelain --untracked-files=no)
[[ -z "$source_status" ]] || build_id+="-dirty"

# Une même liste canonique porte les empreintes et le transfert. Aucun fichier
# temporaire dans les sources ; le trap ne retire que cette réservation privée.
manifest_dir=$(mktemp -d "${TMPDIR:-/tmp}/bridget-deploy-manifest.XXXXXXXX")
trap 'rm -f "$manifest_dir/files.nul" "$manifest_dir/SOURCE-MANIFEST.sha256"; rmdir "$manifest_dir"' EXIT
python3 -B - "$source_dir" "$manifest_dir" "$expect_commit" <<'PY'
import fnmatch
import hashlib
from pathlib import Path
import stat
import subprocess
import sys

source, output = map(Path, sys.argv[1:3])
if sys.argv[3]:
    flags = subprocess.check_output(['git', '-C', str(source), 'ls-files', '-v', '-z'])
    if any(entry[:1] != b'H' for entry in flags.split(b'\0') if entry):
        raise SystemExit('index masqué ou non régulier interdit en mode strict')
entries = subprocess.check_output(['git', '-C', str(source), 'ls-files', '--stage', '-z'])
files = []
for entry in entries.split(b'\0'):
    if not entry:
        continue
    header, raw = entry.split(b'\t', 1)
    mode, _, stage = header.split()
    name = raw.decode('ascii')
    parts = name.split('/')
    if any(ord(c) < 32 or ord(c) > 126 or c == '\\' for c in name):
        raise SystemExit('nom de source non représentable')
    if any(part in ('', '.', '..') for part in parts):
        raise SystemExit('chemin source ambigu')
    if stage != b'0':
        raise SystemExit('index non fusionné')
    excluded = mode not in (b'100644', b'100755') or any(
        part in ('.git', 'target') or any(fnmatch.fnmatchcase(part, pattern)
        for pattern in ('*.db', '*.db-*', '*.sock')) for part in parts)
    if excluded:
        print('source exclue (mode Git ou filtre) : ' + name, file=sys.stderr)
        continue
    path = source / name
    if any(parent.is_symlink() for parent in path.parents if parent != source):
        raise SystemExit('ancêtre symbolique interdit : ' + name)
    if not stat.S_ISREG(path.lstat().st_mode):
        raise SystemExit('source suivie non régulière : ' + name)
    files.append((name, hashlib.sha256(path.read_bytes()).hexdigest()))
if not {'Cargo.toml', 'Cargo.lock'}.issubset(name for name, _ in files):
    raise SystemExit('sources Cargo absentes du manifest')
(output / 'files.nul').write_bytes(b''.join(name.encode('ascii') + b'\0' for name, _ in files))
(output / 'SOURCE-MANIFEST.sha256').write_text(''.join(digest + '  ' + name + '\n' for name, digest in files), encoding='ascii')
PY
manifest_hash=$(federation_hash "$manifest_dir/SOURCE-MANIFEST.sha256")

# Préflight outil avant réservation : un Cargo absent ne laisse aucun préfixe.
federation_remote_command "$remote_prefix" "$remote_cargo"
{
  federation_remote_guards
  printf '%s\n' \
    'federation_state_path "$1"; federation_ancestors "$1"; federation_path "$2"' \
    '[[ ! -e "$1" && ! -L "$1" ]] || federation_fail "préfixe distant déjà occupé"' \
    '[[ -f "$2" && -x "$2" ]] || federation_fail "toolchain absente : --remote-cargo doit désigner Cargo déjà installé"' \
    'command -v python3 >/dev/null && command -v nice >/dev/null || federation_fail "python3 ou nice absent"' \
    'export RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true' \
    '"$2" --version >/dev/null || federation_fail "toolchain indisponible ; aucune installation automatique"' \
    'federation_new_dir "$1"; mkdir -m 700 "$1/source" "$1/bin"'
} | ssh "${ssh_args[@]}" "$target" "$remote_command"

# Le parseur de rsync -e reconnaît les quotes, PAS les échappements Bash %q.
# Les chemins/arguments ont déjà un alphabet fermé sans apostrophe ; les
# quotes simples préservent aussi les quotes doubles de UserKnownHostsFile.
printf -v rsync_shell "'%s' " ssh "${ssh_args[@]}"
rsync -rltz --no-links --chmod=Du=rwx,Dgo=,Fu=rw,Fgo= \
  --from0 --files-from=- --exclude=.git --exclude=target --exclude='*.db' --exclude='*.db-*' --exclude='*.sock' \
  --rsync-path='umask 077 && rsync' -e "$rsync_shell" \
  "$source_dir/" "$target:'$remote_prefix/source/'" < "$manifest_dir/files.nul"

federation_remote_command "$remote_prefix" "$remote_cargo" "$build_id" "$manifest_hash"
{
  federation_remote_guards
  declare -f federation_hash
  printf '%s\n' \
    'federation_private_dir "$1"; federation_private_dir "$1/source"; federation_private_dir "$1/bin"' \
    '[[ ! -e "$1/SOURCE-MANIFEST.sha256" && ! -L "$1/SOURCE-MANIFEST.sha256" ]] || federation_fail "manifest destination déjà occupé"'
  printf '%s\n' "cat > \"\$1/SOURCE-MANIFEST.sha256\" <<'fixture-manifest-end'"
  cat "$manifest_dir/SOURCE-MANIFEST.sha256"
  printf '%s\n' 'fixture-manifest-end'
  cat <<'REMOTE_VERIFY'
[[ $(federation_hash "$1/SOURCE-MANIFEST.sha256") == "$4" ]] || federation_fail "manifest de transfert altéré"
python3 -B - "$1" <<'PY'
import hashlib
import os
from pathlib import Path
import stat
import sys

prefix = Path(sys.argv[1])
source = prefix / 'source'
expected = {}
for line in (prefix / 'SOURCE-MANIFEST.sha256').read_text(encoding='ascii').splitlines():
    digest, name = line.split('  ', 1)
    if name in expected:
        raise SystemExit('entrée manifest dupliquée')
    expected[name] = digest
observed = set()
for root, directories, files in os.walk(source, followlinks=False):
    for name in directories + files:
        path = Path(root) / name
        mode = path.lstat().st_mode
        if stat.S_ISDIR(mode):
            continue
        relative = str(path.relative_to(source))
        if not stat.S_ISREG(mode) or relative not in expected:
            raise SystemExit('source distante inattendue ou lien : ' + relative)
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected[relative]:
            raise SystemExit('empreinte source différente : ' + relative)
        observed.add(relative)
if observed != set(expected):
    raise SystemExit('ensemble de sources distant incomplet')
PY
REMOTE_VERIFY
  printf '%s\n' \
    'federation_private_dir "$1"; federation_private_dir "$1/source"; federation_private_dir "$1/bin"' \
    '[[ ! -e "$1/bin/bridget" && ! -L "$1/bin/bridget" ]] || federation_fail "binaire destination déjà occupé"' \
    'export RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true' \
    'export PATH="${2%/*}:/usr/bin:/bin" BRIDGET_BUILD_ID="$3"' \
    'export CARGO_TARGET_DIR="$1/source/target"' \
    'cd "$1/source"' \
    'nice -n 10 "$2" build --locked --offline --jobs 2 --release -p bridget-daemon --bin bridget 2>&1 | tail -n 20' \
    '[[ -f target/release/bridget && ! -L target/release/bridget && -x target/release/bridget ]] || federation_fail "binaire de compilation absent"' \
    'install -m 700 target/release/bridget "$1/bin/bridget"'
} | ssh "${ssh_args[@]}" "$target" "$remote_command"
printf 'Client installé [%s] : %s:%s/bin/bridget ; aucun daemon démarré.\n' "$label" "$target" "$remote_prefix"
