# Livraison SPEC168 — reçu initial avant commit A

Date : 15 octobre 2026. **Statut : In Progress.** Ce reçu décrit un développement vérifié dont la livraison reste à effectuer. Il ne déclare pas la session Implemented ou Livré.

## État observé

| Étape | État de ce checkpoint |
|---|---|
| Code et contrat | Réalisés dans le worktree de session ; source initiale 1f69e5d8e651dd27907fd244aee6f90b6b734565. |
| Tâches | 45/56 cochées après T036 ; cette observation ne modifie pas Tasks. |
| Converge et contre-revue | P3 CONVERGED ; T037 close après corrections et contrôles affectés. Verdict Sonnet conservé : APPROVE_WITH_CHANGES, sans prétendre à une seconde revue intégrale. |
| Audit T038 | Décision pass, release_blocked=false, A et compteur de pénalités 98 ; 14 contrôles, 0 critique/élevé/moyen actif, 2 LOW et 1 INFO. Axes qualité non notés. |
| FULL T036 | Exécutée une fois ; résultat et limites détaillés ci-dessous. |
| Documentation T039 | En publication. Son cochage reste à ROOT. |
| Version visée | 0.4.0, évolution compatible après 0.3.0 ; métadonnées et commit exact encore à fixer par T040. |
| Git du lot 168 | Pas encore committé, intégré ou poussé à ce checkpoint ; aucun SHA source futur n’est inventé. |
| Paquets 0.4.0 | Non construits ; aucune signature ni empreinte de paquet n’est anticipée. |
| Production | 0.3.0 inchangée par ce lot ; aucun service ni configuration de production modifié par ce checkpoint documentaire. |
| Qualification, activation et recettes réelles | À effectuer après les prérequis T041–T044. Les brouillons privés ne valent pas exécution. |
| Tag, release et nettoyage | Futurs ; aucune suppression de branche, worktree, TMP ou sauvegarde déclarée faite. |

Source de travail actuelle : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte

## Tests et limites conservées

La FULL unique termine RC101 en 775,777964 s : **2 820 PASS / 32 FAIL / 71 ignorés**. Les huit sommaires de harnais enfants sont exclus du total ; aucun double comptage de leurs PASS. Les sorties sont complètes et les sources restent stables pendant cette exécution.

Chronologie des reprises : neuf migrations PASS au même binaire FULL original, avec l’entrée de sauvegarde réelle auparavant absente ; lot r1 à 12 PASS / 3 FAIL / 8 différés ; lot r2 à exactement 8 PASS, feature existante test-support et umask 0077, sources stables. **29 des 32 échecs initiaux passent après reprise**. Les trois annulations 159 restantes dépendent de watch_marker/kqueue Darwin/BSD et ne sont pas validables sur Cartae Linux. Aucun ignored n’est ajouté. Les échecs et logs intermédiaires restent conservés ; aucun GREEN global n’est déduit.

Le replay instrumenté d’identité T3 passe sans atteindre le diagnostic conditionnel. La cause initiale de process_arguments=None reste inconnue. Les neuf fichiers modifiés après FULL ne changent que les tests et leur diagnostic ; le code produit reste inchangé. Deux contre-lectures indépendantes sont reliées au reçu T036, avec leurs limites.

Preuve de tests, noms exacts, logs et SHA :

/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md

Audit et archive privée canonique :

/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/audit168.md

/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/converge168.md

/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/adversarial-review-code168.md

L’audit conserve deux risques LOW rustls-webpki (RUSTSEC-2026-0098 et RUSTSEC-2026-0099) et une INFO de maintenance unsafe-libyaml (RUSTSEC-2024-0449). Aucun succès global de production ne vient du grade A.

## Source binaire et documents : éviter une référence circulaire

Le **commit A** doit fixer le code et les métadonnées de version autorisées par T040. Son SHA ne peut pas être écrit dans son propre contenu avant création. Ce reçu conserve donc ce champ non établi à ce checkpoint.

Après création et contrôle du commit A, ROOT consignera son SHA exact comme source des binaires. Les builds et paquets devront provenir de cet arbre propre. Les commits documentaires ultérieurs pourront référencer A et les hashes des paquets, puis les faits d’activation, de recette, de release et de nettoyage. Ils ne changeront pas rétroactivement la provenance des binaires. Un dernier commit documentaire ne peut pas s’autociter : son SHA sera vérifié séparément dans le reçu Git privé ou un checkpoint ultérieur.

Aucun commit, push ou changement de version n’est exécuté par cette mise à jour documentaire.

## Étapes restant à prouver

T040 : version et commit source exact ; T041 : builds/paquets/signatures/sommes ; T042 : intégration, push et rapprochement des checkouts propres ; T043 : qualification du registre et sauvegardes cohérentes ; T044 : installation/activation contrôlée et vérification des services, DB et configuration ; T045 : recettes réelles avec résultats corrélés et rejeu ; T046 : tag et release v0.4.0 vérifiés ; T047–T049 : reçus durables, nettoyage justifié et contrôle final.

Les données metadata_version=2 exigent une procédure de rollback compatible ; un ancien binaire ne doit pas ouvrir ces lignes. Les sauvegardes et le contrôle des schémas appartiennent au préflight d’activation à venir ; aucune restauration automatique de DB n’est annoncée.

Le statut final ne passera à Implemented/Livré qu’après preuve des étapes applicables. Le développement, les tests et l’audit seuls ne clôturent pas la livraison demandée.


## Checkpoint après commit source A — 15 octobre 2026

Le reçu initial ci-dessus reste historique. **Statut : In Progress, 47/56 tâches cochées par ROOT après T039 et T040.** La source et les métadonnées 0.4.0 sont maintenant committées. L’intégration, le push, la construction validée des paquets et l’activation restent à prouver.

- Commit source A : `269de64e172f71f8856373fe8b09cb2d8642da52`.
- Parent : `1f69e5d8e651dd27907fd244aee6f90b6b734565`.
- Périmètre : 74 fichiers. Manifeste privé : /home/moi/.local/state/bridget-delivery168/git/commitA-scope168-v2.json ; SHA-256 `6f0f2be3cad274cf4fca53edf04371b602d19a9efc3fcee5fb4e632aa63c2e79`.
- Reçu de création : /home/moi/.local/state/bridget-delivery168/git/commitA-receipt168.json ; SHA-256 `8b86e96f9fde684d63cd0ae14693b3d9ecb5e2366a01055fa4f7f666bb948eaf`. Il atteste l’arbre source propre, les blobs contrôlés et les hooks conservés ; `integrated=false`, `pushed=false` à ce checkpoint.

Le workspace et ses trois paquets portent la version 0.4.0. Les dépendances sont inchangées. Ce commit fixe la provenance des builds en cours ; les futurs commits de preuves documentaires pourront le référencer sans s’attribuer leur propre SHA ni changer la source binaire.

### Export de source propre

L’export du commit A contient **3 433 fichiers** :

- /home/moi/.local/state/bridget-delivery168/build/exports/0.4.0-269de64e172f/source.tar ; SHA-256 `e8036348d3e543cee260a5dc63d2e206280d5da4056f49af8e66316d53f83ab0`.
- /home/moi/.local/state/bridget-delivery168/build/exports/0.4.0-269de64e172f/export-manifest.json ; SHA-256 `9ca7fe06873edd6c9ffcf8998d35891d522b7449cb4474f6c4e0a114996bd232`.
- Reçu de commande : /home/moi/.local/state/bridget-delivery168/build/export168-269de64e172f.command.json ; SHA-256 `06be903716c44e6e80907a2ed10898e4db18f50000f86e99050194b060f02f57`.

Le helper relu omet exactement trois liens symboliques historiques de l’export, sans supprimer leurs originaux :

- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/audits/latest
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/149-sous-agents-lineage/validation/native-network-recipes/node_modules
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/149-sous-agents-lineage/validation/ui-recipes/node_modules

### Builds démarrés, résultats non anticipés

La construction Linux a démarré dans :

/home/moi/.local/state/bridget-delivery168/build/runs/0.4.0-269de64e172f-linux

La construction macOS a démarré dans :

/Users/moi/.cache/bridget-release168/0.4.0-269de64e172f-mac

Aucun succès de build, paquet, signature ou somme finale n’est présumé ici. Aucun build supplémentaire n’est lancé par cette écriture. Le lot n’est encore ni intégré ni poussé. La production 0.3.0 reste conservée ; aucun service ni configuration de production n’est modifié par ce checkpoint.

Les limites T036 restent inchangées : FULL unique 2 820 PASS / 32 FAIL / 71 ignorés ; 29 des 32 échecs initiaux PASS après reprise, trois annulations non validables sur Linux ; cause initiale T3 inconnue malgré replay PASS. Aucune recette réelle, release ni nettoyage n’est déclaré terminé.


## Checkpoint T042–T044 — source intégrée et activation vérifiée

Les checkpoints précédents restent historiques. Source A `269de64e172f71f8856373fe8b09cb2d8642da52` intégrée en ff-only et poussée sur main, puis rapprochée dans le checkout Mac propre. Reçus privés :

- /home/moi/.local/state/bridget-delivery168/git/integration-pushA-cartae.json — SHA-256 `2f0f8750f5b7a975281d11fe55a0ae06cf2280813e8fdba8d03666cd2de34fd0`, remote origin, distant vérifié et arbre durable propre.
- /Users/moi/.cache/bridget-delivery168/git/integration-pullA-mac.json — SHA-256 `1bb0f3d9741ecdc0f87d860d6b36e449910dcc4df8a386c648e4e714add03446`, remote github, distant vérifié et arbre durable propre.

La qualification des deux registres est PASS dans deux namespaces sans T3 ni appel modèle : catalogue Mac 27=17+10, worker 10=0+10 ; zéro Task/reçu/exécution privés, quick_check ok et FK0. Sauvegardes cohérentes effectuées. Reçu : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/preflight-activation168.md.

L’activation 0.4.0 est exécutée et suivie par les contrôles natifs et MCP : daemon Mac 47823, pont 47926 et worker Cartae 3712026, Build-id A ; registre ON, catalogue 27, DB 7/12 quick_check ok/FK0. T3 65544/65608 et tunnel 60206 sont conservés. Le premier freeze refusé avant mutation et le correctif de garde du renderer restent documentés. Aucun connection_epoch worker non exposé n’est inventé. Reçu : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/activation168.md.

Preuve finale : /Users/moi/.cache/bridget-delivery168/activation/runtime-followup168.json — SHA-256 `af31f8e20f63db8283cfb1b2cfde1cd26adb57bbe920cf3c339f1dbe63cdabdc`, phase active_verified. Le reçu d’activation antérieur active_pending_mcp_checks est conservé, sans réécriture.

État ROOT reçu avant cochage T043/T044 : 49/56. Cette rédaction ne modifie pas Tasks. **Session In Progress** : recettes privées réelles T045 encore à faire, release et nettoyage futurs. Les limites de tests et d’entitlement restent inchangées ; aucun GREEN global.


## Checkpoint T045B — correctif de délai validé, recettes non closes

État inchangé : 51/56, T045 ouverte. La première recette privée Laya sur deux cibles a produit une Task distante host cartae, au SHA 269de64e172f71f8856373fe8b09cb2d8642da52, avec résultat et rejeu PASS. Sa source de décision est toutefois deterministic avec caller_deadline : la voie Laya valide n’est pas prouvée.

Après redémarrage privé, ROOT rapporte la même Task, le même canon, le BLOB metadata et le résultat inchangés, avec une Task, un reçu et une exécution. Le payload binaire Task n’est pas identique ; seule sa projection métier est stable. Le compteur réel HTTP n’a pas été observé. Les refus initiaux avant admission restent distincts : CLI EPERM ; JSON sans project_id refusé avec zéro reçu, puis même clé avec project_id bridget admise par MCP. Les droits des fichiers temporaires privés ont été corrigés à 0600 avant le redémarrage privé ; aucun succès d’admission n’est tiré de ces refus.

Le plafond service fixe de 2 s est corrigé : service 1 à 30 s dans le budget global restant, défaut 2 s. Une seule sélection contrôlée exacte est PASS, avec trois sous-cas de délai et source stable SHA-256 5668719253fb44f96ccf28a878c952da559c0a3349f73adf0ebf56462c90da7c. Reçu : /home/moi/.local/state/bridget-delivery168/tests/t045-budget-service-config-r1.json ; SHA-256 738ac4d040cd13d59d5273821b229f323aa5da2cb406a0de991b0308f6f43149. Contre-lecture statique APPROVE selon ROOT ; détails dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md.

Les builds et l’activation A restent des faits historiques actifs. Le futur commit B n’est pas encore nommé ici. Ses paquets, son activation et les trois recettes réelles Laya, Haiku et score restent requis avant publication. Aucun nouveau build, activation, release, scanner V15, FULL ou passage P4 n’est présumé.
