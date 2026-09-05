#!/usr/bin/env bash
# Fédération 089 : un transfert Unix, aucune installation de service implicite.
# Gardes communes à deploy-remote.sh ; sourcer ce fichier n'a aucun effet.

federation_fail() { printf '%s\n' "$*" >&2; exit 2; }

federation_path() {
  local path=$1
  # Alphabet fermé : espaces admis, expansions shell et séparateur Unix exclus.
  [[ "$path" =~ ^/[a-zA-Z0-9_./\ -]+$ ]] || federation_fail "chemin absolu invalide : $path"
  case "$path" in
    /|*/|*//*|*/./*|*/.|*/../*|*/..) federation_fail "chemin non normalisé : $path" ;;
  esac
}

federation_state_path() {
  federation_path "$1"
  case "$1/" in
    /tmp/|/private/tmp/|/var/|/private/var/|/Users/|/home/|/root/|"${HOME:-/}/"|\
    */.ssh/*|*/.codex/*|*/.claude/*|*/.gemini/*|*/.cache/bridget/*|*/.config/bridget/*|\
    */.local/share/bridget/*|*/.local/state/bridget/*|*/.local/bin/*|\
    */Library/LaunchAgents/*|*/Library/Application\ Support/bridget/*|\
    */.config/systemd/*|*/bridget/|*/bridget/*|*/bridget.sock/)
      federation_fail "cible historique ou globale interdite : $1" ;;
  esac
}

federation_ancestors() {
  local part current= rest=${1#/}
  while [[ -n "$rest" ]]; do
    part=${rest%%/*}; current="$current/$part"
    [[ ! -L "$current" ]] || federation_fail "lien symbolique interdit : $current"
    [[ "$rest" == */* ]] || break
    [[ ! -e "$current" || -d "$current" ]] || federation_fail "parent non répertoire : $current"
    rest=${rest#*/}
  done
}

federation_metadata() {
  if [[ $(uname -s) == Darwin ]]; then stat -f '%u %Lp' "$1"; else stat -c '%u %a' "$1"; fi
}

federation_private_dir() {
  federation_ancestors "$1"
  [[ -d "$1" && $(federation_metadata "$1") == "$(id -u) 700" ]] ||
    federation_fail "répertoire 0700 possédé par le compte requis : $1"
}

federation_private_file() {
  federation_path "$1"
  federation_ancestors "$1"
  [[ -f "$1" && $(federation_metadata "$1") == "$(id -u) 600" ]] ||
    federation_fail "fichier régulier privé 0600 requis : $1"
}

federation_new_dir() {
  federation_state_path "$1"
  federation_ancestors "$1"
  [[ ! -e "$1" && ! -L "$1" ]] || federation_fail "destination déjà occupée : $1"
  [[ -d ${1%/*} ]] || federation_fail "parent à préparer explicitement : ${1%/*}"
  mkdir -m 700 "$1"
  federation_private_dir "$1"
}

federation_connection() {
  [[ "$host" =~ ^[a-zA-Z0-9][a-zA-Z0-9.-]*$ && "$host" != *..* ]] || federation_fail "hôte invalide"
  [[ "$user" =~ ^[a-zA-Z_][a-zA-Z0-9_-]*$ ]] || federation_fail "utilisateur invalide"
  [[ "$port" =~ ^[0-9]{1,5}$ ]] && (( 10#$port >= 1 && 10#$port <= 65535 )) || federation_fail "port invalide"
  [[ "$label" =~ ^[a-zA-Z0-9][a-zA-Z0-9_-]{0,47}$ ]] || federation_fail "label privé invalide"
  federation_private_file "$identity"
  federation_private_file "$known_hosts"
  target="$user@$host"
  # -F neutralise LocalCommand/ProxyCommand/RemoteForward hérités. Aucun TOFU :
  # le fichier de clés d'hôte est explicite, prérempli, jamais modifié.
  ssh_args=(-F /dev/null -p "$port" -i "$identity"
    -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=yes
    -o "UserKnownHostsFile=\"$known_hosts\"" -o GlobalKnownHostsFile=/dev/null
    -o UpdateHostKeys=no -o ControlMaster=no -o ControlPath=none -o ControlPersist=no
    -o ExitOnForwardFailure=yes -o ServerAliveInterval=15 -o ServerAliveCountMax=3
    -o ConnectTimeout=10 -o ConnectionAttempts=1 -o RequestTTY=no
    -o ForwardAgent=no -o ForwardX11=no -o PermitLocalCommand=no
    -o StreamLocalBindUnlink=no -o StreamLocalBindMask=0177)
}

# SSH joint ses arguments avant le shell distant : transmettre les quotes.
federation_remote_command() {
  remote_command='bash -s --'
  local value
  for value in "$@"; do
    [[ "$value" != *"'"* && "$value" != *$'\n'* && "$value" != *$'\r'* ]] || federation_fail "argument distant invalide"
    remote_command+=" '$value'"
  done
}

federation_remote_guards() {
  printf 'set -euo pipefail\numask 077\n'
  declare -f federation_fail federation_path federation_state_path federation_ancestors federation_metadata federation_private_dir federation_new_dir
}

federation_usage() {
  printf '%s\n' \
    'Usage: federate-ssh.sh run --label NAME --host HOST --user USER --identity FILE --known-hosts FILE' \
    '       --root DIR --socket PATH --remote-root DIR --remote-socket PATH [--port 22] [--dry-run]' \
    'Racines/socket absolues privées ; fichiers SSH déjà en 0600.' \
    'run au premier plan. Aucun install/status/remove, launchd ni configuration historique.' \
    'Clients distants : BRIDGET_HOME=remote-root BRIDGET_SOCKET=remote-socket BRIDGET_CHANNEL=ssh-unix.' \
    'dry-run : lectures locales seulement, aucune écriture ni commande SSH/rsync.' \
    'Socket distante même stale refusée ; aucun effacement automatique. Le serveur SSH doit créer' \
    'ses sockets StreamLocal en privé (StreamLocalBindMask 0177, à vérifier à la recette).' \
    'Après coupure, vérifier son absence avant reprise au même chemin. La recette SSH réelle doit' \
    'encore prouver le nettoyage OpenSSH ; si stale, nettoyage explicite avec preuve de propriété requis.'
}

federation_main() {
  set -euo pipefail
  umask 077
  [[ ${1:-} != --help ]] || { federation_usage; return; }
  [[ ${1:-} == run ]] || federation_fail "usage changé : run au premier plan uniquement ; voir --help"
  shift
  local host= user= port=22 label= identity= known_hosts= root= socket= remote_root= remote_socket= dry_run=false target remote_command
  local -a ssh_args
  while (( $# )); do
    case "$1" in
      --dry-run) dry_run=true; shift; continue ;;
      --host|--user|--port|--label|--identity|--known-hosts|--root|--socket|--remote-root|--remote-socket)
        (( $# >= 2 )) || federation_fail "valeur absente pour $1" ;;
      *) federation_fail "option inconnue : $1" ;;
    esac
    case "$1" in
      --host) host=$2 ;; --user) user=$2 ;; --port) port=$2 ;; --label) label=$2 ;;
      --identity) identity=$2 ;; --known-hosts) known_hosts=$2 ;; --root) root=$2 ;;
      --socket) socket=$2 ;; --remote-root) remote_root=$2 ;; --remote-socket) remote_socket=$2 ;;
    esac
    shift 2
  done
  federation_connection
  federation_state_path "$root"
  federation_state_path "$remote_root"
  federation_path "$socket"
  federation_path "$remote_socket"
  [[ ${socket%/*} == "$root" && ${remote_socket%/*} == "$remote_root" ]] || federation_fail "socket hors de sa racine explicite"
  (( ${#socket} <= 100 && ${#remote_socket} <= 100 )) || federation_fail "socket Unix trop longue (100 octets maximum)"
  federation_private_dir "$root"
  federation_ancestors "$socket"
  # bind(2) sous umask 077 crée une socket 0700 ; OpenSSH sous 0177 crée
  # 0600. Le bit x propriétaire n'ouvre aucun accès réseau supplémentaire.
  local socket_metadata
  socket_metadata=$(federation_metadata "$socket")
  [[ -S "$socket" && ( "$socket_metadata" == "$(id -u) 600" || "$socket_metadata" == "$(id -u) 700" ) ]] || federation_fail "socket maître privée absente : $socket"
  if $dry_run; then
    printf 'dry-run [%s] : %s:%s, transfert Unix %s -> %s ; préflight distant non exécuté\n' "$label" "$target" "$port" "$remote_socket" "$socket"
    return
  fi
  federation_remote_command "$remote_root" "$remote_socket"
  {
    federation_remote_guards
    printf '%s\n' \
      'federation_state_path "$1"; federation_path "$2"; federation_ancestors "$2"' \
      '[[ ${2%/*} == "$1" ]] || federation_fail "socket hors racine"' \
      '[[ ! -e "$2" && ! -L "$2" ]] || federation_fail "socket distante déjà occupée (même stale)"' \
      'if [[ -e "$1" ]]; then federation_private_dir "$1"; else federation_new_dir "$1"; fi'
  } | ssh "${ssh_args[@]}" "$target" "$remote_command"
  printf 'Tunnel [%s] au premier plan ; aucun service installé.\n' "$label" >&2
  printf 'Environnement des clients distants : BRIDGET_HOME=%q BRIDGET_SOCKET=%q BRIDGET_CHANNEL=ssh-unix\n' "$remote_root" "$remote_socket" >&2
  exec ssh "${ssh_args[@]}" -N -R "$remote_socket:$socket" "$target"
}

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then federation_main "$@"; fi
