# Implémentation156 — guidage et lecture de file

État de clôture documentaire : T029/T030 sont validées par le principal dans
leur périmètre. Les lectures T031 attribuées et le contrôle du principal sont terminés.
ROOT valide T031 et le statut Implemented, 31/31 tâches.
Aucun Converge, audit final ou livraison annoncé.
Base livrée155 : 59da3968c7d14dbd7acd1370875dd9d135ba7dcb ; métadonnées be7957ee.

## Comportement et réemploi

Send expose queue/steer/interrupt. Absent ou queue garde les octets ordinaires.
L’intention est posée avant le canon. Le reçu contrôle rend effect=unconfirmed,
sans promesse d’effet fournisseur ; interrupt garde la FIFO. La garde pure
partagée vérifie connexion, parenté, projet, pause, mission et rythme. From ou
origin client ne donnent aucun droit. Attach contrôle explicitement ; l’UI
attestée demeure Client. ChildProof ferme les façades contrôle/queue avant connexion.

SendIdempotent admet guidance_events dans begin_send_delivery_with_guidance,
la même transaction de remise, suivi, lien et journal. Une erreur d’INSERT
annule ces écritures. Historique/Control/Cancel inscrivent l’audit durable avant
émission physique ; un crash peut consommer le rythme sans émission. Control
réserve canon/rejeu avant la garde, puis vérifie avant dispatched/comptage/émission.

Le rythme agent est de 1 interruption/30 s par couple et 3/600 s par cible.
LAST/THIRD_LATEST et now rendent la frontière et retry_after exacts. Attach
et pause interne sont exempts. Une mission native active interdit interrupt
agent ;152 reprend pause/panne, pas une interruption utilisateur terminale.
Aucun steer agent expiré ne devient automatiquement une interruption.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/implementation.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/implementation.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/implementation.md#extrait-3).

## Tests et limites de preuve

Une seule suite complète : 700,10 s, RC101, 2350 PASS Cargo/11 FAIL/73 ignored.
Les trois PASS de processus enfants imbriqués sont séparés, sans double comptage.
Les 216 sources sont stables pendant cette exécution. Les 80 unitaires156 et
les 3 E2E passent dans cette suite ; ils ne s’ajoutent pas à ses 2350 PASS.

Trois nouvelles fixtures : deux témoins lib corrigés par identité/parenté réelles
(2 PASS, 8,27 s), puis spec138 adapté au refus durable attendu pour un client nu
(1 PASS, 7,67 s). Ce dernier ne prouve pas une admission de parent. Les huit
modes d’échec hérités sont confirmés par les mêmes messages/sites155 et restent
hors périmètre. Leurs causes fines non établies demeurent déclarées.
Le FULL initial conserve RC101/11FAIL. Aucune seconde suite complète ni succès
global sur le dernier hash de fixtures n’est revendiqué.

La comparaison T029 finale prouve 763/763 PASS distincts par cumul : 762 dans
le FULL et 1 témoin control dans la reprise ciblée. Zéro manquant,7 ignored
inchangés. Les 774 PASS bruts initiaux incluaient 2 enfants imbriqués et 9
recouvrements de filtres : 763 noms distincts, pas 774. Les anciens wrappers gardent leur code jusqu’à relance.
Les fournisseurs E2E sont privés/factices : aucun modèle réel ou nouvelle
capacité runtime Claude n’est démontré. Le modèle ADR011/027 ne protège pas
contre un processus hostile du même compte qui forge Attach/lit les preuves.

## Articles XIX et XX — revue et responsabilité

La lecture daemon attribuée couvre 78 hunks et ses deux fixtures corrigées ;
aucun défaut nouveau confirmé dans ce périmètre. La capture initiale 49 fichiers,
36 Rust inclut les trois fichiers nouveaux ; une capture n’est pas une lecture.
ROOT couvre ses sources/documents, git154 les 78 hunks daemon, et le lecteur
complémentaire les 179 hunks de 34 autres fichiers Rust, 5973 lignes de patch,
guidance 392 / E2E 1044 inclus. Son delta spec138 couvre 3 hunks / 84 lignes et les
lignes 157–221 actuelles. Fin attribuée 05:26:01 UTC ; aucun défaut fonctionnel
nouveau confirmé, aucun C/H. La validation globale T031 est acquise par ROOT sur ces lectures et ses documents.

Les ajouts servent des parcours déjà présents, avec la même garde et les mêmes
stores. Le module pur centralise plusieurs puits ; la table justifie le rythme
persistant. QueueList pure diffère de la reprise mutable ; cancel_queued ne peut
atteindre l’actif. Aucun refactor ni test de confort n’est proposé. Zéro ligne
supprimable à comportement constant est étayée dans les portions déjà relues ;
le lecteur complémentaire retient une réserve LOW : environ 2 lignes froides
de boucle parent_source en test, placées au backlog sans correction additionnelle.

Réponses XX : besoin explicite de contrôle/file ; réemploi des mécanismes
existants ; hypothèses de provenance/effet fermées par gardes ; ownership ROOT
pour l’application/livraison ; abstractions liées à droits/atomicité/compatibilité ;
suppression potentielle LOW limitée aux 2 lignes froides de test, sans
modification supplémentaire du périmètre. L’humain dispose
des limites de reçu, de preuve, de rythme et de capacité, sans succès inventé.

ROOT compare aussi 216 hashes FULL/état final : 214 inchangés, deux deltas de
fixtures seulement (daemon.rs et spec138_project_test.rs). Cela ne constitue
pas un garde de sources avant/après chaque reprise ciblée. Les reçus ciblés
n’ont pas ces snapshots ; aucune stabilité rétroactive n’est inventée.

## Reçus

/Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-full156-once.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T029-full156-once-comparison.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-two-old-fixtures-green.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-spec138-refusal-green.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/final-docs/T031-daemon-review.md
/Users/moi/.cache/bridget-delivery156.iuy6stuo/final-docs/T031-daemon-readmanifest.json
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/specs/156-guidage-file/validation/full156-source-delta-final.json

Log FULL SHA-256 :fdb7deaff78eb9fa82a7af2879f59c3a0dd5858300450e8e198d80bd891e0274.
Diff lu daemon SHA-256 :1991ad415fa4573d6cfd6a3eea077528f322e6d95bb4d9d62e7695fd91dbfaa0.

Reçus complémentaires T031 lus par ROOT :
/Users/moi/.cache/bridget-delivery156.iuy6stuo/final-docs/T031-other-rust-review.md
/Users/moi/.cache/bridget-delivery156.iuy6stuo/final-docs/T031-other-rust-readmanifest.json

## Complément T032 — FR016 corrigée après le FULL

Le correctif trace les premiers refus Steer/Interrupt de préflight et les refus
antérieurs à une admission réussie. record_guidance_outcome reste réemployé avec
faits disponibles ; record_guidance_refusal, privé, couvre quatre préflights et
l’indisponibilité des faits. Identité, labels et store sont réemployés. Aucune
nouvelle table, API publique ou couche service. Les traces refused n’admettent
ni quota ni remise. Rejeu, refus authorize déjà tracé, pause interne et échec
fournisseur après admitted ne créent pas de seconde trace. Une panne du journal
conserve le warning et le refus, sans autoriser d’effet.

L’oracle existant guidance156_rate_refus_non_compte a un RED réel : 0 PASS/1 FAIL,
RC101, 13,75 s ; le premier refus projet avait zéro trace au lieu d’une. Après
correctif, 93 PASS/0 FAIL, RC0, 21,91 s : 80 guidance156, 11 spec087, 2 témoins lib.
Une reprise distincte des trois E2E prévus passe : 3 PASS/0 FAIL, RC0, 17,87 s.
Chaque exécution conserve ses 216 hashes sources avant/après. Les deux GREEN
portent le même ensemble de sources ; daemon final SHA-256 :
`a11310b737d1ac186309a57f306babe3146ec8af7f57a921bc6adf6b398c3c3c`.

L’extension prouve les refus pair/projet/persistance idempotents, leurs faits
réels et le rejeu projet sans nouvelle trace, quota ou émission. Les autres
puits sont raccordés et relus ; aucun oracle nouveau couvrant chaque branche
n’est inventé. Aucun helper de test, scénario nommé ou modèle réel ajouté.

La production T032 est postérieure au FULL unique. Les affirmations précédentes
« aucun changement de production après FULL » et le reçu de deux seules fixtures
sont des checkpoints antérieurs à T032. Le FULL reste 2350 PASS Cargo + 3 PASS
enfants séparés/11 FAIL/73 ignored ; huit modes hérités confirmés et trois
fixtures corrigées par reprises 2+1. Les93+ 3 ciblés ne sont pas ajoutés au FULL.
Aucune seconde suite complète, couverture globale du hash final, réussite
Converge, audit final ou livraison n’est annoncée par ce complément.

Après les deux GREEN, ROOT a restauré les blobs de départ exacts de cinq
sous-modules suivis par rustfmt. La reproduction de cette mise en forme est
byte-identique pour chacun : aucun changement métier. Les 216 sources étaient
stables pendant les runs ; cinq hashes de forme ont ensuite changé par cette
restauration. Daemon production/test reste au SHA a113 ci-dessus. Aucun rejeu
pour cette seule mise en forme, ni couverture globale actuelle inventée.
Reçu : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T032-format-only-restored.json

Reçus T032 :
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T032-fr016-red.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T032-fr016-green-regressions.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T032-fr016-e2e-green.json
