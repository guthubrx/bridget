# Converge222 — passage1, source intégrée à main91083f5d

Statut : mécanismes implémentés ; preuves de livraison ouvertes. Aucun CONVERGED global ni livraison annoncés avant ces preuves. Analyse manuelle par le responsable, pas une revue indépendante.

| FR | Mécanisme contrôlé | Témoins ciblés | Constat |
|---|---|---|---|
| 001–003 | AgentProfileStore DeclareRoles/List/Search/Get ; request_role ; autorité RELEASE | claim_scope, contact, release | Source et tests ciblés vérifiés ; circuit natif final ouvert |
| 004–006 | definitions immuables, specialist_initial_activations, RequestTemporary, RequestApproval et grant révisé | profiles_grants_and_versions, temporary_profile, approval_requires | Source/tests vérifiés |
| 007–009 | wake/FleetSupervisor, SpecialistLaunchAuthority::lock et preferences épinglées | release_checks, constraints | Source/tests vérifiés ; réveil réel final ouvert |
| 010–011 | requirements typés, refus avec noms et état unverified ; préparation explicite dans launch_policy ; default_mandate | constraints_and_no_budget | Source/tests vérifiés ; aucune preuve de prérequis inconnue inventée |
| 012 | SpecialistBudget absent par défaut, budget_roots et agrégation native des observations | optional_budget_is_inherited | Source/tests vérifiés ; plafond fournisseur reste distinct |
| 013–016 | specialist_requests + work_submissions prepared atomiques ; promotion, suivi natif, sélection stable et présence vivante | prepared_reservation, contact, team_partial | Source/tests vérifiés ; pas de réaffectation ni interruption silencieuse |
| 017–019 | capture_task/correlated_reply/observed_history ; source cursors/savepoint ; rappel lexical puis projet ; contexte borné et sources conservées | memory_prefers_project, contact | Source/tests vérifiés ; historique en fonctionnement final ouvert |
| 020–021 | MemoryCorrect/Forget/Versions ; tombstone ; versions de consignes distinctes | memory_prefers_project, profiles_grants_and_versions | Source/tests vérifiés |
| 022–024 | outil MCP, CLI humaine, HTTP authentifié, catalogue Android et ouverture conversation existante ; arrêt par contrôles natifs déjà présents | old_client, neuf tests SpecialistsModel | Compilations et modèle vérifiés ; circuit UI authentifié final ouvert |
| 025–027 | Export définition seule, Import inactif, Convert identité existante, CaptureHistory paginée | profiles_grants_and_versions et contrôle source des branches | Source vérifiée ; recette CLI import/conversion finale ouverte |
| 028 | composition immuable + admissions/answers natifs ; mêmes sous-clés après admission partielle | team_partial_admission | Source/tests vérifiés ; aucune copie des états métier |
| 029–030 | failure fermé avec action/portée ; négociation SpecialistsV1 ; plancher specialists.requests.v1 | old_client, request_rejects, storage_floor | Source/tests vérifiés ; retour compatible à qualifier |
| 031 | scripts de compilation/maintenance/update writer existants ; manifeste et contrôles PID/SHA | à produire à la livraison | Ouvert : build final, intégration/push, activation, preuves et nettoyage |

## Écarts trouvés et traités
- Rôle hors portée : refus avant enregistrement, preuve claim_scope.
- Lecture humaine restreinte : List/Get/capture_history respectent aussi la portée de l'appareil. Une décision humaine ne vaut pas scope global. Test limité à un autre projet passé.
- Plancher de retour absent : réutilisation du contrôle de formats, specialists.requests.v1 empêche le drain d'un lecteur ancien. Test ciblé passé.
- Intégration amont : import SHA local requis par la garde RELEASE après changement de l'amont ; corrigé sans changer ce contrat.

Aucune nouvelle tâche fonctionnelle ajoutée : ces corrections ferment T003/T008/T010/T020/T027/T030. T027–T032 restent ouverts pour les preuves finales.

## Passage2 — correction du coût de sélection
Le JOIN fournit la désignation officielle avec chaque candidat ; plus de lecture SQL par candidat dans le minimum. Test de projection du booléen ajouté à claim_scope, contact/composition inchangés. Groupe final13 daemon +1 transport +1 autorité humaine PASS. Code FR001–030 présent ; aucune nouvelle tâche fonctionnelle ajoutée. FR031 et recettes de T027/T031 restent ouverts, donc pas de CONVERGED global avant chargement.

## Clôture de la lecture de code après qualification
Raccord mobile corrigé en T033, test vrai handler et circuit Android passés. Aucun manque de code supplémentaire FR001–030 identifié. Les revendications de livraison FR031 restent ouvertes : le client de ce fil ne s'est pas rechargé. Les tâches existantes couvrent ce reste, aucune tâche de confort ajoutée. Le statut global reste In Progress ; aucun CONVERGED global inventé. Les deux passages de confrontation du code sont conservés.
