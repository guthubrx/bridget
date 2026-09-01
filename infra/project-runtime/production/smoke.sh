#!/usr/bin/env bash
set -euo pipefail

required=(bash git rg jq curl cargo node pnpm python3 /usr/local/bin/bridget)
for binary in "${required[@]}"; do
  command -v "$binary" >/dev/null
done

test "$(id -u)" = "1002"
test -n "${HOME:-}"
test -d "$HOME"
test ! -e /var/run/docker.sock

git --version
cargo --version
/usr/local/bin/bridget --help >/dev/null
node --version
pnpm --version
python3 --version
