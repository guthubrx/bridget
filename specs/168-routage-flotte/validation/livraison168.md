# Livraison SPEC168 — reçu courant avant clôture documentaire

## État courant — 15/10/2026 à 17:13 CEST

**In Progress, 53/56 tâches cochées par ROOT.** T045 et T046 sont closes. T047, commit documentaire C1, est en cours ; T048 et T049 restent ouvertes. Le nettoyage final de la source appartient à ROOT. Aucun statut Livré ou Implemented n’est annoncé.

La source binaire B `431b78f72bdd73eac4772af17bf36879d7a629d2`, version 0.4.0, est construite et activée sur Mac/Cartae. Les trois voies réelles ont résultats et rejeux vérifiés : Laya `decision_service` sur 2 candidats, Haiku `lightweight_model` sur 21 candidats, dernier repli `deterministic` sur 21 candidats. Détail : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/recette168.md.

**Release publiée et vérifiée :** https://github.com/guthubrx/bridget/releases/tag/v0.4.0 . Tag annoté `43202c9de66ce466b76ba05bd3077699ac88ca3d`, cible exacte B ; quatre actifs retéléchargés et hashes conformes. Reçu : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/release168.md, SHA-256 `15a7b44610ebe060c47164e6c511606e67c37c4d2e4e85a71c2db5be879b9f5d`. Preuve privée : /home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/publication168.json, SHA-256 `d815ca75ca354ec4941577cb39a29aab62cf641ac71895f2ba69e3f0c8ea9666`.

Santé finale reçue : B en ligne, catalogue 27, pont 32 fils vers 97328 ; DB readonly quick_check ok/FK0, versions 7/12 ; worker B 3767260 actif, configuration inchangée. Application T3 65544 et tunnel 60206 préservés ; le serveur 65608 a été remplacé extérieurement par 97328 né à 16:51:02 CEST, sans signal T3 de notre part. La restriction temporaire de 16:56 CEST est levée.

Preuves existantes :

- /Users/moi/.cache/bridget-delivery168/final-health/mac-health.json — SHA-256 `be3ec722c86f8fcdd6de90353e1748e318b0de3beb086690cc66fdd276d114ba` ;
- /Users/moi/.cache/bridget-delivery168/final-health/database-health.json — SHA-256 `98feb24442412b8e2ce12d5c6a92e75605c500bd6f67b4e7fc349fc6cebab147` ;
- /home/moi/.local/state/bridget-delivery168/final-health/cartae-health.json — SHA-256 `a409e7698a2f7ff7bdede77a2a2126eeda0d835052765473f0ef60850818f32a`.

Limites : FULL unique 2820 PASS/32 FAIL/71 ignorés ; 29 échecs initiaux PASS après reprises, trois annulations 159 non validables sur Linux, cause initiale T3 inconnue malgré replay PASS. Deux LOW rustls-webpki et une INFO unsafe-libyaml restent documentés ; aucun GREEN global. Aucun nouvel audit/FULL/P4/build/service/test/modèle par cette rédaction. Aucun retour direct au binaire 0.3.0 après reçu admission v2. Les commits documentaires descendants conservent la source binaire B et ne nécessitent aucune reconstruction.

Cet état daté prévaut sur les checkpoints historiques ci-dessous, dont les anciennes mentions de publication en préparation. Les DB, preuves et sauvegardes sont protégées.

---

> État courant : source B 431b78f72bdd73eac4772af17bf36879d7a629d2 ; checkpoints A historiques.
> Référence : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/livraison168.md (section « Checkpoint B courant »).

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


## Checkpoint B courant — source intégrée, paquets et activation

**État courant : In Progress, 51/56 au mandat ROOT.** Aucun cochage effectué ici. Source binaire active et source des paquets : `431b78f72bdd73eac4772af17bf36879d7a629d2`, version 0.4.0. Les checkpoints A ci-dessus sont historiques ; les futurs commits documentaires ne changent pas cette provenance et ne s’attribuent pas leur propre SHA.

Source B intégrée/poussée en ff-only et checkouts durables rapprochés :

- /home/moi/.local/state/bridget-delivery168/git/integration-pushB-cartae.json ; SHA-256 `14a6e094fd6cec68108304505ebf9a020955523cd89de46496ead5716f0a3434` ; remote origin, HEAD et distant égaux B, arbre propre au reçu.
- /Users/moi/.cache/bridget-delivery168/git/integration-pullB-mac.json ; SHA-256 `7197a15dbc8930f4907a56c59cca558be0f0ad4db2bd30bc948cbeea55b0d361` ; remote github, HEAD et distant égaux B, arbre propre au reçu.

Les builds B Linux et Mac sont RC0, issus du même manifeste de paquet source `23088ff43c90289f511827466e641855321adc798c950343bab7228ac3c540fc`. Binaire Mac signé `3502daf850bae303a9ad0f4bdd2dbc073f6b36fdd51ced8ac62f52014aa15bed`, Linux `3b86b8fd48698e567d5f190a67f946d84d99fecd4526441ba6702bba9c3bf95a`. Paquets, sommes et hashes de reçus : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/paquets168.md (checkpoint B courant).

Les deux registres B sont qualifiés PASS : catalogue 27/17/10 côté Mac, 10/0/10 côté worker, zéro Task/reçu/exécution privés, quick_check ok/FK0. Seuls les délais du coordinateur local changent à 30 s / 20 s ; Mac A d104f2… devient B `59802f1a8e42bdca8fc01935b0638f448f493a8dcf30973bf46d0f846ef7bf47`, worker `bac0293a069ed4d90a3fcff7feb439c42fd5c49d5b1a3d86ad605a975bfe1083` inchangé. Les connexions/authentifications et 27 options restent conservées. Préflight et sauvegardes B réels : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/preflight-activation168.md.

Activation B active_verified : daemon 71927 né 15:57:58 CEST, pont 72468 né 15:58:00, worker 3767260 naissance 158783879. Build-id B, registre ON et catalogue 27=17+10 ; DB quick_check ok/FK0, schémas 7/12. T3 65544/65608 et tunnel 60206 conservés. Statut natif en ligne, pont running, MCP who réussi sans avertissement de projet. Le reçu d’activation active_pending_mcp_checks reste conservé ; le suivi séparé ferme ces contrôles.

/Users/moi/.cache/bridget-delivery168/activation/B-runtime-followup168.json ; SHA-256 `6d50c3181012c13b2334722f3e3ae13468800c0f8c75d988cfeca27be2c8f86e`.

Détail des services et preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/activation168.md.

État ROOT au présent mandat : 51/56, sans cochage par cette rédaction. Les recettes réelles T045 sur B sont en cours ; la release n’est pas publiée. La session reste In Progress. Les limites de T036 sont conservées : FULL unique 2 820 PASS / 32 FAIL / 71 ignorés ; 29 des 32 échecs initiaux PASS après reprise ; trois annulations 159 non validables sur Linux ; cause initiale T3 inconnue malgré replay PASS. Aucun GREEN global. Aucun résultat de recette encore en cours, tag, asset publié ou nettoyage n’est anticipé.


> Historique importé sans réécriture depuis /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/168-routage-flotte/validation/livraison168.md. La restriction décrite ci-après est levée ; voir la mise à jour finale.

## Reprise du 15 octobre 2026 à 16 h 56 CEST — source B, accès restreint

Les checkpoints précédents restent historiques. **Statut : In Progress, 51/56 tâches cochées.** T045 à T049 restent ouvertes. Ce complément documentaire local ne remplace pas les documents plus récents non committés du worktree Cartae.

### Constats relus sur le Mac pendant cette reprise

- HEAD et la référence locale github/main désignent tous deux `431b78f72bdd73eac4772af17bf36879d7a629d2`. Cette comparaison locale ne constitue pas une nouvelle lecture du distant GitHub.
- Le binaire installé /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget porte le SHA-256 `3502daf850bae303a9ad0f4bdd2dbc073f6b36fdd51ced8ac62f52014aa15bed`.
- Le reçu d’activation B a été relu : version 0.4.0, phase active_verified, catalogue 27, routage activé, DB quick_check ok/FK0. Il constate les PID daemon 71927, pont 72468 et worker Cartae 3767260 au moment de l’activation. Ces PID ne sont pas revalidés par cette reprise.

Reçu d’activation : /Users/moi/.cache/bridget-delivery168/activation/B-runtime-followup168.json

SHA-256 : `6d50c3181012c13b2334722f3e3ae13468800c0f8c75d988cfeca27be2c8f86e`.

| Recette B relue | Décideur réel | Candidats | Résultat et rejeu après redémarrage privé |
|---|---|---:|---|
| Laya | decision_service | 2 | Task 5b08fb9c-1900-40ca-9757-4e268d82d0bb ; résultat Cartae au SHA B, ACK présent, cleanup_done=true. Même Task et trois empreintes canon/metadata/delegate après reprise. Un reçu, une Task, une exécution. |
| Dernier repli | deterministic | 21 | Task 3147c551-4415-4bd7-8359-b6161edc26ad ; résultat Cartae au SHA B, ACK présent, cleanup_done=true. Même Task et trois empreintes canon/metadata/delegate après reprise. Un reçu, une Task, une exécution. |

Preuve Laya : /Users/moi/.cache/bridget-delivery168/r168-b-laya-0c2c55/after-restart-receipt.json

SHA-256 : `8098115f4ec4a6fb2515a871ecd6819e146eeaa6ece4ed0b425bc24b926d270d`.

Preuve du dernier repli : /Users/moi/.cache/bridget-delivery168/r168-b-score-824a08/after-restart-receipt.json

SHA-256 : `49020adfed54f87f411ac8e17ef92064b19642ec5f3bf1121a2781bbd0779971`.

Les payloads Task bruts peuvent différer après reprise. La projection métier reste stable. Les nombres réels d’appels HTTP et de nouveaux processus ne sont pas instrumentés ; aucun zéro physique n’est revendiqué.

### Dernier point Haiku transmis avant la restriction

Le contexte de passation rapporte une décision `lightweight_model` sur 21 candidats et un résultat `RECETTE168_HAIKU_MATRIX_OK`, Linux/cartae, au SHA B. Cette preuve distante n’a pas pu être relue pendant cette reprise. Le dernier ACK connu était absent. Le rejeu après redémarrage privé et le nettoyage restent à vérifier.

- Namespace : /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18
- Parent : `8f2f787f-9eb7-4ce9-8cd0-070f50c577a5`.
- Clé : `recette-haiku-cartae-759bee18-matrix`.
- Task : `67f97d3e-be24-4f0d-a31f-258a56c744e5`.
- Arguments exacts à relire avant tout rejeu : /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/matrix-mcp-send-receipt.json

Ne pas créer une nouvelle clé pour poursuivre cette recette. Ne pas confondre ce cas avec le premier essai à sept candidats, ni avec le repli déterministe observé sur le Mac non authentifié à Claude.

### Nettoyage déjà attesté, puis limites de cette reprise

Le reçu local confirme le retrait normal des deux anciens worktrees Mac et la conservation de leurs archives vérifiées et stashes. Volume logique retiré : 218 486 707 octets ; aucun gain physique n’est déduit.

Reçu : /Users/moi/.cache/bridget-delivery168/cleanup-T048-two-worktrees-20261015T162414/receipt.json

SHA-256 : `f3b7b4ee7dbd034ec55277fe1c9bb454d537abde34b2b3493e5ff0ff9d937b3d`.

Le reçu privé des anciens workers de recette a aussi été relu : /Users/moi/.cache/bridget-delivery168/cleanup/private-workers-after-recipes.json ; SHA-256 `2b0fff6c5dec9c13cee04e425bea5a2934315fb832b5868aa42bcbb1a8b7aa19`. Il ne couvre pas la dernière topologie Haiku sur Cartae.

L’accès SSH a d’abord rendu Linux sur une sonde simple, puis la lecture distante nécessaire a échoué : `ssh: connect to host 37.59.185.67 port 2222: Operation not permitted`. Le contrôle local des processus a aussi été refusé : `zsh: operation not permitted: ps`. Le profil courant interdit l’écriture dans .git et dans les caches hors dépôt. Aucun contournement, signal de processus, commit ou publication n’a été exécuté pendant cette reprise.

### Ordre exact de reprise

1. T045 : relire la Task Haiku et son ACK, capturer le reçu complet, puis effectuer le rejeu exact après redémarrage du seul coordinateur privé. Comparer Task, résultat, trois empreintes et comptes. Arrêter ensuite la topologie privée après vérification fraîche des identités de processus.
2. T046 : actualiser les notes préparées sur Cartae et publier v0.4.0 sur le commit B exact avec les quatre actifs déjà construits. Vérifier cible du tag et sommes distantes. Aucune release n’est déclarée publiée par ce document.
3. T047 : rapprocher ce complément des documents non committés du worktree Cartae, sans écraser leurs preuves. Finaliser les critères et intégrer/pousser les documents.
4. T048 : retirer le worktree et la branche Cartae seulement après récupération vérifiée. Conserver les DB, preuves, sauvegardes et tout candidat dont la garde d’usage reste incertaine.
5. T049 : vérifier les deux checkouts, les actifs, l’état installé et les 35 FR/11 SC, puis finaliser les statuts sur preuves.

Les paquets ne nécessitent pas de nouvelle compilation pour un complément documentaire. La FULL unique, ses 32 échecs initiaux et leurs limites restent conservés. Aucun nouveau test, FULL, audit, redémarrage T3 ou changement de configuration de production n’a été lancé ici.


## Mise à jour après levée des restrictions — 15/10/2026 à 17:05 CEST

Le complément Mac de 16 h 56 CEST ci-dessus est conservé sans réécriture. Ses limites décrivent cette reprise passée. L’accès SSH et les droits nécessaires sont désormais rétablis selon ROOT ; cette restriction ne bloque plus la clôture opérationnelle.

La recette matrice Haiku finale est maintenant vérifiée par /home/moi/.local/state/bridget-delivery168/r168-ch-759bee18/after-restart-matrix-receipt.json. SHA-256 : `f6d3d046bfe8cd1c3665b3a583931a3dbc170aa1b9603be20ef239a52c740581`. Le reçu atteste 21 candidats, une décision `lightweight_model`, le rejeu privé 3804689 → 3834811, la même Task, les trois BLOB canon/metadata/delegate identiques et la même projection métier, avec ACK présent et `cleanup_done=true`. Le payload brut de Task n’est pas byte-identique ; les appels HTTP et nouveaux processus ne sont pas instrumentés. Aucun zéro physique n’est revendiqué. Le contexte antérieur « ACK et rejeu à vérifier » est donc historique. Les détails de cette recette et des rejeux Laya/score restent dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/recette168.md, sans réécriture de ce document ici.

La source B, les paquets et l’activation déjà documentés restent acquis. État : 51/56 au mandat ROOT, T045 prête à clore par ROOT. La publication reste en préparation, sans release publiée ni statut Livré/Implemented déclaré. La suite T046–T049 relève de ROOT ; aucun nouvel audit, FULL, P4 ou build n’est requis par ce seul rapprochement documentaire.

Les PID de production du checkpoint d’activation ne sont pas recontrôlés par cette rédaction. Pour T3, ROOT précise : application 65544 et tunnel 60206 inchangés ; remplacement extérieur du serveur 65608 par 97328, né à 16:51:02 CEST pendant les restrictions. Aucun signal T3 de notre part.
