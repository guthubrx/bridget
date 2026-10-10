#!/bin/zsh
# Usage : run-r3.sh <recette.ts> ; vérifie SHA/empreinte avant et après, journalise dans results-r3/.
umask 077
export PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:/usr/bin:/bin:/usr/sbin:/sbin
cd "${0:A:h}" || exit 2
name=${1:r}
./check-r3.sh "début $name" | tee -a results-r3/shacheck.log || exit 3
node "$1" > "results-r3/$name.stdout.txt" 2>&1
rc=$?
./check-r3.sh "fin $name" | tee -a results-r3/shacheck.log
echo "rc=$rc"
tail -3 "results-r3/$name.stdout.txt"
exit $rc
