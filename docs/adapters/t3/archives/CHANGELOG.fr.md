# Archives techniques de l’adaptateur T3

Document d’origine : [CHANGELOG.fr.md](../../../../CHANGELOG.fr.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Retenir les messages mobiles dans Bridget pendant le travail de l’agent. Afficher un volet repliable au-dessus de la saisie, avec édition, annulation et guidage explicite.
- Conserver les actions incertaines. Rapprocher les identifiants du journal et conserver le guidage après reprise de transmission.
- Prendre en charge le guidage Codex explicite via l’adaptateur T3 V2 existant ; les notifications ordinaires attendent toujours.

<a id="extrait-2"></a>

## Extrait 2

- Aucun changement de protocole, de schéma, de dépendance ou de configuration T3. Les anciens états de demandes déjà expirées restent historiques.
- 46 tests ciblés de compatibilité distante et 3 tests du correctif réussissent. Après le dernier ajustement du test SQL sélectif, seul cet oracle a été rejoué et réussit. Aucune suite complète n’a été relancée.
- Les paquets, l’activation et la recette réelle sont attestés séparément dans le reçu de livraison de la session 170.

<a id="extrait-3"></a>

## Extrait 3

- L’audit V15 r2 conclut `PASS`, avec `release_blocked=false` et une note A. Le compteur fondé sur les pénalités vaut 98. Les axes de qualité indiqués `N/A` restent non notés. Aucun constat critique, élevé ou moyen n’est ouvert.
- Deux constats faibles restent suivis : `RUSTSEC-2026-0098` et `RUSTSEC-2026-0099`, sur `rustls-webpki` 0.101.7. `RUSTSEC-2024-0449` signale la maintenance de `unsafe-libyaml` 0.2.11. Cette version ne modifie aucune dépendance et ne prétend pas résoudre ces avis.
- La campagne complète unique a compté 2 820 tests réussis, 32 échecs et 71 tests ignorés. Pour 29 de ces échecs, le dernier résultat ciblé est désormais réussi. Trois scénarios d’annulation de la session 159 ne sont pas validables sous Linux. La cause initiale de l’échec T3 reste inconnue. Aucun succès global du workspace n’est déduit de ces résultats.
- Le reçu de livraison distingue les preuves Git, les paquets, l’installation et le fonctionnement. Aucun succès de construction, d’activation ou de recette réelle n’est déduit des seuls résultats de tests ou d’audit.

<a id="extrait-4"></a>

## Extrait 4

- Le pont T3 ferme les liens des sous-agents terminés après avoir drainé leurs derniers événements.
  Les conversations restent dans l'historique, mais elles ne gardent plus des milliers de fichiers
  ouverts. Les reprises nécessaires sont limitées à 32 liens simultanés.
- Avec T3 Code `0.0.45-local.166` ou plus récent, la reprise d'un fil Codex remonte la preuve privée
  Bridget courante avant son premier tour. Une ancienne preuve reste refusée et un échec de remontage
  bloque le tour, sans emprunter l'identité d'un autre fil.

<a id="extrait-5"></a>

## Extrait 5

- La limite souple normale de 4 096 fichiers suffit à nouveau. Augmenter cette limite ne remplace pas
  la fermeture des liens et ne constitue plus le correctif recommandé.
- Un fil repris n'a plus besoin d'une boucle de désinstallation et de réinstallation du pont pour
  retrouver Bridget. Vérifier d'abord que les versions T3 et Bridget compatibles sont chargées.

<a id="extrait-6"></a>

## Extrait 6

- Les tests ciblés du pont passent : 13 scénarios de cycle de vie, 102 tests du module T3 et 22 tests
  de contrat ; un test de contrat reste ignoré.
- La recette réelle est passée de 1 130 à 32 liens et de 4 539 à 167 fichiers ouverts. Aucun `EMFILE`
  ni `Too many open files` n'a été observé après activation.
- La suite complète du crate conserve 91 échecs hérités hors des fichiers affectés. Aucun résultat
  GREEN global n'est revendiqué.

<a id="extrait-7"></a>

## Extrait 7

- Sous-agents internes T3 attestés : annuaire et envoi par MCP seulement, provenance visible,
  réponses routées au parent. Aucun agent enfant durable ni droit d'administration ajouté (133).
- Messages et lots : nom humain avec UUID complet ; le routage et l'historique ne changent pas (134).
- Contrôleur Agent Loop extérieur : prise en charge, progrès vérifiable, rappels regroupés et
  décision de suite. Une file vide ou une fin de tour n'est pas une clôture de mission (135).
- Fils partagés : preuves `kind:history` silencieuses, consignes courtes et remplacement explicite
  `supersedes_seq`, sans effacer ni rejouer l'histoire (136).
- Annuaire du même projet par défaut, `who/agents --global` volontaire et motif interprojets
  `--cross-project-reason` / `cross_project_reason`. Les projets inconnus restent explicitement
  inconnus ; les envois legacy avertis restent compatibles. Les rappels Loop des UUID/rôles déjà
  mandatés restent actifs, sans recrutement extérieur automatique (138).
- Délégation native sous autorité unique, admission contrôlée, suivi durable des tâches et reprise
  par rondes ; le moteur et la machine sont choisis par le maître, sans bascule implicite.
- File de messages au repos, réveils et faits manqués, édition contrôlée, catalogue de capacités
  et prise en compte des crédits Codex du compte cible.

<a id="extrait-8"></a>

## Extrait 8

- Le connecteur T3 reste un composant indépendant ; son symbole visuel est livré séparément et
  cette release ne prouve pas le rechargement d'une application T3 déjà ouverte.
- Les validations associées sont ciblées. Aucun résultat GREEN global du workspace n'est revendiqué.

<a id="extrait-9"></a>

## Extrait 9

- `bridget spawn` attend la durée demandée par `--timeout` (plus une marge) avant de conclure ;
  un fournisseur lent à démarrer était lancé mais annoncé « outcome_unknown » au bout de 10 s.
- Pont T3 Code : les notifications d'observation qui se suivent partagent un seul tour au lieu de
  réveiller l'abonné une fois chacune ; elles ne sont jamais mêlées aux messages.
- Pont T3 Code : un message sans réponse attendue destiné à un fil Claude ou Cursor occupé est remis
  aussitôt au tour en cours, au lieu d'attendre sa fin (jusqu'à 8 h 45 observées). Codex, les
  demandes suivies et les notifications attendent toujours la fin du tour.

<a id="extrait-10"></a>

## Extrait 10

- Pont T3 Code : une demande avec réponse attendue invite l'agent à ouvrir sa réponse par
  « ↪ Réponse à <expéditeur> (relayée par Bridget) : ». Relayée à l'expéditeur, elle reste aussi
  affichée dans le fil de l'utilisateur, qui la prenait pour lui.
- La consigne des messages sans réponse attendue sépare l'accusé de réception de l'action :
  « Pas d'accusé de réception… Ce n'est pas une absence de tâche ». L'ancienne formule était lue
  comme « rien à faire ».
- `send` accepte un début d'UUID d'au moins 6 caractères (CLI et MCP) ou un nom d'affichage exact
  (CLI) quand il ne désigne qu'un seul agent.
- Un lancement d'équipier d'un type absent du registre est refusé « type d'agent inconnu », avec
  la liste des types connus, au lieu d'une capacité manquante.

<a id="extrait-11"></a>

## Extrait 11

- Pont T3 Code : un tour aussitôt suivi d'un autre, entre deux lectures du pont, était signalé
  comme une lacune d'observation ; sa fin est désormais annoncée normalement quand l'origine du
  tour a été relevée pendant qu'il tournait.

<a id="extrait-12"></a>

## Extrait 12

- **Abonnements durables.** Les abonnements d'observation survivent à un redémarrage du daemon :
  ils reprennent seuls au retour de leur source, et leur propriétaire est averti que les faits de
  la coupure sont perdus, au lieu de devoir se réabonner.
- **Avis amortis.** Un changement d'état d'une source n'est annoncé qu'après 30 secondes de
  stabilité ; une source qui clignote donne au plus un avis « source instable » toutes les 5
  minutes. Chaque avis réveillant un agent, cela évite des tours dépensés pour rien.
- **Pont T3 Code : tours spontanés.** Un fil dont le dernier tour est un réveil d'arrière-plan (fin
  d'un sous-agent) reste observable ; sa fin de tour est signalée.
- **Pont T3 Code : réponses incertaines.** Quand une réponse ne peut pas être reliée avec certitude
  à sa demande, elle est transmise avec la mention « appariement incertain » au lieu d'être gardée.
- **Pont T3 Code : seconde application.** Une seconde application T3 ouverte sur les mêmes données
  déclenche une notification qui la nomme ; le fichier d'état qu'elle efface en se fermant est
  rétabli automatiquement.
- **Appariement par origine prouvée.** La réponse relayée est celle du tour réellement déclenché par
  la demande, prouvé par horodatage, et non plus déduite d'un décompte.
- **Entretien automatique.** Le daemon purge chaque heure les remises expirées et l'état d'identité
  des processus disparus, et fait tourner ses journaux ; la construction retire les worktrees
  fusionnés et inoccupés.
- **Remises regroupées.** Les messages sans réponse attendue destinés à un même fil partagent un tour.

<a id="extrait-13"></a>

## Extrait 13

- Un grand fil (plus de 1 000 messages) n'est plus recopié en boucle dans le journal d'observation.
- Le pont n'annonce plus une source prête avant d'avoir vérifié son journal, ce qui produisait des
  notifications en alternance.
- Un processus disparu pendant l'inventaire des fichiers ouverts n'invalide plus l'identité de
  tous les autres fils.
- La CLI lancée depuis un fil T3 attesté ne signe plus ses envois « human ».
- L'identifiant de build n'est plus figé au premier passage du script de compilation.
- Les tests d'intégration ne dépendent plus de la charge de la machine.
