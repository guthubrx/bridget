#!/usr/bin/env bash
set -euo pipefail
script_dir="$(cd "$(dirname "$0")" && pwd -P)"
bash -n "$script_dir/federate-ssh.sh"
bash -n "$script_dir/deploy-remote.sh"
# Les anciens grep validaient l'écriture du fichier de configuration global,
# retirée en 089. Les oracles vivent désormais dans le banc sans réseau :
# BRIDGET_CHANNEL explicite, paramètres fermés, anciennes actions refusées,
# dry-run sans effets et aucun écrasement de socket.
printf '%s\n' 'Recette 089 : doublures SSH/rsync uniquement, aucune connexion réelle.' >&2
exec bash "$script_dir/tests/federation_089_test.sh"
