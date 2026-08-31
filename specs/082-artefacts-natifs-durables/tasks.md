# Tâches - SPEC-082 Artefacts natifs durables et vérifiables

**Entrée**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/`,
`quickstart.md`, `reuse-audit.md`
**Statut de ce fichier**: planifié uniquement. Aucune tâche ci-dessous n'est
implémentée dans cette session de préparation.

## Dépendances et stratégie

~~~text
Fondations
  └─ US1 Publication fiable
      ├─ US2 Consultation native
      ├─ US3 Conservation, restauration et cache
      └─ US4 Recherche, partage et références
          └─ US5 Erreurs et opérations interrompues
SPEC-083 dépend de US1, US2 et US3 de cette spec.
~~~

MVP: US1 puis US2 - publier une version structurée et la consulter de manière
lisible, sourcée et stable. US3 doit être achevé avant toute promesse de
durabilité aux utilisateurs. Aucun vieux message n'est migré.

## Phase 1 - Préparation et dépendances

- [x] T001 Créer `crates/bridget-daemon/src/artifact_types.rs` avec les modèles sérialisables de `data-model.md`, sans accepter de champ inconnu et avec versions de schéma explicites.
- [x] T002 [P] Ajouter le packaging local reproductible des dépendances UI dans `crates/bridget-daemon/assets/ui/package.json`, `package-lock.json` et `vendor/manifest.json`, sans CDN et avec versions figées.
- [x] T003 [P] Mettre à jour `crates/bridget-daemon/assets/ui/vendor/NOTICE.md` avec ECharts et Tabulator, leurs licences et les sommes d'intégrité attendues.
- [x] T004 [P] Ajouter les fixtures de publication dans `crates/bridget-daemon/tests/fixtures/artifacts/`, incluant données externes sourcées, données partielles, image, fichier et provenance générée.
- [x] T005 Documenter les réglages globaux d'artefact dans `docs/decisions/023-artefacts-structures-et-sandbox.md`, notamment 1 Gio/30 jours, alerte 8 Gio, blocage 10 Gio et exclusion de toute migration.

## Phase 2 - Fondations bloquantes

- [x] T006 Ajouter les migrations additives et les indexes de métadonnées définis dans `crates/bridget-daemon/src/artifact_store.rs`, en s'appuyant sur l'ouverture SQLite existante de `crates/bridget-daemon/src/store.rs`.
- [x] T007 Implémenter dans `crates/bridget-daemon/src/artifact_policy.rs` les limites de manifeste 512 KiB, payload structuré 16 MiB, blob 128 MiB, 100 sources et les seuils de cache globaux documentés.
- [x] T008 Implémenter le magasin de blobs canoniques dans `crates/bridget-daemon/src/artifact_blob_store.rs`, avec SHA-256, écriture atomique, permissions privées, références et racine durable distincte de `.cache/bridget`.
- [x] T009 Étendre `crates/bridget-daemon/src/daemon.rs` afin de configurer explicitement la racine canonique des artefacts sans réutiliser le répertoire de cache volatil.
- [x] T010 [P] Écrire les tests unitaires de migration, références, suppression récupérable et corruption de manifeste dans `crates/bridget-daemon/tests/artifact_store_test.rs`.
- [x] T011 [P] Écrire les tests de limites, quota, rétention, pinning et calcul de cache dans `crates/bridget-daemon/tests/artifact_policy_test.rs`.
- [x] T012 Ajouter à `apps/bridget-desktop/src-tauri/src/preferences_store.rs` les préférences globales de cache local et de conservation, avec migration du document de préférences et libellé de portée `Ce Mac`.

## Phase 3 - US1 Publication Bridget structurée (P1)

**Objectif**: un agent capable publie un unique payload versionné par
`bridget_publish_artifact`; Bridget le valide, l'atteste et le rattache au bon
tour/projet. Un agent incapable l'indique honnêtement.

**Critère indépendant**: publier un graphique avec provenance externe et une
table dérivée, répéter le même appel puis un payload invalide, et constater un
reçu idempotent ou une erreur structurée sans écriture partielle.

- [x] T013 [US1] Implémenter `ArtifactService` dans `crates/bridget-daemon/src/artifact_service.rs` pour valider `ArtifactPublicationV1`, créer une version et produire un reçu atomique conforme à `contracts/artifact-publication-v1.md`.
- [x] T014 [US1] Ajouter la validation de provenance externe, calculée et déclarée dans `crates/bridget-daemon/src/artifact_service.rs`, en refusant les résultats factuels sans source exploitable.
- [x] T015 [US1] Ajouter l'outil MCP `bridget_publish_artifact` dans `crates/bridget-daemon/src/mcp.rs`, avec schéma fermé, contexte projet/tour attesté et refus des identités ou visibilités imposées par l'appelant.
- [x] T016 [US1] Ajouter les actes de journal de publication dans `crates/bridget-transport/src/act_kind.rs` et `crates/bridget-daemon/src/ui.rs`, sans créer de timeline parallèle.
- [x] T017 [US1] Écrire les tests de contrat MCP dans `crates/bridget-daemon/tests/artifact_publication_test.rs` pour succès, paramètres inconnus, provenance insuffisante, erreur structurée et rejeu idempotent.
- [x] T018 [US1] Écrire le test d'isolement projet et de rattachement de tour dans `crates/bridget-daemon/tests/artifact_publication_test.rs`, y compris le refus d'une portée demandée plus large.

## Phase 4 - US2 Consultation native, accessible et sourcée (P1)

**Objectif**: le même payload est rendu inline sous une forme adaptée sans
convertir le Markdown normal en artefact ni demander au moteur de produire des
doublons texte/image.

**Critère indépendant**: ouvrir une conversation avec graphique, KPI, table,
timeline, image et fichier, vérifier résumé/table de même source, thèmes,
clavier, copie/export et absence de requête réseau renderer.

- [x] T019 [US2] Ajouter la projection de références d'artefacts dans `crates/bridget-daemon/assets/ui/app.js` au-dessus de `projectTimeline`, en conservant ordre, ancre de lecture et historique existant.
- [x] T020 [US2] Créer `crates/bridget-daemon/assets/ui/artifact-renderer.js` pour les types `chart`, `kpi`, `table`, `timeline`, `image` et `file`, avec rendu différé et aucune interprétation de pseudo-balise Markdown.
- [x] T021 [US2] Intégrer ECharts localement dans `crates/bridget-daemon/assets/ui/vendor/` et son adaptateur dans `artifact-renderer.js`, avec `aria` activé, résumé de valeurs clés et table dérivée du même dataset.
- [x] T022 [US2] Intégrer Tabulator localement dans `crates/bridget-daemon/assets/ui/vendor/` seulement pour les tables dépassant le seuil documenté, avec table HTML sémantique de repli.
- [x] T023 [US2] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` la carte Données et source, les indicateurs de qualité, la version exacte, copie et export par type sans exécuter de contenu actif.
- [x] T024 [US2] Ajouter les routes relayées de détail, données et export dans `crates/bridget-daemon/src/ui.rs`, avec pagination, bornes de taille et contrôle de portée projet.
- [x] T025 [US2] Ajouter les styles clair/sombre, focus visible, chargement différé et erreur lisible dans `crates/bridget-daemon/assets/ui/theme.css`.
- [x] T026 [US2] Écrire les tests Node de rendu et accessibilité dans `crates/bridget-daemon/assets/ui/artifact-renderer.test.mjs`, couvrant résumé/table, code Markdown inchangé, grande table et données partielles.
- [x] T027 [US2] Écrire les tests de routes relayées dans `crates/bridget-daemon/tests/artifact_relay_test.rs`, couvrant contenu autorisé, pagination, export et refus inter-projet.

## Phase 5 - US3 Conservation, restauration et cache (P1)

**Objectif**: retrouver une version après éviction du cache et restaurer sans
altérer l'historique. Un changement de source crée toujours une version enfant.

**Critère indépendant**: évincer le cache local, restaurer une version canonique,
actualiser une source différente et vérifier version enfant, puis épingler une
version et exécuter la purge sans la supprimer.

- [x] T028 [US3] Ajouter le collecteur externe borné dans `crates/bridget-daemon/src/artifact_fetch.rs`, avec validation de destination, redirections contrôlées, plafonds, empreinte, date de collecte et provenance.
- [x] T029 [US3] Ajouter dans `crates/bridget-daemon/src/artifact_service.rs` les opérations explicites restaurer, actualiser, épingler et supprimer, avec même hash réactivant la version exacte et hash différent créant une enfant.
- [x] T030 [US3] Ajouter le cache de consultation Desktop et son nettoyage déterministe dans `apps/bridget-desktop/src-tauri/src/artifact_cache.rs`, sans autorité sur les blobs canoniques.
- [x] T031 [US3] Ajouter les commandes/écrans de réglages correspondants dans `apps/bridget-desktop/ui/fleet-app.js` et `apps/bridget-desktop/ui/fleet.html`, étiquetés `Ce Mac` ou `Serveur Bridget` selon l'autorité réelle.
- [x] T032 [US3] Écrire les tests de collecte contre destinations refusées, redirections, tailles, offline et source disparue dans `crates/bridget-daemon/tests/artifact_lifecycle_test.rs`.
- [x] T033 [US3] Écrire les tests de version immuable, restauration même hash, version enfant, cache évincé et pinning dans `crates/bridget-daemon/tests/artifact_lifecycle_test.rs`.

## Phase 6 - US4 Recherche, références et partage explicite (P2)

**Objectif**: retrouver au bon projet une version exacte, sans rendre les
artefacts globaux par défaut ni transformer le panneau en explorateur de disque.

**Critère indépendant**: publier dans deux projets, retrouver par défaut dans
le bon projet, passer volontairement en global, suivre une référence ancienne
et vérifier la visibilité après partage explicite seulement.

- [x] T034 [US4] Ajouter les requêtes indexées de liste, recherche et référence de version dans `crates/bridget-daemon/src/artifact_store.rs`, avec portée projet par défaut et pagination.
- [x] T035 [US4] Ajouter les contrats relay de recherche, partage explicite et version actuelle dans `crates/bridget-daemon/src/ui.rs`, sans exposer de chemin système.
- [x] T036 [US4] Ajouter l'onglet Artefacts et la carte de référence compacte dans `crates/bridget-daemon/assets/ui/app.js`, avec navigation vers la version exacte et indicateur de version plus récente.
- [x] T037 [US4] Écrire les tests d'index, visibilité, partage et recherche globale volontaire dans `crates/bridget-daemon/tests/artifact_store_test.rs`.

## Phase 7 - US5 Échecs explicites, annulation et reprise (P2)

**Objectif**: aucun résultat final ne se termine sur un placeholder muet et une
publication interrompue garde un état durable honnête.

**Critère indépendant**: interrompre collecte et publication, simuler un blob
manquant et vérifier cause, manifeste, export et action de restauration sans
faux succès.

- [x] T038 [US5] Ajouter les codes d'erreur et reçus de récupération dans `crates/bridget-daemon/src/artifact_types.rs`, couvrant source indisponible, timeout, quota, blob absent et validation refusée.
- [x] T039 [US5] Raccorder l'annulation de tour existante de SPEC-063 aux opérations d'artefact dans `crates/bridget-daemon/src/artifact_service.rs`, en laissant le résultat partiel marqué interrompu par vous.
- [x] T040 [US5] Ajouter les cartes d'échec et actions restaurer/exporter/consulter manifeste dans `crates/bridget-daemon/assets/ui/artifact-renderer.js` et `theme.css`.
- [x] T041 [US5] Écrire les tests d'annulation, d'erreur finale non muette et de récupération dans `crates/bridget-daemon/tests/artifact_lifecycle_test.rs` et `artifact-renderer.test.mjs`.

## Phase 8 - Finition, sécurité et preuve de maintenance

- [x] T042 Vérifier et supprimer toute tentative de parse de pseudo-balises d'artefact dans `crates/bridget-daemon/assets/ui/app.js`, afin de garder le contrat MCP unique.
- [x] T043 Ajouter les métriques non sensibles de volume, éviction, erreur de publication et restauration dans `crates/bridget-daemon/src/artifact_service.rs` et `crates/bridget-daemon/src/ui.rs`.
- [x] T044 Exécuter les suites Rust déclarées dans `Cargo.toml`, les tests Node de `crates/bridget-daemon/assets/ui/` et la recette `specs/082-artefacts-natifs-durables/quickstart.md`, en consignant les preuves dans `specs/082-artefacts-natifs-durables/implementation.md`.
- [x] T045 Réviser les dépendances locales, licences, hashes, chemins de sauvegarde canonique et procédure de restauration dans `docs/decisions/023-artefacts-structures-et-sandbox.md` et `crates/bridget-daemon/assets/ui/vendor/NOTICE.md`.

## Opportunités de parallélisme

- Après T006-T009 : T010, T011 et T012 peuvent être menées séparément.
- Après T013 : T017 et T018 peuvent être menées en parallèle avec T016.
- Après T019 : le renderer ECharts/T023 et la route relay T024 sont séparables.
- Après T028 : les tests lifecycle T032/T033 peuvent progresser avec le cache T030.

## Vérification de format

Les 45 tâches sont toutes des checkboxes, portent un identifiant séquentiel et
un chemin cible. Les tâches de récits portent un label US. Les tâches T042 et
T045 réduisent explicitement la dette future : elles empêchent une seconde voie
de publication et rendent dépendances, sauvegarde et licences auditables.
