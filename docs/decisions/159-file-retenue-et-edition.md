# ADR159 — file retenue par le daemon et édition

Date : 12 octobre 2026.
Statut A : livré sous le source 53863bd09a4ff364dc2b412519b343c77e163685.
Les preuves de conception, SC004 et T011 ci-dessous restent historiques.
Statut B : adoptée, implémentée et livrée le 12 octobre 2026 ; source 928631df00ac, activation et nettoyage attestés par le reçu canonique.
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/validation/livraison159B.md

## Contexte

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/159-file-retenue-et-edition.md#extrait-1).

## Décision A

Réemployer IdempotencyStore, send_deliveries, le ledger et tracked_requests.
La phase held ne crée pas de lien d'exécution pour un envoi sans intention.
Une occurrence System/Human, une mission native ou une alerte de fil ne devient
pas un message retenu. held_queue reste false par défaut. La rétention exige
la capacité turn_attest déclarée par la vraie session gérée du destinataire.

L'admission réemploie begin_send_delivery_with_guidance. La libération garde
State de la lecture du repos jusqu'au commit Immediate dans ce même magasin.
control_state et control_pause_interruptions sont relus dans cette transaction,
sur config.db_path partagé. Aucun magasin miroir ni seconde connexion dans
le callback. Préserver l'ordre existant des verrous, sans writer socket dans
la transaction. Le commit linéarise la libération ; le push est différé hors
verrou. Une pause validée après ce commit ne reprend pas une remise déjà libérée.

queue_rank est INTEGER PRIMARY KEY AUTOINCREMENT. L'INSERT omet le rang ;
SQLite conserve la séquence après purge, sans nouveau compteur métier.
delivery_id reste UNIQUE NOT NULL et cible des FK. La migration v7 copie les
anciens rowid dans queue_rank et conserve les liens d'exécution. foreign_keys
reste activé et foreign_key_check doit être à zéro. La v7 est à sens unique ;
l'ancien binaire refuse la base v7. Sauvegarde .backup cohérente avant toute
ouverture migrante et vraie restauration isolée avec le binaire 158B installé,
empreinté et conservé avant reconstruction 159 : gate T011 obligatoire.

List et recovery trient par queue_rank. L'index non unique existant
idx_send_deliveries_recipient_phase est étendu à
(recipient_instance_id, phase, queue_rank), sans second index recipient.
AUTOINCREMENT impose un surcoût SQLite/CPU/E/S non mesuré ici ; FR008 justifie
ce coût pour interdire le recyclage du rang après purge.

La minuterie Instant du heartbeat est examinée à chaque boucle, même sous
trafic continu, pour réattester le vrai état périodiquement (30 s au plus).
Pour toute instance turn_attest, busy_period_start porte le début du busy ;
les anciennes instances gardent busy_since. La sortie forcée suit le délai
configurable notify_timeout_secs + 30 s, soit 2730 s au défaut 2700 s.
Aucun scheduler supplémentaire. Les mesures T001/T010 restent limitées au
proxy Python explicité dans le reçu SC004 ci-dessous.

La libération suit le repos attesté ou la sortie forcée explicitement prévue
par FR025. La sortie forcée expire la période occupée ; elle ne prouve pas le
repos du fournisseur et ne fabrique pas une nouvelle attestation. Un true
répété conserve busy_period_start. Cette exception et le repos réattesté ne
sont pas le même scénario ni la même garantie.

La v7 ajoute quatre colonnes : queue_rank, recipient_agent, turn_gate et acked_at.
acked_at INTEGER nullable en v7 porte la date du premier ACK, écrite avec
phase=acked ; un rejeu ne la modifie pas. La grâce de 30 s et turn_gate survivent
au redémarrage. Anciennes lignes : acked_at=NULL et turn_gate=0 ; aucune date
antérieure inventée, aucun nouveau magasin ou compteur métier.

SC001 garde l'égalité du message/remise (canon, phase et push) lorsque le
réglage est fermé, ainsi que les suites vertes et zéro nouvelle régression.
FR023 exige séparément le flux auxiliaire de réattestation ; aucune identité
de tout le flux auxiliaire n'est promise par SC001.

## Conséquences et suite

La file reste durable avant libération. Après libération, les garanties 099
et les issues indéterminées existantes s'appliquent ; aucune injection fournisseur
exactly-once illimitée n'est promise. Après SC004, la livraison A conserve
le réglage fermé.

## Mesure SC004 et raccord de reprise

Le réglage reste fermé après la mesure privée SC004. Le p95 passe de
27,763792 ms à 1028,132375 ms ; le delta vaut 1000,368583 ms et dépasse 250 ms.
La mesure suit le proxy Python « avant émission de TurnCompleted vers début
du tour suivant reçu ». Elle ne mesure pas le repos attesté du daemon.
Chaque lot a 20 échantillons, avec p95 au rang 19. Après seulement, une
précondition ListAgents vérifie busy avant admission ; cette attente est hors
mesure. Les fixtures complètes ne sont donc pas byte-identiques. Aucun SLA,
fournisseur réel ou cause du surcoût n'est établi. Le reçu canonique est :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/validation/latence-159a.md

À l'enregistrement d'une nouvelle instance attestée, avec held_queue ouvert
et dans un terminal, le daemon rattache d'abord les retenus et les remises ordinaires
encore dispatching, non ACKées et non expirées de l'ancienne instance. Le
rattachement conserve identité, génération, octets et rang ; la remise
encore dispatching réserve le tour avant libération du message suivant.
Ce raccord ne transforme pas incidents, notices ou missions natives en
messages ordinaires. La reprise reste celle de 099. Aucun ACK ni signal
terminal 149/152 n'est fabriqué ou relancé pour provoquer une libération.

T012 complète cet ADR avec les preuves A avant livraison. T025 étend le même
fichier au lot B. Les permutations B garderont les rangs existants, avec valeurs
provisoires négatives dans leur transaction unique, jamais publiées ; aucune
contrainte positive incompatible. Aucun code B ni résultat de test revendiqué.

Références primaires : [SQLite AUTOINCREMENT](https://www.sqlite.org/autoinc.html),
[SQLite Foreign Keys](https://www.sqlite.org/foreignkeys.html).

## Répétition de la restauration T011

La procédure complète reste dans :
/Users/moi/Nextcloud/10.Scripts/64.bridget/docs/reference-communication.md
Références privées réellement préparées par ROOT :
/Users/moi/.cache/bridget-restoration159A-reference-20261012/bridget158B-real-snapshot.db
/Users/moi/.cache/bridget-restoration159A-reference-20261012/bridget158B-reference
Ces références initiales restent historiques. Le PASS frais utilise :
/Users/moi/.cache/bridget-restoration159A-reference-fresh-20261012/bridget158B-real-snapshot.db
Migration et restauration emploient deux copies privées distinctes. La
sauvegarde avant remplacement de production reste à créer par ROOT.
Preuve de répétition :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/validation/restauration-v7.md
T012 conserve les preuves A et leurs limites ; T025 étend le même ADR à B.

T011 a réussi sur la référence v6 fraîche avec le debug 159A testé :
migration v7 (RC0), ancien binaire 158B refusant v7 (RC1, DB inchangée), puis
restauration v6 ouverte par le vrai binaire 158B (RC0). Les 11384 records,
57 demandes suivies, 10892 remises et zéro lien gardent leurs empreintes
logiques, échantillons et rangs. quick_check=ok et foreign_key_check=0 aux
trois étapes. Le debug testé est empreinté :
48c5ea41e10f4f0867d3754204df351bff548a2256309c5f48be3db520a2bebf
Cette preuve ne décrit pas le futur binaire release livré.

Le premier FAIL reste historique : l'ancienne référence passait de 62 à 57
demandes suivies à l'ouverture. Les cinq lignes terminées depuis plus de sept
jours ont été retirées par la maintenance de démarrage, byte-identique à 158B ;
la migration T003 n'en est pas la cause. Aucune donnée ni date n'a été corrigée
pour obtenir le PASS frais. Les phases réelles sont acked/indeterminate ;
zéro lien reste zéro lien. Les scénarios synthétiques T003 sont distincts.
Reçu FAIL conservé :
/Users/moi/.cache/bridget-docs159A-T012-20261012/private/T011-FAIL-original-private.json
Reçu PASS réel :
/Users/moi/.cache/bridget-docs159A-T012-20261012/private/T011-PASS-original-private.json
Les JSON exacts restent privés, hors dépôt. Les résumés sans corps conservent
identités, empreintes des tables et SHA256 de chaque message_bytes.
Reçu canonique et diagnostic :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/validation/restauration-v7.md


## Décision B — édition des retenus

Statut B : adoptée, implémentée et livrée ; le reçu distingue code, activation et métadonnées.
Les preuves A ci-dessus sont historiques. A est livrée sous le source
53863bd09a4ff364dc2b412519b343c77e163685 ; sa migration v7 et sa restauration
T011 ne deviennent pas une nouvelle migration B.

Cancel/reorder/promote partagent la file durable `send_deliveries` du daemon.
`queue_edits` et ses deux index sont créés dans `init_schema`, même sur une
base déjà v7, sans version 8. `queue_edit_once` est le puits privé commun aux
trois mutations : reçu et effet dans la même transaction, sans nouveau store.
State protège tour et identité jusqu'au commit. La transaction relit pause,
interruptions, réservation, saga et demande sur la même connexion. Aucun writer
socket sous transaction ; le push vient après commit et relâchement de State.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/159-file-retenue-et-edition.md#extrait-2).

Cancel réemploie clôture et notice durable existantes, sans interrupt. Reorder
permute les rangs existants dans une transaction, avec valeurs temporaires
négatives jamais publiées. Promote réemploie Steer156, le tour actif et la
réservation ; `queue_fallback` n'accorde aucun droit. HeldRelease reste différé
par la pause et la priorité de reprise. Seul l'humain Attach attesté garde
l'exception explicite prévue, avec toutes les autres gardes.

Après promotion, `applied` signifie commit et `effect: "unconfirmed"` signifie
absence de preuve d'injection. FIFO pleine : rejet et remise indéterminée ;
Codex ambigu après guidage : doublon possible. Aucun avis universel ajouté
pour `reply=false`. Rejeu sous droits frais : reçu historique, jamais nouvelle
émission.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/159-file-retenue-et-edition.md#extrait-3).

Contrat et droits :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/contracts/edition159.md
Référence d'usage :
/Users/moi/Nextcloud/10.Scripts/64.bridget/docs/reference-communication.md
