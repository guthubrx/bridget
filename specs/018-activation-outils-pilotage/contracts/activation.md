# Contrat d'activation des outils de pilotage

## Entrées

- outil suivi par Git : `bridget-idle.py` ou `bridget-ronde.py` ;
- checkout depuis lequel l'installateur est exécuté ;
- option explicite `--force` pour remplacer une entrée active différente.

## Préconditions, dans l'ordre observable

1. Le checkout est le principal du dépôt, jamais un worktree lié.
2. `HEAD` est attachée à la branche locale `main`.
3. L'arbre suivi et non suivi est propre.
4. L'entrée active est absente, déjà gérée au même SHA, ou son remplacement est explicitement autorisé.
5. `refs/remotes/origin/main` existe et contient `HEAD` dans son ascendance.

Chaque précondition non satisfaite produit un refus non nul avant toute
écriture de release, de commande ou d'unité.

## Sortie sûre

- Artefact : `$HOME/.local/share/bridget/pilotage/releases/<SHA>/<outil>`.
- Origine : fichier adjacent `<outil>.origin` contenant format, remote,
  référence, SHA, nom d'artefact et SHA-256, jamais l'URL Git brute.
- Entrée active : `$HOME/.local/bin/<outil>`, lien absolu vers l'artefact.
- Mode de l'artefact : lecture/exécution, sans écriture ordinaire.
- Représentation : l'artefact est un fichier régulier non lien au mode `0555` ;
  sa preuve est un fichier régulier non lien au mode `0444`.
- Résolution : le chemin canonique du répertoire de release reste hors du
  dépôt source, même si le répertoire SHA ou un parent a été remplacé par un
  lien.
- Postcondition : l'entrée active est relue après remplacement et sa cible
  textuelle doit être exactement l'artefact attendu.

## Idempotence

Rejouer l'installation depuis le même SHA avec une release intacte et la même
cible rend un succès explicite sans changer les octets. Une release existante
mais différente rend un échec de corruption et n'est jamais réparée
silencieusement.

Pour la ronde, l'idempotence porte aussi sur le contenu et le mode exacts des
unités. Une configuration différente sans `--force` est un refus non nul ; elle
ne peut jamais être masquée par une annonce finale de succès.

## Refus spécifique à la ronde

Si l'entrée active est une copie régulière et que `--force` est absent,
l'installateur s'arrête avant de créer le répertoire d'archives, les unités
launchd/systemd ou leur activation.
