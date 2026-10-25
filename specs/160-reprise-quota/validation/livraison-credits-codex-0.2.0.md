# Livraison du correctif crédits Codex et de Bridget 0.2.0

Le contrôle de quota tient désormais compte des crédits attestés. Une fenêtre incluse à 100 % reste affichée comme épuisée. Elle n’empêche plus un lancement lorsque le fournisseur confirme des crédits utilisables et aucun plafond de dépenses atteint. Une absence de preuve de crédits conserve le refus. Les quotas et les crédits restent distincts.

## Sources et fonctionnement livré

- Correctif intégré : `5872f48ec3267d8acb6d758bf6dcae4664a9a4f3`.
- Release sémantique : `v0.2.0`, commit `cd64d4ee07d7ad0c5ce2ff98d287ce6e9a8bb793`.
- Publication : https://github.com/guthubrx/bridget/releases/tag/v0.2.0
- Le delta des sources `crates` entre le correctif et la release est vide. Le commit de release porte la version et ses documents.
- Runtime installé : version 0.1.3, BuildID `5872f48ec326`. Ce runtime contient le correctif. La release publiée 0.2.0 constitue une preuve distincte. Aucun rebuild supplémentaire n’est requis pour rendre le correctif actif.
- Binaire Mac : /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget
- SHA-256 Mac : `3e3884b2e9bbd43e59a7a7cfe1c6148f0bb26618e3141045cc3095bf7504a43b`.
- Binaire Linux installé : /home/moi/.local/lib/bridget-communication/credits-5872f48ec326/bin/bridget
- SHA-256 Linux : `a6efc339200af5fec4c3626dedc977fead59e9b2150bee91f4a7a15db092faf0`.

## Paquet publié

- Publication GitHub : 14 octobre 2026 à 07:12:51 UTC.
- Tag annoté : `1373bc9bd065f214117ecfdce456f84edb4bee3e`, cible `cd64d4ee07d7ad0c5ce2ff98d287ce6e9a8bb793`.
- Archive : https://github.com/guthubrx/bridget/releases/download/v0.2.0/bridget-0.2.0-linux-x86_64.tar.gz
- SHA-256 archive : `3224755d9512db2bce4153868f853dbb62f2e973a3cb086226f091adcad9cd89`.
- SHA-256 binaire de release : `89f5c801e463bf73c3b504e33d18aa731ba8c2848e3bd37db1c91861700ebfc3`.
- L’API GitHub confirme le digest de l’archive ci-dessus et celui du fichier `.sha256` : `9c4094d42fc73b6816a28198377d61900d83dd701dd89817fd9c0dd0375ed43c`.
- Construction `--locked --offline --release`, deux jobs. Version du binaire empaqueté : `bridget 0.2.0`, BuildID correspondant au commit source. Sommes internes vérifiées par le développeur distant.
- Reçu de publication : /home/moi/.cache/bridget-release-0.2.0.kCU3Vv/receipt-publication.md

## Validation ciblée

- Parseur Codex : 1 PASS ; flux natif d’observation : 1 PASS ; roundtrip wire : 1 PASS.
- Compatibilité Claude : parseur 1 PASS et wrapper 1 PASS.
- ProviderLimits : 12 PASS.
- Reprise et mise en attente 160 : 55 PASS ; l’oracle de qualification modifié a ensuite été rejoué seul, 1 PASS.
- Admission distante 162 : 1 PASS sur la portée de l’instance cible dans la fixture. La tâche de release décrite plus bas constitue la preuve réelle distincte.
- Groupe native157 : 39 PASS, 5 FAIL, 1 ignored. Les cinq échecs ont été rejoués exactement sur la baseline propre : 5 FAIL identiques. Ils portent sur les prérequis Git et l’environnement, pas sur le budget. Aucun succès global n’est déduit.
- Contrôles finaux : roundtrip wire 1 PASS ; `cargo check -p bridget-daemon --tests`, format et `git diff --check` RC 0.
- Les compilations intermédiaires échouées et la première tentative de baseline contaminée par le cache sont conservées. Cette première tentative n’est pas comptée comme preuve. La baseline correcte et les contrôles finaux ont ensuite été exécutés avec les sources attestées.
- Aucun nouveau FULL n’a été exécuté. Les avertissements Rust préexistants restent documentés.

Les crédits sont conservés par type de moteur et instance, sans transfert entre comptes ou machines. Le solde n’est pas interprété. Les champs inconnus ou absents n’autorisent pas le lancement. Un contrôle de dépenses atteint bloque même après le reset d’une fenêtre. L’admission locale conserve sa limite : elle suppose l’environnement du profil déclaré et n’atteste pas le compte d’un futur processus distinct.

## Exécution distante réelle

La tâche native `acf5d5e4-8121-4953-b842-0cf5b6270337` a exécuté la release sur Cartae avec Codex. Le résultat durable atteste `result_available`, `result_sent=true`, `cleanup_done=true` et `guarded=false`. Le journal a montré l’exécution du travail distant. La disponibilité de crédits a donc été exercée par une tâche réelle, au-delà de la seule lecture du catalogue.

La sonde du compte annonçait un quota inclus à 100 %, `hasCredits=true`, `unlimited=false` et `spendControlReached=false`. Son mode d’authentification est ChatGPT. Le solde reste privé et n’est ni reproduit ni interprété dans ce reçu. Aucun changement de fournisseur, de profil, d’authentification ou de permissions n’a été nécessaire.

## Vérifications et limites des observations

La nouvelle observation est validée dans /Users/moi/.cache/bridget-credits-delivery-20261014/delivery-observation-1791963162372313000.json. Elle vérifie les binaires, signature Mac, exécutables des services, unité et lien CLI Linux, registre et credential. La base passe `quick_check` et ne comporte aucune erreur de clés étrangères ; versions idempotence 7 et exécutions 12. Le worker courant est 1740608, naissance 147622282 ; les services Mac sont 27267 et 27297. Le reçu conserve les observations historiques et distingue les processus courants.

Le premier ordre d’arrêt a provoqué le départ automatique du worker quand son maître a disparu. Le second passage a livré les binaires et démarré les services. Le premier worker `1740440` a ensuite quitté sur `worker_capability_unavailable` pendant le démarrage. Le worker `1740608` a pris la suite. Les erreurs et les anciens reçus restent conservés.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/160-reprise-quota/validation/livraison-credits-codex-0.2.0.md#extrait-1).

Fleet a pu évoluer lors de la création de la nouvelle tâche. Son égalité complète avec le préflight n’est pas déduite. Le registre et les LaunchAgents sont comparés aux preuves d’installation.

Le diagnostic MCP du parent est distinct du correctif et de l’exécution distante aboutie. Une fluctuation `t3_session_unavailable` ne remet pas en cause le résultat et les états de nettoyage natifs durables. Aucun succès global d’admission pour tous les profils n’est déduit.

## Nettoyage du lot

Les deux worktrees propres et fusionnés retirés dans ce lot sont :

- /home/moi/10.Scripts/64.bridget/.worktrees/fix-codex-credits-20261014
- /home/moi/10.Scripts/64.bridget/.worktrees/acf5d5e4

Ces deux worktrees et leurs branches ont été retirés sans force. Reçu : /Users/moi/.cache/bridget-credits-delivery-20261014/cleanup-owned-1791963312355198000.json. Les états natifs étaient terminaux. La garde a vérifié une commande de supervision fermée et 455 processus, sans descripteur ni processus dans les deux racines. Les commandes utilisées sont `git worktree remove` et `git branch -d`.

Le dossier /home/moi/.local/state/bridget-release-0.2.0-preflight/schema-codex-0.162.1 contient une preuve de schéma et reste conservé. Aucun dossier baseline supplémentaire n’a été identifié.

Le périmètre exact des stages et TMP propres figure dans /Users/moi/.cache/bridget-credits-delivery-20261014/cleanup-scope.json. Les binaires installés, preuves, sauvegardes, cache Cargo partagé, ancien worktree 163, connecteur 164 et cache de release /home/moi/.cache/bridget-release-0.2.0.kCU3Vv sont protégés. Aucun nettoyage global TMP n’est prévu.

Le nettoyage des stages et TMP a été exécuté. Reçu : /Users/moi/.cache/bridget-credits-delivery-20261014/cleanup-stages-1791963594344849000.json. Les deux répertoires Mac suivants ont été retirés :

- /Users/moi/.cache/bridget-credits-delivery-20261014/credits-mac-5872f48ec326-1791960481745542000-efp98osl/source
- /Users/moi/.cache/bridget-credits-delivery-20261014/credits-mac-5872f48ec326-1791960481745542000-efp98osl/target

La source Linux /home/moi/.local/lib/bridget-communication/credits-5872f48ec326/source a été retirée après conservation de son manifeste dans /home/moi/.local/lib/bridget-communication/credits-5872f48ec326/source-cleanup-manifest.json. Le manifeste porte 3 347 fichiers. Le binaire installé et son reçu de build sont conservés.

Les deux fichiers TMP ont été archivés byte-identiques avant leur retrait :

- /tmp/bridget-0.2.0-release-parent → /home/moi/.cache/bridget-release-0.2.0.kCU3Vv/cleanup-tmp/bridget-0.2.0-release-parent ; SHA-256 `855827493436ecbf1d23b7b0319c4437be913418f6621d2304b48799e0d0d888`.
- /tmp/bridget-0.2.0-metadata.json → /home/moi/.cache/bridget-release-0.2.0.kCU3Vv/cleanup-tmp/bridget-0.2.0-metadata.json ; SHA-256 `00335ce7eeb04b1bf50ada3b73fde65979f12242e5496888fcc25ea93c133b0d`.

Le binaire Mac signé de préparation reste disponible dans /Users/moi/.cache/bridget-credits-delivery-20261014/credits-mac-5872f48ec326-1791960481745542000-efp98osl/bridget. Il est conservé pour vérifier et reproduire l’installation. Les sauvegardes et preuves conservent les erreurs historiques et les états avant activation. Le cache Cargo partagé reste utile aux développements suivants. L’ancien worktree 163 et le connecteur 164 restent hors du lot. Le cache de release conserve les artefacts publiés et leurs preuves. Aucun service, configuration ou base de données n’a été modifié par ce nettoyage. Aucun gain physique de disque n’est revendiqué.

## Preuves complémentaires

- /home/moi/.local/state/bridget-release-0.2.0-preflight/credits-fix-handoff.md
- /home/moi/.local/state/bridget-release-0.2.0-preflight/credits-final-fixed.json
- /Users/moi/.cache/bridget-credits-delivery-20261014/backup-5872f48ec326/activation.json
- /Users/moi/.cache/bridget-credits-delivery-20261014/backup-5872f48ec326-r2/activation.json


## Empreintes des preuves ciblées

- /home/moi/.local/state/bridget-release-0.2.0-preflight/credits-fix-handoff.md : `bf7db030b18fc2c844fac05128defc22dff9482570d73279d4f4405abbf64a7d`.
- /home/moi/.local/state/bridget-release-0.2.0-preflight/credits-final-fixed.json : `f4681f0dc0f9b991906a7f60a1a368e6dd888a348e9c89409e38fe48b65dd479`.
- /home/moi/.local/state/bridget-release-0.2.0-preflight/credits-fix-manifest.json : `a083ae3b33064777f916dceddf965e27f291dba5a0942553aafe01150359b7c6`.
- /home/moi/.local/state/bridget-release-0.2.0-preflight/credit-probe.json : `32599d3a1929a7058b3e7646f638ddb26caf9dfc62d9c64b33ab25a04420066c`. Cette empreinte identifie la sonde privée sans publier son contenu sensible.
