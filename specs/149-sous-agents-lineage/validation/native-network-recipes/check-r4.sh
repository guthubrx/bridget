#!/bin/zsh
# Contrôle début/fin de ronde r4 : SHA-256 du binaire release r9 + empreinte de production (lecture seule).
umask 077
BIN=/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-abc850858975
WANT_BIN=abc850858975fb5c7d733eb052be4fff6e3cb24537ce8b7d4475019b79abf6d8
WANT_PROD=b2b87458cf3cec7989debeb91352b838c417c3eb48329d57099bec4c9a0d5f29
V=/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation
got_bin=$(shasum -a 256 "$BIN" | cut -d' ' -f1)
got_prod=$(python3 -I "$V/native-r5-fingerprint.py" | python3 -I -c 'import json,sys;print(json.load(sys.stdin)["prod"]["value"])')
echo "$(date +%H:%M:%S) $1 bin=$([ "$got_bin" = "$WANT_BIN" ] && echo OK || echo DIFF:$got_bin) prod=$([ "$got_prod" = "$WANT_PROD" ] && echo OK || echo DIFF:$got_prod)"
[ "$got_bin" = "$WANT_BIN" ]
