# Journal d’implémentation — Noms humains dans les messages Bridget

## Métadonnées

- **Spec** : 134-noms-humains-messages
- **Branche** : session-134-noms-humains-messages
- **Démarré** : 2026-10-05
- **Terminé** : 2026-10-05
- **Statut** : Livré en production

## Reprise — saturation des noms provisoires, 2026-10-05

- Correctif en cours sur `fix-134-profils-satures`, dans le worktree dédié.
- Observation réelle : les 1 000 noms `Agent` à `Agent (1000)` sont occupés.
  Le profil de `29aaed9b-9f6f-4849-87a5-1a23bbe01948` est absent. Le pont
  connaît son titre `opus-city-coder-1` mais reçoit `IdentityUnavailable`.
- Cause confirmée : `available_display_name()` limite les essais à 1 000.
  L'erreur `noms affichés épuisés` est ignorée par l'enregistrement.
- Limite de la livraison initiale : les tests couvraient le profil absent,
  pas la saturation. La recette avec `3D-collision` ne prouvait pas la
  réparation de tous les fils. La clôture initiale était donc trop large.
- Test rouge : `spec134_profils_satures_se_reparent_puis_prennent_le_titre_t3`
  échoue sur `Invalid("noms affichés épuisés")` avant correction.
- Correctif : une lecture des noms occupés, un ensemble en mémoire, au plus
  p+1 variantes pour p profils ; aucun plafond arbitraire. Le suffixe reste
  dans la longueur maximale, y compris pour les noms Unicode de 80 caractères.
  Les erreurs de création du profil sont désormais journalisées.
- Tests verts : création et réparation sur schémas actuel/historique,
  idempotence, titre T3, Unicode et réutilisation d'un nom libéré ; test du
  chemin daemon Register → DisplayNameSet → profil → enveloppe T3.
- Six tests ciblés daemon SPEC-134 passent. Le premier passage ciblé sans
  TMPDIR court avait une erreur de harnais `socket trop longue` ; relancé
  avec une racine temporaire courte et privée, il passe sans mutation produit.
- Self-review XVIII/XIX/XX : deux fonctions de production existantes modifiées,
  aucune table ni dépendance ajoutée. Une allocation est O(p) en temps/mémoire.
  Les chemins de lecture ordinaires restent inchangés. Les erreurs de stockage
  ne coupent pas le routage ; elles sont visibles, sans contenu de message.
- `cargo fmt --all -- --check`, `git diff --check` et Clippy tous targets sans
  avertissement passent sur l'état final. Les tests complets passent avec une
  racine isolée : 44 tests core, 1 005 tests daemon (10 ignorés), 294 tests
  transport (1 ignoré), intégrations et doctests sans échec. L'intégration
  Register ajoutée après la compilation du passage complet a passé parmi les
  six tests ciblés frais ; le code de production n'a plus changé.
  Journal complet : `/tmp/b134.yaTk9E/tests.log`.
- Les messages historiques ne sont pas réécrits. Sauvegarde cohérente SQLite,
  binaire et plists avant livraison dans
  `/Users/moi/.cache/bridget-deploy-backups/spec134-saturation-H3XqQq`.

## Revue du plan

- Auditeur externe : agent Claude `29aaed9b-9f6f-4849-87a5-1a23bbe01948`.
- Verdict : `APPROVE_WITH_CHANGES`.
- Intégré : repli sur nom blanc et validation commune de la publication T3.
- Vérifié comme déjà satisfait : la branche part de `main` après SPEC-133.
- Première demande externe : expirée sans verdict. Aucune approbation inventée.

## Progression

### T001 — Préflight et conception

- **Statut** : Complété
- **Base** : `main` au commit `54414d391493ee232f40a2048d79487f800db5d5`
- **Worktree** : branche `session-134-noms-humains-messages`, état isolé
- **Tests exécutés** :
  - `cargo test -p bridget-core message` : 13 tests passés
  - `cargo test -p bridget-daemon spec110` : 3 tests passés
  - `cargo test -p bridget-daemon spec114_lot` : 3 tests passés
- **Analyse SpecKit** : 9 exigences sur 9 couvertes par 7 tâches ; aucune
  incohérence, ambiguïté ou violation constitutionnelle détectée.
- **Notes** : le runtime officiel SpecKit ne fournit pas ses scripts ou modèles
  dans ce dépôt. Les artefacts ont suivi les formats existants du projet.

### T002 — Libellé commun

- **Statut** : Complété
- **Fichier** : `crates/bridget-core/src/message.rs`
- **Test rouge** : 3 tests `spec134`, 2 échecs prouvant que le nom était ignoré.
- **Test vert** : 3 tests `spec134` passés ; le test de compatibilité `spec133`
  passe aussi.
- **Résultat** : le nom nettoyé précède l’UUID complet. Les noms absents, vides,
  blancs ou égaux à l’UUID gardent le format historique. La provenance déléguée
  reste après le parent.
- **Self-review XIX/XX** : une seule fonction centrale change. Aucun helper,
  état, dépendance ou branche hors besoin. Le calcul reste O(1).

### T003 — Lots T3

- **Statut** : Complété
- **Fichier** : `crates/bridget-daemon/src/t3code.rs`
- **Test rouge** : le lot affichait les deux UUID sans les noms attendus.
- **Test vert** : le nouveau test `spec134` et les 3 tests `spec114_lot`
  passent.
- **Résultat** : chaque élément du lot appelle le même `sender_label()` que le
  rendu unitaire.
- **Self-review XIX/XX** : une seule expression remplace un accès direct
  divergent. Aucun format parallèle n’est ajouté. Le lot reste O(n), avec sa
  borne existante.

### T004 — Réparation des profils

- **Statut** : Complété
- **Fichier** : `crates/bridget-daemon/src/agent_profile.rs`
- **Test rouge** : une identité conservée sans profil restait avec zéro profil
  après deux appels à `ensure_agent_ids()`.
- **Test vert** : 1 test `spec134` puis les 9 tests du module de profils passent.
- **Résultat** : une seule lecture joint identité et profil. La transaction crée
  l’identité si nécessaire, crée le profil s’il manque, puis garantit l’état
  d’application avec un insert idempotent.
- **État transitoire attendu** : le profil réparé reçoit d’abord un nom unique
  `Agent N`. Le pont T3 republie ensuite le titre du fil. L’en-tête transitoire
  `Agent N (UUID)` est sûr et ne change jamais l’identité routable.
- **Self-review XIX/XX** : le contrat de la fonction existante est renforcé.
  Aucune migration, table, commande ou dépendance. Complexité O(n), avec une
  requête de lecture et des écritures conditionnelles par identifiant.

### T005 — Validation

- **Formatage** : `cargo fmt --all -- --check` passé.
- **Tests SPEC-134** : 6 tests passés. Ils couvrent le libellé, les replis, la
  provenance, les lots, la réparation idempotente et l’autorité du profil.
- **Régressions ciblées** : 3 tests `spec114_lot` et 9 tests `agent_profile`
  passés.
- **Espace de travail** : 44 tests `bridget-core` passés. Le module principal
  `bridget-daemon` a passé 1001 tests, avec 10 tests ignorés et 2 tests de CLI
  dépendants de l’identité du processus parent. Ces deux tests ont ensuite
  passé séparément dans un `BRIDGET_HOME` privé sans identité héritée.
- **Cadre stable macOS** : `TMPDIR` court et privé, masque `077`, tests
  sérialisés. Ce cadre évite la limite `SUN_LEN` et les fichiers temporaires
  partagés. Les échecs initiaux venaient du harnais, pas du code SPEC-134.
- **Analyse statique** : `cargo clippy --workspace --all-targets -- -D warnings`
  passé.
- **Construction** : `cargo build --locked --release -p bridget-daemon` passé.

### T006 — Revue et convergence

- **Contre-revue externe** : `APPROVE_WITH_CHANGES` par l’agent Claude
  `29aaed9b-9f6f-4849-87a5-1a23bbe01948`.
- **Condition validation** : satisfaite par T005.
- **Condition anti-usurpation** : satisfaite par le test
  `spec134_le_nom_livre_vient_du_profil_et_jamais_du_message_entrant`. Un faux
  `from_display_name` client est remplacé par le nom du profil.
- **Condition documentaire** : le nom transitoire `Agent N` est documenté dans
  T004 ci-dessus.
- **Convergence** : spec, plan, tâches, contrat, journal et résultats décrivent
  le même comportement. Aucune exigence orpheline ni tâche ouverte avant la
  livraison.
- **Audit final** : aucun défaut fonctionnel ou de sécurité ouvert dans le
  périmètre. Le nom reste une projection. L’UUID reste l’autorité.

## Self-review Article XIX/XX

- Pourquoi cette solution est nécessaire : l’UUID seul rend les messages
  difficiles à attribuer. Le nom existe déjà dans le profil et dans le message.
- Pourquoi elle est maintenable : une seule fonction construit le libellé. Les
  lots appellent la même fonction. La réparation reste dans la transaction
  existante.
- Hypothèses prises : le nom est informatif et peut changer. Le daemon reste
  l’autorité qui enrichit le message au moment de la remise.
- Vérifications réalisées : tests rouges puis verts, régressions ciblées,
  espace de travail, formatage, Clippy, release et contre-revue externe.
- Vérification de livraison : les deux services installés ont redémarré et un
  message réel a été reçu avec le nouveau binaire.
- Code supprimé ou évité : aucun format parallèle, aucun annuaire, aucune table,
  aucune migration globale et aucune dépendance.
- Complexité ajoutée : O(1) par libellé. La réparation reste O(n) pour n
  identifiants enregistrés.

### T007 — Livraison

- **Fusion locale** : branche fusionnée en avance rapide dans `main`.
- **Historique distant** : l’historique GitHub séparé a été rattaché par une
  fusion sans perte et sans `force push`. Le contenu local validé reste la
  version de référence.
- **Construction de production** : binaire release verrouillé construit depuis
  `main` dans un répertoire isolé. SHA-256 :
  `8e7fddc4f48458ec4104f2332ef858b57c0d214cbf7548abc8e12c9ec2910301`.
- **Sauvegarde** : le binaire précédent est conservé dans
  `/Users/moi/.cache/bridget-deploy-backups/bridget-before-spec134-20261005-081337`.
- **Déploiement** : remplacement atomique du binaire actif. Seuls
  `com.bridget.daemon` et `com.bridget.t3` ont été redémarrés.
- **Santé** : les deux services sont `running`. Le daemon publie le build-id
  `e057c41f9f54`.
- **Recette réelle** : la demande `mcp-97667-6ac34040-5` a reçu `SPEC134 OK`
  de l’agent nommé `3D-collision`, identité
  `fdef03b0-c651-4f5b-8f2e-3f14ec8218c1`.
- **Publication** : `main` a été poussé vers
  `https://github.com/guthubrx/bridget.git` sans réécriture d’historique.
- **Contre-revue finale** : la demande de second passage a expiré. Aucun verdict
  favorable n’est inventé. Les trois conditions de la contre-revue précédente
  sont corrigées, testées et consignées ci-dessus.
