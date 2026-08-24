#!/usr/bin/env bash
set -euo pipefail

script="$(cd "$(dirname "$0")" && pwd)/open-remote-ui.sh"

bash -n "$script"
if "$script" user@host --maicie-config relative 2>/dev/null; then exit 1; fi
if "$script" user@host --maicie-config /tmp/config --local-port 0 2>/dev/null; then exit 1; fi
if "$script" --maicie-config /tmp/config 2>/dev/null; then exit 1; fi

# Le lanceur ne peut ouvrir qu'un forward TCP local et démarrer la projection
# `bridget ui`; aucune sous-commande d'action distante n'est présente.
rg -q -- '-L "127\.0\.0\.1:' "$script"
rg -q 'exec %q ui --maicie-config %q' "$script"
if rg -q -- '(^|[^[:alpha:]])(spawn|stop|approve|cancel)([^[:alpha:]]|$)|-R' "$script"; then
    echo "le tunnel de lecture ne doit contenir aucune action ni forward inverse" >&2
    exit 1
fi

echo "lanceur UI distant : syntaxe, garde loopback et absence d'action vérifiées"
