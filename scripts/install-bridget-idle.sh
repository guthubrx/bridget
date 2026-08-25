#!/usr/bin/env bash
# Pose ~/.local/bin/bridget-idle comme LIEN vers le script versionné (jamais une copie).
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: install-bridget-idle.sh [--force]

Remplace ~/.local/bin/bridget-idle par un symlink vers scripts/bridget-idle.py
du dépôt (worktree ou clone) d'où cette commande est lancée. Une copie locale
dissocie l'outil de pilotage de la revue et du jury — c'est ce qu'on ferme ici.
EOF
}

force=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --force) force=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "option inconnue: $1" >&2; usage >&2; exit 2 ;;
  esac
done

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_command="${root_dir}/scripts/bridget-idle.py"
installed_command="${HOME}/.local/bin/bridget-idle"
[[ -f "$source_command" ]] || { echo "commande source absente: $source_command" >&2; exit 1; }
mkdir -p "${HOME}/.local/bin"

if [[ -e "$installed_command" || -L "$installed_command" ]]; then
  if [[ -L "$installed_command" && "$(readlink "$installed_command")" == "$source_command" ]]; then
    echo "déjà en place: $installed_command -> $source_command" >&2
    exit 0
  fi
  if [[ "$force" != 1 ]]; then
    echo "déjà en place: $installed_command (passer --force pour remplacer par un symlink)" >&2
    exit 1
  fi
  rm -f "$installed_command"
fi

ln -sfn "$source_command" "$installed_command"
echo "posé: $installed_command -> $source_command" >&2
