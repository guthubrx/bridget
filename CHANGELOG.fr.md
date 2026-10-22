# Journal des changements

[English](CHANGELOG.md) · [Français](CHANGELOG.fr.md)


Chaque version publiée s'ajoute à l'historique du dépôt public, avec son tag `vX.Y.Z`. Les numéros
suivent le versionnage sémantique : le dernier chiffre pour les corrections et la documentation, le
deuxième pour les nouvelles capacités, le premier pour les ruptures de contrat. Les dates sont au
format AAAA-MM-JJ.

## [0.4.1] - 2026-10-16

### Corrigé

- Une réponse distante acceptée ferme le suivi de sa demande, même sans accusé de transport du parent. Elle ne déclenche plus une fausse alerte de non-réponse.
- Après une panne du stockage, l’entretien reprend cette clôture, même si la tâche est déjà nettoyée. Le résultat reste conservé et le travail ne redémarre pas.
- La corrélation vérifie la mission, la ronde, l’enfant et le parent. Une vraie absence de réponse reste soumise au délai configuré.

### Compatibilité et validation

- Aucun changement de protocole, de schéma, de dépendance ou de configuration T3. Les anciens états de demandes déjà expirées restent historiques.
- 46 tests ciblés de compatibilité distante et 3 tests du correctif réussissent. Après le dernier ajustement du test SQL sélectif, seul cet oracle a été rejoué et réussit. Aucune suite complète n’a été relancée.
- Les paquets, l’activation et la recette réelle sont attestés séparément dans le reçu de livraison de la session 170.

## [0.4.0] - 2026-10-15

### Ajouté

- Le catalogue de routage présente les modèles déclarés sur chaque machine avec leur moteur, leur connexion et leurs efforts pris en charge. `bridget delegate --catalogue --json` permet de le consulter sans T3.
- Une tâche peut imposer séparément le modèle, le moteur, la machine, la connexion ou l’effort. Les axes non imposés restent libres dans les limites du registre. Un effort absent conserve le comportement natif du fournisseur.
- Le choix tient compte des faits de charge frais, de la continuité du travail Git, des capacités de compilation déclarées et de l’état attesté des quotas ou crédits de la connexion cible. La continuité reste bornée : elle ne bloque pas le déplacement vers une autre machine disponible.
- La chaîne de décision est configurable et désactivable par fichier de configuration. Elle applique les filtres déterministes, puis peut consulter un serveur de décision compatible, tel que Laya ou Jev, puis un modèle léger de secours. Le classement déterministe reste le dernier recours. Le modèle de secours choisit la combinaison d’exécution ; il n’effectue pas la tâche à sa place.
- L’accès au service de décision est déclaré par machine. Un service désactivé ou indisponible n’empêche pas les niveaux de secours autorisés. Les décideurs ne peuvent choisir que parmi les combinaisons déjà admissibles.
- L’état durable et la projection Lineage v4 conservent le choix, sa provenance et les faits bornés utilisés. Un rejeu exact retrouve le reçu enregistré sans nouvelle décision.

### Git et exécution distante

- Le démarrage distant vérifie le commit de départ attendu avant la préparation du travail. Une divergence Git ne devient pas une tâche exécutée sur une autre révision.
- Des travaux distincts peuvent utiliser leurs propres worktrees. Le moteur de routage ne lance aucun `git fetch`, `git push` ou merge : la synchronisation et la livraison Git restent des actions explicites de coordination.
- Le catalogue reste distinct des droits réels. Un modèle déclaré ne prouve ni l’accès du compte, ni un quota ou des crédits inconnus. Les connexions restent celles autorisées et configurées sur chaque machine.

### Compatibilité et exploitation

- Les contrats existants, dont les projections Lineage v1 à v3, restent conservés. Lineage v4 ajoute les métadonnées de décision bornées ; les lectures anciennes gardent leur forme et n’obtiennent pas ces nouveaux détails.
- L’admission v2 et les nouvelles capacités de probe sont négociées explicitement. Une capacité absente ne déclenche pas de conversion silencieuse vers une ancienne admission.
- La migration du stockage conserve les reçus antérieurs. Après l’écriture d’un reçu d’admission v2, ne pas revenir à un binaire 0.3.0, même si le routage est ensuite désactivé. Utiliser un binaire compatible avec cette migration, ou une restauration cohérente validée qui ne perd aucun travail.
- Bridget reste utilisable depuis le terminal et indépendant de T3.

### Validation et limites connues

- L’audit V15 r2 conclut `PASS`, avec `release_blocked=false` et une note A. Le compteur fondé sur les pénalités vaut 98. Les axes de qualité indiqués `N/A` restent non notés. Aucun constat critique, élevé ou moyen n’est ouvert.
- Deux constats faibles restent suivis : `RUSTSEC-2026-0098` et `RUSTSEC-2026-0099`, sur `rustls-webpki` 0.101.7. `RUSTSEC-2024-0449` signale la maintenance de `unsafe-libyaml` 0.2.11. Cette version ne modifie aucune dépendance et ne prétend pas résoudre ces avis.
- La campagne complète unique a compté 2 820 tests réussis, 32 échecs et 71 tests ignorés. Pour 29 de ces échecs, le dernier résultat ciblé est désormais réussi. Trois scénarios d’annulation de la session 159 ne sont pas validables sous Linux. La cause initiale de l’échec T3 reste inconnue. Aucun succès global du workspace n’est déduit de ces résultats.
- Le reçu de livraison distingue les preuves Git, les paquets, l’installation et le fonctionnement. Aucun succès de construction, d’activation ou de recette réelle n’est déduit des seuls résultats de tests ou d’audit.

## [0.3.0] - 2026-10-15

### Ajouté

- `bridget delegate <MISSION>` donne à un agent coordinateur un parcours court pour choisir un profil, créer un enfant et lui confier une mission. Les options avancées restent facultatives.
- `bridget task` expose la liste, l’état, l’attente, la continuation, les réponses, l’annulation et l’accusé de lecture. Ces commandes réutilisent le moteur natif et fonctionnent sans T3.
- La sortie `--json` reste exploitable par script. Un terminal sans identité d’agent reçoit un refus avant tout effet et une instruction de rattachement.

### Limites de validation

- La coordination directe exige une identité d’agent Bridget vivante. Elle ne transforme pas un terminal humain en agent.
- `task list` est une projection humaine globale. Les autres opérations restent limitées au coordinateur propriétaire.

## [0.2.1] - 2026-10-15

### Corrigé

- Le pont T3 ferme les liens des sous-agents terminés après avoir drainé leurs derniers événements.
  Les conversations restent dans l'historique, mais elles ne gardent plus des milliers de fichiers
  ouverts. Les reprises nécessaires sont limitées à 32 liens simultanés.
- Avec T3 Code `0.0.45-local.166` ou plus récent, la reprise d'un fil Codex remonte la preuve privée
  Bridget courante avant son premier tour. Une ancienne preuve reste refusée et un échec de remontage
  bloque le tour, sans emprunter l'identité d'un autre fil.

### Exploitation

- La limite souple normale de 4 096 fichiers suffit à nouveau. Augmenter cette limite ne remplace pas
  la fermeture des liens et ne constitue plus le correctif recommandé.
- Un fil repris n'a plus besoin d'une boucle de désinstallation et de réinstallation du pont pour
  retrouver Bridget. Vérifier d'abord que les versions T3 et Bridget compatibles sont chargées.

### Limites de validation

- Les tests ciblés du pont passent : 13 scénarios de cycle de vie, 102 tests du module T3 et 22 tests
  de contrat ; un test de contrat reste ignoré.
- La recette réelle est passée de 1 130 à 32 liens et de 4 539 à 167 fichiers ouverts. Aucun `EMFILE`
  ni `Too many open files` n'a été observé après activation.
- La suite complète du crate conserve 91 échecs hérités hors des fichiers affectés. Aucun résultat
  GREEN global n'est revendiqué.

## [0.2.0] - 2026-10-14

### Ajouté

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

### Corrigé

- Le calcul de quota intègre les crédits Codex du compte cible (5872f48e).

### Limites de validation

- Le connecteur T3 reste un composant indépendant ; son symbole visuel est livré séparément et
  cette release ne prouve pas le rechargement d'une application T3 déjà ouverte.
- Les validations associées sont ciblées. Aucun résultat GREEN global du workspace n'est revendiqué.

Voir la référence de communication et le guide d'installation. Toutes les entrées publiées restent
conservées.

## [0.1.3] - 2026-10-02

### Corrigé

- `bridget spawn` attend la durée demandée par `--timeout` (plus une marge) avant de conclure ;
  un fournisseur lent à démarrer était lancé mais annoncé « outcome_unknown » au bout de 10 s.
- Pont T3 Code : les notifications d'observation qui se suivent partagent un seul tour au lieu de
  réveiller l'abonné une fois chacune ; elles ne sont jamais mêlées aux messages.
- Pont T3 Code : un message sans réponse attendue destiné à un fil Claude ou Cursor occupé est remis
  aussitôt au tour en cours, au lieu d'attendre sa fin (jusqu'à 8 h 45 observées). Codex, les
  demandes suivies et les notifications attendent toujours la fin du tour.

### Documentation

- Guide d'installation : déclarer un fournisseur compatible Claude (exemple GLM) ou un second
  compte Codex dans le registre, avec sa variante lecture seule automatique.

### Modifié

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

## [0.1.2] - 2026-10-02

### Documentation

- Skill : lancer des équipiers en lecture seule est autorisé d'office pour une tâche de lecture,
  sans terminal ; seul le droit d'écrire (`--posture development`, Codex) exige un terminal
  humain ; un refus « posture_decouverte » signale un type non déclaré dans le registre.

### Corrigé

- Pont T3 Code : un tour aussitôt suivi d'un autre, entre deux lectures du pont, était signalé
  comme une lacune d'observation ; sa fin est désormais annoncée normalement quand l'origine du
  tour a été relevée pendant qu'il tournait.

## [0.1.1] - 2026-09-25

### Modifié

- `bridget --help` affiche la version réelle du paquet ; elle était figée à 0.1.0 dans le texte.

### Documentation

- README à jour des capacités récentes, et ce journal des changements.
- La publication est désormais versionnée : une nouvelle version s'ajoute à l'historique au lieu de
  le remplacer, ce qui permet de comparer deux versions.

## [0.1.0] - 2026-09-25

Première version publique versionnée. Les publications précédentes remplaçaient l'historique ;
celle-ci en est le point de départ. Elle contient notamment les changements suivants.

### Ajouté

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

### Corrigé

- Un grand fil (plus de 1 000 messages) n'est plus recopié en boucle dans le journal d'observation.
- Le pont n'annonce plus une source prête avant d'avoir vérifié son journal, ce qui produisait des
  notifications en alternance.
- Un processus disparu pendant l'inventaire des fichiers ouverts n'invalide plus l'identité de
  tous les autres fils.
- La CLI lancée depuis un fil T3 attesté ne signe plus ses envois « human ».
- L'identifiant de build n'est plus figé au premier passage du script de compilation.
- Les tests d'intégration ne dépendent plus de la charge de la machine.
