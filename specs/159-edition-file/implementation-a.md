# Mise en œuvre159A — lot A livré

Lot A : 14 tâches (T001–T013 et T028). Lot B : 14 tâches encore ouvertes. Lot A14 coché et livré par ROOT ; spec159 InProgress car B14 reste ouvert. Aucun succès global de tests déduit. Source de travail actuelle, daemon.rs :
abf654cd44116dc277c80b3f9ff9e188e6ba5dae47d5c03df75ddbcdbbdaaddc
Base de travail b235998638b3eadf8f8edc0a6e323b762e95cab0 ; source 158B livré e52a96240b6168bf4466712e1bda8852b4ecb43f. Aucun commit 159A futur inventé.

P3 et son annexe ciblée FR023/FR025/T006 sont CONVERGED. L'annexe conserve les tâches byte-identiques et les autres fichiers du guardP3 ; elle n'est pas un quatrième passage global. R2 Claude est APPROVE_WITH_CHANGES ; ROOT a retenu et corrigé FR023, puis exécuté GREEN 8 sur le source courant. Audit final : A99,95, 0C/0H/0M/1 LOW accepté ; validateur RC0/0 erreur/0 warning. Lecture partielle 16/64 fichiers, 4275/174030 lignes (2,4565%). La capture 18ad/448 hunks reste historique ; le delta courant relit la modification FR023, avec les 64 empreintes AV/AP égales. Aucune lecture globale du dépôt n'est prétendue.

La file réemploie send_deliveries, la saga099, Store/IdempotencyStore et leur DB commune, begin_send_delivery_with_guidance, canons, suivi, garde de pause, poussée et reprise. SQLite AUTOINCREMENT fournit le rang durable. Ajouts prévus : held_queue fermé, attestation et période occupée, quatre colonnes v7 (queue_rank/recipient_agent/turn_gate/acked_at), phases held/cancelled et HeldRelease. Aucun service, framework, compteur métier ou outil d'édition en A.

La rétention accepte absence et QueueOnly pour une instance attestée, avec exclusions FR002. La libération lit repos, pause, interruption, réservation et échéance sous State et transaction Immediate ; commit avant push hors writer. La sortie forcée exige silence> 90 s ET période occupée >= notify_timeout_secs + 30 s ; repos frais exige busy_period_start=None. Un true après expiration conserve le début jusqu'au vrai false. Le premier ACK est durable ; ancienne date NULL sans invention. Le balayage nettoie après COMMIT, y compris les lignes déjà réglées avant une erreur tardive, puis renvoie l'erreur. T028 réemploie le getter SELECT pour les remises propres du client nu négocié : scope, clé et delivery_id inchangés, aucun gain de droits. data_version seul n'est pas une preuve universelle ; getter pur et list_ne_modifie_rien complètent.

Articles XIX/XX : nécessité par les exigences existantes ; réemploi des puits et transactions ; erreurs fermées conservées ; aucun helper/framework supplémentaire pour DUP001. Les57 lignes froides restent LOW accepté. Les lectures et corrections ROOT, les preuves par tâche et les limites figurent dans la matrice A14. Ce reçu documentaire ne se substitue pas aux manifestes de lecture.

Chronologie probatoire : T013 ciblé 920 PASS principaux/6 ignored,903/903 références PASS inchangées ; FULL unique 2551 PASS/9 FAIL/75 ignored RC101, 779,06 s ; neuf échecs résolus par derniers replays ciblés exacts. R1 :RED 3 FAIL puis GREEN 54 ; T028 :RED 1 FAIL puis GREEN 7 ; R2 FR023 :RED 1 FAIL puis GREEN 8 courant. Chaque lot garde son empreinte propre, sans seconde FULL ni transposition de2551 tests au dernier hash.

SC004 : proxy Python avant TurnCompleted→début du tour suivant reçu,20 échantillons/rang19. p95 avant 27,763792 ms/après 1028,132375 ms ; delta 1000,368583 ms > 250 ms. Sampler after PASS en 54,72 s, mais seuil dépassé : held_queue reste fermé. Aucune horloge daemon_idle exacte ni SLA. La précondition busy supplémentaire après est hors mesure ; fixtures complètes non byte-identiques. Ancienne metadata396804 conservée historique ; p95 inchangé, métadonnées 54,72/log db134 corrigée.

T011 réel : migration6→7 avec debug48c5…, ancien158B refusantv7 RC1 sans changerDB, restaurationv6 ouverte RC0. Hash/logiques/samples/rangs égaux ; quick_check=ok/FK0. Premier FAIL conservé : maintenance à7jours, byte-identique158B, pas migration. Snapshot réel :acked/indeterminate et zéro lien ; synthétiques T003 distincts. JSON publics sans corps, originaux privés jamaisGit. Cette répétition ne décrit pas le futur release ou un rollback production.

Sauvegarde réelle pré-v7 acquise à 17:14:26 UTC : native0,quickOK,FK0. DB et binaire rollback privés, non copiés au dépôt :
/Users/moi/.cache/bridget-delivery159A-preparation-20261012/rollback158B-prod-v6.db
SHA256 ef4c01667fb9102efbffca25234eb81bbdb8d57c0dda6533ab02d5a147b9bf4d
/Users/moi/.cache/bridget-delivery159A-preparation-20261012/rollback158B-bridget
SHA256 cfb77a6552e97b28fd092ecbd73e82fbfcaa59694264b354fe593cb63e64018c
Préserver les faits plus récents avant un éventuel rollback. Native0 reste daté ; recontrôle avant activation.

ROOT a coché A14, intégré et poussé source53863bd09a4ff364dc2b412519b343c77e163685, installé et activé le release signé. Basev7 contrôlée, T3 inchangé ; branche/worktree/stage retirés, archive vérifiée et sauvegardes conservées. Reçu actuel : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/validation/livraison159A.md

Preuves/gates A14 et détails de tests :
/Users/moi/.cache/bridget-docs159A-final-preparation-20261012/matrice-A14-gates.md
/Users/moi/.cache/bridget-docs159A-final-preparation-20261012/tests-159a.md
