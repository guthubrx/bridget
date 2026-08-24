#!/bin/sh
# Vérifie la base gelée 015 avant toute extension du protocole 016.
set -eu

root=$(git rev-parse --show-toplevel)
cd "$root"

merge=0b2b83e807c6055c0014d5fa21b0efbfe8fd6311
contract=specs/015-guichet-maicie/contracts/protocole-guichet.md
source_fixture=specs/015-guichet-maicie/contracts/fixtures/service-negotiation-v1.jsonl
pinned_fixture=specs/016-coordination-active/contracts/fixtures/service-negotiation-v1.jsonl
expected_sha=494bc21a9eacb47296ab9ef0b22d6b830060c77b6d5c8cbc5ef0bb7534244303
normative_fixture=specs/016-coordination-active/contracts/fixtures/service-frames-v1.jsonl
normative_sha=42e0ea32690e267221804a8af9aee538162f7c141a9cffacf6d6a44a9a8cda1f
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT HUP INT TERM
normative_source="$tmpdir/service-frames-v1.jsonl"

git merge-base --is-ancestor "$merge" HEAD || {
    echo "G-1600 invalide : le merge 015 $merge n'est pas ancêtre de HEAD" >&2
    exit 1
}

git show "$merge:$contract" | grep -Fq '**Statut** : gelé pour G1501' || {
    echo "G-1600 invalide : contrat 015 non gelé au merge" >&2
    exit 1
}

git show "$merge:$contract" | awk '
    /^### 5\.1 Trames valides$/ { section = 1; next }
    section && /^```json$/ { block = 1; next }
    block && /^```$/ { exit }
    block { print }
' > "$normative_source"

test "$(wc -l < "$normative_source" | tr -d ' ')" = 7 || {
    echo "G-1600 invalide : impossible d'extraire les sept trames normatives" >&2
    exit 1
}

for fixture in "$source_fixture" "$pinned_fixture"; do
    actual=$(shasum -a 256 "$fixture" | awk '{print $1}')
    test "$actual" = "$expected_sha" || {
        echo "fixture divergente : $fixture" >&2
        exit 1
    }
done

cmp -s "$source_fixture" "$pinned_fixture" || {
    echo "fixture 015 et copie 016 différentes" >&2
    exit 1
}

verify_normative_fixture() {
    fixture=$1
    actual=$(shasum -a 256 "$fixture" | awk '{print $1}')
    test "$actual" = "$normative_sha" || {
        echo "corpus normatif divergent : $fixture" >&2
        return 1
    }
    cmp -s "$normative_source" "$fixture" || {
        echo "corpus normatif et objet Git épinglé différents" >&2
        return 1
    }
}

if [ "${1:-}" = "--self-test" ]; then
    mutated="$tmpdir/service-frames-v1-mutated.jsonl"
    sed '1s/{/[/' "$normative_fixture" > "$mutated"
    if verify_normative_fixture "$mutated"; then
        echo "auto-test invalide : la mutation d'un octet a été acceptée" >&2
        exit 1
    fi
    echo "auto-test vérifié : la mutation d'un octet est refusée"
    exit 0
fi

verify_normative_fixture "$normative_fixture"

echo "G-1600 vérifié : amont 015 et corpus filaire normatif épinglés"
