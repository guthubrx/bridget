# Contrat 097 — Claude interactif sans tmux

## Lancement

- `bridget claude [--name N] [args Claude…]` exige stdin et stdout sur un
  terminal ; sinon refus : « Claude interactif exige un terminal ; utilisez
  bridget spawn claude pour un agent détaché ».
- Échec d'ouverture du PTY ou de lancement du fournisseur : refus nommé avant
  toute présence Bridget.
- Les arguments utilisateur sont relayés tels quels après validation existante ;
  `--strict-mcp-config`, `--mcp-config` éphémère et les outils autorisés
  Bridget sont ajoutés comme aujourd'hui. Aucun bypass de permissions n'est
  ajouté implicitement ; un bypass explicite de l'utilisateur est relayé.

## Présence

- `who` : TYPE `claude`, TRANSPORT `claude_pty`, MODE `cli`, LOCALISATION `—`.
- Un type resté tmux (`gemini`, `gclaude`, `--`) sans pane est refusé au
  lancement : « aucun pane tmux ; lancez dans tmux ou utilisez bridget spawn ».

## Remise

- Message idempotent : écriture `ESC[200~` + enveloppe + `ESC[201~`, attente de
  digestion bornée, puis `\r`. Accusé `DeliverAcked` après succès ; sinon
  `DeliveryIndeterminate`. Rappels et notifications : même voie.
- Contenu refusé (séquences de contrôle, dépassement du plafond) : aucune
  écriture, état indéterminé nommé, trace journal.
- Saisie humaine en cours : le collage s'ajoute au composer natif ; le wrapper ne
  vide ni ne réécrit la saisie.

## Journal et attach

- `turn_start` pour chaque message Bridget remis ; `turn_start {from:"human"}`,
  `update {kind:"text"}` et `turn_end` issus du transcript de la session courante.
- `bridget attach <UUID>` accepté (mode `cli`, journal attesté).

## Fin de session

- Sortie du fournisseur → restauration du terminal → code de sortie relayé →
  déconnexion Bridget. Signaux INT/TERM/HUP : même chemin.
- Perte du daemon : l'enfant continue, reconnexion avec la même identité,
  notification « reconnecté » remise par le PTY.

## Claude géré (rappel du contrat 089)

- Lancement par le daemon avec l'environnement du daemon (HOME, USER hérités),
  registre `claude_stream_json`. Recette : demande suivie → `answered`, journal
  attachable, arrêt < 10 s, zéro processus survivant, aucune clé d'API.
