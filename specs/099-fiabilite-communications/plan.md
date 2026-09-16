# Plan 099 — Fiabilité et identité des communications

Statut : Implemented. Branche : session-099-fiabilite-communications.
Spec : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/099-fiabilite-communications/spec.md.

## Contexte technique

Rust 2024, std::thread, sockets Unix, serde, SQLite/rusqlite ; dépendances inchangées.
Autorité unique conservée. Essais sur états temporaires privés et faux t3code déjà
présent ; aucune opération sur le daemon installé. Les scripts et templates du
moteur SpecKit local sont absents : protocole des skills appliqué directement.

## Choix et réutilisation de l'existant

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

## Fichiers impactés

- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs : remise, admission, rattachement, annulation.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs : contrat d'identité étendu.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp_identity.rs : identité privée et preuve.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/communication/client.rs : client auxiliaire.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs : admission des envois idempotents.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs : conservation après enregistrement.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs : contrôle, réponses et journal.
- Tests existants correspondants et tests 099 ciblés ; ajuster les constructeurs
  du protocole affectés sans changer leurs scénarios hors périmètre.

## Modèle de menace et compatibilité

L'attaquant connaît agent/instance et peut ouvrir le socket mais ne possède pas la
preuve privée. Un processus hostile du même compte pouvant lire cette preuve
reste hors cloisonnement garanti. Le jeton ne sort ni dans who/status ni dans logs.
Un token ancien ne survit pas à un changement de connexion propriétaire.
Ancien propriétaire sans auxiliaire demeure compatible ; auxiliaire ancien :
refus nommé, mise à jour wrapper/client nécessaire. Tester les accès légitimes
locaux et la lecture d'identité privée côté distant sans supposer que le chemin
du daemon existe sur la machine du wrapper.

## Stratégie de validation

Tests avant corrections pour chaque famille. Combiner tests unitaires des états
et tests d'intégration aux frontières (vrais sockets, base temporaire, faux HTTP).
Réutiliser les fixtures existantes ; ne pas remplacer leur comportement métier
par des mocks internes. Feature Gherkin 099 décrivant les acceptations, tests Rust
nommés spec099 et commandes explicites dans quickstart.md.
Tests de régression + workspace selon compatibilité et ressources, fmt, clippy.
Ne pas interpréter le nombre de tests comme une preuve d'exhaustivité.

## Gates constitutionnels et Article XIX/XX

- PASS : demande explicite, nouvelle session, français, aucun commit/déploiement.
- PASS : source/état isolés ; aucun secret réel dans les essais.
- PASS : protections existantes réutilisées, zéro nouvelle dépendance/service/table.
- Extension de protocole justifiée par trois usages réels : mutations auxiliaires,
  Send classique et SendIdempotent ; identité déclarée insuffisante aujourd'hui.
- État supplémentaire de réponse nécessaire à la reprise, stocké dans l'état
  existant du fil. Pas de nouvelle file de jobs générale.
- Le code de compatibilité historique n'est pas supprimé sans preuve d'inutilité.
- Aucun test utilisateur payant, aucune modification de configuration fournisseur.

## Ordre d'exécution

Tests/US1 puis US2 dans daemon.rs (pas d'écritures concurrentes du même fichier).
US3–US5 peuvent être travaillées ensemble dans t3code.rs, indépendamment du cœur.
Intégration, contre-revue, convergence exigence → preuve, puis audit du diff.
