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

git merge-base --is-ancestor "$merge" HEAD || {
    echo "G-1600 invalide : le merge 015 $merge n'est pas ancêtre de HEAD" >&2
    exit 1
}

git show "$merge:$contract" | grep -Fq '**Statut** : gelé pour G1501' || {
    echo "G-1600 invalide : contrat 015 non gelé au merge" >&2
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

echo "G-1600 vérifié : amont 015 et fixture publique épinglés"
