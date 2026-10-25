# Archives techniques de l’adaptateur T3

Document d’origine : [specs/134-noms-humains-messages/implementation.md](../../../../../../specs/134-noms-humains-messages/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Correctif livré depuis `fix-134-profils-satures`, dans le worktree dédié.
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
- Commit source : `c1712153b484`, fusion en avance rapide dans main. Construction
  release verrouillée depuis main, dans une cible séparée de la production.
- Binaire remplacé atomiquement, SHA-256
  `8ab9d061c674521bc25a3a16f7b39c374cfd1cac8ecbf3fc71d57e751faca1ee`.
  Seuls les services `com.bridget.daemon` et `com.bridget.t3` ont été relancés.
  Le build-id du daemon est `c1712153b484` ; les deux services sont running.
- Avant : 38 fils T3, 33 sans nom. Après : 38 fils, zéro sans nom et zéro nom
  provisoire. Un nouveau fil apparu ensuite porte déjà son titre réel :
  contrôle frais à 39 fils, zéro sans nom.
- L'expéditeur du signalement porte maintenant `opus-city-coder-1`.
  L'envoi par ce nom est résolu au bon UUID. Recette `8f654d6acfdd4`, réponse
  `t3-b8b7ebac-07a4-441b-bf8e-ec7bb951e73d` contenant `NOM134 OK`.
  L'affichage final de la réponse dans l'interface n'a pas été inspecté :
  la réparation de l'annuaire, le trajet réel et le rendu par test sont vérifiés.
- Comparaison avec la sauvegarde : aucun corps ni expéditeur de ledger conservé
  n'a changé. Le nettoyage standard de rétention au redémarrage a expiré deux
  entrées du 28/09, âgées de plus de sept jours. Le log de purge le confirme.
  Ces deux entrées restent intégrales dans la sauvegarde SQLite. Aucune
  restauration ancienne ni modification de la politique de rétention.
- Limites : les anciens en-têtes T3 restent inchangés ; la nouvelle livraison
  utilise les profils réparés. Pas de nouvelle contre-revue externe revendiquée
  pour ce correctif, seulement self-review et preuves exécutées.

<a id="extrait-2"></a>

## Extrait 2

- Auditeur externe : agent Claude `29aaed9b-9f6f-4849-87a5-1a23bbe01948`.
- Verdict : `APPROVE_WITH_CHANGES`.
- Intégré : repli sur nom blanc et validation commune de la publication T3.
- Vérifié comme déjà satisfait : la branche part de `main` après SPEC-133.
- Première demande externe : expirée sans verdict. Aucune approbation inventée.

<a id="extrait-3"></a>

## Extrait 3

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

<a id="extrait-4"></a>

## Extrait 4

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

<a id="extrait-5"></a>

## Extrait 5

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
