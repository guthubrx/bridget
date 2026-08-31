# Specification Analysis Report - SPEC-081

**Méthode** : analyse manuelle non destructive des artefacts. Le binaire `specify` et les scripts `.specify` ne sont pas installés dans ce worktree, conformément au choix de ne pas les installer.

## Résultat

| ID | Catégorie | Sévérité | Emplacement | Synthèse | Résolution |
|---|---|---|---|---|---|
| A1 | Couverture | HIGH, corrigé | FR-8114, plan, tâches | Le modèle Desktop actuel exclut les profils locaux, donc « Cet ordinateur » n'était pas réalisable. | Découverte dynamique de l'endpoint local sans profil ni jeton manuel, tâches T002, T004, T005, T008, T011, T020 et T026. |
| A2 | Cohérence | MEDIUM, corrigé | FR-8108, data-model, tâches | Le tri Nom et la persistance des groupes repliés manquaient du modèle. | Ajout de `name` et `collapsed_group_keys`, tâches T007, T028, T030 et T031. |
| A3 | Sécurité | MEDIUM, corrigé | plan, data-model | Le mot `local` était mélangé à l'état de connexion. | Ajout du champ `kind: ssh | local`; une source locale n'apparaît qu'après lecture de snapshot réussie. |

Aucun finding ouvert de niveau CRITICAL, HIGH ou MEDIUM ne reste.

## Couverture

| Exigence | Tâches | Note |
|---|---|---|
| FR-8101 à FR-8104 | T005, T008, T011, T015-T019 | Flotte agrégée, origine et conversation correcte. |
| FR-8105 à FR-8107 | T020-T021, T025-T026 | Sources et chips de filtre indépendants. |
| FR-8108 à FR-8109 | T027-T031 | Critères, direction, ordre et groupes. |
| FR-8110 | T032-T035 | Coordinateur explicite, épingle locale. |
| FR-8111 à FR-8112 | T008, T016, T018, T025 | Défaillance isolée et gestionnaire atteignable. |
| FR-8113 | T022-T024 | Source cible avant création ou import. |
| FR-8114 | T002, T004-T005, T008, T011, T020, T026 | Découverte conditionnelle de « Cet ordinateur ». |
| FR-8115 | T007, T030, T034 | Préférences exclusivement locales. |
| NFR-8101 à NFR-8102 | T005, T008, T031 | Complexité et budget de 150 ms. |
| NFR-8103 | T021, T029, T026 | Chips et déplacement accessibles au clavier. |
| NFR-8104 à NFR-8105 | T002, T005, T009, T013-T014, T018 | Données réduites et WebView sans privilège. |
| SC-8101 à SC-8107 | T018-T019, T026, T031, T035-T038 | Témoins techniques, performance et parcours manuel. |

## Constitution

- Worktree dédié et base explicite : conforme.
- Une seule abstraction nouvelle, `fleet.rs`, justifiée par réduction de données privées, agrégation multi-sources et tests purs : conforme.
- Aucun second transport, store de profils, registre de panneaux ou messagerie : conforme.
- Tous les travaux non triviaux ont une preuve observable : conforme.
- Livraison volontairement exclue de T041 : conforme au mandat utilisateur.

## Tâches non mappées directement

T001, T036-T041 sont des gates de preuve, convergence, relecture et documentation. Elles sont nécessaires au workflow et ne constituent pas du travail orphelin.

## Métriques

- Exigences fonctionnelles : 15
- Exigences non fonctionnelles : 5
- Critères de succès constructibles : 7
- Tâches : 41
- Couverture : 27/27, 100 %
- Ambiguïtés ouvertes : 0
- Duplications : 0
- Issues CRITICAL : 0
- Issues Article XIX/XX : 0

## Décision

PASS. Les artefacts sont cohérents pour commencer l'implémentation.

## Analyse après implémentation

| Vérification | Verdict | Preuve |
|---|---|---|
| Une origine défaillante n'efface pas les autres. | PASS | SourceProjection isolée et test source_illisible_est_isolee_sans_effacer_les_autres_sources. |
| Aucun chemin de projet, jeton, URL ni détail SSH ne traverse fleet_snapshot. | PASS | DTO public réduit et test projection_publique_ne_transporte_ni_chemin_ni_secret. |
| L'origine sélectionnée est la seule qui reçoit une action. | PASS | panel_open accepte source_id, reconstruit localement son URL et n'accepte que create_project ou import_project. |
| La source locale n'est pas un profil persistant. | PASS | découverte par commande constante, conditionnée à une lecture réussie de snapshot. |
| Panneau unique et coque visible. | PASS | PanelRegistry inchangé, géométrie réservée à 520 px et relais desktop_shell. |
| Filtres, tris et épingles restent locaux. | PASS | PreferencesStore étendu, fleet-presentation sans accès réseau. |

### Note Article XIX/XX

Deux modules nouveaux, et non un seul comme envisagé initialement, existent finalement :

1. fleet.rs réduit une frontière de sécurité avant IPC.
2. fleet-presentation.js porte les règles pures nécessaires à des tests sans WebView.

Ils remplacent une logique implicite et non testable, ne créent aucun framework interne et sont chacun appelés par l'interface produite. Aucun finding critique ne reste.

## Convergence finale

Passage : 2
Verdict : CONVERGED

La vérification finale a comparé contrats, tâches, DTO IPC, interface et tests. Une divergence a été détectée avant ce passage formel : les regroupements imbriqués devaient refléter tous les critères de tri, alors que la première projection n'utilisait que le premier. Elle a été corrigée dans fleet-presentation.js et couverte par le témoin trois_criteres_conservent_les_agents_et_la_stabilite_des_regroupements.

Aucune nouvelle exigence ni tâche n'est nécessaire. Les 41 tâches existantes couvrent l'implémentation et ses gates.
