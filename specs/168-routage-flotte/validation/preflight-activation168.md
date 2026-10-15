# T043 — qualification et préflight d’activation 168

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
> Référence : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/preflight-activation168.md (section « Checkpoint B courant »).

Date : 15 octobre 2026. Source binaire : `269de64e172f71f8856373fe8b09cb2d8642da52`, version 0.4.0. Qualification et préflight exécutés ; ce reçu ne coche aucune tâche. Le checkpoint ROOT avant cochage T043/T044 porte 49/56. La livraison complète reste In Progress.

## Connexions et configuration

Deux comptes Codex distincts sont conservés sur le Mac (normal et pro), un sur Cartae. Les observations model/list déjà recueillies donnent sept modèles par connexion : gpt-6.1-sol, gpt-6-astra, gpt-6-sol, gpt-6-luna, gpt-5.6-sol, gpt-5.6-terra et gpt-5.6-luna. Leurs efforts et défauts viennent de chaque observation ; aucun effort supplémentaire n’est inventé. Les connexions Claude locale et Cartae déclarent Haiku/Sonnet/Opus 5.5, sans effort ajouté quand le moteur n’en expose pas. Les attestations existantes Claude indiquent OAuth Max ; aucune nouvelle connexion interactive ni copie de secret fournisseur vers un moteur n’est effectuée pour cette qualification.

Preuves déjà recueillies :

- /home/moi/.local/state/bridget-delivery168/config-draft/model-list-codex-mac-normal.json
- /home/moi/.local/state/bridget-delivery168/config-draft/model-list-codex-mac-pro.json
- /home/moi/.local/state/bridget-delivery168/config-draft/model-list-codex-cartae-normal.json
- /home/moi/.local/state/bridget-delivery168/config-draft/claude-mac-auth-status.json
- /home/moi/.local/state/bridget-delivery168/config-draft/claude-cartae-qualification.reference.json
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/qualification-claude168.md

Les registres sont fusionnés récursivement avec leurs fragments ; les listes et scalaires remplacent la valeur précédente, null est conservé. Les JSON UTF-8 canoniques sont validés par le vrai binaire 168. Le namespace de production reste maître Mac / worker Cartae ; le daemon Cartae décrit ci-dessous est temporaire et privé.

| Registre fusionné | SHA-256 | Catalogue exact |
|---|---|---|
| Mac | `d104f2e82ec93ef8aae487e73326ba3da7f3e8adb9ad043ed3587beb54ce67e7` | 27 global, 17 local, 10 Cartae |
| Worker Cartae | `bac0293a069ed4d90a3fcff7feb439c42fd5c49d5b1a3d86ad605a975bfe1083` | 10 global, 0 local, 10 Cartae |

Ces nombres représentent des combinaisons déclarées/observées, pas un droit universel d’utiliser chaque modèle, ni une promesse de quota ou de coût. La lecture du catalogue ne lance aucun modèle.

## Deux qualifications isolées réellement exécutées

Le binaire source A démarre un daemon de premier plan dans chaque namespace neuf, sans identité T3 ni copie d’authentification. Les lectures version, catalogue JSON global/local/Cartae et catalogue humain terminent RC0. Après lecture, seul le daemon enfant attesté est arrêté par SIGTERM ; aucun enfant fournisseur n’est présent. L’environnement utilise HOME réel et les seuls réglages Bridget privés nécessaires.

- Mac : /private/tmp/bq168-m-f7f9378117/namespace-receipt.json ; SHA-256 `6102143e8e1e9482f269ea93dfecd751813be1b4a6b65786bdd4313d791fedce`.
- Cartae : /tmp/bq168-w-186aa1df7e/namespace-receipt.json ; SHA-256 `d75123b1c0942e6810e85ba24f1035c7ff314a42f7064d7e4e63f69c533bafec`.

Les deux reçus portent PASS. Leurs DB privées ont quick_check=ok, zéro erreur de clé étrangère et exactement zéro Task native, reçu automatique et exécution. La DDL des reçus est comparée à la référence `a44bb506ce63f23d048720efccb0810044d5e28bcf03b107d92dae9503f0fb8b`. Les fichiers protégés et les registres de production restent identiques pendant ces qualifications ; aucun modèle ni appel HTTP de décision n’est exécuté.

Reçu commun consommé avant tout arrêt :

- /Users/moi/.cache/bridget-delivery168/activation/validation-receipt-269de64e172f.json — SHA-256 `0ba2d2bbe2d1ac7168287d1f9c9efb629169acf0cc84fa925eb299fccc58689c`.

## Préflight, quiescence et sauvegardes cohérentes

Le préflight r2 termine RC0 avec total_active=0. Les huit compteurs maître (délégations, exécutions, file, demandes enfant, spawns, pause, contrôles, places automatiques) sont à zéro. Le worker est préparé avant l’arrêt du maître et du pont ; son état et ses commandes/slots sont contrôlés avant remplacement. Les nombres 23 commandes/50 slots de son inventaire ne sont pas présentés comme des travaux actifs.

La garde root fuser a été qualifiée puis exécutée avant la copie froide, selon la procédure ROOT. Les sauvegardes réelles conservent binaires antérieurs, registres, unité/plists, état worker et DB cohérente. La sauvegarde privée du credential worker sert au rollback ; elle n’est ni un nouveau login ni une injection de secret dans un moteur.

- Mac : /Users/moi/.cache/bridget-delivery168/activation/backup-269de64e172f-r2
- Cartae : /home/moi/.local/state/bridget-delivery168/activation/backup-269de64e172f-r2

La copie SQLite Mac par l’API de sauvegarde porte SHA-256 `8515e26223c976773cc08d08d794bebffba0374ed3dd3177b8ccc8d0d2326dd2`, quick_check=ok, FK0. Le hash du fichier DB froid avant remplacement est `6ebfd66c2fbf343c0518fcc5c1aa3c4e1d3556b5096b7a05b48401a7c3612a4d` ; ces deux artefacts ont des représentations physiques distinctes et ne sont pas prétendus identiques. Les schémas de migration sont 7/12.

- /Users/moi/.cache/bridget-delivery168/activation/r2-preflight-command.json — SHA-256 `3189b27a31f8b1f61aa36f604a7e6d6bd6863d8422093c6731ad5eeaab0f1f3a`.
- /Users/moi/.cache/bridget-delivery168/activation/backup-269de64e172f-r2/activation.json — SHA-256 `04b0dcf71522597c36788c47ed573a9aea4bf3c451dbd8a0141f9264e98bee77`.

## Historique et rollback

Le premier freeze est refusé avant mutation après la sortie naturelle du renderer T3 34864. Le garde corrigé et contre-relu conserve les deux ancres stables application/serveur par chemin exact, parenté, PID et naissance, ainsi que le tunnel. Il ne prétend pas préserver un renderer déjà sorti. Les anciens reçus restent conservés.

- /Users/moi/.cache/bridget-delivery168/activation/renderer-guard168.json — SHA-256 `4b4f1b29e46ea3136a37b15ff55dc6efac300998944fbb4c59d07e3877e73e46`.

Diff conservé : /Users/moi/.cache/bridget-delivery168/activation/renderer-guard168.diff.

Le rollback automatique est interdit par le reçu. Dès qu’une ligne metadata_version=2 existe, un ancien binaire est incompatible. Préférer un binaire compatible 168 avec automatismes désactivés, ou une procédure cohérente sans perte relue par ROOT. Aucune restauration de DB n’a été faite.

L’activation exécutée et ses contrôles sont consignés séparément : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/activation168.md. Les recettes réelles T045 restent à faire ; ni catalogue ni qualification privée ne les remplacent.


## Checkpoint B courant — qualification et préflight du registre

Source B : `431b78f72bdd73eac4772af17bf36879d7a629d2`. Les qualifications A restent historiques. Les deux registres B ont été qualifiés par le même helper relu, sans modèle ni fournisseur, sans T3 et sans mutation de production pendant la qualification. Les nouveaux namespaces sont distincts de ceux de A :

- /private/tmp/bq168-mB-ddec62bb33/namespace-receipt.json ; SHA-256 `c157657acd053ba04b26325c581b4eb21e380ac99580434bdefd6487c3baf325`.
- /tmp/bq168-wB-9506f9378c/namespace-receipt.json ; SHA-256 `6e2d7bf1ae19afd0cfc8020e0a22e16c966167d8b250f77ab0910496c08625f2`.

Les deux reçus portent PASS : catalogue Mac exactement 27/17/10, worker 10/0/10 ; zéro Task native, reçu automatique et exécution ; quick_check ok et FK0. Les fichiers protégés avant/après sont identiques. Les deux daemons privés ont été arrêtés seuls par SIGTERM après contrôle de leur identité ; aucun enfant fournisseur. Copies durables :

- /Users/moi/.cache/bridget-delivery168/activation/configB-431b78f72bdd/qualify-mac.receipt.json
- /Users/moi/.cache/bridget-delivery168/activation/configB-431b78f72bdd/qualify-worker.receipt.json
- /home/moi/.local/state/bridget-delivery168/activation/configB-431b78f72bdd/qualify-mac.receipt.json
- /home/moi/.local/state/bridget-delivery168/activation/configB-431b78f72bdd/qualify-worker.receipt.json

Le fragment Mac change uniquement les deux budgets du coordinateur local : decision_timeout_ms=30000 et service_timeout_ms=20000, contre 10000/2000 avant B. Aucun modèle, connexion, auth, droit ou autre valeur de registre n’est modifié. Le fragment worker est {}.

CAS Mac avant application : `d104f2e82ec93ef8aae487e73326ba3da7f3e8adb9ad043ed3587beb54ce67e7` ; fusion B installée : `59802f1a8e42bdca8fc01935b0638f448f493a8dcf30973bf46d0f846ef7bf47`. Worker avant/après : `bac0293a069ed4d90a3fcff7feb439c42fd5c49d5b1a3d86ad605a975bfe1083`. Les 27 combinaisons restent conservées ; leur entitlement n’est pas garanti par le catalogue.

Reçu de validation consommé par l’activateur : /Users/moi/.cache/bridget-delivery168/activation/configB-431b78f72bdd/validation-receipt-B.json ; SHA-256 `9a012713173ad50a21a265f3bd7d13712a60f5222710dd9adc0d3702e7e39908`. Paramètres complets : /Users/moi/.cache/bridget-delivery168/activation/configB-431b78f72bdd/activation.params.json ; SHA-256 `16d886bf5c9569c3336b3ae2fc1a0c9cb9c064be45a5dc4343439eb1fcce7259`. Ils lient source, binaires, fusion et DDL inchangée `a44bb506ce63f23d048720efccb0810044d5e28bcf03b107d92dae9503f0fb8b`.

Le préflight B termine RC0 avec total_active=0 et les huit compteurs de quiescence maître à zéro. Les sauvegardes B neuves sont réellement créées, sans remplacer A :

/Users/moi/.cache/bridget-delivery168/activation/backup-431b78f72bdd-B

/home/moi/.local/state/bridget-delivery168/activation/backup-431b78f72bdd-B

La copie SQLite de sauvegarde B porte SHA-256 `2f35681bf722dc6f62610c9349cd439b767771dfbc5f7774a8142a45c03035a7`, quick_check ok/FK0. Les schémas de migration restent 7/12. Les unités/plists, état worker, anciens binaires et registres restent sauvegardés. Les gardes T3/tunnel, CAS et absence d’écrivain sont conservées. Aucun rollback automatique de DB ; un ancien binaire reste interdit sur des lignes metadata_version=2.

- /Users/moi/.cache/bridget-delivery168/activation/B-preflight-command.json ; SHA-256 `0dec216d0ebc179c5ba0cadd7034d4f61bcefa88ac7564308541f0da5303cc83`.
- /Users/moi/.cache/bridget-delivery168/activation/backup-431b78f72bdd-B/activation.json ; SHA-256 `a55769f921f1cdd6e1fa51075c726a55309aa399a67f6cca5583180688369ba2`.

État ROOT au présent mandat : 51/56, sans cochage par cette rédaction. Les recettes réelles T045 sur B sont en cours ; la release n’est pas publiée. La session reste In Progress. Les limites de T036 sont conservées : FULL unique 2 820 PASS / 32 FAIL / 71 ignorés ; 29 des 32 échecs initiaux PASS après reprise ; trois annulations 159 non validables sur Linux ; cause initiale T3 inconnue malgré replay PASS. Aucun GREEN global.
