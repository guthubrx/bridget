# Plan technique — Session 051

## Décision

Faire porter à la collecte de statut un résultat explicite : un statut normal
inclut le cas hors ligne et le cas ancien non attesté ; une négociation engagée
mais illisible rend une erreur. La sonde pose un délai de lecture de deux
secondes sur la socket clonée avant son premier `read_line`.

Le délai de deux secondes reprend la borne déjà utilisée par les sondes CLI de
runtime. Il borne chaque lecture du petit protocole local sans ajouter de
configuration ni de dépendance. Après une erreur de sonde, `get_status`
s’arrête : il n’ouvre pas la seconde connexion qui collecte l’annuaire.

## Propagation

- `status`, `agents` et `who` rendent un diagnostic non nul ;
- la carte de reprise conserve l’erreur dans sa source `status` existante ;
- le reaper interrompt son observation au lieu de fabriquer une liste vide ;
- un daemon ancien qui a répondu au `ClientWelcome` mais ne fournit pas le
  rapport récent reste un succès non attesté.

## Étapes

1. Monter un pair Unix qui accepte, capture la trame puis se tait ; mesurer le
   blocage de la base avec une échéance externe qui termine proprement l’enfant.
2. Faire retourner à la sonde un résultat distinguant absence et
   indisponibilité ; poser le délai avant la première lecture.
3. Propager l’erreur aux consommateurs sans modifier le contrat filaire.
4. Rejouer les contrôles voisins et le mutant qui retire la pose du délai.

## Fichiers prévus

- `crates/bridget-daemon/src/daemon.rs`
- `crates/bridget-daemon/src/cli.rs`
- `crates/bridget-daemon/src/reprise.rs`
- `crates/bridget-daemon/src/reaper.rs`
- `crates/bridget-daemon/tests/identity_probe_timeout_test.rs`
- `specs/051-borner-sonde-identite-daemon/*`

## Risques et contrôles

- **Faux hors ligne** : l’oracle exige un diagnostic d’indisponibilité et une
  connexion effectivement acceptée.
- **Compatibilité rompue** : un rapport ancien incomplet reste un succès sans
  attestation ; le contrôle dédié garde cette frontière.
- **Seconde suspension** : le pair compte les connexions et en exige une seule.
- **Oracle pendu sous mutant** : le parent du banc impose une échéance externe
  et envoie seulement SIGTERM à son propre enfant avant d’échouer.
- **Périmètre diffus** : aucune dépendance, option ou trame nouvelle.

## Contrôle constitutionnel

- La solution réutilise la bibliothèque standard et les résultats déjà portés
  par la carte de reprise.
- La complexité reste O(1) : nombre fixe de trois échanges de négociation.
- Les effets sont observés sur le vrai binaire et une vraie socket Unix.
- L’état incertain est déclaré ; aucune absence n’est déduite d’une lecture
  aveugle.

