#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
if ! command -v gitleaks >/dev/null 2>&1; then
  printf '%s\n' 'Install Gitleaks 8.30.1, then rerun this command.' >&2
  exit 2
fi
if [[ "$(git rev-parse --is-shallow-repository)" == true ]]; then
  printf '%s\n' 'Refusing an incomplete history scan: fetch the full history first.' >&2
  exit 2
fi
# Exact historical fingerprints only. New commits still produce new findings.
gitleaks git "$repo_root" --log-opts=--all --redact=100 --no-banner --no-color \
  --ignore-gitleaks-allow --max-archive-depth=2 \
  --gitleaks-ignore-path "$repo_root/.gitleaksignore"
