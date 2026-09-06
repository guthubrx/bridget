#!/usr/bin/env bash
# Sources autonomes : aucune installation, réseau ou flotte de production.
set -euo pipefail
umask 077
source_root=$(cd "$(dirname "$0")/.." && pwd -P)
if [[ $# != 1 || "$1" != /* || -e "$1" || -L "$1" ]]; then
  echo 'usage: package-089-core.sh /chemin/absolu/nouveau-paquet (doit être absent)' >&2
  exit 2
fi
package_root=$1
# Refuser les chemins indirects, y compris un parent symbolique. Aucun mkdir
# avant la validation complète : un refus ne laisse pas de paquet partiel.
case "$package_root/" in *'/../'*|*'/./'*|*'//'*) echo 'refus : chemin non normalisé' >&2; exit 2 ;; esac
parent=${package_root%/*}
[[ -d "$parent" ]] || { echo 'refus : parent absent' >&2; exit 2; }
while [[ -n "$parent" && "$parent" != / ]]; do
  [[ ! -L "$parent" ]] || { echo 'refus : parent symbolique' >&2; exit 2; }
  parent=${parent%/*}
done
# Liste fermée : les fixtures documentaires sont des entrées de tests, pas
# une dépendance au produit historique. Ni checkout complet ni features cachées.
entries=(
  Cargo.toml Cargo.lock rust-toolchain.toml LICENSE README.md README.en.md
  crates skills/bridget
  docs/communication-installation.md
  scripts/package-089-core.sh scripts/federate-ssh.sh scripts/deploy-remote.sh
  scripts/tests/federation_089_test.sh scripts/tests/package_089_test.sh
  specs/089-communication-core
  specs/090-codex-interactif
  specs/015-guichet-maicie/contracts/fixtures
  specs/016-coordination-active/contracts/fixtures
  specs/010-mcp/spike/fake-mcp-server.py
  specs/026-operations-greffe-central/contracts/greffe-authorization.example.json
  specs/038-regeneration-politique/contracts/greffe-authorization-refresh.example.json
)
for entry in "${entries[@]}"; do
  # find ne voit pas un lien dans les ancêtres de son point de départ.
  source_parent="$source_root/$entry"
  source_parent=${source_parent%/*}
  while [[ "$source_parent" != "$source_root" ]]; do
    [[ ! -L "$source_parent" ]] || { echo "refus : parent source symbolique : $entry" >&2; exit 2; }
    source_parent=${source_parent%/*}
  done
  [[ -e "$source_root/$entry" ]] || { echo "refus : source absente : $entry" >&2; exit 2; }
  [[ ! -L "$source_root/$entry" && -z $(find "$source_root/$entry" -type l -print -quit) ]] || {
    echo "refus : source symbolique : $entry" >&2; exit 2;
  }
done
build_id=unknown
if git -C "$source_root" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  build_id=$(git -C "$source_root" rev-parse --short=12 HEAD)
  [[ -z $(git -C "$source_root" status --porcelain) ]] || build_id+="-dirty"
elif [[ -f "$source_root/BUILD_ID" && ! -L "$source_root/BUILD_ID" ]]; then
  read -r build_id < "$source_root/BUILD_ID"
fi
mkdir "$package_root"
for entry in "${entries[@]}"; do
  mkdir -p "$(dirname "$package_root/$entry")"
  cp -R "$source_root/$entry" "$package_root/$entry"
done
printf '%s\n' "$build_id" > "$package_root/BUILD_ID"
# Empreintes du contenu réellement livré (le build-id seul n'identifie pas les
# changements non committés). Vérification possible sans dépôt Git ni réseau.
(cd "$package_root" && find . -type f ! -path ./SOURCE-MANIFEST.sha256 -exec shasum -a 256 {} \; | LC_ALL=C sort > SOURCE-MANIFEST.sha256)
printf '%s\n' "$package_root"
