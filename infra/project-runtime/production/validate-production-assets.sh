#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dockerfile="$root/Dockerfile"
lockfile="$root/toolchain.lock"
runtime_policy="$root/config/project-runtime-policy.example.json"
resource_catalog="$root/config/project-resource-catalog.example.json"

grep -Eq '^FROM .+@sha256:[a-f0-9]{64}$' "$dockerfile"
grep -Fq 'USER bridget' "$dockerfile"
grep -Fq 'TARGETARCH' "$dockerfile"
grep -Fq 'target/release/bridget /usr/local/bin/bridget' "$dockerfile"
grep -Fq 'COPY Cargo.toml Cargo.lock /opt/bridget-src/' "$dockerfile"
grep -Fq 'linux/amd64' "$lockfile"
grep -Fq 'provider_clients=none' "$lockfile"
grep -Fq 'secrets=none' "$lockfile"
! grep -Eqi '(api[_-]?key|token|password|secret)[[:space:]]*[:=][[:space:]]*[^[:space:]]+' "$root"/*.md "$root"/*.lock "$root"/*.sh "$root"/Dockerfile "$root"/config/*.json
jq -e '
  .contract_version == 1
  and (.policies | length == 1)
  and .policies[0].image_reference_kind == "local_image_id"
  and (.policies[0].image_reference | test("^sha256:[a-f0-9]{64}$"))
  and .policies[0].run_as_uid == 1002
  and .policies[0].runtime_launcher == "/usr/local/bin/bridget"
  and .policies[0].runtime_executables == []
' "$runtime_policy" >/dev/null
jq -e '
  .contract_version == 1
  and .sources == []
  and (.extension_roots | all(startswith("/")))
  and (.secret_roots | all(startswith("/")))
' "$resource_catalog" >/dev/null
