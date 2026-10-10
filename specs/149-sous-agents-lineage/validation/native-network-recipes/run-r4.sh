#!/bin/zsh
# Usage : run-r4.sh <recette.ts> ; vérifie SHA/empreinte avant et après, journalise dans results-r4/.
umask 077
export PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:/usr/bin:/bin:/usr/sbin:/sbin
export NATIVE149_BIN=/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-abc850858975
export NATIVE149_BIN_SHA256=abc850858975fb5c7d733eb052be4fff6e3cb24537ce8b7d4475019b79abf6d8
export NATIVE149_RESULTS=results-r4
cd "${0:A:h}" || exit 2
name=${1:r}
./check-r4.sh "début $name" | tee -a results-r4/shacheck.log || exit 3
node "$1" > "results-r4/$name.stdout.txt" 2>&1
rc=$?
./check-r4.sh "fin $name" | tee -a results-r4/shacheck.log
echo "rc=$rc"
tail -3 "results-r4/$name.stdout.txt"
exit $rc
