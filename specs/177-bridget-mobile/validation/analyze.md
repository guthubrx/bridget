# Analyze 177 — 2026-10-24

Primitive : protocole speckit-analyze lu et exécuté directement. Les scripts projet de prérequis et setup sont absents ; la résolution du dossier s'appuie sur .specify/feature.json et sur les fichiers réellement présents.

## Premier passage

| ID | Catégorie | Gravité | Constat | Correction retenue |
|---|---|---|---|---|
| A1 | Couverture | HIGH | T026 ne nommait qu'une preuve de migration ; l'adaptation requise par les attestations d'origine n'était pas une action explicite | Nommer l'implémentation de l'adaptation avant sa preuve |
| A2 | Sécurité | HIGH | La réconciliation d'une association dont la réponse est perdue n'avait pas d'identité de tentative figée | Exiger tentative et clé appareil persistées avant échange |
| A3 | Ressources | MEDIUM | Les bornes HTTP, pages et abonnements n'étaient pas chiffrées | Fixer les limites initiales dans le contrat |
| A4 | Cohérence | MEDIUM | Les nouveaux modules et les chemins des tâches n'étaient pas nommés dans le plan | Ajouter le plan de fichiers |
| A5 | Couverture | MEDIUM | L'admission humaine à un profil risquait d'être confondue avec une identité d'agent | Exiger autorité humaine explicite, profil et projet déjà autorisés |

Les corrections non ambiguës ont été appliquées dans le périmètre validé, puis les trois artefacts ont été relus.

## Second passage

27 exigences FR, 41 tâches distinctes, 27/27 exigences couvertes. Aucun marqueur non résolu. Les huit scénarios sont couverts. Les dix critères SC sont rattachés aux tâches de test et de livraison ci-dessous.
Aucun CRITICAL persistant. Les preuves runtime de migration et les préconditions de notification/téléphone ne sont pas supposées acquises.

| Critère | Tâches de preuve |
|---|---|
| SC-001 | T006, T021, T036 |
| SC-002 | T009, T022, T036 |
| SC-003 | T013, T025 |
| SC-004 | T007, T025, T036 |
| SC-005 | T036 |
| SC-006 | T026, T029, T030 |
| SC-007 | T017, T024, T036 |
| SC-008 | T025, T035, T036 |
| SC-009 | T038, T039, T040, T041 |
| SC-010 | T009, T012, T017, T036 |

## Constitution et minimalisme

Worktree serveur isolé ; moteur, reçu, journal et question humaine réutilisés. La nouvelle responsabilité d'appareil est justifiée. Pas de framework d'orchestration, d'agent synthétique ou de second ledger métier.
Une dépendance HTTP et les bibliothèques Android restent soumises à recherche, justification et versions verrouillées avant ajout.
La migration d'une autorité n'est pas remplacée par un proxy. Les tâches de code et les tâches de recette physique restent séparées.
Potentiel minimalisme : aucune ligne de code à évaluer à ce stade ; le périmètre neuf sera revu après implémentation.

## Limites

Analyse d'artefacts et lectures ciblées du code, pas validation de l'application. Contre-revue refusée avant démarrage, aucun verdict indépendant. Aucun build, aucune migration ni installation ne sont annoncés par cette analyse.

## Prochaine action

Reprendre T001, puis T002 et T003. La livraison complète exige toutes les tâches et les gates ; le statut reste In Progress.

## Reprise du 24 octobre — décision et cohérence

La création directe est exclue sur décision utilisateur ; T014 devient l’envoi humain à un agent existant. La capacité task_create est refusée par le magasin et absente du profil --all. Les exigences restantes ne sont pas réduites : fédération, tâches, historique, pièces jointes, notifications, serveur permanent et livraison restent ouverts. Analyse manuelle : outils shell SpecKit toujours absents du checkout ; comparaison des artefacts et du code actuel. Aucun CRITICAL de cohérence documentaire identifié ; aucun verdict de sécurité global n’est acquis.

## Reprise complète — 24 octobre, 14 h 11 CEST

Sync projet exécutée : aucun changement nécessaire. Primitives analyze et implement relues. check-prerequisites.sh et extensions.yml sont absents dans ce checkout ; dossier résolu par .specify/feature.json. Checklist exigences : 12/12. Cadrage, 27 exigences et 41 tâches conservés ; 12 tâches vérifiées, 29 ouvertes. Relecture spec/plan/contrats/tasks et code courant : aucun CRITICAL de cohérence documentaire. Les actions directes de création restent exclues. La configuration des notifications, la migration et les recettes réseau restent des préconditions non acquises.

Constats non ambigus : la liste des appareils doit être paginée pour rester sous la borne de réponse Android ; le suivi doit réutiliser le flux attach natif, avec annulation par échéance/révocation et sans interrogation répétée du worker. Ces points sont couverts par T007/T011 et ne nécessitent pas une nouvelle fonctionnalité. Les adaptations de commandes, brouillons et renouvellements à leur origine sont vérifiées par les tests précédents ; aucune sécurité globale n’en est déduite. L’analyse est exécutée dans ce tour ; Converge reste réservé à la dernière implémentation.

## Reprise après preuve d'origine sur deux machines

T026 reste ouverte malgré le succès du relais d'attestation : le chemin prepare_routing_projects suppose encore la source locale au coordinateur. L'observation et la création natives locales supposent aussi cette égalité. Le plan doit adapter ces deux points avant migration. Réutiliser prepare-project.py et sa publication de mapping, puis le worker existant pour une exécution sur le coordinateur depuis une source étrangère. Aucun second moteur Git, transfert de secret au téléphone ou identité fournisseur synthétique.

T026 cite un fichier de tests d'intégration absent. Les preuves sont dans les modules natifs origin_proof, les fixtures de daemon et les tests ciblés existants. La tâche est corrigée vers ces chemins réels. Aucun CRITICAL documentaire persistant ; la sécurité et la bascule restent des gates ouvertes.

## Reprise Firebase — 24 octobre, 17 h 25 CEST

Analyse exécutée à nouveau dans ce tour. La primitive check-prerequisites.sh est absente (code 127), donc comparaison manuelle de spec.md, plan.md, tasks.md, contrat d’origine et code. Les 27 exigences restent couvertes par les 41 tâches. Les 19 cases cochées restent inchangées. Les gardes de préparation relevées par la contre-revue sont corrigées et les 54 tests ciblés passent. Le contrat d’origine décrit maintenant PrepareProject ; sa version précédente ne décrivait que les lectures. Aucun CRITICAL documentaire persistant. La migration, les notifications réelles et la recette complète restent ouvertes ; aucune version livrée n’est annoncée.

Observation de production utile à T027 : les 111 anciennes Tasks locales sont terminales, aucune n’est continuable et toutes sont nettoyées lors de cette lecture. Cette observation devra être refaite juste avant la bascule ; elle ne vaut pas gel de la production. Les Tasks distantes restent à réconcilier sur leur placement d’origine.
