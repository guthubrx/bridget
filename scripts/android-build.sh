#!/usr/bin/env bash
set -euo pipefail
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
export JAVA_HOME="${JAVA_HOME:-$HOME/.local/share/bridget/toolchains/jdk-17.0.21+9}"
if [[ ! -x "$JAVA_HOME/bin/java" ]]; then
  echo "JDK 17.0.21+ requis : définir JAVA_HOME vers un JDK corrigé." >&2
  exit 2
fi
version=$("$JAVA_HOME/bin/java" -XshowSettings:properties -version 2>&1 | sed -n 's/^[[:space:]]*java.version = //p')
if [[ ! "$version" =~ ^17\.0\.([0-9]+) ]] || (( BASH_REMATCH[1] < 21 )); then
  echo "JDK 17.0.21+ requis ; version détectée : $version" >&2
  exit 2
fi
cd "$repo/android"
exec ./gradlew --no-daemon "$@"
