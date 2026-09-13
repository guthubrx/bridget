# 090 — Codex interactif natif, connecté à Bridget sans tmux

## Correctif reprise par nom — 2026-09-07

Incident : `resume horizon-calliope --name calliope` échoue sur `thread/list`
avant reprise, puis l'arrêt du groupe est non confirmé dans les quatre secondes.
La sonde du CLI 0.153.4 prouve que le mode par défaut rescane/répare les métadonnées
JSONL : page 1 6,623 s, page 2 29,588 s, recherche filtrée seule 46,993 s.

Le catalogue interactif doit demander `useStateDbOnly: true` par l'API publique
Codex, sans accès direct de Bridget à sa base ni aux rollouts. Cela utilise la
projection publiée par Codex, pas un cache client. Borne 1000 fils / 10 s,
pagination complète, refus absent/ambigu et contrôle UUID conservés. Aucun
scan/réparation implicite déclenché pour ouvrir le sélecteur ou résoudre un nom.
Une ancienne conversation non publiée dans ce catalogue reste absente ; ne pas
inventer de fil ni lancer une réparation de son propre chef.

Acceptation : une fixture simulant le silence du scan historique rend le cas
rouge sans ce drapeau ; avec lui, reprise par nom exacte jusqu'à la TUI sur le
même fil. Vérifier le champ à chaque page, les refus et le nettoyage des seuls
enfants détenus. Rejouer sur Codex installé avec le nom fourni, sans tour envoyé,
sans démarrer une seconde session de travail ni couper les agents présents.

## Amendement 2026-09-06 — options et reprise initiale

- Reprise humaine étendue : `resume <UUID|nom-Codex>` ou `resume` seul.
  Résolution depuis le protocole public `thread/list`, sans lire l'index ni
  la base Codex depuis Bridget. Nom exact absent/ambigu : refus. Menu numéroté
  Bridget avant la TUI native, pagination et annulation sans session ni prompt.
  Catalogue non archivé borné 1 000 entrées/10 s, incomplet refusé. Le nom
  de conversation Codex est distinct du nom Bridget `--name`.
- `bridget codex --yolo resume <UUID>` reprend explicitement ce fil ; `--yolo`
  est l'alias exact de `--dangerously-bypass-approvals-and-sandbox`. Sans option
  explicite, aucun contournement n'est activé. La TUI reste l'autorité des décisions.
- `--name <nom>` définit le nom d'affichage Bridget via le service de renommage
  existant, sans changer l'UUID (`--agent-id`). Nom invalide ou déjà attribué :
  refus explicite, jamais réussite affichée. Le nom n'est pas transmis à Codex.
- La reprise conserve UUID, historique et titre du fil fournisseur. Un fil absent,
  incompatible ou remplacé par une réponse serveur différente est refusé avant
  présence. Le choix initial ne permet toujours pas de changer de fil en cours.
- Oracles : alias long/court identiques ; vraie TUI + daemon isolés reprennent un
  historique réel, ajoutent un tour, publient le nom demandé sous le même UUID
  Bridget et attestent les permissions dans le contexte durable Codex.
- Claude interactif sans tmux est hors de cet amendement ; le mode géré existe.

Date : 2026-09-05. Statut : Implemented ; adoption autorisée et exécutée le 2026-09-06.
Branche : session-090-codex-interactif. Dépendance : noyau 089.

## Besoin et périmètre

L'utilisateur veut converser avec Codex dans son interface terminal habituelle,
tout en permettant aux autres agents de joindre cette même conversation par
Bridget. Aucun tmux, simulation de clavier, nouvelle GUI ou orchestration métier.
Le fournisseur reste accessible par son CLI et son abonnement, sans API payante
de remplacement. Les autres fournisseurs et les agents gérés restent inchangés.

## Scénarios utilisateur et acceptation

### US1 — Une session interactive joignable (P1)

L'utilisateur lance `bridget codex` dans son terminal. Codex conserve son interface
native et ses fonctions interactives ; Bridget annonce l'identité de cette session.
Un message envoyé par un autre agent rejoint la conversation sélectionnée, et non
une deuxième session invisible. Codex peut répondre par les outils Bridget.

Acceptation : sans tmux dans l'environnement, une saisie humaine et une demande
Bridget sont observées dans la même conversation ; la réponse liée clôt la demande.

### US2 — Saisie, permissions et observation honnêtes (P1)

L'utilisateur peut écrire pendant le travail de l'agent. Une demande interagent
arrivée pendant un tour ne détruit pas la saisie et ne crée pas un tour concurrent
invisible. Le journal Bridget contient aussi les tours initiés par l'humain.
Les demandes de permission restent une décision de l'utilisateur, selon ses
réglages ; Bridget ne les accepte pas automatiquement pour faciliter le raccord.

Acceptation : corpus tour actif/inactif, annulation, permission refusée/acceptée,
événement inconnu et octets bruts ; aucun succès de remise inventé.

### US3 — Fermeture et pannes explicites (P1)

Quitter l'interface termine cette session interactive et sa présence Bridget.
La fermeture du terminal ne promet pas la persistance d'un agent géré. Une perte
de connexion au daemon ne recrée pas la conversation Codex ; après rétablissement,
l'identité, les corrélations et les garanties de rejeu restent celles du noyau.

Acceptation : fermeture normale, terminal coupé, fournisseur arrêté et daemon
momentanément indisponible ne laissent pas une présence fausse ni un enfant oublié.

## Exigences fonctionnelles

- FR-09001 : `bridget codex` hors tmux ouvre l'interface native, avec un nom
  explicite optionnel et le répertoire courant choisi par le shell ; dépendance non compatible = refus clair.
- FR-09002 : humain et Bridget ciblent un même identifiant de conversation attesté.
  Aucun second fil créé pour simuler la réception d'un message.
- FR-09003 : réutiliser envoi, suivi, réponse liée, canon, ledger et idempotence
  Bridget. Un retry ne double pas la remise dans le domaine de garantie existant.
- FR-09004 : afficher une présence conforme au chemin réel, jamais tmux sans tmux.
  Modèle/effort/activité restent des faits observés, absence = inconnu.
- FR-09005 : journal et attache couvrent tours humains et interagents ; conserver
  les octets source, leur provenance et la distinction lacune/indisponibilité.
- FR-09006 : conserver l'autorité des permissions natives ; aucun nouveau bypass,
  aucune approbation automatique due au raccord, aucune modification de config globale.
- FR-09007 : sérialiser ou refuser explicitement les interactions incompatibles
  pendant un tour. Ne pas assimiler l'acceptation dans une file à une remise certaine.
- FR-09008 : arrêter uniquement les ressources appartenant à la session interactive.
  Nettoyage borné ; aucune dépendance au terminal d'un autre utilisateur/agent.
- FR-09009 : le rétablissement du lien Bridget conserve session et identité ; une
  session fournisseur perdue est annoncée, jamais remplacée silencieusement.
- FR-09010 : rester compatible avec le socket Bridget configurable et la fédération
  SSH existante. Aucun nouveau port réseau public ni accès navigateur à la socket.
- FR-09011 : préserver les parcours gérés Codex et les autres adaptateurs. Documentation
  explicite de la différence interactif/agent persistant/attach.

## Critères de réussite

| Critère | Preuve exigée |
|---|---|
| SC-09001 | Vraie interface Codex sous pseudo-terminal, sans tmux : entrée humaine + message Bridget dans le même fil, réponse liée visible au ledger. |
| SC-09002 | Retry même clé : une remise, même issue ; message pendant tour actif traité sans perte de saisie ni duplication. |
| SC-09003 | Journal/attach montre les deux origines ; fixture atypique conservée octet pour octet. |
| SC-09004 | Permission demandée : aucune décision automatique par Bridget ; refus humain respecté. |
| SC-09005 | Sortie/EOF/panne : arrêt observé sous 10 s hors éventuelle demande fournisseur déjà partie, zéro enfant de test survivant ; production inchangée. |
| SC-09006 | Coupure puis reconnexion Bridget : même identité et même fil, aucun prompt de reconstruction. |
| SC-09007 | Tests ciblés puis tests workspace, fmt et clippy ; régressions historiques identifiées séparément, aucun test supprimé pour obtenir le vert. |

## Hypothèses et limites

Codex uniquement. Les paramètres natifs influant sur la session doivent être
propagés ou explicitement refusés, jamais ignorés. Aucune promesse de survie à la
fermeture du terminal : pour cela, le parcours spawn persistant reste distinct.
Amendement utilisateur du 2026-09-06 : une identité Bridget reste liée au fil
initial, mais le serveur Codex peut charger d'autres fils. Les sous-agents
internes relèvent uniquement de Codex : ni inscription Bridget, ni supervision,
ni arrêt du parent à leur création/reprise. Une notification globale concernant
un autre fil n'atteste pas une navigation humaine et ne ferme pas la session.
Les envois et observations Bridget restent corrélés au fil initial ; aucun
événement ou acte de permission d'un autre fil ne lui est attribué. Bridget ne
promet pas de suivre une navigation native `/new` ou `/resume` : utiliser un
nouveau lancement pour joindre une autre conversation. Les autres agents Bridget
restent indépendants.
Aucun déploiement/commit automatique.
