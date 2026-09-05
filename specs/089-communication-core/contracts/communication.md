# Contrat d'extraction — communication et fédération

Ce contrat conserve la sémantique de la référence Git dfa2134dcfe2a2522e3ae77d93561e6ae72556b3. Il ne prétend pas encore en avoir gelé toutes les fixtures : c'est le livrable testable T002.

## Surfaces conservées

CLI/MCP pour send/reply/who/ledger, lifecycle spawn/stop, observation attach, découverte et négociation de capacités, interfaces publiques guichet/événements pour service externe. Les détails de signature restent ceux du corpus source ; `in_reply_to`, `id`, `issued_at` et le scope stable ne sont pas optionnels au sens du retry : tous les éléments du canon doivent être rejoués sans modification.

Les appels CLI/MCP utilisent une même construction canonique et une même lecture du ledger. Aucune façade ne lit une DB distante présumée locale. Les outils MCP n'écrivent sur stdout que leur protocole négocié. Une opération de GUI/projet retirée est refusée explicitement ; elle n'est pas transformée en appel métier implicite.

## Tableau de vérité minimal

| Fait observé | Ce qui peut être affirmé | Ce qui ne peut pas être affirmé |
|---|---|---|
| Connexion/authentification valide | Identité et capacités de cette connexion | Travail effectué |
| Dépôt durable / in_flight attesté | La demande est enregistrée et en cours de remise | Le fournisseur a traité la mission |
| ACK de remise valide | L'injection reconnue par le transport est durablement attestée | Le résultat est juste ou complet |
| Réponse liée enregistrée | Demande answered ; rappels de cette demande arrêtés | Objectif Maicie réussi |
| Transmission commencée, pas d'ACK valide | outcome_unknown suivant le contrat | Échec certain, donc droit de renvoyer sous un ID neuf automatiquement |
| Gap | Séquence manquante attestée | Flux frais parce qu'un fragment ultérieur arrive |
| Unavailable | Source indisponible | Existence d'une lacune de données déjà prouvée |

Le tableau ne remplace pas les issues fermées existantes : le corpus couvrira aussi les refus et terminaux historiques, dont expiration et orphaned. La skill doit expliquer leur conduite sans uniformiser tous les cas en « réessayer ».

## Limite de l'idempotence

L'identité du retry est stable avant connexion. Même clé + même canon retrouve la même histoire ; canon divergent est refusé sans mutation. Un horizon expiré n'autorise aucune réémission automatique sous une nouvelle clé.

La preuve d'injection unique doit nommer les processus qui survivent au crash. Une panne du wrapper entre un effet fournisseur et son accusé peut laisser une ambiguïté irréductible ; aucun contrat externe ne la résout par magie. Conserver le refus/indétermination prévus par le socle plutôt que promettre une exécution exactement une fois sans transaction fournisseur.

## SSH : même protocole, un maître

Le tunnel transporte la socket Unix du même daemon maître. La connexion réseau n'est pas le mode d'agent. Les sockets, répertoires et identités d'instance test sont explicites ; aucun remplacement d'une socket existante non possédée. Vérification de clé d'hôte SSH conservée, échec de forwarding fatal au tunnel, keepalive borné, aucune ouverture d'un port applicatif public.

Reconnexion : même identité logique, nouveau contexte de connexion, nouvelles autorisations négociées ; aucune conservation accidentelle de capacité d'une connexion morte. Retry et curseur proviennent de l'état durable, jamais du dernier nom affiché. Un curseur hors rétention annonce Gap ; une coupure annonce Unavailable, sans effacer le fait de Gap antérieur.

## Sécurité : modèle de confiance déclaré

Socket accessible seulement au compte/aux accès SSH autorisés. Répertoires et fichiers sensibles privés dès création ; validation du propriétaire, du type et des symlinks. Bornes de trames incluant le délimiteur, de mémoire/file et de temps total sur connect/handshake/write/read. Identité stable vérifiée, scopes isolés et capacité retirée lors de toute sortie de connexion.

Un contenu d'agent est une donnée non fiable, pas une autorité d'administration. Les changements de registre/permissions et les approbations humaines restent hors des messages libres. Il n'existe pas d'isolation forte contre un adversaire exécutant déjà du code sous le même UID : SSH n'est pas une sandbox d'agent. Les protections fournisseur demeurent nécessaires.

## Corpus à épingler avant modification

T002 doit matérialiser les bytes réels, leur SHA-256 et leur origine Git pour : négociations, envois et replies, lookup et issues, annuaire/ledger, attach snapshot/live/Gap/End, lifecycle de processus, services/claims/replies/événements. Chaque famille conservée reçoit un test de lecture dans le daemon et un consommateur ; une mutation d'un octet doit être détectée. Ne pas geler seulement le handshake.

L'évolution additive nécessaire à l'isolation est versionnée et testée avec le client précédent. Une inconnue n'est ni ignorée silencieusement ni déclarée disponible. Ce document ne crée aucun champ filaire nouveau.
