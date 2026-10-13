# Contre-revues du plan et du code SPEC-162

## Ronde 1

**Verdict du reviewer : APPROVE_WITH_CHANGES.** Revue en lecture seule, aucun test ni build. Texte brut : /Users/moi/.cache/bridget-delivery162-20261013/review-plan-r1.txt. Les corrections ci-dessous ont été vérifiées avec ROOT et l’agent code161 ; elles sont documentaires, pas des preuves d’implémentation.

| Objection | Vérification et décision |
|---|---|
| O1 — ReceiptStore et deux écrivains incompatibles | Retenue. Store technique distinct, gardes réemployées. Worker seul écrivain metadata ; wrapper seul écrivain du blob immutable dans slot privé. Pas de nouveau canal IPC ni de FD managed-status détourné. Budget sur enveloppe sérialisée de 2 Mio, metadata et temporaires ; pas de promesse de 4 096 résultats dans 16 Mio. |
| O2 — RetainedResultAuthority utilisé à l’entrée et absence d’issue terminale | Retenue. capture_remote_result vérifie worker/epoch/placement/commande/exécution sans connexion du wrapper mort. Mutation métier et disposition dans une transaction. RetainedResultAuthority uniquement à la sortie. accepted, discarded_terminal, rejected_conflict et deferred explicites ; quota_hold temporaire conserve son slot. Les issues finales arrêtent les replays sans écraser un résultat différent. |
| O3 — Contrat de cinq opérations incomplet | Retenue. Ajouter parent_proof, journal_page, final_result et result_disposition. Attach est un lecteur local à réemployer sur le worker. Le flux vivant conserve LiveJournalFragment/AttachView existants ; aucune nouvelle famille follow/stop. La racine froide est déduite du reçu authentifié. La proposition de retirer les parents distants est rejetée par ROOT : elle réduit le plan complet demandé. Leur preuve est relue sur leur hôte avec profondeur 8 et échanges hors verrou ; un timeout refuse sans danger. |
| O4 — Reconnexion bloquée par ancienne connexion fantôme | Retenue. Même identité, credential et révision autorisés remplacent atomiquement la connexion précédente. Nouvelle époque et anciennes trames refusées. Toute commande doit être placée sur ce worker. Le store reste à écrivain unique. Aucun nouveau permis ou respawn déduit du handshake. |
| O5 — Supervision et sortie unknown absentes | Retenue. Unité systemd utilisateur et paths privés définis ; tunnel 095 conservé au Mac. KillMode=process est justifié par les wrappers gérés qui doivent survivre puis être réconciliés. Unknown conserve la capacité jusqu’à preuve terminale ou intervention hors périmètre ; aucun reset destructif ajouté. |

## Corrections complémentaires

- NativeDelegationRequest est fermé ; seul l’ancien registre ignore machines. La lecture fraîche de delegation_mcp.rs:91–120 montre que l’ancien MCP refuse aussi les champs inconnus, contrairement à l’hypothèse du reviewer. Garde de placement lors de la demande courante.
- Succès de lecture enrichie version=2 ; erreur historique version=1 reconnue explicitement par le nouveau CLI. Authentification humaine v1 inchangée.
- Contact affiché comme observation du snapshot ; aucun âge vivant supposé. Heartbeat prend son époque dans la connexion.
- Quotas Fleet globaux par agent_type conservés. Réserves worker techniques additionnelles.
- Préparation de 30 secondes terminée avant ReadyToRelease ; permis de 5 secondes depuis cet événement local.
- Aucun réseau sous DaemonState, pour toutes les opérations, pas seulement Prepare.
- Collecte après disposition durable ET fin du producteur, fermeture irrévocable et rétention. Les octets conservés restent dans le budget après solde de la réserve.

## Portée et gates

Le PASS de réemploi R1 concernait 39 éléments. Son actualisation R2 est confiée à l’owner du rapport. La contre-revue R2 ciblée et la relecture ROOT sont encore attendues. Aucun tasks.md avant leur feu vert. Aucun succès runtime ou validation globale déduit de cette révision documentaire.

## Ronde 2 — réception et premières corrections

**Verdict : APPROVE_WITH_CHANGES, aucun blocage déclaré par le reviewer.** Texte brut : /Users/moi/.cache/bridget-delivery162-20261013/review-plan-r2.txt. O1 à O5 sont considérées refermées. Aucun test ni build exécuté par le reviewer.

La lecture fraîche de fleet.rs:1010 corrige les formulations de R1/R2 : quota Fleet unique toutes catégories, plus limites fournisseur 157 par agent_type. Les tâches locales et distantes les partagent. Un unknown distant immobilise donc aussi de la capacité locale.

Expiration Prepare/Release : appeler abandon et fermer le canal, puis observer la fin ; EOF fait sortir le bootstrap bloqué. La racine du journal se dérive de l’instance et de la génération attestées selon le patron ManagedStderrStore, sans champ de chemin venant du CLI.

Les objections A à D et les choix de contact/rétention attendent encore les vérifications ciblées de ROOT et des chercheurs. Pas de tâches générées pendant cette vérification.

Décisions ROOT sur les points mineurs R2 : le snapshot existant ne fige que generation/seq. Les transitions de contact_state invalident le curseur ; last_contact_at est observé par page, sans gel global ni âge vivant. Cette décision remplace l’hypothèse de gel volatile R1. La collecte du blob est permise après disposition finale durable, producteur terminal et clôture irrévocable commande/slot ; conserver seulement le reçu compact et son empreinte/disposition 7 jours. Cela remplace la rétention de 7 jours du blob proposée en R1. Un slot vivant non clôturé peut encore saturer le budget ; aucun nouvel outil de maintenance ajouté.

**D retenue et vérifiée :** Le worker, les wrappers et les fournisseurs sous le même UID partagent un domaine de confiance. Les modes 0600/0700 ne les isolent pas entre eux. Un fournisseur full-access compromis peut lire le credential ou modifier l’état ; l’authentification prouve la possession du credential, pas l’intégrité du processus. À Prepare et avant RELEASE, refuser tout chevauchement canonique dans les deux sens entre cwd, mappings, writableRoots ou additional_directories et le credential, son répertoire privé ou l’état worker. Une résolution impossible entraîne un refus. Cette garde prévient l’exposition explicite ou accidentelle, pas un accès global dangerFullAccess/bypassPermissions. Le wrapper d’infrastructure peut publier uniquement dans son slot préalloué ; ce slot n’est jamais ajouté aux droits du fournisseur. Un UID séparé ou une sandbox OS restent hors périmètre. Journaliser les remplacements de connexion avec machine, worker et anciennes/nouvelles époques, jamais le secret.

## Arbitrages finaux A à D, vérifiés après R2

| Point | Décision retenue et preuve |
|---|---|
| A — Stores stricts et retour arrière | Retenu. Task.remote_placement et DesiredEquipier.remote_placement Option default, Fleet schéma 5 explicite. Request garde ses champs. Aucun downgrade sur état 162 inchangé. Sauvegarde jointe DB/Fleet/registre/worker après quiescence ; restauration seulement sans perdre l’activité postérieure. Sinon route désactivée sous 162. Preuves : delegation.rs:212/231, protocol.rs:2154, desired_state.rs:17/87/120. |
| B — Deux entrées de résultat | Retenu. Wrapper distant : blob seulement. Worker seul final_result. capture_reply refuse par remote_result_requires_worker une mission distante courante ou passée ; les deux appelants Send/SendIdempotent répondent Nack, sans Ack ni dispatch de secours. RetainedResultAuthority reste en sortie. Preuves : daemon.rs:11968/21821/21849. |
| C — Réservation des rondes | Retenu. reserve_result_slot idempotent avant RELEASE, après begin_round avant send_as, après reconstruction quota et sauvegarde resume_execution_id avant deliver_reconstructed. Identité figée, E/S hors verrou puis revalidation. Refus sans nouvel UUID ; clôture caduque seulement sur preuve de non-dispatch. Preuves : native_delegation.rs:2466/3444/3502/3504/4480. |
| D — Même UID | Retenu avec limite explicite, voir décision ci-dessus. Chevauchements bidirectionnels interdits ; credential uniquement en mémoire worker, jamais dans l’environnement enfant. Aucun confinement OS prétendu. |

Ces décisions ferment les corrections documentaires vérifiées. L’actualisation de l’audit et la relecture ROOT restent les gates avant Tasks. Aucune ronde complète supplémentaire n’est déduite automatiquement.

## Contre-revue de code R1 — BLOCKED, corrections T044 ouvertes

**Verdict reçu et acquitté : BLOCKED.** La contre-revue de code est distincte des rondes de plan ci-dessus. Identifiant consultable dans l’historique de délégation : node:delegated-task:command%3Amcp%3A3fa5c3d9-0ca4-4322-8649-ec020a6fcfe7%3Adelegate-task%3A162-code-review-20261013-r1. Le présent compte rendu reprend les objections vérifiées par ROOT ; il ne revendique pas une nouvelle exécution ou une lecture indépendante du retour brut non recopié ici.

ROOT a confirmé trois manifestations d’une même cause : les issues de démarrage et de refus ne sont pas toutes raccordées à une réconciliation durable. Les ancrages ci-dessous sont ceux de sa lecture avant correction ; les lignes peuvent évoluer.

| Objection | Vérification ROOT | Décision et preuve attendue |
|---|---|---|
| Démarrage distant encore starting sans Reconcile | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/daemon.rs:18427 et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/daemon/native_delegation.rs:5651 | Retenue. Réconciliation bornée hors mutex, puis revalidation avant mutation. Aucun progrès terminal déduit d’une absence de réponse. |
| Observation Refused ignorée dans le contrôle/annulation | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/daemon/native_delegation.rs:6517, avec le raccord daemon ci-dessus | Retenue. Consommer le refus durable corrélé si absence de naissance ou arrêt prouvé ; conserver l’inconnu sinon. Ne pas injecter un DesiredStopped pour rendre la recette verte. |
| Réserve Created puis erreur avant spawn laisse Reserved, ensuite Unknown | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/remote_worker.rs:1928 | Retenue. Après erreur avant naissance prouvée, persister le refus dans le reçu existant. Une réservation ambiguë ne permet ni nouvelle création ni libération de capacité. |

**Nuance de preuve corrigée par ROOT :** il serait faux d’affirmer qu’aucun processus n’est toujours né sur ces chemins. Après création, EOF ou StartupFailed ne prouve pas à lui seul la mort du bootstrap ou du fournisseur. La correction doit observer l’arrêt de l’identité attestée ; sans cette preuve, Unknown et sa capacité restent conservés. La fermeture du canal n’est pas un substitut à cette preuve.

T044 porte la correction et deux E2E réels, sans DesiredStopped injecté : fournisseur StartupFailed après RELEASE, puis vrai ReadyToRelease retardé de six secondes face à un permis de cinq secondes. Chaque scénario doit prouver arrêt, remote_start_failed et réutilisation du quota 1. Les refus avant naissance et EOF/StartupFailed avant Ready relèvent des tests de primitives distincts, pas de ces deux E2E. MASTER, WORKER et FIXTURES corrigent leurs surfaces sous coordination ROOT. Aucun correctif n’est déclaré testé à ce checkpoint, aucune tâche de confort ajoutée et aucune seconde FULL prévue.

État : In Progress, 43/44. Converge2 CONVERGED reste historique, puis la contre-revue R1 rouvre le développement ; aucun passage 3 n’est encore exécuté. La préparation de l’audit sur sources pré-R1 est suspendue. G8/G9 restent ouverts et aucune livraison ne peut être déduite de la convergence historique.



### Précision ROOT sur la borne — objection partiellement retenue

ROOT a relu le prédicat de wait_ready_or_cancel appelé par remote_worker.start : il inclut prepared.expires, soit la fenêtre de préparation de 30 secondes, en plus de deadline_at de 300 secondes. L’affirmation « le worker attend 300 secondes sans borne 30 » est donc inexacte et n’est pas retenue. Le manque de Reconcile reste réel et doit être corrigé ; cette nuance ne l’efface pas.

Les E2E finaux prévus ne simulent pas une attente de 30 secondes : un fournisseur produit StartupFailed après RELEASE ; un autre scénario retarde un vrai ReadyToRelease de six secondes, au-delà du permis de cinq secondes. Chacun doit observer la fin réelle, remote_start_failed et la réutilisation d’un quota de 1. Aucun DesiredStopped ne sera injecté pour fabriquer ces conclusions. EOF/StartupFailed avant Ready est couvert par le test de primitive WORKER prévu, avec conservation du Child et arrêt attesté. Aucun de ces nouveaux essais n’est déclaré exécuté ici.

Les extensions réemployées annoncées sont les variantes retained des primitives de processus et un curseur de réconciliation volatil. Leurs lignes d’audit de réemploi seront consignées après le gel des propriétaires ; ce checkpoint ne crée ni nouvelle API métier ni helper permanent.

## R1/T044 — clôture du slot sans résultat encore à valider

ROOT et l’owner ont confirmé un reliquat du même défaut : finish_command ne fermait pas les slots sans blob. Après remote_start_failed, la réservation durable de 2 Mio restait comptée ; le nettoyage maître ne transmettait pas Cancel ResultSlot. La correction worker réemploie close_result_slot après arrêt attesté et réessaie la clôture sur Terminal/Refused si le stockage a échoué. Elle ne collecte pas les blobs Deferred et ne libère aucune preuve ambiguë.

T044 inclut désormais la libération de cette réservation sans résultat et sa preuve ciblée, encore en attente. Aucun cochage : 43/44. La sonde MASTER se limite maintenant à un seul candidat coûteux par cadence ; le curseur avance aussi si le candidat est invalide. Aucun parcours coûteux N+1 ajouté. Ces points décrivent les corrections en cours, pas des tests réussis.

### R1/T044 — preuves R4, clôture encore suspendue

R4 passe : 73 tests lib et 4 E2E, zéro échec, quatre helpers ignorés. Les preuves privées sont référencées dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/tests162.md. Les primitives retained, les refus après arrêt attesté et la clôture des slots vides sont testés. Les scénarios E2E couvrent StartupFailed après RELEASE et retard Ready de six secondes contre permis cinq secondes, pas le timeout de préparation 30 secondes.

La relecture ROOT postérieure découvre un inventaire répété par commande terminale dans reconcile_all. Une lecture groupée est en correction ; replay attendu. T044 demeure ouverte à 43/44 et Converge3 n’a pas commencé. Il ne s’agit ni d’un verdict favorable du reviewer ni d’une livraison. Le BLOCKED R1 reste historique ; G8/G9 restent ouverts.

### Clôture interne T044 après R5

R5 corrige puis valide le reliquat de parcours répété : 17 worker PASS et 4 E2E PASS, RC0, source stable. ROOT autorise le cochage T044. Développement/préparation 44/44 ; Converge3 reste à exécuter. Cette clôture interne ne remplace pas un verdict du reviewer : R1 BLOCKED reste historique et aucun APPROVE futur n’est revendiqué. G8/G9 restent ouverts.

## Contre-revue code R2 — en attente après Converge3

Converge3 est CONVERGED, zéro tâche ajoutée, 44/44 et tâches byte-identiques. ROOT lance la contre-revue code R2 sous taskId node:delegated-task:command%3Amcp%3A3fa5c3d9-0ca4-4322-8649-ec020a6fcfe7%3Adelegate-task%3A162-code-review-20261013-r2, borne quinze minutes. Aucun verdict R2 anticipé ; le BLOCKED R1 reste historique. Les modules d’audit sont terminés selon ROOT, avec 0 CRITICAL/0 HIGH/3 MEDIUM/1 LOW ; scoring et validateur encore en cours. G8/G9 restent ouverts.

## Contre-revue code R2 — APPROVE_WITH_CHANGES, D1 requis avant G8

ROOT a reçu et acquitté le verdict **APPROVE_WITH_CHANGES** de la contre-revue R2. Les objections R1 sont résolues. Le taskId reste node:delegated-task:command%3Amcp%3A3fa5c3d9-0ca4-4322-8649-ec020a6fcfe7%3Adelegate-task%3A162-code-review-20261013-r2.

| Point | Vérification et disposition |
|---|---|
| D1 — perte du Child lors d’une erreur de libération | Constat moyen retenu et requis avant G8. ROOT a confirmé dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/remote_worker.rs:2183 et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/managed_process.rs:356 que release().map_err()? consomme le Child et perd ce handle si l’écriture RELEASE échoue. Même défaut si set_status_nonblocking échoue après libération. L’owner corrige ces deux fichiers ; oracle de primitive et replay R6 ciblé attendus. Aucun résultat R6 anticipé. |
| L1 — crash entre Created et marqueur | Limite conservée : un crash worker dans cette fenêtre peut laisser Unknown et sa réservation de quota. La fermeture du bootstrap ne justifie pas un redispatch. Cette limite doit figurer dans le reçu de livraison. Aucun scénario de zombie Linux n’a été exécuté ; l’analyse du reviewer est une inférence, pas une preuve de recette. |

Converge3 reste un verdict historique correct sur la source examinée avant D1. Le correctif D1 appartient à la phase de revue/audit, avec réattestation ciblée après correction. Aucun Converge4 ni nouvelle ronde complète n’est demandé. Les **44/44 tâches planifiées restent figées**, SHA-256 746adf1b544e476ffe21e79131cb9ec2ad0b89c6bd90c0870475ecd242e800ea ; ce compteur ne vaut pas gate final. **Pipeline In Progress : D1 en cours, G8/G9 ouverts.**

## D1 — R6 réussi, transition corrigée après relecture, R7 attendu

R6 donne **29 lib PASS et 4 E2E PASS, zéro échec, quatre helpers ignorés, RC0**, en 59,578 s au total. Les journaux donnent 2,38 s et 30,08 s de tests. Le reçu atteste sources_unchanged=true, test-support, umask 0077 et TMP canonique /private/tmp/bq0davs1t. Commandes exactes, binaires et manifest sont conservés dans les preuves suivantes :

| Fichier | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-D1-release-r6.json | 65c686b7bd5b095552889a99e991e0b2aae9db8479572b78a77a31dd93e98358 |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-D1-release-r6.log | 84e3577dc29263bdb20eaae36fbc466cc5db9efb18e4c700bff0fa2bd8d2a2dd |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-D1-release-r6.sources | fcae9dbbe088fd0b05c09836d33e185ff9a12c1248d1d9817b6206b978b57197 |

Le correctif réemploie release_retained : RunningManagedChild et marqueur restent disponibles en cas d’erreur EPIPE, write ou flush, et l’API release historique reste compatible. Le worker conserve le handle avant set_status_nonblocking et avant stockage. L’état transitoire du canal en erreur empêche une lecture bloquante ou un faux NativeReady en cas d’échec de stockage ; l’arrêt attesté est demandé avant persistance. Aucun nouveau store ni framework.

La lecture indépendante a confirmé la conservation du handle, puis trouvé que native_status refusait failed=true lorsque la phase était ReleaseAuthorized. ROOT a vérifié et fait corriger cette garde et l’oracle existant. **R6 reste historique sur ses sources ; le replay exact R7 worker162_marqueur_et_permis_precedent_release est en cours, sans résultat anticipé.** D1 reste ouvert avant G8, en attente de cette preuve et de la relecture finale ciblée. Les 44 tâches restent figées ; aucun Converge4, test de confort ou nouvelle FULL.

Le budget temporaire de rétention du cache passe de 45 à 46 Gio pour R7, afin de conserver le profil debug observé à 44,69 Gio et encore utile à 163. Il s’agit d’une règle de rétention, pas d’un plafond strict ni d’un objectif de consommation. Aucun gain physique n’est revendiqué. G8/G9 restent ouverts.

## D1 clos — replay R7 et relecture ciblée conformes

ROOT autorise la clôture de D1 après **R7 : 1 PASS, zéro échec/ignoré, RC0, 9,804 s au total**. Compilation 7,95 s selon ROOT ; le journal donne 0,15 s de test. Le reçu atteste sources_unchanged=true, umask 0077, TMP canonique et rétention temporaire 46 Gio.

L’oracle exact est remote_worker::tests::worker162_marqueur_et_permis_precedent_release. La commande conservée dans le JSON utilise nice 10, scripts/build.py, cargo test --locked --offline --jobs 2, bridget-daemon, test-support, --lib, --exact et deux threads. Le rédacteur a relu les preuves et vérifié les deux empreintes de source, sans exécuter de test.

| Preuve ou source | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-D1-transition-r7.json | efadfd309186e1c99fad5c57f9ac1d8b8cb509eebf83ae9d7c2b5b4a6ff183ba |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-D1-transition-r7.log | a037aaeed1ef85ae70024782960df75632b538baa4fa66f1f23eb107e46ed48d |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/remote_worker.rs | 3ee35d1a54376f40891c2b8d9f4eb763d1d1a2726076ca936471d584721fb2d6 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/managed_process.rs | 1d4937145011edc55ff9875a83f25998dd4a000d7a962b526237d1116d040c83 |

La relecture indépendante ciblée confirme D1 résolu : le handle est conservé ; failed=true est admis en ReleaseAuthorized avec marqueur, sans promouvoir la phase. failed=false reste interdit dans cette situation. ROOT rapporte également diffcheck RC0. R6 reste la preuve des primitives et des quatre E2E sur les sources avant la garde finale ; R7 complète cette preuve pour la transition corrigée. Aucun nouveau passage complet ni Converge4 n’a eu lieu.

Les gates de test et de revue propres à D1 sont clos. Le verdict R2 APPROVE_WITH_CHANGES est conservé avec cette réponse ciblée, sans le renommer APPROVE. **44/44 tâches planifiées, SHA tasks inchangé 746adf1b544e476ffe21e79131cb9ec2ad0b89c6bd90c0870475ecd242e800ea.** Le scoring et le validateur d’audit sont encore en finalisation : aucun grade anticipé. G8/G9 restent ouverts, aucun succès global ni livraison déduit. La limite L1 reste applicable.
