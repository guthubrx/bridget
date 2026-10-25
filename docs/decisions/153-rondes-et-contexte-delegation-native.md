# Décision153 - rondes et brief de la délégation native

Date : 2026-10-11. Statut : Accepté.
Ce statut accepte les décisions du plan. Il ne prouve ni la fin des tests
ni l'activation des processus de production.

## Contexte

La délégation150 nettoie l'enfant après son résultat. Les lectures et l'attente151
restent pures. L'accusé de lecture est distinct de la remise. Le socle152 reprend
une mission après une panne, sous contrôle de l'identité et des droits.
La session153 permet des corrections successives avec le même enfant et un
dossier structuré du parent. Elle ne crée pas un moteur de reprise supplémentaire.

## Décisions et alternatives

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/153-rondes-et-contexte-delegation-native.md#extrait-1).

D12 explicite les invariants I1/I5/I7 et T004 du plan. Elle n'ajoute pas
une exigence. La transaction rend la Task après commit. Elle vide les champs
par ronde, dont résultat, erreur, échéance, dates de fin/lecture/remise et flags.
Elle conserve les compteurs de reprise152 et le checkpoint de lancement de
vie entière, les identités, la filiation, le dossier et les droits.

## Réemploi et limites de structure

Sept champs Task couvrent garde, fin durable, ronde, corps, rejeu et preuves
d'accusé. La garde initiale reste immuable ; sa libération reste monotone.
L'index existant `native_delegations_state` garde son nom et s'étend aux places
tenues par les enfants inactifs. La migration ne reconnaît que les deux
définitions SQL exactes connues, après `trim` des extrémités ; une définition
étrangère refuse la migration et garde ses données. Deux clauses `INDEXED BY`
imposent le même index aux comptages global et propriétaire.

Coûts du code : quota O(log n + a), a ≤128 ; comptage total O(n), préexistant ;
mission passée O(log n) ; corps O(B) ; recontrôles de politique comptés séparément.
Ce sont des analyses du code et des observations EXPLAIN, sans banc de performance.
Le schéma handoff existant est transmis aux adaptateurs, sans copie ni helper neuf.
L'observateur interactif149 publie le fait courant pour Delegate et Continue.

L'attente et le statut restent des lectures. Quatre consommateurs utilisent
le même identifiant de résultat courant ; l'entretien recopie aussi les vrais
accusés des anciennes rondes avant la purge. Une réponse tardive compare
l'identité de l'enfant sans réconciliation ni sauvegarde.
Le magasin, les transactions, la projection, le nettoyage et les gardes
de cible/droits existants sont réutilisés. Aucun module ou dépendance nouveau.
La seule fixture commune neuve est `cfg(test)`, avec quatre usages T028 prévus ;
elle compose trois primitives de daemon existantes et le vrai `begin_round`.

## Conséquences et preuves

L'enfant garde son contexte entre rondes. Le même processus ne garantit pas
le souvenir d'un vrai modèle et peut renforcer un biais. Une contre-revue
indépendante utilise une nouvelle délégation. Le redémarrage du daemon met
fin à la garde ; elle ne reprend pas. Un enfant inactif tient une place jusqu'à
son arrêt confirmé. Le résultat et l'état sont conservés après fin explicite
sur `result_available`. La garde ne peut pas être reposée après cette fin.

Les preuves finales ciblées sont40 tests natifs et1 E2E PASS, puis179 références
lib PASS sur le même binaire final. L’E2E prouve trois rondes sur le même
processus/instance et le refus du17e enfant. FR-019 est corrigé : propriété pure,
tick après nouvelle admission committée seulement. Les16 références d’intégration
gardent la preuve workspace et l’impact final des trois fichiers changés.
Workspace R1 reste2145 PASS/10 FAIL/72 ignorés avant corrections ciblées ;
aucun GREEN global. Mémoire réelle T024 non vérifiée selon quickstart :
aucun modèle ni service lancé pour cette recette.

Références : plan et invariants, contrat rounds153, audit de réemploi et preuves
dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/153-rondes-contexte/.
Guide : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/delegation-native.md.
