# Archives techniques de l’adaptateur T3

Document d’origine : [specs/162-execution-distante/validation/preflight162.md](../../../../../../../specs/162-execution-distante/validation/preflight162.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Worktree Bridget /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante, branche session-162-execution-distante, base84cd24305837b70f866ae09bff86235b9743071e. Seule modification alors relevée : .specify/feature.json, sélection162 intentionnelle ; le worktree ne doit donc pas être présenté comme strictement propre à cet instant.
- Worktree T3 /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant, branche session-162-lineage-distant, based833387203e113ec2a24f947ba8b8064a1c2542b, propre au relevé initial.
- Cibles autorisées : Bridget github/main sur https://github.com/guthubrx/bridget.git ; T3 fork/local/main-20261009 sur https://github.com/guthubrx/t3code.git. Aucun push upstream ni PR T3. Identité Git utilisateur à conserver.

<a id="extrait-2"></a>

## Extrait 2

- /Users/moi/Nextcloud/10.Scripts/64.bridget : branche main, HEAD 84cd24305837b70f866ae09bff86235b9743071e, checkout durable propre=true.
- /Users/moi/11.Repositories/t3code-local : branche local/main-20261009, HEAD d833387203e113ec2a24f947ba8b8064a1c2542b, checkout durable propre=true.

<a id="extrait-3"></a>

## Extrait 3

- Ressources du lot : les deux worktrees/branches162 ci-dessus, dépendances T3 isolées dans leur worktree, preuves /Users/moi/.cache/bridget-delivery162-20261013, artefacts specs162. Aucun préfixe Linux162, credential162 ou service worker activé n’est attesté par ce préflight.
- Bridget : deux entrées worktree observées, checkout durable main et worktree162. Le checkout durable est conservé.
- Git commun T3 : 18 entrées worktree observées. Une appartient à162 ; les 17 autres sont préexistantes ou hors lot. Leur simple présence ne prouve ni abandon, ni fusion, ni absence de processus. Elles sont toutes préservées ; aucune candidate au nettoyage162 sans mandat séparé.

<a id="extrait-4"></a>

## Extrait 4

- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code — refs/heads/fix/codex-sqlite-import.
- Hors lot : /Users/moi/.codex/worktrees/t3-checkpoint-timeout — refs/heads/fix/checkpoint-add-timeout.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local — refs/heads/local/main-20261009.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/149-correctif-identite — refs/heads/session-149-correctif-identite.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/001-codex-import — refs/heads/fix/codex-sqlite-session-import.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/001-pr-branch-isolation — refs/heads/001-pr-branch-isolation.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/001-provider-icons — refs/heads/feat/provider-instance-icon-colors.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/002-local-build-pipeline — refs/heads/002-local-build-pipeline.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/002-provider-icons-v0.0.40 — refs/heads/local-patch/provider-instance-icon-colors-v0.0.40.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/003-compact-thread-cards — refs/heads/003-compact-thread-cards.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/003-compact-thread-cards-pr — refs/heads/feat/compact-thread-cards.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/003-compact-thread-cards-v0.0.40 — refs/heads/local-patch/compact-thread-cards-v0.0.40.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/004-chat-width — refs/heads/004-chat-width.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/005-sidebar-multi-drag — refs/heads/session-005-sidebar-multi-drag.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/005-sidebar-multi-drag-pr — refs/heads/fix/sidebar-multi-drag.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/local-evolutions-v0.0.42 — refs/heads/local-patch/evolutions-v0.0.42.
- Hors lot : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/local-evolutions-v0.0.44 — refs/heads/local-patch/evolutions-v0.0.44.

<a id="extrait-5"></a>

## Extrait 5

- Maître Bridget PID20937, naissance13octobre00:25:51 ; pont PID21097, naissance00:25:54.
- Tunnel SSH PID34764, naissance12octobre17:13:12.
- Application T3 PID60116 et serveur PID60228, naissances11octobre01:13:35/36.
- Aucun processus BridgetCartae observé à ce contrôle161 (pgrepRC1).

<a id="extrait-6"></a>

## Extrait 6

Ce document ne relit pas ps et ne revendique pas que ces PID sont encore actifs aujourd’hui. Avant livraison/activation162, ROOT devra relever les processus réels et leurs naissances. Ne jamais utiliser ces nombres historiques comme cibles de kill. Aucun redémarrage T3 autorisé ; livraison T3 sur disque uniquement, runtime chargé distingué. Le tunnel existant n’a pas à être relancé.

<a id="extrait-7"></a>

## Extrait 7

T3 : installation isolée offline/frozen déjà acquise dans le worktree162, Node24.13.1/pnpm11.10.0, RC0 en194,208s, 2000paquets ajoutés, zéro téléchargement. Manifests/lock inchangés et métadonnées du node_modules durable inchangées. Contrôle6452symlinks dont106workspace : aucun lien vers les anciennes sources durables, aucun lien cassé. Scripts ignore-scripts : prepare/patch Effect/postinstall natifs non exécutés ; cette préparation ne prouve pas encore tests/build/package.

<a id="extrait-8"></a>

## Extrait 8

- FR-024 : racines protégées et credentials identifiés, sans prétendre isoler des fournisseurs mêmeUID full-access.
- FR-026 : distinction préparation/livraison, sauvegarde cohérente future obligatoire ; aucune compatibilité downgrade avec état162 présumée.
- FR-027 : T3 autonome pour son runtime et interdit de relance préservé ; aucun statut «installé/actif162» revendiqué.
- FR-028 : propriété des ressources162 et protection des travaux étrangers explicites.
- SC-011/SC-012 : provenance et limites réutilisables pour les contrôles futurs ; ce document n’est pas une recette de production.

<a id="extrait-9"></a>

## Extrait 9

| Chemin absolu | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/initial162.json | b08d67e9f31cceb77f9dbeba6b3b8555f6617256f7f2a54147b6bf5d11b2148b |
| /Users/moi/.cache/bridget-delivery162-20261013/cache-preflight.txt | e4077e32ef2ef78df2a8ad59372b1892a41a67b39f8d1f9bf469f01098d16c1d |
| /Users/moi/.cache/bridget-delivery162-20261013/preflight/cartae.json | 65bab8a9f504d1a3e30be89731f961638bdcc0071e3f41f0243dd0394afa4f79 |
| /Users/moi/.cache/bridget-delivery162-20261013/preflight/mac.json | 8c17dee78f3f3501ef09e509278baec5f1a4843bb5af1d379e223cc0a99e53fd |
| /Users/moi/.cache/bridget-delivery162-20261013/preflight/summary.json | 1fe4dce0561e3e51ed855535b4b6594e85c4124c3604aac76217b7694fbf121a |
| /Users/moi/.cache/bridget-delivery162-20261013/preflight/cartae-worker-supervisor.json | 736d736d71e35cc8dbdbfc9723653149bc00a67117a489e293223f15a51e5b92 |
| /Users/moi/.cache/bridget-delivery162-20261013/preflight/cartae-worker-supervisor-failed-units.json | 97cd0944b2faae6c0933ff02a4ed88880916e682251c6221cd68e1ebff9d7cc3 |
| /Users/moi/.cache/bridget-delivery162-20261013/preflight/t3-install-offline.json | 9680d4d79bd9101d04cf96c12511907a1bb2e4d59c6af9177ea5d31c5d0849ea |
| /Users/moi/.cache/bridget-delivery162-20261013/preflight/t3-workspace-links.json | 15fc0c9102f5c1152e0ee6bee7a1e7d7de343306342ce55a4840437a238ea9b9 |
| /Users/moi/.cache/bridget-delivery161-20261013/git-final161.json | 9dcf1755fa147ac8ba03a8d0d505a8103f0efd3ed02286511fe4445adee60892 |
| /Users/moi/.cache/bridget-delivery161-20261013/runtime161-final.json | b0478577ecea393f4545c286cfa350c7140fe49683017e7f8f310c69fe78f10f |
