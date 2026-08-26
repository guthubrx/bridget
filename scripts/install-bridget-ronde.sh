#!/usr/bin/env bash
# Pose optionnellement la ronde portable comme unité utilisateur, lecture seule.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: install-bridget-ronde.sh --config CHEMIN_ABSOLU [options]

Active une release admise et, sauf --skip-activate, une unité périodique.
Cette unité archive des rapports locaux ; elle n'envoie rien et ne décide rien.

Lancer depuis le checkout principal, branche main propre, après
`git fetch origin`, le jury et le merge.

Options:
  --config CHEMIN_ABSOLU       configuration Maicie (obligatoire)
  --report-dir CHEMIN_ABSOLU   archives (défaut: ~/.cache/bridget/rondes)
  --interval-seconds N         période >= 60 s (défaut: 420)
  --force                      remplace explicitement les artefacts existants
  --skip-activate              pose l'unité sans l'activer
EOF
}

config=""
report_dir="${HOME}/.cache/bridget/rondes"
interval=420
force=0
skip_activate=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --config) config=${2:-}; shift 2 ;;
    --report-dir) report_dir=${2:-}; shift 2 ;;
    --interval-seconds) interval=${2:-}; shift 2 ;;
    --force) force=1; shift ;;
    --skip-activate) skip_activate=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "option inconnue: $1" >&2; usage >&2; exit 2 ;;
  esac
done
[[ "$config" == /* ]] || { echo "--config doit être un chemin absolu" >&2; exit 2; }
[[ "$report_dir" == /* ]] || { echo "--report-dir doit être un chemin absolu" >&2; exit 2; }
[[ "$interval" =~ ^[0-9]+$ && "$interval" -ge 60 ]] || { echo "--interval-seconds doit être un entier >= 60" >&2; exit 2; }

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# La politique doit s'exécuter avant toute création de rapport ou d'unité :
# un refus n'a ainsi aucun effet et ne peut jamais être annoncé « prêt ».
# shellcheck source=scripts/lib/pilotage-release.sh
source "${root_dir}/scripts/lib/pilotage-release.sh"
pilotage_install_release "$root_dir" "scripts/bridget-ronde.py" "bridget-ronde" "$force"
installed_command="$PILOTAGE_INSTALLED_COMMAND"

write_unit() {
  local path="$1" label="$2" directory prepared expected_hash actual_mode actual_hash
  directory="$(dirname "$path")"
  mkdir -p "$directory"
  prepared="$(mktemp "${directory}/.prepare-$(basename "$path").XXXXXX")"
  cat >"$prepared"
  chmod 0644 "$prepared"
  expected_hash="$(pilotage_release_hash "$prepared")"

  if [[ -e "$path" || -L "$path" ]]; then
    actual_mode="$(pilotage_file_mode "$path" 2>/dev/null || true)"
    if [[ -f "$path" && ! -L "$path" && "$actual_mode" == 644 ]] \
      && cmp -s "$path" "$prepared"; then
      rm "$prepared"
      echo "déjà en place: $label ($path)" >&2
      return 0
    fi
    if [[ "$force" != 1 ]]; then
      rm "$prepared"
      pilotage_refuse "configuration différente: $label ($path); utiliser --force pour remplacer"
      return 1
    fi
  fi

  if ! pilotage_replace_entry "$prepared" "$path"; then
    rm -f "$prepared"
    pilotage_refuse "écriture atomique impossible: $label ($path)"
    return 1
  fi
  actual_mode="$(pilotage_file_mode "$path" 2>/dev/null || true)"
  actual_hash="$(pilotage_release_hash "$path" 2>/dev/null || true)"
  if [[ ! -f "$path" || -L "$path" || "$actual_mode" != 644 || "$actual_hash" != "$expected_hash" ]]; then
    pilotage_refuse "attestation unité invalide: $label ($path)"
    return 1
  fi
  echo "posé: $label ($path)" >&2
}

mkdir -p "$report_dir"
chmod 0700 "$report_dir"

case "$(uname -s)" in
  Darwin)
    unit="${HOME}/Library/LaunchAgents/com.bridget.ronde.plist"
    write_unit "$unit" "unité launchd ronde" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>com.bridget.ronde</string>
  <key>ProgramArguments</key><array>
    <string>/usr/bin/env</string><string>python3</string><string>${installed_command}</string>
    <string>--config</string><string>${config}</string>
    <string>--report-dir</string><string>${report_dir}</string>
  </array>
  <key>StartInterval</key><integer>${interval}</integer>
  <key>RunAtLoad</key><true/>
  <key>EnvironmentVariables</key><dict>
    <key>HOME</key><string>${HOME}</string>
    <key>PATH</key><string>${HOME}/.local/bin:/usr/bin:/bin</string>
  </dict>
  <key>StandardOutPath</key><string>${report_dir}/service-stdout.log</string>
  <key>StandardErrorPath</key><string>${report_dir}/service-stderr.log</string>
</dict></plist>
EOF
    if [[ "$skip_activate" != 1 ]]; then
      launchctl bootstrap "gui/$(id -u)" "$unit" 2>/dev/null || launchctl kickstart -k "gui/$(id -u)/com.bridget.ronde"
      echo "activé: com.bridget.ronde" >&2
    fi
    ;;
  Linux)
    service="${HOME}/.config/systemd/user/bridget-ronde.service"
    timer="${HOME}/.config/systemd/user/bridget-ronde.timer"
    write_unit "$service" "unité systemd ronde" <<EOF
[Unit]
Description=Bridget ronde (lecture seule)

[Service]
Type=oneshot
ExecStart=/usr/bin/env python3 ${installed_command} --config ${config} --report-dir ${report_dir}
Environment=HOME=${HOME}
Environment=PATH=${HOME}/.local/bin:/usr/bin:/bin
EOF
    write_unit "$timer" "timer systemd ronde" <<EOF
[Unit]
Description=Timer Bridget ronde (lecture seule)

[Timer]
OnBootSec=30
OnUnitActiveSec=${interval}
AccuracySec=15
Unit=bridget-ronde.service

[Install]
WantedBy=timers.target
EOF
    if [[ "$skip_activate" != 1 ]]; then
      systemctl --user daemon-reload
      systemctl --user enable --now bridget-ronde.timer
      echo "activé: bridget-ronde.timer" >&2
    fi
    ;;
  *) echo "plateforme non couverte: $(uname -s) (Darwin/Linux seulement)" >&2; exit 2 ;;
esac

echo "ronde portable prête: ${installed_command} --config ${config}" >&2
