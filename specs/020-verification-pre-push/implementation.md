# Journal d'implémentation — Spec 020

## Métadonnées

- **Spec** : 020-verification-pre-push
- **Branche** : session-20-verification-pre-push
- **Démarré** : 2026-08-25
- **Terminé** : en cours
- **Base mesurée** : b6eea777facf929d99a9c4f9ae75fb50e06dc2fd

## Progression

### T001 — Propriété et cas limites

- **Statut** : complété
- **Fichiers** : `spec.md`, `checklists/requirements.md`
- **Validation** : aucun marqueur de clarification ; checklist 10/10.
- **Note** : la propriété adoptée raisonne sur l'union transactionnelle.

### T002 — Plan technique

- **Statut** : complété
- **Fichiers** : `plan.md`, `research.md`, `quickstart.md`
- **Validation** : contrat `pre-push` recoupé avec la documentation Git.
- **Note** : l'activation reste dépendante de SPEC-018 et hors du lot.

### T003–T004 — Banc rouge

- **Statut** : complété
- **Fichier** : `scripts/test-git-pre-push-authorship.sh`
- **Mesure retenue sur stub permissif** : 1 passé / 6 rouges / 0 ignoré.
- **Contrôle positif** : l'héritage déjà distant suivi de commits propres passe.
- **Mesures écartées** : deux exécutions affichaient le même total mais les
  fixtures échouaient avant de créer leurs commits ; elles ne constituent pas
  une preuve et ne sont pas comptées.

### T005–T006 — Hook transactionnel

- **Statut** : complété
- **Fichier** : `scripts/git-pre-push-authorship.sh`
- **Mesure** : 7 passés / 0 rouge / 0 ignoré.
- **Propriétés vérifiées** : union multi-références, branche sans amont,
  force-push, filtre générique, héritage distant et échec fermé.

### T007 — Mutation du filtre réel

- **Statut** : complété
- **Mutation** : ajout d'une virgule dans le motif de co-autorat.
- **Mesure mutée** : 3 passés / 4 rouges / 0 ignoré.
- **Rouges** : branche neuve, force-push, multi-références et filtre direct.
- **Après restauration** : 7 passés / 0 rouge / 0 ignoré.

## Revue hostile

Deux problèmes trouvés et corrigés :

1. Le nettoyage temporaire encodait le chemin dans une chaîne de trap :
   remplacé par une fonction qui valide le préfixe avant suppression.
2. La destination distante pouvait être interprétée comme une option :
   ajout du séparateur d'options avant le chemin.

Portabilité vérifiée avec un `TMPDIR` absolu contenant des espaces :
7 passés / 0 rouge / 0 ignoré.

### T008 — Validation finale sans activation

- **Statut** : complété
- **Commit d'implémentation** :
  `ef845ab2ffe159d5ce2a8c6df917834a2df9769a`
- **Banc ciblé** : 7 passés / 0 rouge / 0 ignoré.
- **Syntaxe Bash** : valide sur les deux scripts.
- **Diff** : aucun espace ou marqueur invalide.
- **Simulation réelle** : création de référence simulée contre `origin`,
  deux commits introduits, acceptée sans envoi.
- **Activation** : `/home/moi/.git-hooks/pre-push` reste absent.
- **Non mesuré** : exécution macOS et analyse ShellCheck, indisponible sur la
  machine ; aucune suite Cargo, car aucun fichier Rust n'est modifié.

## REX

**Date** : 2026-08-25
**Tâches complétées** : 8/8
**Tests** : 7/7

### Ce qui a bien fonctionné

- La soustraction de l'état distant conserve l'héritage accepté tout en
  inspectant chaque commit réellement nouveau.
- Le test direct du motif et sa mutation empêchent qu'un hook seulement
  exécutable soit pris pour un hook efficace.

### Difficultés rencontrées

- Deux premières exécutions rouges avaient des fixtures invalides ; leurs
  comptes ont été écartés avant toute conclusion.
- La revue hostile a trouvé un nettoyage temporaire trop dépendant du quoting
  et une destination distante sans séparateur d'options ; les deux ont été
  corrigés avant livraison.

### Charge future

- Aucune dépendance ajoutée.
- Les deux scripts portent chacun une responsabilité : barrière et banc.
- Potentiel de suppression à comportement constant après revue : environ
  zéro ligne.

### Recommandation

Après jury et merge uniquement, faire passer l'activation par la règle de
projection durable de SPEC-018. Une barrière côté réception restera nécessaire
si la politique doit résister au contournement volontaire d'un hook local.
