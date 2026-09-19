#!/usr/bin/env bash
# Fédération 089/095 : transfert Unix et service utilisateur autonome explicite.
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

federation_label() {
  [[ "$1" =~ ^[a-zA-Z0-9][a-zA-Z0-9_-]{0,47}$ ]] || federation_fail "label privé invalide"
}

federation_hash() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    federation_fail "outil SHA-256 absent"
  fi
}

federation_owned_safe_dir() {
  local metadata uid mode group_digit other_digit
  federation_path "$1"
  federation_ancestors "$1"
  [[ -d "$1" && ! -L "$1" ]] || federation_fail "répertoire possédé requis : $1"
  metadata=$(federation_metadata "$1")
  uid=${metadata%% *}; mode=${metadata#* }
  [[ "$uid" == "$(id -u)" && "$mode" =~ ^[0-7]{3}$ ]] || federation_fail "propriétaire ou mode invalide : $1"
  group_digit=$(( (10#$mode / 10) % 10 )); other_digit=$(( 10#$mode % 10 ))
  (( (group_digit & 2) == 0 && (other_digit & 2) == 0 )) || federation_fail "répertoire inscriptible par un tiers : $1"
}

federation_regular_mode() {
  federation_path "$1"
  federation_ancestors "$1"
  [[ -f "$1" && ! -L "$1" && $(federation_metadata "$1") == "$(id -u) $2" ]] ||
    federation_fail "fichier régulier possédé mode $2 requis : $1"
}

federation_prepare_private_dir() {
  federation_path "$1"
  federation_ancestors "$1"
  if [[ ! -e "$1" && ! -L "$1" ]]; then
    mkdir -p "$1"
    chmod 0700 "$1"
  fi
  federation_private_dir "$1"
}

federation_prepare_service_dir() {
  federation_path "$1"
  federation_ancestors "$1"
  [[ -e "$1" || -L "$1" ]] || mkdir -p "$1"
  federation_owned_safe_dir "$1"
}

federation_atomic_create() {
  local destination=$1 mode=$2 description=$3 directory temporary
  directory=${destination%/*}
  federation_path "$destination"
  federation_owned_safe_dir "$directory"
  [[ ! -e "$destination" && ! -L "$destination" ]] || federation_fail "collision $description : $destination"
  temporary=$(mktemp "$directory/.federation-095.XXXXXX") || federation_fail "préparation impossible : $description"
  chmod "$mode" "$temporary"
  if ! cat >"$temporary"; then
    rm -- "$temporary"
    federation_fail "écriture impossible : $description"
  fi
  if ! ln "$temporary" "$destination"; then
    rm -- "$temporary"
    federation_fail "collision pendant publication : $destination"
  fi
  rm -- "$temporary"
  federation_regular_mode "$destination" "$mode"
}

federation_set_install_paths() {
  local platform data_home config_home
  federation_label "$label"
  federation_path "${HOME:-}"
  platform=$(uname -s)
  # Le gestionnaire utilisateur garantit HOME, contrairement aux variables XDG
  # héritées du shell d'installation. Les chemins restent donc stables au login.
  data_home="$HOME/.local/share"
  config_home="$HOME/.config"
  federation_path "$data_home"
  federation_path "$config_home"
  install_base="$data_home/bridget-federation"
  install_dir="$install_base/$label"
  runner_path="$install_dir/runner.sh"
  config_path="$install_dir/config"
  receipt_path="$install_dir/receipt"
  logs_dir="$install_dir/logs"
  stdout_path="$logs_dir/stdout.log"
  stderr_path="$logs_dir/stderr.log"
  case "$platform" in
    Darwin)
      backend=launchd
      service_label="com.bridget.federation.$label"
      service_dir="$HOME/Library/LaunchAgents"
      service_path="$service_dir/$service_label.plist"
      ;;
    Linux)
      backend=systemd
      service_label="bridget-federation-$label.service"
      service_dir="$config_home/systemd/user"
      service_path="$service_dir/$service_label"
      ;;
    *) federation_fail "plateforme non couverte : $platform (Darwin/Linux seulement)" ;;
  esac
  federation_path "$install_base"; federation_path "$install_dir"
  federation_path "$runner_path"; federation_path "$config_path"; federation_path "$receipt_path"
  federation_path "$logs_dir"; federation_path "$stdout_path"; federation_path "$stderr_path"
  federation_path "$service_dir"; federation_path "$service_path"
}

federation_load_config() {
  local path=$1 line key value count=0
  federation_private_file "$path"
  host= user= port= label= identity= known_hosts= root= socket= remote_root= remote_socket= config_backend=
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ "$line" == *=* ]] || federation_fail "configuration inerte invalide"
    key=${line%%=*}; value=${line#*=}; count=$((count + 1))
    case "$count:$key" in
      1:version) [[ "$value" == 1 ]] || federation_fail "version de configuration inconnue" ;;
      2:label) label=$value ;;
      3:backend) config_backend=$value ;;
      4:host) host=$value ;;
      5:user) user=$value ;;
      6:port) port=$value ;;
      7:identity) identity=$value ;;
      8:known_hosts) known_hosts=$value ;;
      9:root) root=$value ;;
      10:socket) socket=$value ;;
      11:remote_root) remote_root=$value ;;
      12:remote_socket) remote_socket=$value ;;
      *) federation_fail "champ ou ordre de configuration invalide : $key" ;;
    esac
  done <"$path"
  [[ $count == 12 ]] || federation_fail "configuration incomplète"
  federation_label "$label"
  [[ "$config_backend" == launchd || "$config_backend" == systemd ]] || federation_fail "backend de configuration invalide"
  [[ "$host" =~ ^[a-zA-Z0-9][a-zA-Z0-9.-]*$ && "$host" != *..* ]] || federation_fail "hôte invalide"
  [[ "$user" =~ ^[a-zA-Z_][a-zA-Z0-9_-]*$ ]] || federation_fail "utilisateur invalide"
  [[ "$port" =~ ^[0-9]{1,5}$ ]] && (( 10#$port >= 1 && 10#$port <= 65535 )) || federation_fail "port invalide"
  federation_path "$identity"; federation_path "$known_hosts"; federation_path "$root"
  federation_path "$socket"; federation_path "$remote_root"; federation_path "$remote_socket"
}

federation_load_receipt() {
  local path=$1 line key value count=0
  federation_private_file "$path"
  receipt_label= receipt_backend= receipt_runner= receipt_config= receipt_service=
  receipt_runner_hash= receipt_config_hash= receipt_service_hash=
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ "$line" == *=* ]] || federation_fail "reçu inerte invalide"
    key=${line%%=*}; value=${line#*=}; count=$((count + 1))
    case "$count:$key" in
      1:version) [[ "$value" == 1 ]] || federation_fail "version de reçu inconnue" ;;
      2:label) receipt_label=$value ;;
      3:backend) receipt_backend=$value ;;
      4:runner) receipt_runner=$value ;;
      5:config) receipt_config=$value ;;
      6:service) receipt_service=$value ;;
      7:runner_sha256) receipt_runner_hash=$value ;;
      8:config_sha256) receipt_config_hash=$value ;;
      9:service_sha256) receipt_service_hash=$value ;;
      *) federation_fail "champ ou ordre de reçu invalide : $key" ;;
    esac
  done <"$path"
  [[ $count == 9 ]] || federation_fail "reçu incomplet"
  [[ "$receipt_runner_hash" =~ ^[0-9a-f]{64}$ && "$receipt_config_hash" =~ ^[0-9a-f]{64}$ && "$receipt_service_hash" =~ ^[0-9a-f]{64}$ ]] || federation_fail "hash de reçu invalide"
}

federation_validate_install() {
  local requested_label=$label requested_backend=$backend
  federation_private_dir "$install_dir"
  federation_regular_mode "$runner_path" 700
  federation_regular_mode "$config_path" 600
  federation_regular_mode "$receipt_path" 600
  federation_regular_mode "$service_path" 600
  federation_load_receipt "$receipt_path"
  [[ "$receipt_label" == "$requested_label" && "$receipt_backend" == "$requested_backend" ]] || federation_fail "reçu incompatible avec le service demandé"
  [[ "$receipt_runner" == "$runner_path" && "$receipt_config" == "$config_path" && "$receipt_service" == "$service_path" ]] || federation_fail "chemins du reçu incompatibles"
  [[ $(federation_hash "$runner_path") == "$receipt_runner_hash" ]] || federation_fail "runner installé altéré"
  [[ $(federation_hash "$config_path") == "$receipt_config_hash" ]] || federation_fail "configuration installée altérée"
  [[ $(federation_hash "$service_path") == "$receipt_service_hash" ]] || federation_fail "unité installée altérée"
  federation_load_config "$config_path"
  [[ "$label" == "$requested_label" && "$config_backend" == "$requested_backend" ]] || federation_fail "configuration incompatible avec le reçu"
  label=$requested_label; backend=$requested_backend
}

federation_parse_link_args() {
  local allow_dry_run=$1
  shift
  host= user= port=22 label= identity= known_hosts= root= socket= remote_root= remote_socket= dry_run=false
  while (( $# )); do
    case "$1" in
      --dry-run)
        [[ "$allow_dry_run" == true ]] || federation_fail "--dry-run réservé à run"
        dry_run=true; shift; continue ;;
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
}

federation_validate_link() {
  federation_connection
  federation_state_path "$root"
  federation_state_path "$remote_root"
  federation_path "$socket"
  federation_path "$remote_socket"
  [[ ${socket%/*} == "$root" && ${remote_socket%/*} == "$remote_root" ]] || federation_fail "socket hors de sa racine explicite"
  (( ${#socket} <= 100 && ${#remote_socket} <= 100 )) || federation_fail "socket Unix trop longue (100 octets maximum)"
  federation_private_dir "$root"
  federation_ancestors "$socket"
  local socket_metadata
  socket_metadata=$(federation_metadata "$socket")
  [[ -S "$socket" && ( "$socket_metadata" == "$(id -u) 600" || "$socket_metadata" == "$(id -u) 700" ) ]] || federation_fail "socket maître privée absente : $socket"
}

federation_remote_preflight() {
  local mode=$1
  federation_remote_command "$remote_root" "$remote_socket" "$mode"
  {
    federation_remote_guards
    cat <<'REMOTE'
federation_state_path "$1"; federation_path "$2"; federation_ancestors "$2"
[[ ${2%/*} == "$1" ]] || federation_fail "socket hors racine"
if [[ "$3" == remove ]]; then
  if [[ ! -e "$1" && ! -L "$1" && ! -e "$2" && ! -L "$2" ]]; then
    exit 0
  fi
  federation_private_dir "$1"
elif [[ -e "$1" ]]; then
  federation_private_dir "$1"
else
  federation_new_dir "$1"
fi
if [[ "$3" == strict ]]; then
  [[ ! -e "$2" && ! -L "$2" ]] || federation_fail "socket distante déjà occupée (même stale)"
  exit 0
fi
config="$1/federation.env"
federation_path "$config"; federation_ancestors "$config"
if [[ "$3" == recover ]]; then
  if [[ -e "$config" || -L "$config" ]]; then
    [[ -f "$config" && ! -L "$config" && $(federation_metadata "$config") == "$(id -u) 600" ]] || federation_fail "configuration distante étrangère"
    [[ $(cat "$config") == channel=ssh-unix ]] || federation_fail "configuration distante incompatible"
  else
    (umask 077; set -C; printf 'channel=ssh-unix\n' >"$config") || federation_fail "collision configuration distante"
  fi
fi
if [[ -e "$2" || -L "$2" ]]; then
  if ! python3 - "$1" "$2" <<'PY'
import errno, os, socket, stat, sys
root, path = sys.argv[1:]
def refuse(message):
    print(message, file=sys.stderr)
    raise SystemExit(23)
rst = os.lstat(root)
if not stat.S_ISDIR(rst.st_mode) or rst.st_uid != os.getuid() or stat.S_IMODE(rst.st_mode) != 0o700:
    refuse("racine distante privée 0700 requise")
try:
    before = os.lstat(path)
except FileNotFoundError:
    raise SystemExit(0)
if not stat.S_ISSOCK(before.st_mode) or before.st_uid != os.getuid() or stat.S_IMODE(before.st_mode) not in (0o600, 0o700):
    refuse("socket distante étrangère ou non privée")
probe = socket.socket(socket.AF_UNIX)
probe.settimeout(1.0)
try:
    probe.connect(path)
except OSError as exc:
    if exc.errno != errno.ECONNREFUSED:
        refuse("état de socket ambigu : refus de suppression")
else:
    refuse("socket distante vivante : refus de remplacement")
finally:
    probe.close()
try:
    after = os.lstat(path)
except FileNotFoundError:
    refuse("socket remplacée pendant la sonde")
identity = lambda item: (item.st_dev, item.st_ino, item.st_uid, stat.S_IFMT(item.st_mode), stat.S_IMODE(item.st_mode))
if identity(before) != identity(after):
    refuse("socket remplacée pendant la sonde")
os.unlink(path)
PY
  then
    federation_fail "socket distante non récupérable"
  fi
fi
REMOTE
  } | ssh "${ssh_args[@]}" "$target" "$remote_command"
}

federation_run() {
  local mode=$1
  federation_validate_link
  if $dry_run; then
    printf 'dry-run [%s] : %s:%s, transfert Unix %s -> %s ; préflight distant non exécuté\n' "$label" "$target" "$port" "$remote_socket" "$socket"
    return
  fi
  federation_remote_preflight "$mode"
  printf 'Tunnel [%s] au premier plan ; aucun daemon Bridget créé.\n' "$label" >&2
  printf 'Environnement des clients distants : BRIDGET_HOME=%q BRIDGET_SOCKET=%q BRIDGET_CHANNEL=ssh-unix\n' "$remote_root" "$remote_socket" >&2
  exec ssh "${ssh_args[@]}" -N -R "$remote_socket:$socket" "$target"
}

federation_write_config() {
  {
    printf 'version=1\nlabel=%s\nbackend=%s\n' "$label" "$backend"
    printf 'host=%s\nuser=%s\nport=%s\n' "$host" "$user" "$port"
    printf 'identity=%s\nknown_hosts=%s\n' "$identity" "$known_hosts"
    printf 'root=%s\nsocket=%s\nremote_root=%s\nremote_socket=%s\n' "$root" "$socket" "$remote_root" "$remote_socket"
  } | federation_atomic_create "$config_path" 600 "configuration"
}

federation_write_service() {
  case "$backend" in
    launchd)
      {
        printf '%s\n' '<?xml version="1.0" encoding="UTF-8"?>' \
          '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' \
          '<plist version="1.0"><dict>' \
          "<key>Label</key><string>$service_label</string>" \
          '<key>ProgramArguments</key><array>' \
          "<string>$runner_path</string><string>service-run</string><string>--config</string><string>$config_path</string>" \
          '</array>' '<key>KeepAlive</key><true/>' '<key>RunAtLoad</key><true/>' \
          '<key>ThrottleInterval</key><integer>10</integer>' \
          '<key>Umask</key><integer>63</integer>' \
          "<key>StandardOutPath</key><string>$stdout_path</string>" \
          "<key>StandardErrorPath</key><string>$stderr_path</string>" \
          '</dict></plist>'
      } | federation_atomic_create "$service_path" 600 "unité launchd"
      ;;
    systemd)
      {
        printf '%s\n' '[Unit]' 'Description=Bridget federation SSH' \
          'StartLimitIntervalSec=0' '' '[Service]' 'Type=simple' \
          "ExecStart=\"$runner_path\" service-run --config \"$config_path\"" \
          'Restart=always' 'RestartSec=5' 'UMask=0077' 'KillSignal=SIGTERM' 'SendSIGKILL=no' \
          'TimeoutStopSec=10' "StandardOutput=append:$stdout_path" \
          "StandardError=append:$stderr_path" '' '[Install]' 'WantedBy=default.target'
      } | federation_atomic_create "$service_path" 600 "unité systemd utilisateur"
      ;;
  esac
}

federation_write_receipt() {
  receipt_runner_hash=$(federation_hash "$runner_path")
  receipt_config_hash=$(federation_hash "$config_path")
  receipt_service_hash=$(federation_hash "$service_path")
  {
    printf 'version=1\nlabel=%s\nbackend=%s\n' "$label" "$backend"
    printf 'runner=%s\nconfig=%s\nservice=%s\n' "$runner_path" "$config_path" "$service_path"
    printf 'runner_sha256=%s\nconfig_sha256=%s\nservice_sha256=%s\n' "$receipt_runner_hash" "$receipt_config_hash" "$receipt_service_hash"
  } | federation_atomic_create "$receipt_path" 600 "reçu d'installation"
}

federation_activate() {
  case "$backend" in
    launchd)
      # Le préflight natif a déjà attesté l'absence du label : dès cet appel,
      # un succès partiel ou un signal doit passer par l'arrêt contrôlé.
      rollback_activation=true
      launchctl bootstrap "gui/$(id -u)" "$service_path" || federation_fail "activation launchd impossible : $service_label"
      launchctl print "gui/$(id -u)/$service_label" >/dev/null 2>&1 || federation_fail "service launchd non actif : $service_label"
      ;;
    systemd)
      systemctl --user daemon-reload || federation_fail "systemd --user daemon-reload impossible"
      rollback_activation=true
      systemctl --user enable --now "$service_label" || federation_fail "activation systemd utilisateur impossible : $service_label"
      systemctl --user is-active --quiet "$service_label" || federation_fail "service systemd utilisateur non actif : $service_label"
      systemctl --user is-enabled --quiet "$service_label" || federation_fail "service systemd utilisateur non activé : $service_label"
      ;;
  esac
}

federation_native_label_absent() {
  local native_state
  case "$backend" in
    launchd)
      if launchctl print "gui/$(id -u)/$service_label" >/dev/null 2>&1; then
        federation_fail "label launchd déjà chargé hors installation attestée : $service_label"
      fi
      ;;
    systemd)
      native_state=$(systemctl --user show --property=LoadState --value "$service_label" 2>/dev/null) ||
        federation_fail "état du label systemd invérifiable : $service_label"
      [[ "$native_state" == not-found ]] || federation_fail "label systemd déjà connu hors installation attestée : $service_label"
      ;;
  esac
}

federation_remove_if_attested() {
  local path=$1 mode=$2 expected_hash=$3
  [[ -e "$path" || -L "$path" ]] || return 0
  federation_regular_mode "$path" "$mode"
  [[ $(federation_hash "$path") == "$expected_hash" ]] || federation_fail "fichier remplacé avant suppression : $path"
  rm -- "$path"
}

federation_remove_log() {
  local path=$1 metadata uid mode group_digit other_digit
  [[ -e "$path" || -L "$path" ]] || return 0
  [[ -f "$path" && ! -L "$path" ]] || federation_fail "log étranger refusé : $path"
  metadata=$(federation_metadata "$path"); uid=${metadata%% *}; mode=${metadata#* }
  [[ "$uid" == "$(id -u)" && "$mode" =~ ^[0-7]{3}$ ]] || federation_fail "log non possédé refusé : $path"
  group_digit=$(( (10#$mode / 10) % 10 )); other_digit=$(( 10#$mode % 10 ))
  (( (group_digit & 2) == 0 && (other_digit & 2) == 0 )) || federation_fail "log inscriptible par un tiers refusé : $path"
  rm -- "$path"
}

federation_rollback_install() {
  [[ ${rollback_install:-false} == true ]] || return 0
  # Empêche toute récursion si une garde du sous-processus de rollback échoue.
  rollback_install=false
  if [[ ${rollback_activation:-false} == true ]]; then
    if ! (
      trap - EXIT HUP INT TERM
      federation_deactivate
      federation_connection
      federation_remote_preflight remove
    ); then
      printf 'Rollback incomplet [%s] : arrêt ou nettoyage distant non confirmé ; installation conservée pour reprise.\n' "${label:-inconnu}" >&2
      return 0
    fi
  fi
  federation_rollback_file "$receipt_path" 600 "${rollback_receipt_hash:-}"
  federation_rollback_file "$service_path" 600 "${rollback_service_hash:-}"
  federation_rollback_file "$config_path" 600 "${rollback_config_hash:-}"
  federation_rollback_file "$runner_path" 700 "${rollback_runner_hash:-}"
  [[ -n ${logs_dir:-} && -d "$logs_dir" && ! -L "$logs_dir" ]] && rmdir "$logs_dir" 2>/dev/null || true
  [[ -n ${install_dir:-} && -d "$install_dir" && ! -L "$install_dir" ]] && rmdir "$install_dir" 2>/dev/null || true
}

federation_rollback_file() {
  local path=${1:-} mode=${2:-} expected_hash=${3:-}
  [[ -n "$path" && -n "$expected_hash" && -f "$path" && ! -L "$path" ]] || return 0
  [[ $(federation_metadata "$path" 2>/dev/null || true) == "$(id -u) $mode" ]] || return 0
  [[ $(federation_hash "$path" 2>/dev/null || true) == "$expected_hash" ]] || return 0
  rm -- "$path"
}

federation_install() {
  local requested_host requested_user requested_port requested_label requested_identity requested_known_hosts
  local requested_root requested_socket requested_remote_root requested_remote_socket
  federation_parse_link_args false "$@"
  federation_validate_link
  federation_set_install_paths
  requested_host=$host; requested_user=$user; requested_port=$port; requested_label=$label
  requested_identity=$identity; requested_known_hosts=$known_hosts
  requested_root=$root; requested_socket=$socket
  requested_remote_root=$remote_root; requested_remote_socket=$remote_socket
  if [[ -e "$install_dir" || -L "$install_dir" || -e "$service_path" || -L "$service_path" ]]; then
    [[ -d "$install_dir" && ! -L "$install_dir" && -f "$service_path" && ! -L "$service_path" ]] || federation_fail "collision avec une installation étrangère : $label"
    federation_validate_install
    [[ "$host" == "$requested_host" && "$user" == "$requested_user" && "$port" == "$requested_port" && "$label" == "$requested_label" && "$identity" == "$requested_identity" && "$known_hosts" == "$requested_known_hosts" && "$root" == "$requested_root" && "$socket" == "$requested_socket" && "$remote_root" == "$requested_remote_root" && "$remote_socket" == "$requested_remote_socket" ]] || federation_fail "installation existante attestée mais paramètres divergents"
    printf 'Liaison [%s] déjà installée et attestée (%s).\n' "$label" "$backend" >&2
    return 0
  fi
  federation_native_label_absent
  federation_remote_preflight strict
  rollback_install=true; rollback_activation=false
  rollback_runner_hash=; rollback_config_hash=; rollback_service_hash=; rollback_receipt_hash=
  trap federation_rollback_install EXIT
  trap 'exit 129' HUP
  trap 'exit 130' INT
  trap 'exit 143' TERM
  federation_prepare_private_dir "$install_base"
  mkdir -m 0700 "$install_dir"
  federation_private_dir "$install_dir"
  mkdir -m 0700 "$logs_dir"
  federation_private_dir "$logs_dir"
  federation_prepare_service_dir "$service_dir"
  federation_atomic_create "$runner_path" 700 "runner autonome" <"${BASH_SOURCE[0]}"
  rollback_runner_hash=$(federation_hash "$runner_path")
  federation_write_config
  rollback_config_hash=$(federation_hash "$config_path")
  federation_write_service
  rollback_service_hash=$(federation_hash "$service_path")
  federation_write_receipt
  rollback_receipt_hash=$(federation_hash "$receipt_path")
  federation_activate
  rollback_install=false
  trap - EXIT HUP INT TERM
  printf 'Liaison [%s] installée ; unité chargée (%s), connectivité à confirmer par la recette.\n' "$label" "$backend" >&2
  if [[ "$backend" == systemd ]]; then
    printf 'Persistance après déconnexion : vérifier le linger de la session utilisateur ; aucune politique privilégiée n’est modifiée.\n' >&2
  fi
}

federation_status() {
  local native_state
  federation_set_install_paths
  [[ -d "$install_dir" && ! -L "$install_dir" && -f "$service_path" && ! -L "$service_path" ]] || federation_fail "liaison inconnue : $label"
  federation_validate_install
  case "$backend" in
    launchd)
      native_state=$(launchctl print "gui/$(id -u)/$service_label" 2>/dev/null) || { printf 'inactif [%s] (%s)\n' "$label" "$backend"; return 1; }
      [[ "$native_state" == *"state = running"* ]] || { printf 'chargé sans processus actif attesté [%s] (%s)\n' "$label" "$backend"; return 1; }
      ;;
    systemd)
      systemctl --user is-active --quiet "$service_label" || { printf 'inactif [%s] (%s)\n' "$label" "$backend"; return 1; }
      ;;
  esac
  printf 'processus actif [%s] (%s) ; connectivité du tunnel non attestée\n' "$label" "$backend"
}

federation_deactivate() {
  case "$backend" in
    launchd)
      if launchctl print "gui/$(id -u)/$service_label" >/dev/null 2>&1; then
        launchctl bootout "gui/$(id -u)/$service_label" || federation_fail "arrêt launchd impossible : $service_label"
      fi
      launchctl print "gui/$(id -u)/$service_label" >/dev/null 2>&1 && federation_fail "service launchd encore actif : $service_label"
      ;;
    systemd)
      if systemctl --user is-active --quiet "$service_label" || systemctl --user is-enabled --quiet "$service_label"; then
        systemctl --user disable --now "$service_label" || federation_fail "arrêt systemd utilisateur impossible : $service_label"
      fi
      systemctl --user daemon-reload || federation_fail "systemd --user daemon-reload impossible"
      systemctl --user is-active --quiet "$service_label" && federation_fail "service systemd utilisateur encore actif : $service_label"
      systemctl --user is-enabled --quiet "$service_label" && federation_fail "service systemd utilisateur encore activé : $service_label"
      ;;
  esac
  return 0
}

federation_remove() {
  federation_set_install_paths
  [[ -d "$install_dir" && ! -L "$install_dir" && -f "$service_path" && ! -L "$service_path" ]] || federation_fail "liaison inconnue : $label"
  federation_validate_install
  receipt_self_hash=$(federation_hash "$receipt_path")
  federation_deactivate
  federation_connection
  federation_remote_preflight remove
  federation_remove_if_attested "$service_path" 600 "$receipt_service_hash"
  federation_remove_if_attested "$config_path" 600 "$receipt_config_hash"
  federation_remove_if_attested "$runner_path" 700 "$receipt_runner_hash"
  federation_remove_log "$stdout_path"; federation_remove_log "$stderr_path"
  federation_remove_if_attested "$receipt_path" 600 "$receipt_self_hash"
  rmdir "$logs_dir" || federation_fail "répertoire de logs non vide : $logs_dir"
  rmdir "$install_dir" || federation_fail "installation contient des fichiers étrangers : $install_dir"
  printf 'Liaison [%s] retirée ; clés SSH et données Bridget conservées.\n' "$label" >&2
}

federation_cli_usage() {
  printf '%s\n' \
    'Usage: bridget federate ssh://[utilisateur@]hôte[:port] [-p PORT] [OPTIONS]' \
    '       bridget federate status [--label NOM]' \
    '       bridget federate remove [ssh://[utilisateur@]hôte[:port]] [--label NOM] [-p PORT]' \
    '       bridget federate --help' \
    'Options de nouvelle liaison : --label --user --identity --known-hosts --root --socket' \
    '                               --remote-root --remote-socket' \
    'Le port vaut 22 par défaut. Un port URL et -p doivent être identiques.' \
    'Les questions interactives exigent stdin ET stdout sur un terminal.'
}

federation_cli_parse_destination() {
  local parsed uri_user uri_host uri_port
  parsed=$(python3 - "$cli_destination" <<'PY'
import re, sys
from urllib.parse import urlsplit

value = sys.argv[1]
if value.strip() != value or any(ord(character) < 32 or ord(character) == 127 for character in value):
    print("espaces périphériques et caractères de contrôle interdits", file=sys.stderr)
    raise SystemExit(2)
try:
    parsed = urlsplit(value)
    port = parsed.port
except ValueError as error:
    print(f"destination SSH invalide : {error}", file=sys.stderr)
    raise SystemExit(2)
if parsed.scheme != "ssh" or not parsed.hostname:
    print("destination attendue : ssh://[utilisateur@]hôte", file=sys.stderr)
    raise SystemExit(2)
if parsed.password is not None or parsed.path or parsed.query or parsed.fragment or "%" in value:
    print("mot de passe, encodage, chemin, query et fragment sont interdits", file=sys.stderr)
    raise SystemExit(2)
host = parsed.hostname.lower()
user = parsed.username or "-"
if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9.-]*", host) or ".." in host:
    print("hôte SSH invalide", file=sys.stderr)
    raise SystemExit(2)
if user != "-" and not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_-]*", user):
    print("utilisateur SSH invalide", file=sys.stderr)
    raise SystemExit(2)
if port is not None and not 1 <= port <= 65535:
    print("port SSH hors limites", file=sys.stderr)
    raise SystemExit(2)
print(f"{user}|{host}|{port if port is not None else '-'}")
PY
  ) || federation_fail "destination SSH invalide ; utiliser ssh://[utilisateur@]hôte"
  IFS='|' read -r uri_user uri_host uri_port <<<"$parsed"
  [[ "$uri_user" == - ]] && uri_user=
  [[ "$uri_port" == - ]] && uri_port=
  if [[ -n "$cli_port" && -n "$uri_port" && "$cli_port" != "$uri_port" ]]; then
    federation_fail "ports contradictoires : URL=$uri_port option=$cli_port"
  fi
  if [[ -n "$cli_user" && -n "$uri_user" && "$cli_user" != "$uri_user" ]]; then
    federation_fail "utilisateurs contradictoires : URL=$uri_user option=$cli_user"
  fi
  requested_host=$uri_host
  requested_port=${cli_port:-${uri_port:-22}}
  requested_user=${cli_user:-$uri_user}
  [[ "$requested_port" =~ ^[0-9]{1,5}$ ]] && (( 10#$requested_port >= 1 && 10#$requested_port <= 65535 )) ||
    federation_fail "port invalide"
}

federation_cli_parse_options() {
  cli_label= cli_user= cli_port= cli_identity= cli_known_hosts=
  cli_root= cli_socket= cli_remote_root= cli_remote_socket=
  while (( $# )); do
    case "$1" in
      -p|--port|--label|--user|--identity|--known-hosts|--root|--socket|--remote-root|--remote-socket)
        (( $# >= 2 )) || federation_fail "valeur absente pour $1" ;;
      *) federation_fail "option federate inconnue : $1" ;;
    esac
    case "$1" in
      -p|--port) [[ -z "$cli_port" ]] || federation_fail "option port répétée"; cli_port=$2 ;;
      --label) [[ -z "$cli_label" ]] || federation_fail "option --label répétée"; cli_label=$2 ;;
      --user) [[ -z "$cli_user" ]] || federation_fail "option --user répétée"; cli_user=$2 ;;
      --identity) [[ -z "$cli_identity" ]] || federation_fail "option --identity répétée"; cli_identity=$2 ;;
      --known-hosts) [[ -z "$cli_known_hosts" ]] || federation_fail "option --known-hosts répétée"; cli_known_hosts=$2 ;;
      --root) [[ -z "$cli_root" ]] || federation_fail "option --root répétée"; cli_root=$2 ;;
      --socket) [[ -z "$cli_socket" ]] || federation_fail "option --socket répétée"; cli_socket=$2 ;;
      --remote-root) [[ -z "$cli_remote_root" ]] || federation_fail "option --remote-root répétée"; cli_remote_root=$2 ;;
      --remote-socket) [[ -z "$cli_remote_socket" ]] || federation_fail "option --remote-socket répétée"; cli_remote_socket=$2 ;;
    esac
    shift 2
  done
  [[ -z "$cli_label" ]] || federation_label "$cli_label"
}

federation_cli_inventory() {
  local base candidate candidate_count=0 candidate_label candidate_host candidate_user candidate_port index
  local -a candidates
  cli_install_labels=(); cli_install_hosts=(); cli_install_users=(); cli_install_ports=()
  cli_install_count=0
  federation_path "${HOME:-}"
  base="$HOME/.local/share/bridget-federation"
  federation_path "$base"
  [[ ! -L "$base" ]] || federation_fail "inventaire de fédération lié interdit : $base"
  [[ -e "$base" ]] || return 0
  federation_private_dir "$base"
  candidates=()
  for candidate in "$base"/*; do
    [[ -e "$candidate" || -L "$candidate" ]] || continue
    (( candidate_count < 128 )) || federation_fail "inventaire trop grand : maximum 128 installations"
    candidates[$candidate_count]=$candidate
    candidate_count=$((candidate_count + 1))
  done
  # Complexité O(n), n <= 128 : chaque reçu/configuration est validé une fois.
  for (( index=0; index<candidate_count; index++ )); do
    candidate=${candidates[$index]}
    [[ -d "$candidate" && ! -L "$candidate" ]] || federation_fail "entrée d'inventaire étrangère : $candidate"
    candidate_label=${candidate##*/}
    federation_label "$candidate_label"
    label=$candidate_label
    federation_set_install_paths
    [[ "$candidate" == "$install_dir" ]] || federation_fail "chemin d'installation incohérent : $candidate"
    federation_validate_install
    candidate_host=$host; candidate_user=$user; candidate_port=$port
    cli_install_labels+=("$candidate_label")
    cli_install_hosts+=("$candidate_host")
    cli_install_users+=("$candidate_user")
    cli_install_ports+=("$candidate_port")
    cli_install_count=$((cli_install_count + 1))
  done
}

federation_cli_resolve() {
  cli_resolved=$(python3 - "$requested_host" <<'PY'
import ipaddress, signal, socket, sys

def expired(_signum, _frame):
    raise TimeoutError("budget DNS dépassé")

signal.signal(signal.SIGALRM, expired)
signal.alarm(5)
try:
    addresses = {
        str(ipaddress.ip_address(item[4][0]))
        for item in socket.getaddrinfo(sys.argv[1], None, type=socket.SOCK_STREAM)
    }
finally:
    signal.alarm(0)
if not addresses:
    print("résolution DNS vide", file=sys.stderr)
    raise SystemExit(2)
if len(addresses) > 64:
    print("résolution DNS trop grande : maximum 64 adresses", file=sys.stderr)
    raise SystemExit(2)
print("|".join(sorted(addresses)))
PY
  ) || federation_fail "résolution DNS impossible ou hors budget pour $requested_host"
}

federation_cli_find_destination() {
  local index candidate_host_lower
  cli_matches=(); cli_match_count=0; cli_match_kind=none
  # D'abord l'égalité textuelle canonique, qui n'a besoin d'aucun DNS.
  for (( index=0; index<cli_install_count; index++ )); do
    [[ "${cli_install_ports[$index]}" == "$requested_port" ]] || continue
    [[ -z "$requested_user" || "${cli_install_users[$index]}" == "$requested_user" ]] || continue
    candidate_host_lower=$(printf '%s' "${cli_install_hosts[$index]}" | tr '[:upper:]' '[:lower:]')
    [[ "$candidate_host_lower" == "$requested_host" ]] || continue
    cli_matches[$cli_match_count]=$index; cli_match_count=$((cli_match_count + 1))
  done
  if (( cli_match_count )); then
    cli_match_kind=exact
    federation_cli_filter_matches_by_label
    return 0
  fi
  (( cli_install_count )) || return 0
  federation_cli_resolve
  # Complexité O(n*m), bornée à 128 installations x 64 adresses, chemin administratif froid.
  for (( index=0; index<cli_install_count; index++ )); do
    [[ "${cli_install_ports[$index]}" == "$requested_port" ]] || continue
    [[ -z "$requested_user" || "${cli_install_users[$index]}" == "$requested_user" ]] || continue
    case "|$cli_resolved|" in
      *"|${cli_install_hosts[$index]}|"*) cli_matches[$cli_match_count]=$index; cli_match_count=$((cli_match_count + 1)) ;;
    esac
  done
  if (( cli_match_count )); then
    cli_match_kind=dns
    federation_cli_filter_matches_by_label
  fi
}

federation_cli_filter_matches_by_label() {
  local position matched_index
  [[ -n "$cli_label" ]] || return 0
  for (( position=0; position<cli_match_count; position++ )); do
    matched_index=${cli_matches[$position]}
    [[ "${cli_install_labels[$matched_index]}" == "$cli_label" ]] || continue
    cli_matches[0]=$matched_index
    cli_match_count=1
    return 0
  done
  federation_fail "la destination est déjà gérée par une autre installation ; --label $cli_label diverge"
}

federation_cli_require_unique() {
  (( cli_match_count == 1 )) || {
    if (( cli_match_count > 1 )); then
      federation_fail "destination ambiguë ($cli_match_count installations) ; préciser --label"
    fi
    federation_fail "aucune installation ne correspond à la destination"
  }
  cli_selected=${cli_matches[0]}
}

federation_cli_prompt_missing() {
  local variable_name=$1 prompt=$2 value
  [[ -t 0 && -t 1 ]] || return 1
  printf '%s: ' "$prompt"
  IFS= read -r value || federation_fail "saisie interrompue pour $prompt"
  [[ -n "$value" ]] || federation_fail "valeur vide refusée pour $prompt"
  printf -v "$variable_name" '%s' "$value"
}

federation_cli_validate_selected_options() {
  label=${cli_install_labels[$cli_selected]}
  federation_set_install_paths
  federation_validate_install
  [[ -z "$cli_identity" || "$cli_identity" == "$identity" ]] || federation_fail "--identity contredit l'installation [$label]"
  [[ -z "$cli_known_hosts" || "$cli_known_hosts" == "$known_hosts" ]] || federation_fail "--known-hosts contredit l'installation [$label]"
  [[ -z "$cli_root" || "$cli_root" == "$root" ]] || federation_fail "--root contredit l'installation [$label]"
  [[ -z "$cli_socket" || "$cli_socket" == "$socket" ]] || federation_fail "--socket contredit l'installation [$label]"
  [[ -z "$cli_remote_root" || "$cli_remote_root" == "$remote_root" ]] || federation_fail "--remote-root contredit l'installation [$label]"
  [[ -z "$cli_remote_socket" || "$cli_remote_socket" == "$remote_socket" ]] || federation_fail "--remote-socket contredit l'installation [$label]"
}

federation_cli_install_new() {
  local missing= option variable prompt
  local -a requirements
  requirements=(
    '--label|cli_label|Label de la liaison'
    '--user|requested_user|Utilisateur SSH'
    '--identity|cli_identity|Chemin de la clé privée SSH'
    '--known-hosts|cli_known_hosts|Chemin du fichier known_hosts'
    '--root|cli_root|Racine locale Bridget'
    '--socket|cli_socket|Socket locale du maître'
    '--remote-root|cli_remote_root|Racine Bridget distante'
    '--remote-socket|cli_remote_socket|Socket cliente distante'
  )
  for requirement in "${requirements[@]}"; do
    IFS='|' read -r option variable prompt <<<"$requirement"
    [[ -n "${!variable}" ]] && continue
    if ! federation_cli_prompt_missing "$variable" "$prompt"; then
      missing+=" $option"
    fi
  done
  [[ -z "$missing" ]] || federation_fail "paramètres manquants hors double terminal :${missing}"
  federation_install --label "$cli_label" --host "$requested_host" --user "$requested_user" \
    --port "$requested_port" --identity "$cli_identity" --known-hosts "$cli_known_hosts" \
    --root "$cli_root" --socket "$cli_socket" --remote-root "$cli_remote_root" \
    --remote-socket "$cli_remote_socket"
}

federation_cli_status_all() {
  local index failed=false
  federation_cli_inventory
  if [[ -n "$cli_label" ]]; then
    for (( index=0; index<cli_install_count; index++ )); do
      [[ "${cli_install_labels[$index]}" == "$cli_label" ]] || continue
      label=$cli_label
      federation_status
      return
    done
    federation_fail "liaison inconnue : $cli_label"
  fi
  (( cli_install_count )) || { printf 'aucune liaison fédérée installée\n'; return; }
  for (( index=0; index<cli_install_count; index++ )); do
    label=${cli_install_labels[$index]}
    if ! federation_status; then failed=true; fi
  done
  [[ "$failed" == false ]]
}

federation_cli_connect() {
  local selected_label
  federation_cli_parse_destination
  federation_cli_inventory
  federation_cli_find_destination
  if (( cli_match_count > 1 )); then federation_cli_require_unique; fi
  if (( cli_match_count == 1 )); then
    cli_selected=${cli_matches[0]}; selected_label=${cli_install_labels[$cli_selected]}
    federation_cli_validate_selected_options
    federation_status
    printf 'Liaison [%s] réutilisée sans réinstallation ; cible enregistrée %s:%s.\n' \
      "$selected_label" "${cli_install_hosts[$cli_selected]}" "${cli_install_ports[$cli_selected]}"
    return
  fi
  [[ -z "$cli_label" ]] || {
    for (( cli_selected=0; cli_selected<cli_install_count; cli_selected++ )); do
      selected_label=${cli_install_labels[$cli_selected]}
      [[ "$selected_label" == "$cli_label" ]] && federation_fail "--label $cli_label existe mais vise une autre destination"
    done
  }
  federation_cli_install_new
}

federation_cli_remove() {
  local answer
  federation_cli_inventory
  if [[ -z "$cli_destination" ]]; then
    [[ -n "$cli_label" ]] || federation_fail "remove exige une destination ou --label"
    [[ -z "$cli_user$cli_port$cli_identity$cli_known_hosts$cli_root$cli_socket$cli_remote_root$cli_remote_socket" ]] ||
      federation_fail "remove sans destination accepte seulement --label"
    for (( cli_selected=0; cli_selected<cli_install_count; cli_selected++ )); do
      [[ "${cli_install_labels[$cli_selected]}" == "$cli_label" ]] || continue
      label=$cli_label; federation_remove; return
    done
    federation_fail "liaison inconnue : $cli_label"
  fi
  federation_cli_parse_destination
  federation_cli_find_destination
  federation_cli_require_unique
  federation_cli_validate_selected_options
  if [[ "$cli_match_kind" == dns && -z "$cli_label" ]]; then
    if [[ ! -t 0 || ! -t 1 ]]; then
      federation_fail "alias DNS vers [$label] ${cli_install_hosts[$cli_selected]}:${cli_install_ports[$cli_selected]} ; confirmer avec --label $label"
    fi
    printf 'Retirer [%s], cible enregistrée %s:%s ? saisir oui: ' \
      "$label" "${cli_install_hosts[$cli_selected]}" "${cli_install_ports[$cli_selected]}"
    IFS= read -r answer || federation_fail "confirmation interrompue"
    [[ "$answer" == oui ]] || federation_fail "retrait annulé"
  fi
  federation_remove
}

federation_cli() {
  local operation=connect
  cli_destination=
  [[ ${1:-} != --help && ${1:-} != -h ]] || { federation_cli_usage; return; }
  case ${1:-} in
    status) operation=status; shift ;;
    remove) operation=remove; shift ;;
  esac
  if [[ "$operation" != status && -n ${1:-} && ${1:-} != -* ]]; then cli_destination=$1; shift; fi
  if [[ "$operation" == status ]]; then
    federation_cli_parse_options "$@"
    [[ -z "$cli_user$cli_port$cli_identity$cli_known_hosts$cli_root$cli_socket$cli_remote_root$cli_remote_socket" ]] ||
      federation_fail "status accepte seulement --label"
    federation_cli_status_all
    return
  fi
  federation_cli_parse_options "$@"
  case "$operation" in
    connect) [[ -n "$cli_destination" ]] || federation_fail "destination attendue : ssh://[utilisateur@]hôte"; federation_cli_connect ;;
    remove) federation_cli_remove ;;
  esac
}

federation_usage() {
  printf '%s\n' \
    'Usage: federate-ssh.sh run|install --label NAME --host HOST --user USER --identity FILE --known-hosts FILE' \
    '       --root DIR --socket PATH --remote-root DIR --remote-socket PATH [--port 22] [--dry-run]' \
    '       federate-ssh.sh status|remove --label NAME' \
    '       federate-ssh.sh cli URL|status|remove ...   Façade du binaire bridget federate' \
    'Racines/socket absolues privées ; fichiers SSH déjà en 0600.' \
    'run reste au premier plan et refuse toute socket distante occupée, même stale.' \
    'install pose un runner autonome et un service utilisateur launchd/systemd --user.' \
    'Clients distants : BRIDGET_HOME=remote-root BRIDGET_SOCKET=remote-socket BRIDGET_CHANNEL=ssh-unix.' \
    'dry-run : lectures locales seulement, aucune écriture ni commande SSH/rsync.' \
    'La reprise supervisée ne retire une socket stale qu’après propriété privée, ECONNREFUSED' \
    'et revalidation identité/inode ; socket vivante, timeout ou état ambigu sont refusés.'
}

federation_main() {
  set -euo pipefail
  umask 077
  action=${1:-}
  [[ "$action" != -h && "$action" != --help ]] || { federation_usage; return; }
  shift || true
  case "$action" in
    cli) federation_cli "$@" ;;
    run) federation_parse_link_args true "$@"; federation_run strict ;;
    install) federation_install "$@" ;;
    status|remove)
      [[ ${1:-} == --label && -n ${2:-} && $# == 2 ]] || federation_fail "usage : $action --label NAME"
      label=$2
      if [[ "$action" == status ]]; then federation_status; else federation_remove; fi
      ;;
    service-run)
      [[ ${1:-} == --config && -n ${2:-} && $# == 2 ]] || federation_fail "usage interne invalide"
      supplied_config=$2
      federation_load_config "$supplied_config"
      federation_set_install_paths
      [[ "$supplied_config" == "$config_path" && "$config_backend" == "$backend" ]] || federation_fail "runner/config hors installation attestée"
      federation_validate_install
      dry_run=false
      federation_run recover
      ;;
    *) federation_fail "commande inconnue ; voir --help" ;;
  esac
}

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then federation_main "$@"; fi
