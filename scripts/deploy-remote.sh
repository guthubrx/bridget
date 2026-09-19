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
    '       --source DIR --remote-prefix DIR --remote-cargo PATH [--port 22] [--dry-run]' \
    'Client-only exclusivement. Préfixe distant neuf, parent déjà préparé.' \
    'Seuls les fichiers suivis Git sont transférés, modifications suivies incluses, sans les non-suivis.' \
    'Cargo/Rust et leur cache doivent être disponibles : build --locked --offline.' \
    'Aucune installation Rust automatique, aucun profil/skill/service/binaire global modifié.' \
    'dry-run ne contacte pas SSH/rsync et ne crée aucun fichier. Ancienne syntaxe daemon retirée.'
}

host= user= port=22 label= identity= known_hosts= source_dir= remote_prefix= remote_cargo= dry_run=false
while (( $# )); do
  case "$1" in
    --help) usage; exit 0 ;;
    --dry-run) dry_run=true; shift; continue ;;
    --host|--user|--port|--label|--identity|--known-hosts|--source|--remote-prefix|--remote-cargo)
      (( $# >= 2 )) || federation_fail "valeur absente pour $1" ;;
    *) federation_fail "option inconnue (client-only) : $1" ;;
  esac
  case "$1" in
    --host) host=$2 ;; --user) user=$2 ;; --port) port=$2 ;; --label) label=$2 ;;
    --identity) identity=$2 ;; --known-hosts) known_hosts=$2 ;; --source) source_dir=$2 ;;
    --remote-prefix) remote_prefix=$2 ;; --remote-cargo) remote_cargo=$2 ;;
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
# Même représentation que build_identity.rs : ne pas provoquer un faux écart
# entre SHA complet distant et SHA 12 caractères du même code côté maître.
build_id=${build_id:0:12}
[[ -z $(git -C "$source_dir" status --porcelain --untracked-files=no) ]] || build_id+="-dirty"

# Préflight outil avant réservation : un Cargo absent ne laisse aucun préfixe.
federation_remote_command "$remote_prefix" "$remote_cargo"
{
  federation_remote_guards
  printf '%s\n' \
    'federation_state_path "$1"; federation_ancestors "$1"; federation_path "$2"' \
    '[[ ! -e "$1" && ! -L "$1" ]] || federation_fail "préfixe distant déjà occupé"' \
    '[[ -f "$2" && -x "$2" ]] || federation_fail "toolchain absente : --remote-cargo doit désigner Cargo déjà installé"' \
    'export RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true' \
    '"$2" --version >/dev/null || federation_fail "toolchain indisponible ; aucune installation automatique"' \
    'federation_new_dir "$1"; mkdir -m 700 "$1/source" "$1/bin"'
} | ssh "${ssh_args[@]}" "$target" "$remote_command"

# Le parseur de rsync -e reconnaît les quotes, PAS les échappements Bash %q.
# Les chemins/arguments ont déjà un alphabet fermé sans apostrophe ; les
# quotes simples préservent aussi les quotes doubles de UserKnownHostsFile.
printf -v rsync_shell "'%s' " ssh "${ssh_args[@]}"
git -C "$source_dir" ls-files -z | rsync -rltz --no-links --chmod=Du=rwx,Dgo=,Fu=rw,Fgo= \
  --from0 --files-from=- --exclude=.git --exclude=target --exclude='*.db' --exclude='*.db-*' --exclude='*.sock' \
  --rsync-path='umask 077 && rsync' -e "$rsync_shell" \
  "$source_dir/" "$target:'$remote_prefix/source/'"

federation_remote_command "$remote_prefix" "$remote_cargo" "$build_id"
{
  federation_remote_guards
  printf '%s\n' \
    'federation_private_dir "$1"; federation_private_dir "$1/source"; federation_private_dir "$1/bin"' \
    '[[ ! -e "$1/bin/bridget" && ! -L "$1/bin/bridget" ]] || federation_fail "binaire destination déjà occupé"' \
    'export RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true' \
    'export PATH="${2%/*}:/usr/bin:/bin" BRIDGET_BUILD_ID="$3"' \
    'cd "$1/source"' \
    '"$2" build --locked --offline --release -p bridget-daemon --bin bridget 2>&1 | tail -n 20' \
    '[[ -f target/release/bridget && ! -L target/release/bridget && -x target/release/bridget ]] || federation_fail "binaire de compilation absent"' \
    'install -m 700 target/release/bridget "$1/bin/bridget"'
} | ssh "${ssh_args[@]}" "$target" "$remote_command"
printf 'Client installé [%s] : %s:%s/bin/bridget ; aucun daemon démarré.\n' "$label" "$target" "$remote_prefix"
