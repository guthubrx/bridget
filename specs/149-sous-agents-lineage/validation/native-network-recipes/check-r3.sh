#!/bin/zsh
# Contrôle début/fin de ronde r3 : SHA-256 du binaire release r8 + empreinte de production (lecture seule).
umask 077
BIN=/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb
WANT_BIN=0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6
WANT_PROD=3943009ca82db59d850d913b55b18c7a0a17c91c3143ffaeb39ee6cdd600526d
V=/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation
got_bin=$(shasum -a 256 "$BIN" | cut -d' ' -f1)
got_prod=$(python3 -I "$V/native-r5-fingerprint.py" | python3 -I -c 'import json,sys;print(json.load(sys.stdin)["prod"]["value"])')
echo "$(date +%H:%M:%S) $1 bin=$([ "$got_bin" = "$WANT_BIN" ] && echo OK || echo DIFF:$got_bin) prod=$([ "$got_prod" = "$WANT_PROD" ] && echo OK || echo DIFF:$got_prod)"
# Le binaire immuable est la référence : seule sa dérive est fatale. Une source modifiée APRÈS la compilation (autre propriétaire) est journalisée.
[ "$got_bin" = "$WANT_BIN" ]
