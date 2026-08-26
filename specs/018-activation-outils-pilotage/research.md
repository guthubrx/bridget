# Recherche technique — Activation gouvernée des outils de pilotage

## Décision 1 — Vérifier l'admission, pas seulement le rangement

**Décision** : exiger successivement un checkout principal, la branche
symbolique `main`, un arbre propre, puis prouver que `HEAD` est ancêtre de
`refs/remotes/origin/main` avec `git merge-base --is-ancestor`.

**Rationale** : refuser un chemin contenant `.worktrees` ne prouve pas qu'un
commit a été jugé. À l'inverse, l'ancestralité accepte un rollback vers un
commit déjà admis tout en refusant un `main` local en avance ou divergent.

**Alternatives considérées** :

- comparer uniquement le nom de branche : refuse certains gestes mais laisse
  passer un `main` local non poussé ;
- exiger `HEAD == origin/main` : ferme le défaut mais interdit sans raison un
  rollback vers une version déjà admise ;
- lancer `git fetch` dans l'installateur : introduit une mutation réseau
  implicite et rend l'installation dépendante de la disponibilité distante.

## Décision 2 — Extraire l'objet Git, ne pas copier le fichier de travail

**Décision** : matérialiser l'artefact depuis le blob `HEAD:<chemin>` et non
depuis les octets du checkout.

**Rationale** : l'objet Git lie exactement les octets au SHA annoncé. Le
contrôle d'arbre propre reste un refus d'honnêteté opérationnelle, tandis que
l'extraction par objet constitue la preuve mécanique.

**Alternatives considérées** :

- lien vers le checkout principal : une modification locale ou un changement
  de branche devient actif sans nouvelle activation ;
- copie directe du fichier de travail : dépend d'une observation instantanée
  et oblige à refaire séparément la preuve d'identité avec le commit.

## Décision 3 — Release nommée par SHA hors dépôt

**Décision** : stocker chaque outil sous
`$HOME/.local/share/bridget/pilotage/releases/<SHA>/<outil>`, puis faire pointer
`$HOME/.local/bin/<outil>` vers cette version.

**Rationale** : la release survit à la suppression de tous les worktrees. Le
SHA complet visible dans la cible distingue immédiatement une release d'un
espace de travail. Un fichier d'origine adjacent donne le remote, la référence,
le nom et l'empreinte de l'artefact. L'URL brute est volontairement omise car
elle peut contenir des identifiants et varie entre HTTPS et SSH.

**Alternatives considérées** :

- copier directement dans `$HOME/.local/bin` : exécutable mais origine
  indiscernable et rollback non lisible ;
- lier vers le checkout stable : règle l'incident courant sans fermer la
  mutation implicite du checkout.

## Décision 4 — Politique centralisée pour deux consommateurs

**Décision** : porter les gardes et la matérialisation dans un helper Bash
commun aux deux installateurs.

**Rationale** : cette abstraction n'est pas une factorisation cosmétique ;
elle porte une frontière de gouvernance qui doit rester identique pour
`bridget-idle` et `bridget-ronde`. Deux implémentations indépendantes
recréeraient le risque de dérive qui a laissé la ronde annoncer un faux succès.

**Alternatives considérées** : dupliquer le contrôle dans chaque installateur.
Rejeté car toute évolution devrait alors prouver deux politiques distinctes.

## Décision 5 — Activation atomique, corruption fail-closed

**Décision** : préparer artefact, preuve et lien temporaire avant renommage
dans leur emplacement final. Une release existante sous le même SHA mais aux
octets différents est refusée, même avec `--force`.

**Rationale** : `--force` autorise le changement de version active ; il ne doit
jamais réécrire l'histoire associée à un SHA. Une interruption avant le
renommage laisse l'ancienne entrée intacte.

## Sources locales vérifiées

- `git help merge-base` : contrat de `--is-ancestor` et codes de sortie.
- `git help show` : lecture d'un blob adressé par `<commit>:<chemin>`.
- Installateurs et harnais existants aux SHA `aa16dd3` et `b6eea77`.
- Incident et règles opératoires dans `docs/regles-chantier.md`.

La décision est propre au dépôt et ne dépend d'aucun état récent d'un service
tiers ; aucune recherche Web n'est nécessaire.
