# Archives techniques de l’adaptateur T3

Document d’origine : [specs/100-observation-partage/plan.md](../../../../../../specs/100-observation-partage/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. Étendre attach.rs : lecture non interactive via AttachClientState, Subscribe,
   AttachWindow et SnapshotCaughtUp. Réutiliser assemblage, lacunes et erreurs,
   sans second lecteur JSONL sur disque. Extrait JSON sourcé limité à 64 Kio et
   200 entrées ; tail=50 par défaut ; from_seq optionnel ; notices et next_seq.
2. Réutiliser DaemonConnection et son budget global de 10 secondes. Exposer sa
   lecture de réponse au module attach ; fermeture de connexion annule le replay.
3. CLI journal et outil MCP bridget_journal : lecture, ou partage facultatif
   `to` par send existant. `reply` autorisé uniquement avec destinataire. Pas de
   nouvel artefact obligatoire, ni stockage doublon ni moteur de résumé.
4. Ajouter un petit état d'observation daemon, indépendant de Maicie : abonnements
   et écritures récentes. CoordinationSubscribe reste le contrat guichet ;
   Subscribe reste attach. Ajouter le contrat ObservationRequest/Result fermé.
5. Catalogue : turn_ended, permission_required, file_written, file_collision.
   Propriétaire depuis live_connection_identity ; filtres facultatifs agent et
   fichier (* seulement), once, ttl_secs. Limites : 16 par agent, 128 global,
   TTL 1–604800 s ; expiration 3600 s par défaut, conservation mémoire du daemon.
6. Collecter les fins de tour corrélées, permissions et écritures depuis le
   journal confirmé après flush. Ajouter à JournalLiveFeed une file indépendante
   de 256 métadonnées : le flux attach existant n'est drainé que lorsqu'une vue
   est attachée et ne peut servir de second consommateur. Relayer les métadonnées
   sans vue attach ; aucun rejeu des anciens journaux. Ne pas interpréter le texte assistant,
   les commandes shell, ni les arguments tronqués. Étendre les producteurs qui
   ont les chemins bruts pour conserver cette seule métadonnée avant troncature.
7. Collision : chemin absolu lexical normalisé + hôte ; deux auteurs distincts
   dans 30 s ; ignorer reads et doublons de source ; limite de 4096 fichiers,
   anti-répétition de la dernière paire/fichier dans la même fenêtre, complétée
   par la borne de débit pour plusieurs auteurs. Pas de lecture disque
   ou canonicalize sous le verrou. Signaler l'éviction/indisponibilité observée.
8. Notifications par messages système ordinaires, sans reply ; sortie bornée
   hors verrou et file bornée afin qu'un abonné lent n'immobilise pas la source.
   Limite supplémentaire de cinq notifications/s/abonnement, suppressions visibles.
   Les tours corrélés à un message bridget-observation ne nourrissent pas eux-mêmes
   ce catalogue. Un simple état busy→idle sans corrélation ne suffit pas : les
   adaptateurs natifs/T3 ne sont pas annoncés compatibles pour cette observation.
9. CLI events et outil MCP bridget_events : types/sub/list/unsub. Un point de
   validation daemon, erreurs explicites, pas d'exécution de SQL ou scripts.

<a id="extrait-2"></a>

## Extrait 2

Orchestration, objectifs, leases de fichiers, détection de succès métier,
surveillance de tout le filesystem, exécution automatique d'une commande en
réaction, abonnements persistants inter-daemons, évolution du serveur T3.
