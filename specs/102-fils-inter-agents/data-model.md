# Modèle de données 102

Toutes les tables sont dans la base existante ouverte par Store. Noms SQL ici
normatifs ; types Rust à définir dans le contrat transport. Aucun corps de fil
dans le ledger global. UUID canoniques ; séquences entières positives ; instants
Unix en secondes, calculés par le daemon. Le temps ne définit jamais l'ordre.

## Tables

### discussion_threads

`thread_id TEXT PRIMARY KEY`, `creator_id TEXT NOT NULL`, `title TEXT NOT NULL`,
`created_at INTEGER NOT NULL`, `closed_at INTEGER NULL`, `last_seq INTEGER NOT NULL DEFAULT 0`,
`body_bytes INTEGER NOT NULL DEFAULT 0`, `schema_version INTEGER NOT NULL DEFAULT 1`.

last_seq et body_bytes sont des compteurs transactionnels, pas des caches libres :
ils permettent l'allocation monotone et le contrôle de quota sans SUM/COUNT de
l'historique à chaque dépôt. Tests de cohérence avec les entrées. Aucun delete,
edit ou réouverture V1. close est autorisé au seul créateur, de manière rejouable.

### discussion_members

Clé primaire `(thread_id, agent_id)` ; `acked_seq INTEGER DEFAULT 0`,
`last_ack_receipt TEXT NULL`, `last_ack_through_seq INTEGER NULL`.
Clé étrangère thread_id ; index `(agent_id,thread_id)` pour list.
Membres fixés à la création ; créateur inclus, doublons dédupliqués avant validation.
Agent déconnecté autorisé s'il possède une identité connue du catalogue durable.
Pas de suppression en cascade sur disparition de présence ; pas de transfert
implicite d'appartenance à un nouvel UUID portant le même nom.

### discussion_entries

Clé primaire `(thread_id,seq)` ; `message_id TEXT UNIQUE NOT NULL`,
`author_id TEXT NOT NULL`, `body TEXT NOT NULL`, `created_at INTEGER NOT NULL`,
`notify_json TEXT NOT NULL`, `reply_to_seq INTEGER NULL`.
FK `(thread_id,author_id)` vers les membres ; reply_to vérifié dans le même fil.
notify_json contient les UUID effectivement ciblés (triés, dédupliqués, auteur
exclu) et le mode demandé `none|targets|all` pour la restitution.
L'entrée reste lisible de tous les membres, même si seuls deux sont ciblés.

### thread_operations

Clé primaire `(actor_id,operation_id)` ; `action TEXT NOT NULL`,
`thread_id TEXT NOT NULL`, `canonical_hash TEXT NOT NULL`,
`result_json TEXT NOT NULL`, `created_at INTEGER NOT NULL`.
Actions : create/post/close. opération UUID préparée avant le premier appel par
le client ou l'agent qui souhaite survivre à la perte de son premier reçu.
Conserver le reçu de succès tant que le fil existe ; pas de TTL caché de rejeu.
Au plus une création, une clôture et une opération par entrée : close sur fil
déjà clos avec une nouvelle clé est refusé sans insertion. Rejouer exactement
la clé originale conserve son succès. Les erreurs ne s'accumulent pas en table.
Pour create, thread_id et résultat sont créés dans la même transaction que le fil.
Canon JSON stable ; ensemble des membres/cibles trié ; corps conservé octet pour
octet. Ne pas inventer de sérialisation cryptographique : réutiliser les helpers
de canonicalisation/empreinte099 quand leurs signatures conviennent.

### thread_reads

Clé primaire `(thread_id,agent_id)` ; `receipt_id TEXT UNIQUE NOT NULL`,
`base_seq INTEGER NOT NULL`, `through_seq INTEGER NOT NULL`,
`snapshot_seq INTEGER NOT NULL`, `expires_at INTEGER NOT NULL`,
`requested_limit INTEGER NOT NULL`.

Un reçu actif maximum par membre/fil. Reçu opaque UUID aléatoire non assimilé à
une autorisation : l'identité attestée reste obligatoire. La page se reconstruit
depuis les entrées immuables ; pas de copie du corps dans thread_reads.
Invariants `base_seq < through_seq <= snapshot_seq`, aucune page vide avec reçu,
borne réelle through_seq conservée. Après ACK, ligne supprimée et dernier reçu
confirmé conservé dans discussion_members. Après expiration, remplacement
autorisé par read depuis acked_seq inchangé ; l'ACK de l'ancien reçu échoue
seulement après remplacement. Tant que la ligne courante est intacte et que
base_seq correspond, son ACK tardif reste valide, sans relecture forcée.

### thread_wakes

Clé primaire `(thread_id,agent_id)` ; `pending_seq INTEGER NOT NULL DEFAULT 0`,
`generation INTEGER NOT NULL DEFAULT 0`, `active_through_seq INTEGER NULL`,
`delivery_id TEXT NULL`, `delivery_key TEXT NULL`, `recipient_instance_id TEXT NULL`,
`delivery_generation INTEGER NULL`, `issued_at INTEGER NULL`, `expires_at INTEGER NULL`,
`state TEXT NOT NULL`, `reason TEXT NULL`, `dispatched_seq INTEGER NOT NULL DEFAULT 0`,
`last_attempt_at INTEGER NULL`, `last_uncertain_generation INTEGER NULL`,
`last_uncertain_seq INTEGER NULL`.

`pending_seq` = plus grande séquence explicitement adressée à ce membre, conservée
comme watermark même après satisfaction ; ce n'est pas « dernier message du fil ».
`dispatched_seq` = plus haute borne dont l'alerte est confirmée injectée, pas lue.
`active_through_seq` = borne figée d'une remise réservée et possiblement en vol.
`generation` est celle de la sollicitation ; delivery_generation est celle du
transport099. expires_at porte l'échéance d'injection120s, pas le TTL de clé099.
Les identités/clefs de livraison figées ne sont jamais recalculées après crash.
Besoin à envoyer si `pending_seq > max(acked_seq, dispatched_seq)` et aucune
remise active non échue. Si la dernière remise est inconnue et échue, exiger en
plus pending_seq>active_through_seq ; conserver sa borne/génération dans les
champs last_uncertain avant nouvelle réservation. Un ACK tardif est corrélé à sa
génération, jamais appliqué à la suivante. Les messages sans mention ne modifient
pas cette ligne. Les champs uncertain ne conservent qu'un dernier diagnostic,
pas une seconde collection illimitée de reçus099.

Pas de tableau de toutes les alertes en RAM ; historique de transport disponible
dans les reçus099. Une ligne ne peut porter qu'une génération active, et conserve
pending_seq supérieur pour le départ suivant. État synthétique exposé :
`none`, `pending`, `in_flight`, `dispatched`, `refused`, `outcome_unknown`,
`satisfied_by_read`, `cancelled`. Raisons d'attente : `offline`, `dnd`,
`capability_unavailable`, `busy`, `rate_limited`.

## Invariants transactionnels

1. Un succès post implique entrée + opération + intentions de toutes les cibles.
2. Une erreur post ne laisse aucune de ces trois mutations ni ACK piggyback.
3. `(thread_id,seq)` est strictement croissant, sans saut dû à un rollback.
4. acked_seq n'avance que par ACK de reçu valide ; jamais par publication ou
   notification. `0 <= acked_seq <= last_seq`.
5. Un ACK jusqu'à N ne touche ni pending_seq>N ni une remise figée au-delà deN.
6. Réservation de wake et clé de livraison sont durables avant projection vers099.
7. Aucun IO fournisseur sous la transaction Store ; pas de faux commit atomique
   entre les connexions distinctes Store et IdempotencyStore.
8. Un close empêche toute nouvelle réservation ; l'envoi déjà en vol reste tracé.
9. La vérification de quota et l'insertion sont dans la même transaction.
10. Les FK sont activées/vérifiées par connexion pour ces tables ; des tests
    doivent constater le refus effectif, pas seulement la présence du texte SQL.

## Courses critiques à reproduire

| Avant | Action concurrente | Résultat obligatoire |
|---|---|---|
| curseur10, page11–20 émise | post21 mentionne le lecteur | ACK20 laisse21 non lu et encore sollicité |
| pending20, remise figée20 | mention21 | enveloppe20 inchangée ; pending21 conservé |
| lecture11–20 non confirmée | deuxième read | même reçu/plage ; pas de saut à21 |
| reçu11–20 confirmé | rejeu ACK même reçu | already_acknowledged ; curseur20 |
| ancien ACK20, curseur30 | rejeu ACK20 | receipt_obsolete, aucune baisse ni avance |
| post commis, aucune projection | crash | reprise de la même intention et de la même clé |
| injection possible, ACK perdu | redémarrage | outcome_unknown visible ; pas d'envoi sous nouvelle clé |
| outcome_unknown20, échéance passée | mention21, puis ancien ACK20 | nouvelle génération pour21 autorisée ; ACK20 ne l'acquitte pas |
| reçu20 âgé de11min non remplacé | ACK20 | accepté si base_seq correspond, sans relire |
| reçu20 âgé de11min remplacé par read | ACK ancien reçu | receipt_obsolete, aucun changement |
| pending20, pas encore parti | ACK20 ou close | aucune nouvelle remise pour20 |

## Migration et limites

Créer les tables/index en transaction via Store::init_schema ; version des fils
séparée du schéma idempotence. Test sur copie synthétique de base pré102 avec
ledger et demandes099 ; ouvrir deux fois puis vérifier contenu/invariants.
Ne pas utiliser ni copier la base réelle comme fixture.

Les quotas logiques portent sur octets UTF-8 des corps ; index/pages SQLite/WAL
ajoutent un overhead, donc ne pas vendre le quota comme limite exacte du fichier.
Pas de suppression automatique ni de nouvelle politique de rétention globale.
À plafond : lectures, ACK et clôture continuent ; sauvegarde/entretien restent
les procédures existantes, sans outil de purge nouveau dans cette spec.
