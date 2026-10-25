# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/validation/livraison169.md](../../../../../../../specs/169-priorite-bridget/validation/livraison169.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Jalon | État et preuve |
|---|---|
| J1 — Convergence, contre-revue, audit | Clos. P2 CONVERGED,0 tâche ajoutée ; contre-revue R2 APPROVE_WITH_CHANGES arbitrée ; audit source pass, release_blocked=false, finalize/validate RC0. Reçu canonique /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/169-priorite-bridget/validation/audit169.md. |
| J2 — Git | C1 documentaire réalisé par ROOT : source T3 intégrée/poussée sur fork/local/main-20261009 :`58ddcf3a16205a24f940164cba500089f49ec6c0`, Mac et Cartae alignés ; documents Bridget intégrés/poussés sur main :`f4cb623aed57ce251bbd0b1f293f9b0c2b6f2c11`. Le complément documentaire C2 est identifié par /Users/moi/.cache/bridget-delivery169/git-bridget-c2-169.json ; sa source ne change pas celle du paquet T3. |
| J3 — Paquet | Clos le 2026-10-15. Paquet T3 signé `0.0.45-local.169`, construit une fois depuis la source publiée ; ASAR SHA-256 `a23d41088ed6aa3671a129778efc42736089f68b433d7f4dd946bc25888fb52c`, codesign RC0. Preuve opératoire privée : /home/moi/.local/state/bridget-delivery169/operational169/operational169-proofs.json. |
| J4 — Installation | Clos le 2026-10-15. Installation puis activation unique terminées, préférence ON ; nouveaux processus T3 main4717/serveur4792 ; processus protégés conservés : Bridget71927/72468 et tunnel60206. Sauvegarde : /Users/moi/.cache/bridget-delivery169/activation-preparation/backup169-once. |
| J5 — Activation et recette | Clos sur faits limités. Un wake scoped a repris le fil ordinal48 ; `preferBridgetDelegation=true` est présent dans la consigne `turn/start` corrélée. HTTP sujet officiel Bridget : complet7508698 octets avec `projection.visibleTurnItems=[]` et données utiles conservées ; bounded2661116 octets avec curseur et `payloadBudgetExceeded=true`. Recette `bridget_auto_delegate` V2 : 1 RPC initial + 1 RPC de rejeu identique, 1 Task, 1 lancement, aucune nouvelle délégation T3 ni sous-agent fournisseur pour cette recette. Task `6c1c769a-92ab-4369-95ff-bb2f978d681b`, request_id `spec169-bridget-first-recette-20261015-r1`, journal48 événements inchangé, 1 `turn_start`/1 `prompt_dispatched`/1 `turn_end`, gap0, restart_resumes0, `cleanup_done=true`, `result_sent=true`. ACK `bridget_task_ack` informatif acquis à `1792095240` ; `delivery_acked=false` est conservé. FR017, SC003 réel et SC009 peuvent être marqués couverts par ces seuls faits ; aucune généralisation à tous moteurs/modèles. |
| J6 — Reçu final et nettoyage | Examen du périmètre clos par ROOT. Worktree et branche T3 Mac retirés sans force après preuve de fusion/push/propreté et absence d’usage ; 16 fichiers ignorés archivés et vérifiés. Worktree documentaire Mac réservé au commit C2 puis retrait gardé ; 4 fichiers ignorés déjà archivés byte-identiques. Deux worktrees/branches Cartae conservés : sudo lsof RC0 mais avertissement fuse.portal, observation incomplète. Stages, caches de tests, TMP opératoires, paquet signé, DB et rollback conservés avec leurs preuves ; aucun nettoyeur de ces stages qualifié. Détail final : /Users/moi/.cache/bridget-delivery169/final169.json. Aucun gain physique revendiqué. |

<a id="extrait-2"></a>

## Extrait 2

Le moteur Bridget0.4.0 reste inchangé. Aucun code Rust ou nouveau binaire Bridget n'est produit par ce lot documentaire. La source T3 publiée est alignée avec Mac et Cartae ; le paquet signé et le code chargé sont attestés par la preuve privée ci-dessus. Ni token ni corps de conversation ne figurent dans les preuves publiables.

<a id="extrait-3"></a>

## Extrait 3

Checkpoint historique avant construction :la racine et le worktree T3 Cartae sont alignés sur58ddcf3a16205a24f940164cba500089f49ec6c0 ; seul le propre worktree ROOT reste non suivi à la racine. Preuve /home/moi/.local/state/bridget-delivery169/git-t3-alignment169.json. La préparation de construction a échoué avant compilation puis est reprise sur le même checkout ; compteur de compilations0 à ce checkpoint. Aucun résultat de build n'est présumé. Le détail de préparation reste privé.
