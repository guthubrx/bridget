#!/usr/bin/env bash
set -euo pipefail
script="$(cd "$(dirname "$0")" && pwd)/federate-ssh.sh"
bash -n "$script"
bash -n "$(dirname "$script")/deploy-remote.sh"
grep -q "channel=ssh-unix" "$script"
grep -q "transport=ssh-unix" "$script"
if "$script" install 'mauvais nom' --host example.invalid 2>/dev/null; then exit 1; fi
if "$script" invalid test --host example.invalid 2>/dev/null; then exit 1; fi
echo "tests de syntaxe et d'usage réussis"
