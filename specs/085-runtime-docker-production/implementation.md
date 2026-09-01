# Implémentation - SPEC-085 Runtime Docker de production

## Preuves effectives

- Image de production construite le 2026-09-01 sur le serveur cible Linux amd64.
- Image locale attestée : `sha256:0b84145f04295f02640bfe892a2f30b024926a5a493be5dac405cb6c86badd67`.
- Le build compile réellement `/usr/local/bin/bridget` depuis le checkout et
  utilise le toolchain Rust 1.88.0 explicite. Un premier build a échoué avec
  Rust 1.85.0, ce qui a été corrigé avant de produire ce digest.
- Le conteneur de projet reste actif au repos via un exécutable interne borné ; les agents rejoignent ensuite ce même conteneur par `docker exec`.
- Le smoke test a été exécuté en rootfs lecture seule, UID/GID `1002:1002`, avec `/tmp` temporaire et sans socket Docker monté.
- Une inspection éphémère supplémentaire a vérifié le conteneur lancé avec
  `ReadonlyRootfs=true`, `CapDrop=[ALL]`, `no-new-privileges`, aucune liaison
  de port et uniquement deux bind mounts temporaires contrôlés. Les logs sont
  vides et l'environnement/les labels de l'image ne contiennent ni secret,
  ni jeton, ni mot de passe, ni clé API.
- Le socle ne contient ni client fournisseur ni secret. Les clients sont déclarés ultérieurement par politique fermée et ne sont jamais téléchargés à l'exécution de l'image.

## Décisions appliquées

- L'image est Linux amd64 seulement et sa base est référencée par digest.
- Le backend par défaut reste Host tant que le réglage serveur et une activation projet explicite ne sont pas complets.
- Les exemples de politiques sont non déployables tels quels : ils utilisent des répertoires `/srv/bridget/...` à remplacer et aucune référence de secret.
- Aucun service systemd ni daemon actif n'est modifié par ces preuves.
- `execution.default_backend` est un document local atomique, versionné et
  idempotent. Son installation absente migre vers `host`, sans lire ni réécrire
  aucune liaison de projet. Un défaut Docker exige une politique `id@version`
  et les quatre attestations : Docker, politique, catalogue et image.
- L'API de contrôle projette ces attestations avec une raison fermée. La
  création ou l'import affiche le backend et la politique résolus avant
  confirmation ; Host peut toujours être choisi explicitement.
- Le menu projet demande l'état runtime au daemon, propose uniquement les
  opérations fermées pertinentes et attend la réponse puis un rafraîchissement
  serveur avant de refléter un changement. Il ne transporte ni image, ni
  commande Docker, ni chemin libre.
- Une injection Docker qui échoue après `create` prouve la compensation : le
  conteneur est supprimé, l'ingress est retiré et le binding durable reste Host.
- La même compensation est maintenant exercée pour les échecs de création,
  démarrage et inspection du conteneur. Dans les trois cas, aucune liaison
  Docker ni ingress ne survit.
- La projection de capacité est calculée par le daemon puis réutilisée par le
  relais UI. Elle ne publie que `policy_unavailable`,
  `resource_catalog_unavailable`, `image_unattested` ou `docker_unavailable`.
- Les routes runtime vérifient jeton, corps strict, indisponibilité et état
  confirmé. La projection publique ne contient ni identifiant de conteneur ni
  digest interne d'image.
- Stop, remove, recreate et retour Host sont bloqués par un agent effectivement
  connecté. La garde s'applique à recreate avant la résolution de politique,
  afin qu'une configuration manquante ne masque jamais l'activité réelle.
- La projection runtime est testée avec `FAKE_SECRET=ne-doit-jamais-sortir` :
  elle retourne seulement `runtime_unavailable`; la valeur factice, le digest
  interne et l'identifiant de conteneur sont absents du JSON UI.
- Le rollback injecte maintenant cinq coupures : préflight Docker, montage,
  création, démarrage et inspection. Le binding persistant reste Host et
  l'ingress est retiré dans chaque cas.
- La fixture de redémarrage relit un conteneur existant et vérifie son image,
  ses labels de projet/génération/epoch et sa politique. Elle ne demande aucun
  `create`; une transition `creating` résiduelle devient
  `recreate_required`.
- Le test Docker d'intégration `project_runtime_agents_test` a exécuté deux
  agents runtime dans le même conteneur, avec deux projets distincts isolés.
  Les deux inscriptions et les arrêts ciblés passent.
- Une preuve Docker séparée a monté un dépôt Git avec deux worktrees aux mêmes
  chemins absolus, exécuté deux sessions successives dans un même conteneur,
  puis retiré le conteneur. Les états Git, branches, HEAD et worktrees avant et
  après sont octet pour octet identiques.

## Validation en cours

- Les tâches sont cochées uniquement quand le code, les tests ciblés et la preuve correspondante existent.
- Validation exécutée : contrôles statiques de l'image, build Linux amd64,
  smoke test rootfs lecture seule/non-root, test de compensation daemon,
  tests `project_runtime`, tests Store et protocole SPEC-085, 74 tests UI Rust
  et 130 tests Node du relais.
- Revue finale relue contre `reuse-audit.md` et `adversarial-review.md` : la
  limite Linux amd64 reste explicite, aucun socket Docker ni secret ne traverse
  le conteneur, et les agents d'un même projet partagent volontairement le
  domaine de confiance de leurs worktrees et de leur Git common dir.

## Validation opérateur restant volontairement manuelle

Le quickstart réel reste ouvert parce qu'il installe des catalogues opérateur,
peut redémarrer Bridget et active Docker pour un projet choisi. Le backend par
défaut reste `host` tant que l'opérateur ne valide pas explicitement ce
parcours.
