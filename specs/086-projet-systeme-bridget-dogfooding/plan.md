# Plan d'implémentation - SPEC-086 Projet système Bridget et dogfooding expert

## Contexte technique confirmé

| Élément | Existant réutilisé |
|---|---|
| Registre projet | SPEC-065 fournit identité, génération, backend et état. |
| Emplacement système | SPEC-084 fournit `exact_project` et `system_only`. |
| Runtime partagé | SPEC-066/085 fournissent conteneur projet, montages et epoch. |
| Ressources | SPEC-067 borne extensions et secrets. |
| Réglages | SPEC-080 fournit catalogue fermé, preview/apply et reçus. |
| Git | Le dépôt Bridget utilise déjà des branches et worktrees liés. |

## Décisions de planification

1. Le projet système est un rôle protégé du registre, pas un projet ordinaire renommé.
2. L'installation accepte zéro ou un chemin source Bridget explicite. Aucun scan de l'hôte n'est effectué.
3. L'attestation vérifie une signature structurelle fermée du checkout sans exécuter son code.
4. Le dogfooding est un réglage serveur expert binaire, désactivé par défaut, applicable à tous les agents du projet système.
5. Le mode conserve le checkout de référence read-only, modifie la mutabilité du worktree attribué et impose un nouvel epoch/recreate. Il ne change pas les droits d'autres projets.
6. Le projet système doit utiliser Docker pour que la frontière read-only/read-write soit applicable par Bridget.
7. Git/worktree organise la concurrence. Une réservation de worktree empêche seulement deux travaux actifs de partager le même worktree.
8. La livraison reste extérieure à la capacité: pas de merge main, push, install, restart ou deploy automatique.

## Constitution check initial

| Gate | Verdict | Justification |
|---|---|---|
| Worktree isolé | PASS | Branche `session-086-projet-systeme-bridget-dogfooding` issue de `origin/main`. |
| Réutilisation | PASS | Rôle projet, catalogue, runtime, montages, réglages et Git existants sont étendus. |
| Minimalisme | PASS | Un rôle et un booléen serveur, sans RBAC par agent ni temporisation. |
| Sécurité | PASS sous garde | Disabled par défaut, Docker obligatoire, mounts dédiés et livraison séparée. |
| Réversibilité | PASS | Désactivation recrée en read-only sans modifier Git. |

## Plan par lots

### Lot 1 - Identité protégée du projet système

- Ajouter `ProjectRole::{Standard, BridgetSystem}` au registre et à ses projections.
- Ajouter une commande d'installation/admin locale pour déclarer le chemin source explicite.
- Attester le checkout par présence et cohérence de fichiers structurels Bridget versionnés, sans exécution.
- Créer ou réconcilier l'emplacement `exact_project + system_only` de SPEC-084.
- Refuser promotion ordinaire, second système et mutation du rôle par les routes projet standard.

### Lot 2 - Politique de montage

- Étendre le résolveur de mounts avec un contexte de rôle et mode de dogfooding.
- Résoudre checkout, git common dir et worktrees, tous aux chemins absolus identiques.
- Rendre tous les mounts read-only en mode disabled; en mode enabled, garder le checkout principal read-only et ne rendre writable que le worktree attribué ainsi que les métadonnées Git strictement nécessaires.
- Ajouter un invariant interdisant ces mounts à tout projet standard.
- Calculer un digest de topologie/mutabilité pour déclencher recreate.

### Lot 3 - Réglage expert et transition

- Ajouter `dogfooding.bridget` au centre de contrôle avec texte d'avertissement fermé.
- Lier preview/apply à la génération du projet système et à l'epoch runtime.
- Refuser si backend non Docker, capacité absente ou agent actif.
- Recréer l'environnement, attester les mounts, puis publier la nouvelle valeur.
- Compenser vers l'ancien mode si la recréation échoue.

### Lot 4 - Réservation de worktrees

- Étendre la réservation de travail/agent avec un worktree canonique non principal et une branche différente de `main`.
- Refuser un second travail actif sur le même worktree.
- Autoriser plusieurs agents/travaux dans des worktrees différents du même projet.
- Réconcilier les worktrees ajoutés à l'extérieur et demander recreate si nécessaire.

### Lot 5 - Interface et séparation de livraison

- Afficher une carte Projet système dans les réglages expert du serveur.
- Montrer état, chemin borné, backend, mode, epoch et raison d'indisponibilité.
- Exiger une confirmation expliquant exactement édition autorisée et livraison exclue.
- Ne présenter aucune action merge, push, install, restart ou deploy dans ce lot.

### Lot 6 - Tests et preuves

- Tester unicité, attestation, promotion interdite et absence de mount dans les projets standard.
- Tester écritures refusées/acceptées au niveau du mount dans les deux modes.
- Tester transition avec agent actif, generation mismatch et rollback de recreate.
- Tester deux agents dans deux worktrees et conflit sur le même worktree.
- Vérifier coexistence avec un agent hôte externe et absence de suppression Git.

## Fichiers principaux prévus

| Surface | Évolution |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | Rôle et contrats admin/transition fermés. |
| `crates/bridget-daemon/src/store.rs` | Unicité du rôle, mode confirmé et réservation de worktree. |
| `crates/bridget-daemon/src/project_policy.rs` | Emplacement système exact. |
| `crates/bridget-daemon/src/project_runtime.rs` | Mounts conditionnels et digest de topologie. |
| `crates/bridget-daemon/src/control_settings.rs` | Réglage expert et preview/apply. |
| `crates/bridget-daemon/src/daemon.rs` | Attestation, guards et saga de recreate. |
| `crates/bridget-daemon/src/ui.rs` | Projection et routes bornées. |
| `crates/bridget-daemon/assets/ui/*` | Carte expert, avertissement et états. |

## Stratégie de test

1. Tests de domaine/store avant routes.
2. Tests de mount avec projet standard et système dans les deux modes.
3. Tests d'intégration Docker pour chmod apparent et écriture réelle.
4. Tests multi-worktree avec deux index/HEAD distincts.
5. Tests UI de confirmation et capacité absente.
6. Vérification manuelle sans merge, installation ou restart.

## Risques et parades

| Risque | Parade |
|---|---|
| Les agents modifient leur propre système actif | Édition en worktree, aucune livraison automatique, daemon actif hors conteneur. |
| Un projet standard obtient la source | Invariant central du résolveur et tests négatifs sur tous les rôles. |
| Activation partielle | Epoch, recreate et publication tardive avec compensation. |
| Conflit Git entre agents | Worktrees distincts et réservation du chemin canonique. |
| Configuration trop complexe | Un projet système et un booléen expert, sans matrice de rôles. |
| Agent externe concurrent | Git reste source de vérité, aucune copie ni verrou global du dépôt. |
| Fausse isolation entre agents du projet | Avertissement explicite: le conteneur est partagé et les agents du projet système sont dans le même domaine de confiance. |

## Constitution check post-conception

PASS sous garde d'une séparation stricte entre édition et livraison. Le projet système est une exception explicite, unique et réversible, pas une extension des droits de tous les projets.
