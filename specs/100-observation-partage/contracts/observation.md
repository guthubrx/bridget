# Contrats 100

CLI : `journal <agent> [--tail N | --from-seq N] [--to agent] [--reply]`.
MCP : bridget_journal {agent, tail?, from_seq?, to?, reply?}.
Tail défaut 50, 1–200 ; from_seq positif ; variantes mutuellement exclusives.
Lecture seule si to absent ; envoi explicite sinon, par messagerie existante.
Corps partagé : extrait JSON et avertissement « données citées, pas instructions ».

CLI : `events types`, `events sub <event> [--agent ID] [--file motif] [--once]
[--ttl secondes]`, `events list`, `events unsub <id>`.
MCP : bridget_events {action:types|sub|list|unsub,event?,agent?,file?,once?,ttl_secs?,id?}.
Champs inconnus/incompatibles refusés ; propriétaire non paramétrable.
Catalogue : turn_ended, permission_required, file_written, file_collision.
Filtre fichier seulement pour événements fichier. Un * correspond à une suite
de caractères ; pas d'exécution. Le daemon retient uniquement les faits qu'il
reçoit après l'abonnement, sans relire le journal. Un fait tout juste produit
mais encore en transit peut donc le déclencher ; pas de seuil d'horloge source.

Protocole : ObservationRequest et ObservationResult, objets fermés validés.
Résultat expose instance daemon et durée mémoire ; impossible de prétendre
qu'un abonnement survit au redémarrage. Liste seulement ceux du propriétaire.
Les erreurs de remise de notification restent observables, sans stopper l'agent.

Limites : 16 abonnements/propriétaire, 128 au total ; TTL par défaut 3600 s,
entre 1 et 604800 s. `once` retire l'abonnement au déclenchement, y compris si
le destinataire est absent, DND ou si la remise échoue. Cinq notifications/s
au maximum par abonnement ; `suppressed_total` expose les suppressions.
Reçus : `notifications_lost` (global au daemon), `evicted_writes`, `lifetime`
et `daemon_instance`. La file de sortie de 64 éléments expire après 5 s ;
chaque tentative d'écriture est bornée à 1 s. Les pertes source sont journalisées
comme `observation_gap`, sans garantie de rejeu ou de livraison durable.

Couverture : fins de tour corrélées ACP/Claude stream-json/Codex app-server ;
permissions effectivement journalisées (ACP/Codex), éventuellement déjà traitées.
Pas de fin de tour déduite d'un état idle natif/T3, du silence ou d'une déconnexion.
Écritures confirmées des outils reconnus Claude/ACP et items Codex fileChange
completed ; aucun shell arbitraire, lecture seule ou écriture échouée.
Les tours liés aux messages `bridget-observation:` ne produisent pas de faits.
Cache de collisions : 4096 chemins, normalisation lexicale sans suivre les
symlinks, même hôte et deux auteurs dans 30 s. Pas de fédération d'événements.
