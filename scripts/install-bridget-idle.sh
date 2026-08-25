#!/usr/bin/env bash
# Active une release admise de bridget-idle, indépendante de tout worktree.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: install-bridget-idle.sh [--force]

Active dans ~/.local/bin/bridget-idle une release extraite d'un commit admis
sur origin/main. Lancer depuis le checkout principal, branche main propre,
après `git fetch origin`, le jury et le merge.
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
# shellcheck source=scripts/lib/pilotage-release.sh
source "${root_dir}/scripts/lib/pilotage-release.sh"
pilotage_install_release "$root_dir" "scripts/bridget-idle.py" "bridget-idle" "$force"
