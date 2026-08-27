# Spécification 036 — Observer la prise d'un mandat

**Statut** : Implémentée

**Base gelée du lot** : `2078a59bcf43ae8d6b5b9fd7a8fcf14fc26ba88c`

**Objectif** : `66dc001c-6466-486b-9c60-7bf4fa3544d4`

**Dependencies**: SPEC-023-observabilite-tour-refus

## Problème mesuré

Une remise à un agent interactif peut rester collée dans son invite sans être
soumise. Le ledger atteste alors correctement la remise, le daemon atteste
correctement la connexion, mais aucun instrument n'atteste que le client a
accepté l'entrée. Le silence ressemble à tort à un travail en cours.

Le spécimen de référence est le mandat de ce lot :

- `/home/moi/.cache/bridget/sessions/rc5/2026-08-27.jsonl:8` consigne
  `turn_start` à `2026-08-27T09:49:24Z` ;
- `/home/moi/.codex/sessions/2026/08/25/rollout-2026-08-25T15-13-15-01a0390d-b893-7f82-b11a-3116dcfcdd21.jsonl:8444`
  consigne le démarrage réel du client à `2026-08-27T10:01:24.444Z` ;
- le même fichier, ligne 8446, consigne l'entrée utilisateur acceptée à
  `2026-08-27T10:01:24.857Z`.

L'écart réel est donc de 720,444 secondes jusqu'au démarrage du client et de
720,857 secondes jusqu'à l'acceptation de l'entrée. `turn_start` existait
pendant tout l'incident : son nom ne constitue pas la preuve recherchée.

Un second spécimen mesuré ensuite porte la même panne sous une autre apparence :
sur deux agents, le mandat attendait comme texte normal, sans marqueur
`[Pasted Content]`. Une touche Entrée a démarré chacun en sept secondes. La
présence ou l'absence du marqueur de console est donc une explication visuelle
incomplète, jamais un critère de prise en charge. Le calcul repose uniquement
sur ce que le client a accepté.

La première version de l'observateur a révélé un faux positif réel pendant sa
relecture hostile : `cartae0` avait reçu une injection à
`2026-08-27T10:04:24Z`, mais sa trace portait déjà `task_started` à
`2026-08-27T10:03:53.760Z`, sans `task_complete` intermédiaire. Cette remise
était du pilotage pendant un tour actif (steering), pas un démarrage au repos ;
le seuil de 60 secondes ne lui était pas applicable. Le contrôle opposé `rc7`
avait reçu son injection à `2026-08-27T10:40:17Z`, après `task_complete` à
`2026-08-27T10:39:46.868Z`. Ces deux populations restent distinctes.

## Propriété

La ronde locale distingue l'intention de remettre un mandat de son acceptation
par le client interactif. Elle émet exactement l'un des faits suivants pour le
dernier mandat ouvert qu'elle sait corréler :

- `PRISE_ACCEPTEE` lorsque la trace native atteste l'entrée ;
- `MANDAT_NON_SOUMIS` lorsque la source est disponible, qu'au moins une ligne
  y a été lue, qu'aucune acceptation corrélée n'existe et que 60 secondes sont
  écoulées depuis l'injection ;
- `PRISE_EN_ATTENTE` pendant la grâce de 60 secondes ;
- `REMISE_PENDANT_TOUR_ACTIF` lorsqu'aucune acceptation corrélée n'existe mais
  que la trace atteste un tour déjà actif à l'injection, ou une mise en file
  native corrélée ;
- `PRISE_INOBSERVABLE` lorsque la source est distante, absente, vide,
  illisible, ambiguë ou non prise en charge.

Une source indisponible ou un cardinal nul ne vaut jamais « aucun mandat
bloqué ». Chaque diagnostic porte l'état de la source et son cardinal lu.

## Sources attestantes

- Pour Codex interactif, l'acceptation est un message utilisateur de la trace
  native, corrélé à l'identifiant du mandat et postérieur à l'injection. Les
  bornes `task_started`, `task_complete` et `turn_aborted` établissent si le
  client était déjà actif au moment de l'injection.
- Pour Claude interactif, l'acceptation est soit une entrée utilisateur
  directe, soit un `queue-operation=remove`. Un `enqueue` seul signifie mise
  en file et ne vaut pas prise en charge ; il produit
  `REMISE_PENDANT_TOUR_ACTIF`, jamais `MANDAT_NON_SOUMIS`.
- Les pilotes gérés conservent leurs bornes existantes ; ce lot ne redéfinit
  pas leurs transitions.

Le lecteur ne rend jamais le corps des conversations. Il ne projette que le
type de source, les cardinaux, les horodatages et l'état calculé.

## Scénarios

### US1 — Voir un mandat collé non soumis

Une injection ouverte depuis au moins 60 secondes, avec une trace locale
valide qui ne contient aucune acceptation corrélée, apparaît dans une catégorie
nommée `MANDAT_NON_SOUMIS` et non dans l'indétermination générale.

### US2 — Ne pas accuser un agent qui a pris le mandat

Le même montage, muni d'une entrée native corrélée après l'injection, donne
`PRISE_ACCEPTEE` et interdit `MANDAT_NON_SOUMIS`.

### US3 — Dire que l'instrument ne sait pas voir

Une trace absente, distante, vide, illisible ou ambiguë donne
`PRISE_INOBSERVABLE`, avec `records_read=0` lorsque rien n'a été lu. Cet état
est compté séparément et ne devient jamais un état sain.

### US4 — Respecter la mise en file Claude

Un `enqueue` sans `remove` reste non accepté mais est nommé comme remise pendant
un tour actif. Le `remove` corrélé transforme le même mandat en prise acceptée.

### US5 — Ne pas appliquer le seuil idle au steering

Une injection postérieure à `task_started` sans fin intermédiaire ne devient
jamais `MANDAT_NON_SOUMIS`. Elle apparaît comme
`REMISE_PENDANT_TOUR_ACTIF`. Une injection postérieure à `task_complete` reste
éligible au seuil et au diagnostic de non-soumission.

## Exigences fonctionnelles

- **FR-3601** : le chemin de remise tmux et les producteurs de journal ne sont
  pas modifiés ; le lot est exclusivement observateur.
- **FR-3602** : la corrélation exige l'égalité avec l'identifiant porté par la
  première ligne de l'enveloppe canonique et un fait natif horodaté après
  l'injection. Une appartenance de sous-chaîne, un préfixe, un sur-ensemble ou
  une mention dans le corps ne valent jamais acceptation.
- **FR-3603** : le seuil de 60 secondes ne s'applique qu'à l'acceptation par le
  client interactif au repos ; il ne s'applique ni au steering ni au délai
  fournisseur.
- **FR-3604** : une source disponible porte un cardinal de lignes strictement
  positif ; zéro ligne rend la prise inobservable.
- **FR-3605** : une source non locale n'est jamais reconstruite ni supposée ;
  elle rend `PRISE_INOBSERVABLE`.
- **FR-3606** : le texte et le JSON exposent séparément les mandats non soumis
  et les prises inobservables, sans les noyer dans `INDETERMINES`.
- **FR-3607** : la partition complète des agents reste vérifiée, sans omission
  ni recouvrement après ajout des nouvelles catégories.
- **FR-3608** : une ligne partielle, un JSON invalide, une version de source
  non reconnue ou plusieurs traces principales indistinguables invalide
  l'observation complète ; une trace de sous-agent explicitement typée ne rend
  pas la trace principale ambiguë. Une première ligne partielle rencontrée
  pendant la découverte ferme aussi l'observation, même si un autre descripteur
  porte une trace principale valide. Aucune liste partielle n'est présentée
  comme saine.
- **FR-3609** : aucun contenu utilisateur extrait des traces natives n'est
  inclus dans la sortie texte ou JSON.
- **FR-3610** : aucun marqueur visuel de console, y compris
  `[Pasted Content]`, ne participe au calcul de l'état.
- **FR-3611** : l'observation d'une injection interactive ouverte porte sur
  tous les agents vus par le daemon, même lorsque la copie Maicie locale ne
  connaît aucune mission. `MANDAT_NON_SOUMIS`, `PRISE_EN_ATTENTE` et
  `PRISE_INOBSERVABLE` ne peuvent pas être masqués par cette absence. Une
  `PRISE_ACCEPTEE` ne suffit toutefois pas à inventer une mission absente.
- **FR-3612** : une acceptation corrélée prime sur le contexte actif. Sans
  acceptation, un tour actif ou un `enqueue` corrélé produit
  `REMISE_PENDANT_TOUR_ACTIF`, catégorie distincte dans la partition ; il ne
  traverse jamais la branche temporelle de `MANDAT_NON_SOUMIS`.

## Critères de succès

- **SC-3601** : le spécimen à 720,857 secondes, représenté par une fixture
  fidèle, produit `MANDAT_NON_SOUMIS`, avec injection présente, zéro
  acceptation corrélée et cardinal de source positif.
- **SC-3601b** : le même oracle voit aussi un contenu en attente sans marqueur
  `[Pasted Content]` ; les deux formes visuelles donnent le même état métier.
- **SC-3602** : ajouter au même spécimen une entrée native corrélée produit
  `PRISE_ACCEPTEE` et interdit `MANDAT_NON_SOUMIS`.
- **SC-3603** : côté Claude, `enqueue` seul ne suffit pas et `remove` suffit.
- **SC-3604** : supprimer, vider ou corrompre la trace produit
  `PRISE_INOBSERVABLE`, jamais un tableau sain vide.
- **SC-3605** : le harnais historique de `bridget-idle` reste intégralement
  vert et son oracle de partition inclut les deux nouvelles catégories.
- **SC-3606** : un mutant qui traite l'absence d'acceptation comme une prise
  saine meurt sur SC-3601 ; un mutant qui classe toute injection comme bloquée
  meurt sur SC-3602.
- **SC-3607** : le contrôle sans marqueur visuel reste
  `MANDAT_NON_SOUMIS` lorsque son nom est absent de la copie Maicie locale ;
  le cardinal de la source demeure positif.
- **SC-3608** : le spécimen `cartae0` (`task_started` 10:03:53.760Z puis
  injection 10:04:24Z) produit `REMISE_PENDANT_TOUR_ACTIF`, tandis que le
  contrôle `rc7` (`task_complete` 10:39:46.868Z puis injection 10:40:17Z)
  reste `MANDAT_NON_SOUMIS` ; les deux catégories sont disjointes.
- **SC-3609** : l'enveloppe portant exactement `id=mcp-abc` est acceptée ;
  `id=mcp-abc999` et une mention de `id=mcp-abc` dans le corps restent deux
  témoins négatifs indépendants.
- **SC-3610** : un descripteur Codex dont la première ligne n'est pas terminée
  rend `PRISE_INOBSERVABLE` dans l'oracle de découverte et dans la chaîne
  complète, sans disparaître derrière une autre trace valide.

## Limite de portée obligatoire

Cet incrément ne lit que les traces présentes sur la machine où la ronde est
exécutée. La ronde centrale macOS ne voit donc pas, par ce lot seul, les traces
de `/home/moi` sur Cartae. La fédération ou le relais host-aware constitue un
lot séparé. La session 036 rend le fait local calculable et l'absence de source
explicite ; elle ne ferme pas à elle seule le constat central inter-machine.

## Hors périmètre explicite

- Modifier la remise, envoyer automatiquement une touche Entrée ou relancer un
  client interactif.
- Relayer les observations entre machines ou étendre l'annuaire du daemon.
- Calibrer ou émettre `TOUR_NON_DEMARRE` après acceptation par le client.
- Déduire un travail utile de la seule présence d'un tour fournisseur.
- Lire ou afficher le contenu des conversations au-delà de la corrélation
  transitoire en mémoire par identifiant.
