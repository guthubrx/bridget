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

| Décision | Choix retenu | Alternative rejetée |
|---|---|---|
| D1 | Une Task et un enfant pour plusieurs rondes ; chaque ronde change la mission. | Une nouvelle Task liée : casse les index par enfant et le nettoyage. |
| D2 | Enfant inactif en `result_available` avec `idle_deadline_at`. | Un état `idle` : change le contrat fermé de statuts149/T3. |
| D3 | Fin explicite par Cancel et arrêt humain existants ; preuve durable de garde levée. | Un outil release : ajoute une surface publique inutile. |
| D4 | Brief validé et rendu par `handoff::validate_and_render`, mêmes champs et borne de 16 384 octets. | Lecture de l'historique parent ou message séparé : sources indisponibles, ordre et atomicité perdus. |
| D5 | Huit rondes au plus ; garde de 60 à 7 200 secondes. | Contexte et garde sans borne. |
| D6 | Échec d'une ronde : tâche `failed` et nettoyage existant. | Nouvel essai automatique : hors contrat153. |
| D7 | Rejeu par `RoundRecord { round, request_id, digest }`, huit entrées au plus. | Table de clés de ronde supplémentaire. |
| D8 | `native_launch_registry` commun au lancement et à la reprise152 ; contrôles locaux de Continue. | Extraction ou wrapper de droits supplémentaire. |
| D9 | Recalcul de la politique actuelle avec la posture demandée initialement ; égalité complète avec la politique figée. | Révocations seules, posture effective ou retrait de champs de sécurité : réduction de droits non détectée. |
| D10 | Table indexée `native_delegation_past_missions(mission, task_id)` et JOIN de lecture. | Balayage du JSON de toutes les tâches à chaque réponse tardive. |
| D11 | Preuve de remise par ronde dans `unacked_rounds` ; aucune purge tant qu'une preuve manque. | Exiger l'ACK avant Continue : bloque une correction dans le tour où le parent lit le résultat. |
| D12 | `begin_round` atomique sous verrou : heure fraîche, historique, CAS, resets et projection dans une transaction ; signal après commit. | Sauvegardes séparées : ronde partielle ou réveil après rollback. |

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
