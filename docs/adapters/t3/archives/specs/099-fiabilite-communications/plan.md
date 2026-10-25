# Archives techniques de l’adaptateur T3

Document d’origine : [specs/099-fiabilite-communications/plan.md](../../../../../../specs/099-fiabilite-communications/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Rust 2024, std::thread, sockets Unix, serde, SQLite/rusqlite ; dépendances inchangées.
Autorité unique conservée. Essais sur états temporaires privés et faux t3code déjà
présent ; aucune opération sur le daemon installé. Les scripts et templates du
moteur SpecKit local sont absents : protocole des skills appliqué directement.

<a id="extrait-2"></a>

## Extrait 2

1. **US1 — remise bornée hors verrou global.** Réutiliser
   push_control_message_until (daemon.rs:1553), déjà testé pour l'annulation :
   cloner le writer sous verrou, préparer/persister le suivi avant remise pour
   éviter la course d'une réponse immédiate, libérer le verrou global, écrire avec
   budget d'une seconde, puis réacquérir uniquement pour finaliser. Ne jamais
   annoncer Ack sur écriture échouée ; fermer une trame partielle comme le helper
   existant. Conserver les gardes et la monotonie des demandes.
2. **US2 — rattachement explicite.** Étendre Registered d'un credential optionnel,
   émis seulement pour la connexion propriétaire avec instance. Ajouter une seule
   trame RegisterAuxiliary portant agent, instance et preuve aléatoire ; la lier à
   la connexion canonique vivante. Réutiliser conn_names, conn_instances,
   auxiliary_connections et mcp_identity. Owner renouvelle le fichier privé
   d'identité après son enregistrement ; MCP et CLI y lisent la preuve sans
   journalisation. Les clients Role Client s'attestent également, sans transformer
   issuer_scope en autorisation. Contrôler SendIdempotent avant réservation et
   supprimer l'attribution par simple from/from_declared du CLI. Révoquer les
   attestations existantes à la fin de l'incarnation et documenter le refus des
   anciens auxiliaires. Pas de serveur OAuth, nouvelle table ou dépendance.
3. **US3 — attente t3code réactive.** Étendre LinkWorker et sa file existante :
   l'attente d'un fil occupé ne monopolise plus le traitement des contrôles.
   Annulation structurée pour transport t3code dans le daemon, pas un nouveau
   tour contenant une consigne textuelle. Contrôler expiration et perte de
   connexion avant tout dispatch ; état indéterminé si la frontière d'acceptation
   a été franchie sans résultat connu. Maintenir la sérialisation par fil.
4. **US4 — réponse durable jusqu'à confirmation.** Étendre ThreadState/Pending,
   avec valeurs par défaut pour anciens fichiers, pour conserver le texte de
   réponse préparé et son identifiant stable. Réutiliser Ack/Nack et la lecture
   des demandes suivies pour lever une issue inconnue après reconnexion.
   Ne supprimer qu'après confirmation/état terminal attesté. Pas de réexécution
   fournisseur pour rejouer une réponse. Corriger mem::take suivi de sauvegardes
   partielles : chaque sauvegarde contient l'ensemble des attentes restantes.
   Réutiliser write_private_file_atomic pour l'état durable.
5. **US5 — observation honnête.** Retirer la coupe silencieuse à 4 096 caractères
   et utiliser les bornes/fragmentations déjà disponibles du journal. Un refus de
   publication conserve le message à reprendre ou émet une lacune explicite.
   La séquence et les identifiants existants restent l'autorité anti-doublon.

<a id="extrait-3"></a>

## Extrait 3

- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs : remise, admission, rattachement, annulation.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs : contrat d'identité étendu.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp_identity.rs : identité privée et preuve.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/communication/client.rs : client auxiliaire.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs : admission des envois idempotents.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs : conservation après enregistrement.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs : contrôle, réponses et journal.
- Tests existants correspondants et tests 099 ciblés ; ajuster les constructeurs
  du protocole affectés sans changer leurs scénarios hors périmètre.

<a id="extrait-4"></a>

## Extrait 4

Tests/US1 puis US2 dans daemon.rs (pas d'écritures concurrentes du même fichier).
US3–US5 peuvent être travaillées ensemble dans t3code.rs, indépendamment du cœur.
Intégration, contre-revue, convergence exigence → preuve, puis audit du diff.
