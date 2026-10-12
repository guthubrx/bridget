# Implémentation160 — préparation de clôture ciblée

État : Implemented, 37/37 après validation ROOT de T030 et T031.
Audit final, Converge final et livraison restent des gates distinctes. Le diff produit
a été couvert par les relectures attribuées ci-dessous. Cette couverture ne
certifie pas une lecture humaine intégrale de tous les anciens fichiers.
Aucune activation, livraison ou clôture globale du pipeline n’est déduite.

Le contrat ajoute resume_on_limit booléen strict au MCP, sans changement
canonique lorsque le champ est absent. Absent et false sont deux enveloppes
idempotentes distinctes. La tâche reste working ; quota_wait décrit l’attente.
La qualification exige une fenêtre épuisée attestée et un retour futur dans 6h.
Deux rondes sont conservées durablement. L’observation du hold finit après 5s ;
le tick utilise son heure réelle, même retardé. Une attente relance la même
mission après retour+30, sur le même fournisseur/modèle/effort/définition et
sans nouveau fallback ni droits. Le solde exige un ACK. La chaîne reste
réservée et ne consomme pas les reprises après panne 152.

Les magasins existants sont réemployés : Task porte les attentes et le hold ;
ExecutionStore qualifie l’échec et relie la continuation à son identité.
L’ordre qualification puis TaskWait, puis relaunching/reconstruction/remise/ACK
permet le rejeu aux frontières prescrites. La table de qualification et les
champs optionnels sont justifiés par les exigences d’entrée, pas un moteur
supplémentaire. Le journal des pauses, garde NativeMissionDispatch, remise
idempotente, avis durable et erreurs existantes sont conservés.

T012 extrait pause_intervals du calcul 152, garde son ancien oracle inchangé,
puis calcule l’union pause/quota, sans écriture. Coût SQL O(logN+P), union
O((P+Q)log(P+Q)), Q<=2. Le gel porte sur l’enfant seul et finit au retour+30
pour sa part quota. Le wrapper capte la liaison avant transition terminale.
Un refus ancien est ignoré ; un refus sans corrélation après une relance
échoue explicitement. Annulation et échec relaunching réconcilient feuille et
remise avant solde ; une vérification impossible reste explicite.

La préconvergence 25FR/13SC/5US mappe les parcours avec sources/plages/testnames.
C’est une lecture ciblée attribuée, pas une lecture intégrale de tous les corps
des anciens fichiers. La relecture T028 contrôle parser MCP,
projection, qualification, ordre du hold, ACK, garde pause, restart et rappels.
Les 59 tests ciblés passent cumulativement, T027 276PASS/2ignored est identique
à la baseline. Les quatre FAIL d’intégration sont conservés et leurs replays
réussis distincts. Voir validation/tests-160.md/json pour les noms et SHA.

Limites Articles XIX/XX : aucun nouveau service/framework/API de contrôle, reprise
152 distincte et aucune abstraction ajoutée hors tâches. Les extractions ont
les usages prévus. Les relectures T031 ne démontrent aucune ligne suppressible ni défaut bloquant
dans leur périmètre. Ce constat n’affirme pas la minimalité de tout le legacy.
T002 n’a observé aucun quota réel. La mémoire 157 reste volatile. Le quota et
l’échec sont corrélés temporellement, sans causalité prouvée ; répéter la mission
peut répéter des effets externes. ACK/idempotence ne donnent pas exactement
une exécution des effets métier. Revue/audit/Converge/livraison restent à établir.

Provenance ciblée :
/Users/moi/.cache/bridget-converge160-preparation-20261012/preconverge160.json
/Users/moi/.cache/bridget-docs160-T028-final-preparation-20261012/manifest.json
/Users/moi/.cache/bridget-tests160-20261012/ROOT-T027-and-four-oracle-fixes.json


## T031 — couverture du diff et correction ciblée

ROOT a relu la production et les documents. Son arbitrage valide T031 par
couverture du diff, qualification du formatage et lecture fraîche du dernier
delta. Deux reçus complètent cette relecture :

- Natif : 26/26 hunks comptabilisés, 19 de production relus ; nouveaux oracles
  relus et sept hunks de tests examinés par leurs deltas métier. Les 213
  intervalles anciens qualifiés mécaniquement conservent littéraux et
  commentaires. Ce n’est pas une lecture humaine des 9991 lignes de diff.
- Protocole/autres sources : 18 fichiers, 161 hunks lus intégralement ; les
  longues sorties ont été complétées. Les corps historiques hors hunks ne
  sont pas tous relus. L’auto-revue d’un owner reste distincte de la revue adverse.

La capture finale comprend 19 sources. La relecture native initiale portait
sur c7b0a8aa ; ROOT a ensuite relu la correction FR006b avant clôture : un seul
champ de corrélation fourni ne compte pas comme une absence complète.
Le tableau T035 existant est rejoué : 1 PASS, 0 FAIL, RC0, 15.09s, sources stables.
Il ne s’ajoute pas comme un soixantième nom ciblé distinct. Source native finale :
c7b7583eb6e40ef99e392b15eacc9bc3b6942e7b890fe75a4cdb7344cec6b174.
Log T035 : d019e1c93417fbc53e7d73cccc0c54a192afdf6a74762db8a605e20789988797.

L’ADR précise R1 : indeterminate est absorbant ; quarantaine d’échec/annulation
et solde ACK normal sont distincts. Un defer classé Storage conserve le
checkpoint ; une corruption d’identifiants peut bloquer cette tâche sans
clôturer une autre exécution. Les effets externes engagés ne sont pas annulés
ni rendus uniques par cette reprise. RevueR2 et audit final restent en cours ;
aucun résultat futur n’est crédité. Au checkpoint T031, le FULL n’était pas encore lancé ; ce point est supersédé
par le reçu T030 ci-dessous.

Reçus de couverture et capture :
/Users/moi/.cache/bridget-selfreview160-preparation-20261012/native/native-selfreview.md
/Users/moi/.cache/bridget-selfreview160-preparation-20261012/native/read-manifest.json
/Users/moi/.cache/bridget-selfreview160-preparation-20261012/native/format-qualification.json
/Users/moi/.cache/bridget-selfreview160-preparation-20261012/protocol-other/selfreview160.md
/Users/moi/.cache/bridget-selfreview160-preparation-20261012/protocol-other/selfreview160.json
/Users/moi/.cache/bridget-review160-preparation-20261012/scope-files-final.json
/Users/moi/.cache/bridget-tests160-20261012/T035-partial-correlation-prescribed-table-exact-replay.json
/Users/moi/.cache/bridget-review160-preparation-20261012/response-r1/ADR160-R1-complement.candidate.md


## T030 — suite unique close et disposition ROOT

Le FULL unique a duré 900.75s, RC101 : 97 cibles Cargo, 2683 PASS, 6 FAIL et
76 ignored. Les sept PASS des enfants embarqués sont exclus du compte parent.
Les six FAIL restent dans le log original. Ils portent sur deux adaptations
strictement additives d’oracles existants : quota_wait:null dans la projection
legacy 154, et schéma ExecutionStore 11→12 prévu en T011. Aucun code de production
n’a changé. Les 19 sources précédentes sont conservées ; les deux fichiers
additionnels de tests portent les attentes adaptées. La capture 21 courante
est vérifiée par SHA, distincte de la capture historique 19 de T031.

Replay 154 exact : RC0, 1 PASS, 7.88s, sources stables. Les cinq anciens oracles
migration/schéma : RC0, 5 PASS, 4.99s, sources stables. ROOT accepte T030 close sur
preuves et impact. Pas de seconde suite complète ni de GREEN global FULL.
Les compteurs des replays ne remplacent jamais les six échecs du FULL historique.
La réussite des 59 noms ciblés reste cumulative et ne devient pas un run unique.
Les gates restantes ne sont pas supposées acquises : audit final, Converge final,
commit/push, build/install/activation et livraison. Quota réel T002 non observé.

Reçu des comptes Cargo parent :
/Users/moi/.cache/bridget-tests160-20261012/ROOT-FULL160-parent-totals.json
Capture finale 21 :
/Users/moi/.cache/bridget-review160-preparation-20261012/scope-files-final21.json
SHA 57c15398ba4292cbebc9e27b5b22ad06c6eb6b45c155c850d21b085a5c2e6a6c.
Détail des noms/logs/gardes : validation/tests-160.md et tests-160.json.
