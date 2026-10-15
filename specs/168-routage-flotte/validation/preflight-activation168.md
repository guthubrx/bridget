# T043 — qualification et préflight d’activation 168

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
