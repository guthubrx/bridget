# Archives techniques de l’adaptateur T3

Document d’origine : [specs/154-isolation-worktree/adversarial-review-codex.md](../../../../../../specs/154-isolation-worktree/adversarial-review-codex.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | CRITIQUE : `log.showSignature=true` fait lancer `gpg.program`, choisi par le dépôt, pendant le rapport | cohérent avec le plan : le rapport appelait `git log` sans couper les signatures ; `tasks.md` T015 | oui | `--no-show-signature` et `log.showSignature=false` ajoutés (T005) ; mais voir 2 : défense en profondeur seulement |
| 2 | CRITIQUE : la configuration change entre le contrôle et l'extraction ou le rapport ; `status` lance alors un filtre `clean` | cohérent : le contrôle de T008 précède de plusieurs secondes l'extraction et le rapport | oui | non réparable par contrôle. FR-013 devient « garde-fou, pas garantie ». Le choix de l'option d'exécution de Git règle la racine (arbitrage utilisateur) |
| 3 | ÉLEVÉE : course de chemins et retrait récursif interne de Git après un échec de création (`remove_junk`, Git 2.44) | source de Git non disponible ici ; plausible : Git nettoie ce qu'il a créé, et Bridget pré-créait le dossier final | oui | Bridget ne pré-crée plus le dossier final (T009). La course sur `.worktrees` reste, risque accepté par l'arbitrage (même niveau que T3) |
| 4 | ÉLEVÉE : le rapport Git s'exécute sous le verrou global dans le tick (`native_delegation.rs:959`, `:1111`) | lu : le tick tient le verrou pendant ces transitions | oui | D12, T028, SC-011 : noter sous verrou, lire dehors, ranger sous un verrou court |
| 5 | ÉLEVÉE : deux appels de même clé créent deux worktrees ; le plafond de 32 se dépasse (`delegation.rs:120`) | cohérent : la première passe ne gardait aucune trace entre les deux passes | oui | FR-027, T027, SC-010 : table de réservations en mémoire, `workspace_creation_in_progress`, plafond qui compte les réservations |
| 6 | ÉLEVÉE : le wrapper utilise `.output()` sans borne et ne neutralise pas les filtres de `status` (`wrapper.rs:625`, `:641`) | lu : `Command::output()` sans délai ni limite ; `status` peut lancer un filtre `clean` | oui | FR-028, T022 : les lectures passent par l'exécuteur durci ; `status` omis si un filtre est configuré |
| 7 | ÉLEVÉE : revalidation contradictoire (la branche réservée existe, la 32e place est la nôtre) | cohérent avec le contrat : la seconde passe refaisait « tous » les contrôles | oui | FR-017, contrat, T009 : la seconde passe ne refait pas les contrôles d'état Git créés par la requête et compte le plafond sans la réservation propre |

<a id="extrait-2"></a>

## Extrait 2

**Arbitrage utilisateur (2026-10-11)** : option « exécution directe de Git, durcie ». Le daemon
exécute Git pour tout parent autorisé, avec tous les durcissements prévus. Fait
vérifié dans T3 : `GitVcsDriverCore.ts:3418-3419`, `:3789`, `GitVcsDriver.ts:430`
(Git exécuté avec les droits du serveur, seul `fsmonitor` coupé). **Risque
résiduel accepté tel qu'écrit** : Un parent confiné qui peut modifier la configuration du dépôt entre le contrôle et l'exécution de Git peut faire exécuter du code avec les droits du daemon. Ce risque est accepté explicitement, selon les gardes natives, dans le cadre de menace de l'ADR 011. Les questions 1 à 3 gardent leurs valeurs par défaut. Les points 1, 2
et 3 sont donc traités en défense en profondeur et non comme une frontière.
`tasks.md` compte 28 tâches.
