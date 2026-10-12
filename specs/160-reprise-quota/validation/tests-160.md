# Tests160 — preuves ciblées et régressions

Clôture d’implémentation préparée 37/37 après validation ROOT de T030/T031.
Audit final, Converge final et livraison restent des gates distinctes.
Aucun verdict Converge, audit ou livraison n’est déduit de ces résultats.

| Exécution ROOT | Résultat réel | Durée / RC |
|---|---|---|
| Baseline T001, sept filtres prescrits | 276PASS, 0FAIL,2ignored | 208.39s / RC0 |
| Premier RED | Compilation seulement, aucune assertion | 5.25s / RC101 |
| RED compilé avant fondations | 0PASS,11FAIL réels | 14.43s / RC101 |
| RED base avec protocole legacy | 0PASS,11FAIL réels | 9.70s / RC101 |
| T037 oracle avis corrigé | 0PASS,1FAIL réel | 14.75s / RC101 |
| Après fondations, constructeurs corrigés | 13PASS,10FAIL réels | 67.96s / RC101 |
| Protocol/MCP ciblé | 4PASS, 0FAIL | 20.53s / RC0 |
| Intégration des filtres160 | 55PASS,4FAIL | 123.90s / RC101 |
| Quatre oracles après diagnostic de fixture | 4PASS, 0FAIL | 84.68s / RC0 |
| T027 mêmes sept filtres de baseline | 276PASS, 0FAIL,2ignored | 176.61s / RC0 |

Les 59 noms ciblés distincts passent par cumul : 55 succès de l’intégration
et quatre replays exacts. Ce n’est pas une exécution unique 59PASS/RC0.
Les quatre FAIL restent historiques dans leur log. Les premiers essais non
compilables ne sont pas présentés comme des assertions métier rouges.
Les quatre tests Protocol/MCP séparés recouvrent l’intégration : ne pas les
additionner comme quatre tests distincts supplémentaires.

Quatre corrections de montage, selon le reçu ROOT-T027-and-four-oracle-fixes :
annulation/hold, purge après annulation, remise/résultat même mission et crash
sur l’avis durable. ROOT atteste aucune modification de logique production ;
le fichier natif contient aussi les fixtures. Le JSON garde noms exacts,
empreintes des logs, résumés et gardes de sources pour chaque exécution.
La comparaison T027 donne les mêmes 276 noms PASS et 2 ignored que T001, sans manque
ni changement de statut. Au checkpoint ciblé, les sources du replay étaient identiques au WT.
Ce constat est historique ; la capture finale21 ci-dessous inclut les deux
adaptations d’oracles post-FULL, sans modification de production.

Logs principaux :
/Users/moi/.cache/bridget-tests160-20261012/T003-T026-T032-T037-native160-integrated-compile-fixes.log
SHA e6f9df46562bdca8d0e7afb83b5e4e709986e158070203ce64963675f2a998ab
/Users/moi/.cache/bridget-tests160-20261012/T015-T020-T024-four-runtime-oracles-replay-after-diagnosis.log
SHA 292137eddab69e27f9c2babe0685ae60b45ba185f92ef251de4b158c26a51bd1
/Users/moi/.cache/bridget-tests160-20261012/T027-baseline-seven-prescribed-regressions-after-core.log
SHA 701091309dea9dec34784e2c9b5d7b56a78a7746848787eb2ce075cbc5bbe545

Limites : quota réel non observé en T002, aucun fournisseur appelé ici.
Les fixtures ne prouvent pas la causalité quota/échec ni l’unicité des effets
externes. La résolution d’un hold peut être retardée au tick ; seule sa fenêtre
d’observation est bornée. Le checkpoint ci-dessus précédait T030. Le FULL historique terminé et ses
replays sont consignés ci-dessous, sans GREEN global.
Manifestes et tableau des noms :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/160-reprise-quota/validation/tests-160.json


## T030 — FULL unique et six replays exacts

| Exécution | Résultat réel | Durée / RC |
|---|---|---|
| FULL unique, 97 cibles parent | 2683 PASS, 6 FAIL, 76 ignored | 900.75s / RC101 |
| Oracle 154 existant adapté quota_wait:null | 1 PASS, 0 FAIL | 7.88s / RC0 |
| Cinq oracles existants adaptés au schéma 12 | 5 PASS, 0 FAIL | 4.99s / RC0 |

Les sept PASS enfants embarqués sont exclus du compte Cargo parent. Le log
FULL garde ses six échecs. Le résultat des replays 1+5 clôt la disposition T030,
pas le code de retour de cette suite. Aucune seconde suite ni GREEN global.
Les 19 sources précédentes restent inchangées. Les deux deltas de fichiers
tests adaptent uniquement quota_wait:null et schéma 11→12, prévu par T011.
La capture finale 21 et les sources des replays sont tracées dans le JSON.
Aucun quota réel ou effet fournisseur unique n’est démontré.

FULL log :
/Users/moi/.cache/bridget-tests160-20261012/T030-FULL160-once-final-source.log
SHA c0702cc9f7ac8e35ad3b7dfb45e97ebf97827654eea96987db07e91fcb3b390d
Replay 154 :
/Users/moi/.cache/bridget-tests160-20261012/T030-six-oracles-native154-exact-replay.log
SHA 2914173fd0732e4c2dc0fb1dc0c4daeb37b24cff7e3e4f4100bcf4bb1f6cb889
Replay cinq oracles :
/Users/moi/.cache/bridget-tests160-20261012/T030-five-existing-migration12-oracles-exact-replay.log
SHA e4b17b1cc8eb08bbdc70a0333bccac318f2bfb26e5efada1bdcdcdb73874da31
