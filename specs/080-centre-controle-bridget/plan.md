# Plan d'implémentation - SPEC-080 Centre de contrôle Bridget

## Contexte technique

| Élément | Valeur confirmée |
|---|---|
| Serveur Bridget | Rust 2024, daemon SQLite, relais HTTP loopback dans `crates/bridget-daemon/src/ui.rs` |
| Panneau d'un serveur | HTML, CSS et JavaScript sans framework dans `crates/bridget-daemon/assets/ui/`, chargé à travers le tunnel SSH déjà vérifié |
| Client macOS | Tauri v2, Rust et WebView; profils SSH persistés localement |
| Usage existant | `usage_samples` horodatés par agent et source, plus agrégats d'exécution corrélés |
| Configuration modifiable existante | `ProjectRootPolicy`, déjà atomique et protégée par génération |
| Dépendances | aucune nouvelle dépendance ou service externe |

## Décisions de planification

1. L'engrenage est ajouté au bas de la barre gauche du panneau distant. Comme chaque panneau est lié à un seul profil SSH, il ouvre naturellement les réglages de ce serveur et non une copie globale ambiguë.
2. Les préférences de présentation restent sur le Mac, dans un document versionné séparé des profils et des données du serveur. Elles s'appliquent aux fenêtres Bridget Desktop; aucun nom, thème, fuseau ou réglage de lisibilité ne traverse le tunnel vers le serveur dans ce lot. Le panneau distant conserve son thème serveur sombre.
3. Le serveur publie un catalogue fermé. La première clé réellement modifiable est la politique des racines de projets, car elle possède déjà un stockage atomique et une génération. Les catégories fournisseurs, exécution, projets, observabilité, maintenance et sécurité sont visibles mais lecture seule tant qu'un contrat dédié ne leur donne pas une écriture sûre.
4. Le parcours d'écriture réutilise les invariants existants: lecture, prévisualisation sans effet, confirmation locale par l'opérateur, génération attendue, application atomique et reçu. Les routes historiques de projets restent compatibles pendant la migration.
5. L'usage réutilise les échantillons immuables déjà présents. Les dimensions fournisseur et modèle sont enrichies seulement lorsqu'elles sont attestées au moment de l'observation. Une dimension inconnue reste inconnue.
6. Les tarifs sont un catalogue explicite, versionné et daté, sans récupération automatique. Sans tarif applicable, Bridget affiche les jetons et la couverture mais aucun faux coût. Les montants sont toujours intitulés Estimation API.
7. Les mises à jour restent informatives. Aucun téléchargement, redémarrage ou commande d'hôte ne fait partie de la fonctionnalité.

## Constitution check initial

| Gate | Verdict | Justification |
|---|---|---|
| Worktree isolé | PASS | La branche `session-080-centre-controle-bridget` est isolée de `main`. Les numéros 078 et 079 étaient déjà occupés par des worktrees actifs, la session a donc été renumérotée avant tout code. |
| Réutilisation avant création | PASS | Réutilisation du relais UI, du tunnel SSH, de `ProjectRootPolicy`, de `usage_samples`, de la validation de génération et des tests JavaScript existants. |
| Minimalisme | PASS | Aucun terminal distant, éditeur de fichiers, OAuth, serveur central, dépendance frontend ou mise à jour automatisée. Une seule clé serveur devient réellement éditable dans ce lot. |
| Sécurité et secrets | PASS sous garde | Les valeurs secrètes, profils, approbations et commandes libres restent exclus. La confirmation est faite par le WebView local connecté par tunnel approuvé. |
| Observabilité | PASS | Seuls les compteurs, dimensions techniques et reçus non sensibles sont stockés. |
| Complexité | PASS sous test | Les agrégats de période sont calculés par requêtes indexées sur `usage_samples`; tout regroupement UI est linéaire dans le nombre de lignes retournées. |
| Article XX | PASS | Les décisions, contrats, limites et procédures de vérification sont écrits dans les artefacts afin qu'un mainteneur puisse les modifier sans contexte implicite. |
| WIP | WARN | Le dépôt dépasse largement cinq worktrees actifs. Cette SPEC ne ferme, ne déplace ni ne modifie aucun autre worktree. |

## Plan par lots

### Lot 1 - Contrats et données sûres

- Créer le module de catalogue de contrôle serveur fermé et son état versionné.
- Encapsuler la lecture et la mutation de `ProjectRootPolicy` dans ce catalogue, sans dupliquer sa validation ni son écriture atomique.
- Ajouter les routes relay v1 de lecture, prévisualisation, application et reçu. Conserver les routes `/v1/projects/*` existantes pendant ce lot.
- Ajouter une table de reçus non secrets et des tests d'idempotence, conflit de génération, refus et échec atomique.
- Étendre la conservation des échantillons d'usage avec les dimensions attestées disponibles, sans reconstruire modèle ou fournisseur après coup.
- Ajouter le catalogue de tarifs daté et versionné. Le catalogue vide est valide et conduit à une estimation indisponible, jamais à zéro.

### Lot 2 - Écrans de contrôle

- Ajouter l'engrenage et une vue de réglages dans la barre d'agents sans perturber les conversations, les menus d'agents ni le redimensionnement de barre existant.
- Construire les sections Vue d'ensemble, Réglages, Usage, Mises à jour et Diagnostics à partir des capacités de la réponse serveur.
- Rendre les lignes par des descripteurs de portée et d'état. Les réglages projet et secrets sont informatifs ou redirigent vers leur surface existante, jamais modifiables ici.
- Prévisualiser un delta et demander une confirmation locale explicite avant POST d'application. Montrer ensuite le reçu réellement retourné.
- Créer les agrégats de période, les filtres et le détail par fournisseur, modèle et projet lorsque la dimension est attestée. Afficher couverture et dimensions inconnues.

### Lot 3 - Préférences macOS et intégration

- Ajouter un stockage local versionné pour nom d'opérateur, thème, fuseau IANA, taille de police et réduction de mouvement.
- Ajouter l'écran de préférences local dans Bridget Desktop. Une modification de préférence ne modifie aucun serveur, tunnel ni panneau distant.
- Exposer l'état de chaque profil enregistré: connecté, version relay attestée, dernière synchronisation et disponibilité des capacités, sans enregistrer le jeton du relais dans le rendu.

### Lot 4 - Validation et documentation

- Étendre les tests unitaires Rust et JavaScript avant chaque comportement visible.
- Exécuter formatage, tests ciblés, suites workspace pertinentes et vérification de diff.
- Vérifier manuellement sur un serveur enregistré le parcours de lecture, prévisualisation, refus concurrent et reçu. Aucune mise à jour de serveur n'est déclenchée.
- Consigner les limitations de couverture et le statut de l'estimation dans les preuves.

## Fichiers principaux prévus

| Surface | Évolution |
|---|---|
| `crates/bridget-daemon/src/control_settings.rs` | Nouveau catalogue fermé, prévisualisation, application et reçu. |
| `crates/bridget-daemon/src/ui.rs` | Routes relay v1 et projections non sensibles. |
| `crates/bridget-daemon/src/store.rs` | Reçus, tarifs et requêtes d'usage indexées. |
| `crates/bridget-daemon/src/daemon.rs` | Enrichissement des observations attestées, sans inférence. |
| `crates/bridget-daemon/assets/ui/index.html` | Engrenage et conteneur de la vue contrôle. |
| `crates/bridget-daemon/assets/ui/app.js` | État, requêtes, rendu, accessibilité et tests Node. |
| `crates/bridget-daemon/assets/ui/theme.css` | Mise en page et préférences de rendu. |
| `apps/bridget-desktop/src-tauri/src/*` | Préférences locales versionnées, commandes Tauri et URL de panneau. |
| `apps/bridget-desktop/ui/*` | Écran de préférences macOS. |

## Stratégie de test

1. Écrire les tests de contrat et de refus avant la mutation serveur.
2. Tester le catalogue avec capacité absente, lecture seule, valeur invalide, conflit de génération, idempotence et rollback atomique.
3. Tester les agrégats usage avec données absentes, inconnues, filtrées, cache, tarif absent et tarif daté.
4. Exécuter le programme de test Node intégré à `crates/bridget-daemon/assets/ui/app.js` pour l'état et le rendu pur.
5. Exécuter les tests de crate du daemon, du transport et de Bridget Desktop, puis les tests workspace applicables.
6. Demander une validation visuelle humaine avant toute installation de paquet macOS. La construction ne remplace pas cette validation.

## Risques et parades

| Risque | Parade |
|---|---|
| Une page de réglages devient un shell distant déguisé | Identifiants fermés, schémas typés, aucune chaîne de commande ou chemin libre hors politique de projets déjà validée. |
| Confusion Mac, serveur et projet | Badge de portée obligatoire sur chaque ligne et séparation des stockages. |
| Coût trompeur | Tarif absent signifie estimation indisponible; aucune valeur par défaut. |
| Historique modèle incomplet | Dimension inconnue explicite; pas de jointure rétrospective sur un modèle courant. |
| Régression du panneau conversation | Vue contrôle indépendante, retour conservant agent, défilement et brouillon. |
| Concurrent update | Prévisualisation et application liées à une génération attendue; reçu uniquement après succès serveur. |

## Constitution check post-conception

PASS. Le design s'appuie sur les chemins d'autorité existants et ajoute seulement les types, routes et stockages nécessaires aux exigences. Les abstractions `ControlSetting` et `UsageRate` ont au moins trois responsabilités concrètes: contrat relay, validation serveur et rendu UI. Aucune dépendance n'est ajoutée. Les objets restent inspectables, supprimables et testables indépendamment.
