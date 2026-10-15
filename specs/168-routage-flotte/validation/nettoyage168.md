# Nettoyage de la session 168 — bilan d’examen T048

## État courant — 15/10/2026 à 17:24 CEST

**Implemented — 56/56 tâches cochées. Pipeline : Livré, avec conservations explicites T048.** ROOT a validé le contrôle final des 35 exigences, 11 critères et 7 histoires. Aucune tâche ne reste ouverte.

La récupération documentaire T047 est prouvée : C1 `db46c7816777199da7c41bb9bc10063cd361dae7` intégré et poussé, main Mac/Cartae alignés et propres aux reçus. Source binaire B et release v0.4.0 inchangées ; aucun rebuild. Preuves : /home/moi/.local/state/bridget-delivery168/git/integration-pushC1-cartae.json, SHA-256 `e32d1529d986fff8af4441928353d8b1a5fd163e4aefaf7e3b90651d9ce90454` ; /Users/moi/.cache/bridget-delivery168/git/integration-pullC1-mac.json, SHA-256 `4cf769d0707b2b28535a6f91052d786ff5c5b1c861738e645f6a20df84111766`.

L’examen T048 est terminé avec retraits et conservations motivées. Le worktree source Cartae et sa branche sont conservés : C1 fusionné/poussé, arbre propre et zéro ignoré ; garde lsof RC0 mais 4 337 octets stderr, donc observation incomplète. Aucun match observé ne vaut preuve d’absence d’usage. Aucun contournement ni suppression inventée. Les 18 anciens candidats conservent leurs motifs. Reçu : /home/moi/.local/state/bridget-delivery168/cleanup/t048-source-final-guard.json, SHA-256 `e2410c3dc3d838b1fb49cdb065f0a9c067cda717a105cce4d0458b94ea25cfc3`. Bilan : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/nettoyage168.md. Le contrôle final T049 est validé. Le SHA du commit documentaire final est vérifié hors de son propre contenu dans les reçus Git privés.

Les sections ci-dessous sont les observations historiques des opérations : leurs mentions T048 ouverte sont dépassées par ce bilan. Toutes DB, preuves, archives et sauvegardes restent protégées. Aucune nouvelle opération de nettoyage par cette rédaction.

---

État arrêté au 15 octobre 2026, 16:29 CEST. **T048 reste ouverte.** Ce document rassemble des preuves existantes. Sa rédaction n’a lancé aucun nettoyage, arrêt, contrôle de processus, test ou commande Git supplémentaire.

Source B intégrée et poussée sur les deux machines : `431b78f72bdd73eac4772af17bf36879d7a629d2`.

## Nettoyages effectués sur le Mac

Le nettoyeur existant `/Users/moi/Nextcloud/10.Scripts/64.bridget/scripts/build.py clean` a retiré **23,68 Gio logiques** de caches sous `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug` avant la construction B. Il a conservé `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release`, car des fichiers étaient utilisés. Commande RC 0, durée 14,32 s ; aucun retrait forcé.

Preuve : `/Users/moi/.cache/bridget-delivery168/cleanup/mac-project-cache-prune-before-buildB.json`.
SHA-256 : `fb97a663cf167df69a7ece7df0523ea9014cb87c6c3c65b60543813e06f5b0cc`.

Les repères de disque physique sont distincts des volumes logiques : environ **18,5 Gio disponibles avant**, puis **32,25 Gio après** selon le relevé de livraison. Le JSON ci-dessus enregistre précisément 32,25288009643555 Gio après. Cette variation ne mesure pas un gain physique exclusivement imputable au nettoyage ; les clones APFS et les autres opérations empêchent cette assimilation.

Deux anciennes copies de travail ont ensuite été retirées par `git worktree remove`, sans `--force` :

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/168-routage-flotte` ;
- `/private/tmp/bridget-audit168-source-20261015`.

La branche locale Mac `session-168-routage-flotte` a été retirée avec `git branch -d`. Le reçu final ne recense plus que le dépôt principal. Les autres copies de travail sont restées intactes.

Avant ces retraits, les contenus complets, y compris les fichiers ignorés, ont été archivés et vérifiés. Les modifications et fichiers non suivis ont aussi été conservés dans des remises Git temporaires (stashes). Aucun stash n’a été effacé. Les contrôles d’usage du projet ont été renouvelés avant les mutations ; ils ne constituent pas une garantie d’inactivité future.

Preuve de conservation : `/Users/moi/.cache/bridget-delivery168/cleanup-preservation-T048-20261015T161917/receipt.json`.
SHA-256 : `d2ef5bbd33baca446e28fefb1f0b6edcca7507c4e4e2d99b46fd33af9607cc7a`.

Archives conservées :

- `/Users/moi/.cache/bridget-delivery168/cleanup-preservation-T048-20261015T161917/mac-worktree168.tar.gz` — SHA-256 `c884990fe51441d1ec9ea70be8054993f905abab6a670269c7035d4223d81bee` ; 97 269 918 octets de fichiers source ;
- `/Users/moi/.cache/bridget-delivery168/cleanup-preservation-T048-20261015T161917/audit-source168.tar.gz` — SHA-256 `c3a1967943830492d66ff41f21c1b4c7292fc5eeb4b57f0c6cafe732ea1fe5de` ; 121 216 789 octets de fichiers source.

Manifestes conservés :

- `/Users/moi/.cache/bridget-delivery168/cleanup-preservation-T048-20261015T161917/mac-worktree168.tar.gz.manifest.json` — SHA-256 `222df8e77b4f6b19dc8f2de961ad8279e96524432be5035cd0b7de9dc6035be0` ;
- `/Users/moi/.cache/bridget-delivery168/cleanup-preservation-T048-20261015T161917/audit-source168.tar.gz.manifest.json` — SHA-256 `e66f14997e79a31c44a1b0ae61af8e6a0a2baedbf56c926b67f1ba0717f01795`.

Preuve des retraits : `/Users/moi/.cache/bridget-delivery168/cleanup-T048-two-worktrees-20261015T162414/receipt.json`.
SHA-256 : `f3b7b4ee7dbd034ec55277fe1c9bb454d537abde34b2b3493e5ff0ff9d937b3d`.
Volume retiré : **218 486 707 octets logiques** de fichiers source. Aucun gain physique n’est revendiqué. Les archives restent présentes. Les stashes `52c3ec5831ef24a0343cb5eb509c7ae19ebcb186` et `8847c19551678a8d61d177534f15c6e063bb4095`, ainsi que le stash antérieur, sont conservés.

## Dix-huit anciens candidats Cartae conservés

Aucun de ces 18 chemins n’a été retiré : neuf namespaces ne contiennent que des répertoires vides ; huit racines contiennent des DB, fichiers associés ou reçus protégés ; un cache de qualification contient des preuves possibles dont la dispensabilité n’est pas attestée.

La garde du projet `scripts/build.py::opened_paths` a lancé une seule observation `lsof -nP -Fpn`. Elle a obtenu RC 0, mais aussi 4 337 octets d’avertissements indiquant une sortie potentiellement incomplète. La garde est donc restée fermée. Aucun warning n’a été ignoré, aucun autre outil n’a remplacé cette garde, et aucune absence de PID, de handle ou de référence active n’est déduite de cette sortie.

Les 18 chemins exacts et leurs motifs de conservation figurent dans `/home/moi/.local/state/bridget-delivery168/cleanup/t048-old-candidates-attempt.json`.
SHA-256 : `c03eb859880668bae3e8abf614f20f29231af21e425af784fd4f5b3f2765a0c9`.
Volume logique conservé : **1 385 075 236 octets** ; volume logique retiré : **0 octet**. Aucun gain physique revendiqué.

## Processus des recettes privées Mac

Les reçus suivants attestent l’arrêt propre du parent de recette et l’absence observée des processus identifiés après arrêt ciblé. Les dossiers de preuves privés restent conservés ; « nettoyage » désigne ici les processus.

- Laya : `/Users/moi/.cache/bridget-delivery168/r168-b-laya-0c2c55/cleanup-runtime.json` — SHA-256 `570144914ec0f5e89cb0e1be90f785e3125268c8d921b58469ee43faefa66b6f` ; daemon 96580 et tunnel SSH 84590 absents après arrêt.
- Score : `/Users/moi/.cache/bridget-delivery168/r168-b-score-824a08/cleanup-runtime.json` — SHA-256 `d9cb1eef4f4f46de0d9fe622c6e442869f07fb3dfa230d6427c56ba2da640be6` ; daemon 32922 et tunnel SSH 16960 absents après arrêt.
- Haiku, tentative initiale avec repli : `/Users/moi/.cache/bridget-delivery168/r168-b-haiku-ee884a/cleanup-runtime.json` — SHA-256 `5bf262f03567ce8250a0ffb6f66ba372a00ec0e2144eef37f7dd7c0b9aaf59b0` ; daemon 90446 absent après arrêt ; aucun worker ni tunnel lancé dans cette tentative.

ROOT a aussi signalé l’absence observée des workers Cartae Laya 3772508 et Score 3784783. Le reçu durable complémentaire de ROOT reste attendu : les trois JSON Mac ci-dessus ne sont pas présentés comme une preuve autonome de ces deux observations distantes.

## Éléments encore en cours et protections

La copie de travail source Cartae `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte` et sa branche `session-168-routage-flotte` restent à traiter. La future topologie Cartae de la recette Haiku devra également recevoir son propre bilan. Ce document ne les déclare ni arrêtées ni retirées.

Toutes les DB et leurs fichiers associés, preuves, archives, sauvegardes de retour arrière, paquets A/B, configurations actives et ressources utiles aux opérations de livraison restent protégés. La copie de travail 163 et les travaux étrangers sont hors périmètre. **T048 n’est pas clôturée par ce reçu.**


## Checkpoint de clôture documentaire — 15 octobre 2026, 17:09 CEST

Ce checkpoint complète le reçu intermédiaire de 16:29. **T048 reste ouverte jusqu’au traitement de T047 par ROOT.** Les observations de cette tranche sont en lecture seule : lecture des reçus, existence de chemins précis, liste Git des worktrees et relevé ciblé des PID. Aucun retrait, signal, test, construction, écriture Git ou nouvelle garde globale n’a été exécuté.

### Dernière recette Haiku : processus privés arrêtés

Le parent réel `8f2f787f-9eb7-4ce9-8cd0-070f50c577a5` a été arrêté par la primitive native, RC 0. Le daemon privé Cartae 3 834 811, naissance `159155731`, et le relais 3 813 547, naissance `159042167`, ont reçu chacun SIGTERM après vérification de leur identité. Leur absence a été vérifiée après trois secondes. Les DB et les preuves restent conservées. Aucun service de production n’a reçu de signal.

Reçu : `/home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/cleanup-runtime-cartae.json`.
SHA-256 : `fbf22a7028aad700ddc05dd4345f9b974db7b4504c710ec2cc920f231552e2e9`.

Sur le Mac, le worker privé 74 195 était déjà absent et n’a reçu aucun signal. Le tunnel privé 74 191, né le 15 octobre à 16:41:23, a reçu SIGTERM après vérification de son exécutable `/usr/bin/ssh` et de sa naissance. Son absence a été vérifiée après trois secondes.

Reçu : `/Users/moi/.cache/bridget-delivery168/r168-mh-759bee18/cleanup-runtime-mac.json`.
SHA-256 : `829c9e658dae179945a8c48025745d11d39940867f86f4b591c9bd98a1641a8a`.

Les dossiers suivants restent des preuves privées ; l’arrêt de leurs processus ne constitue pas leur suppression :

- `/home/moi/.local/state/bridget-delivery168/r168-ch-759bee18` ;
- `/Users/moi/.cache/bridget-delivery168/r168-mh-759bee18`.

### Inventaire final borné

Observation le 15 octobre 2026 vers 17:08:52 CEST, soit 15:08:52 UTC. Les SHA des six reçus Mac et des deux reçus Cartae cités dans ce document ont été relus et concordent. Les tailles conservées restent celles des reçus ; aucun nouveau parcours volumétrique n’a été lancé.

| Ressource | État observé et motif |
|---|---|
| Deux anciennes racines Mac indiquées plus haut | Absentes. Retrait normal déjà attesté ; 218 486 707 octets logiques. Archives et stashes conservés. |
| Dix-huit anciens candidats Cartae | Les 18 chemins existent encore. Conservation de 1 385 075 236 octets logiques selon leur reçu. La garde incomplète reste une limite ; elle n’a pas été contournée. |
| Source Cartae `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte` | Worktree encore enregistré, HEAD B `431b78f72bdd73eac4772af17bf36879d7a629d2`, branche `session-168-routage-flotte`. Nécessaire aux documents de livraison ; retrait non effectué. |
| Worktree Cartae `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission` | Toujours enregistré ; hors périmètre 168, conservé. |
| Derniers daemon et relais privés Cartae | PID 3 834 811 et 3 813 547 absents au relevé ciblé. |
| Anciens workers privés Laya et Score Cartae | PID 3 772 508 et 3 784 783 absents au relevé ciblé. Ce constat complète les reçus Mac ; il ne prétend pas couvrir tous les descendants ou handles du système. |
| Derniers worker et tunnel privés Mac | PID 74 195 et 74 191 absents au relevé ciblé. |
| Production Mac | Daemon 71 927 né à 15:57:58 et pont 72 468 né à 15:58:00 présents. Aucun signal de cette tranche. |
| Production Cartae | Worker 3 767 260 présent, né à 13:58:00 UTC. Aucun signal de cette tranche. |
| T3 et tunnel de production | Application 65 544 née à 04:07:14 et tunnel 60 206 né le 13 octobre à 20:14:11 présents. Serveur T3 actuel 97 328, fils de 65 544, né à 16:51:02. Ce remplacement externe ne constitue pas une conservation de l’ancien PID 65 608 ; aucun signal T3 n’a été envoyé par ce lot. |

Les builds A et B existent toujours aux quatre chemins suivants. Ils ne sont pas déclarés dispensables par cette observation :

- `/Users/moi/.cache/bridget-release168/0.4.0-269de64e172f-mac` ;
- `/Users/moi/.cache/bridget-release168/0.4.0-431b78f72bdd-mac` ;
- `/home/moi/.local/state/bridget-delivery168/build/runs/0.4.0-269de64e172f-linux` ;
- `/home/moi/.local/state/bridget-delivery168/build/runs/0.4.0-431b78f72bdd-linux`.

Les préfixes Linux `/home/moi/.local/lib/bridget-communication/0.4.0-269de64e172f` et `/home/moi/.local/lib/bridget-communication/0.4.0-431b78f72bdd` sont conservés. B sert le worker actif ; A reste une conservation de retour arrière. Aucun nettoyage de ces exécutables, des paquets, des DB, des sauvegardes d’activation ou des preuves n’est autorisé par ce document.

Les namespaces Laya, Score et Haiku des deux machines restent conservés avec leurs DB et reçus. Leur inventaire a été borné aux enfants directs de `/Users/moi/.cache/bridget-delivery168` et `/home/moi/.local/state/bridget-delivery168`, puis aux noms de recettes déjà connus. Aucun scan global TMP, aucun `lsof` global supplémentaire et aucune recherche de processus fournisseur n’ont été lancés.

### Prochains candidats après T047

1. Après commit, intégration et push des documents, ROOT pourra traiter le worktree Cartae `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte` et sa branche `session-168-routage-flotte`. Il faudra renouveler l’ascendance, l’état Git, la conservation des ignorés et la garde d’usage avant un retrait normal sans force. L’état observé ici ne vaut pas autorisation de retirer des modifications ultérieures.
2. Les neuf anciens namespaces vides restent des candidats conditionnels : `/tmp/b168ag-0q8195ie`, `/tmp/b168ar2-z_4n4d0j`, `/tmp/b168p1-m6y_5jyp`, `/tmp/b168p2-h4hznyr5`, `/tmp/b168r1-v546hg07`, `/tmp/b168red-196p4nw8`, `/tmp/b168red-19c_770v`, `/tmp/b168red-_hq9u08s` et `/tmp/b168red-dvzzn_0w`. Leur garde complète doit d’abord réussir. Leur présence vide ne prouve pas l’absence d’usage.
3. Les huit racines contenant des DB ou reçus et le cache de qualification restent protégés, avec leurs chemins exacts dans le reçu des 18 candidats. Ils ne rejoignent pas une liste de suppression automatique. Les namespaces des recettes réelles, archives, stashes, sauvegardes et paquets actifs restent également conservés comme preuves ou moyens de retour arrière.

Ce bilan ne revendique aucun gain physique, aucun nettoyage global et aucune clôture anticipée de T048. Les conservations sont explicites et les retraits déjà faits restent traçables par leurs reçus.


## Bilan final T048 — récupération et conservation motivée, 15/10/2026 à 17:18 CEST

La récupération documentaire T047 est prouvée : C1 `db46c7816777199da7c41bb9bc10063cd361dae7` intégré et poussé, main Mac/Cartae alignés et propres aux reçus. Source binaire B et release v0.4.0 inchangées ; aucun rebuild. Preuves : /home/moi/.local/state/bridget-delivery168/git/integration-pushC1-cartae.json, SHA-256 `e32d1529d986fff8af4441928353d8b1a5fd163e4aefaf7e3b90651d9ce90454` ; /Users/moi/.cache/bridget-delivery168/git/integration-pullC1-mac.json, SHA-256 `4cf769d0707b2b28535a6f91052d786ff5c5b1c861738e645f6a20df84111766`.

L’examen T048 est terminé avec retraits et conservations motivées. Le worktree source Cartae et sa branche sont conservés : C1 fusionné/poussé, arbre propre et zéro ignoré ; garde lsof RC0 mais 4 337 octets stderr, donc observation incomplète. Aucun match observé ne vaut preuve d’absence d’usage. Aucun contournement ni suppression inventée. Les 18 anciens candidats conservent leurs motifs. Reçu : /home/moi/.local/state/bridget-delivery168/cleanup/t048-source-final-guard.json, SHA-256 `e2410c3dc3d838b1fb49cdb065f0a9c067cda717a105cce4d0458b94ea25cfc3`. Bilan : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/nettoyage168.md. Seul le contrôle final T049 et l’identification du futur commit documentaire restent à établir par ROOT.

Le worktree examiné est /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte ; branche session-168-routage-flotte. Le reçu indique removed=false et branch_removed=false. Le worktree163 /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission est protégé. Conservation motivée conforme à l’examen autorisé T048 ; volume retiré pour ce dernier candidat : 0 octet. Aucun gain physique revendiqué. T048 est cochée par ROOT ; T049 est validée par ROOT.
