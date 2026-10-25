# Archives techniques de l’adaptateur T3

Document d’origine : [specs/116-fiabilite-hygiene/implementation.md](../../../../../../specs/116-fiabilite-hygiene/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

### Mesures
- `horizon-terra-captation` (Codex) : 38 messages, 38 tours selon T3, 34 tours écrits. Les messages de
  08:06:32 et 08:25:53 le 20/09 ont chacun eu un tour muet (`01a0bdda`, `01a0bdeb`) ; deux demandes
  voisines ont été abandonnées.
- `opus_city_ai` (Claude) : 58 messages, 139 tours écrits, 320 points de contrôle — réveils
  d'arrière-plan à la fin des sous-agents, et messages tapés pendant un tour.
- Points de contrôle T3 écartés comme décompte : absents sur 5 fils sur 20 (hors Git), non un pour
  un sur 9. Hypothèse abandonnée avant d'écrire du code.
- `latestTurn.requestedAt` égale le `createdAt` d'un message unique sur 3 fils Codex et 1 fil Claude ;
  aucun message pour un réveil d'arrière-plan. C'est la preuve retenue.

<a id="extrait-2"></a>

## Extrait 2

### Correction
- `record_turn_origin` relève l'origine du dernier tour à chaque lecture (avant, seulement si le
  journal d'observation était à jour) ; `prune_turn_origins` borne la table sans élaguer une origine
  qu'une demande attend encore.
- `correlate` consulte d'abord l'origine prouvée ; `proven_turn_answer` rend le texte de ce seul
  tour, `Waiting` s'il est ouvert ou en écriture, `Silent` s'il est clos sans texte. Le rang reste en
  repli, inchangé : les tests historiques passent sans modification.
- `Correlation::Silent` n'est engagé qu'après SETTLE_ATTEMPTS lectures (relecture critique : T3
  projette de façon asynchrone ; un texte arrivé tard l'emporte). Annonce attribuée à Bridget.

<a id="extrait-3"></a>

## Extrait 3

## C. Entretien automatique
- `settle_expired_dispatching` : remises expirées encore `dispatching` → `indeterminate`.
- `purge_expired_sends` : envois seuls, 30 jours après expiration ; `purge_expired`, jamais appelée
  en production, aurait emporté `spawn_commands` en cascade.
- `purge_stale_identity_files` : marqueurs de processus morts ou recyclés ; noms et preuves de plus de
  7 jours ni référencés ni connectés. Réutilise `read_marker_file`. Une preuve d'instance connectée
  est gardée même sans marqueur (fil T3 au repos) : la supprimer aurait recréé la panne de la 115.
- `rotate_log_if_large` / `service_logs` : copie puis troncature au-delà de 20 Mio, une génération
  `.1` ; journaux ouverts en mode ajout par launchd (vérifié `lsof +fg` : AP).
- `run_maintenance`, fil `entretien` : 2 min après le démarrage puis toutes les heures ; verrou tenu
  pour les deux requêtes SQL seulement ; sortie sur arrêt par `sleep_until_shutdown`.
- `prune_merged_worktrees` (`scripts/build.py`) : fusionné dans main, propre (non suivis compris),
  dernier commit de plus de 24 h, aucun processus dont le répertoire courant s'y trouve (un `lsof`,
  échec = rien retiré) ; branche conservée. Racine prise sur le dépôt principal. Simulation sur le vrai
  dépôt : exactement les 21 attendus, les 3 sales, 2 non fusionnés et 2 récents épargnés.

<a id="extrait-4"></a>

## Extrait 4

## Livraison
- 07:52 : fusion, construction ; le script retire de lui-même les 21 worktrees fusionnés.
- 07:52 puis 07:57 : relance du daemon et du pont (la seconde pour l'identifiant de build corrigé).
  `bridget status` annonce `259b85793d50`, commit réel ; 26 agents, 22 fils T3.
- 07:54:53 : premier passage d'entretien — 33 remises en vol passées en sort inconnu ; 55 marqueurs,
  49 noms, 2 preuves retirés. `spawn_commands` intact (449).
- Identités vivantes vérifiées après ce passage : les 10 marqueurs restants désignent tous un agent
  vivant ; `requests` et la sonde d'envoi aboutissent depuis le fil `bdget`.
- 52 fils ont déjà des origines de tour relevées par le nouvel appariement.
