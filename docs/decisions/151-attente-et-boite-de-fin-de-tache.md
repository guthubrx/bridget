# Décision 151 - Attente et boîte de fin de tâche

Date : 2026-10-11. Statut : Accepté.

## Contexte

Le parent pouvait déléguer une mission ou envoyer une demande suivie. Il recevait
le résultat par message, mais ne pouvait pas l'attendre dans le même tour sans
relire le statut. Les indicateurs d'envoi ne prouvaient pas la remise du message.
Un retour d'attente devait donc rester distinct d'un accusé de livraison.

Le socle 150 fournit déjà les tâches durables, les phases de remise, la preuve
`delivery_acked`, la rétention et le délai de mission. La session 151 les réutilise.

## Décisions finales

| Recherche | Décision et raison |
|---|---|
| R1 | Deux outils dédiés : `bridget_task_wait` et `bridget_request_wait`. Les reçus de `bridget_delegate` et `bridget_send` restent immédiats. Un rappel de l'attente après une coupure n'émet aucun nouveau message. |
| R2 | L'attente s'exécute dans le fil de la connexion daemon. Le verrou d'état est tenu seulement pour lire, puis relâché avant le sommeil. Aucun moteur ni service ajouté. |
| R3 | Un petit compteur commun de changements, avec variable de condition, réveille plusieurs lecteurs. Le canal Lineage existant a un consommateur unique et ne signale que les changements projetés. Le compteur est lu avant chaque instantané ; son verrou est relâché avant de prendre celui de l'état. Les écrivains notifient après leur écriture durable. |
| R4 | L'expiration rend `timed_out` sans annuler l'enfant. `wait_timeout` borne l'appel ; `reply_timeout` borne la mission ou la réponse. |
| R5 | La boîte de fin est une vue des faits de tâche et de remise existants. `pending`, `claimed` et `delivered` n'exigent aucune table nouvelle. Une phase orpheline ou absente reste `claimed`, sans preuve d'accusé. |
| R6 | Attente et statut sont des lectures pures. Préparer une réponse de socket ne prouve pas que le parent l'a reçue. Le message de résultat reste envoyé et sa preuve de remise continue à protéger la purge. Un doublon visible est accepté. |
| R7 | Les vues ajoutent `work_state`, résumé stable, taille, référence de détail et boîte. Le résumé de tâche garde 2048 octets UTF-8 au plus. L'attente omet le texte entier ; le statut le conserve. |
| R8 | Quatre attentes par processus MCP, 64 au total dans le daemon. L'annulation ferme le socket, y compris pendant la sonde. Le registre couvre aussi l'annulation avant enregistrement et l'EOF avant jonction des fils. Libération en 6 secondes au plus. |
| R9 | Le client d'attente garde sa connexion `wait_timeout + 10` secondes. Le montage Codex de Bridget seul fixe `tool_timeout_sec = 3700`. Dans un autre hôte, le délai demandé doit rester sous le délai d'outil de l'hôte. |
| R10 | Le premier passage à `answered` écrit un résumé JSON figé dans la même transaction : `id`, `from`, `summary`, `summary_truncated`. Le corps garde 1024 octets UTF-8 au plus. Une autre émission peut remplacer la clé `(id, target)` du ledger : elle ne sert pas à relire cette réponse. Le guichet garde `NULL`. Une lecture filtre l'émetteur et prend état, échéance et résumé dans un seul SELECT. |
| R11 | La sonde ancienne `DaemonIdentityRequest`, budget 3 secondes, vérifie la présence de `wait_v1` dans `features`. Sans cette valeur, refus `wait_unsupported` avant toute trame nouvelle. Aucun nouveau membre d'énumération de capacité n'est négocié avec l'ancien daemon. |
| R12 | `bridget_task_ack` écrit une fois `acknowledged_at` pour les tâches `result_available` ou `failed` du parent. Il déclare la lecture ; il ne prouve aucune livraison et ne change ni message, nettoyage, purge ou projection Lineage. |
| R13 | La propriété suit la garde du statut, sans reprise implicite du propriétaire. Les attentes restent possibles pendant la pause. Une tâche purgée donne `task_unavailable`. Toute liste est validée avant de rendre un résultat ou d'écrire les accusés. |
| R14 | L'annulation native reste inchangée. Après redémarrage du daemon, l'appel ouvert échoue ; le parent peut rappeler l'attente. Reprise des missions et pause : session 152. Liste globale des boîtes : session 158. |

Le compteur reste dans un module existant. Les écritures de demandes le notifient
dans le daemon après commit : réponse idempotente, réponse héritée, guichet,
annulation et expiration. Le Store ne porte aucun compteur de notification.
Le filet de 5 secondes est borné par le délai restant, et par l'échéance d'une demande.
Par réveil, la lecture est O(k), k au plus 16 ; la recherche d'une demande est indexée
par id. La migration de résumé est additive, gardée et sérialisée entre ouvreurs.

## Conséquences

- Le parent attend dans le même tour. Les contrôles restent possibles, avec des
  ressources bornées et sans verrou d'état tenu pendant le sommeil.
- Les lectures ne déclenchent pas de remise ni de purge. Seul l'accusé du message
  prouve sa livraison ; `acknowledged` est une déclaration de lecture indépendante.
- Un résultat peut être vu par l'attente puis par message. Une réponse de demande
  suivie est aussi injectée normalement : doublon visible connu, FR-030.
- Une demande `answered` sans résumé conservé rend `reply: null` et
  `reply_reason: "reply_unavailable"`. Une réponse tardive après le guichet garde ce
  résultat absent. Aucun balayage de messages n'essaie de fabriquer une réponse.
- Aucun service, module, table ou dépendance externe ajouté. Le coût de maintenance
  porte sur trois outils, le compteur et deux champs durables optionnels :
  `reply_summary` et `acknowledged_at`.
- La reprise après panne, le guidage, la suppression de copies et la liste globale
  des boîtes gardent leurs sessions propres. Aucune commande CLI nouvelle.

## Sources du choix

Recherche 151 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/research.md

Contrat 151 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/contracts/attente151.md
