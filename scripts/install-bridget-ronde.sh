#!/usr/bin/env bash
# Pose optionnellement la ronde portable comme unité utilisateur, lecture seule.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: install-bridget-ronde.sh --config CHEMIN_ABSOLU [options]

Pose la commande versionnée et, sauf --skip-activate, une unité périodique.
Cette unité archive des rapports locaux ; elle n'envoie rien et ne décide rien.

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
source_command="${root_dir}/scripts/bridget-ronde.py"
installed_command="${HOME}/.local/bin/bridget-ronde"
[[ -f "$source_command" ]] || { echo "commande source absente: $source_command" >&2; exit 1; }

write_unit() {
  local path="$1" label="$2"
  if [[ -e "$path" && "$force" != 1 ]]; then
    echo "déjà en place: $label ($path)" >&2
    return 1
  fi
  mkdir -p "$(dirname "$path")"
  cat >"$path"
  chmod 0644 "$path"
  echo "posé: $label ($path)" >&2
}

mkdir -p "${HOME}/.local/bin" "$report_dir"
chmod 0700 "$report_dir"
# Lien, pas copie : une copie ~/.local/bin échappe à la revue et au jury
# (constat 2026-08-25 — mêmes outils de pilotage que bridget-idle).
if [[ -L "$installed_command" && "$(readlink "$installed_command")" == "$source_command" ]]; then
  echo "déjà en place: commande ($installed_command -> $source_command)" >&2
elif [[ ! -e "$installed_command" || "$force" == 1 ]]; then
  rm -f "$installed_command"
  ln -sfn "$source_command" "$installed_command"
  echo "posé: commande ($installed_command -> $source_command)" >&2
else
  echo "déjà en place: commande ($installed_command) (passer --force pour symlink versionné)" >&2
fi

case "$(uname -s)" in
  Darwin)
    unit="${HOME}/Library/LaunchAgents/com.bridget.ronde.plist"
    write_unit "$unit" "unité launchd ronde" <<EOF || true
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
    write_unit "$service" "unité systemd ronde" <<EOF || true
[Unit]
Description=Bridget ronde (lecture seule)

[Service]
Type=oneshot
ExecStart=/usr/bin/env python3 ${installed_command} --config ${config} --report-dir ${report_dir}
Environment=HOME=${HOME}
Environment=PATH=${HOME}/.local/bin:/usr/bin:/bin
EOF
    write_unit "$timer" "timer systemd ronde" <<EOF || true
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
