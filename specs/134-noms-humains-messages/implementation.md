# Journal d’implémentation — Noms humains dans les messages Bridget

## Métadonnées

- **Spec** : 134-noms-humains-messages
- **Branche** : session-134-noms-humains-messages
- **Démarré** : 2026-10-05
- **Terminé** : En cours
- **Statut** : In Progress

## Revue du plan

- Auditeur externe : agent Claude `29aaed9b-9f6f-4849-87a5-1a23bbe01948`.
- Verdict : `APPROVE_WITH_CHANGES`.
- Intégré : repli sur nom blanc et validation commune de la publication T3.
- Vérifié comme déjà satisfait : la branche part de `main` après SPEC-133.
- Première demande externe : expirée sans verdict. Aucune approbation inventée.

## Progression

### T001 — Préflight et conception

- **Statut** : Complété
- **Base** : `main` au commit `54414d391493ee232f40a2048d79487f800db5d5`
- **Worktree** : branche `session-134-noms-humains-messages`, état isolé
- **Tests exécutés** :
  - `cargo test -p bridget-core message` : 13 tests passés
  - `cargo test -p bridget-daemon spec110` : 3 tests passés
  - `cargo test -p bridget-daemon spec114_lot` : 3 tests passés
- **Analyse SpecKit** : 9 exigences sur 9 couvertes par 7 tâches ; aucune
  incohérence, ambiguïté ou violation constitutionnelle détectée.
- **Notes** : le runtime officiel SpecKit ne fournit pas ses scripts ou modèles
  dans ce dépôt. Les artefacts ont suivi les formats existants du projet.

### T002 — Libellé commun

- **Statut** : Complété
- **Fichier** : `crates/bridget-core/src/message.rs`
- **Test rouge** : 3 tests `spec134`, 2 échecs prouvant que le nom était ignoré.
- **Test vert** : 3 tests `spec134` passés ; le test de compatibilité `spec133`
  passe aussi.
- **Résultat** : le nom nettoyé précède l’UUID complet. Les noms absents, vides,
  blancs ou égaux à l’UUID gardent le format historique. La provenance déléguée
  reste après le parent.
- **Self-review XIX/XX** : une seule fonction centrale change. Aucun helper,
  état, dépendance ou branche hors besoin. Le calcul reste O(1).

### T003 — Lots T3

- **Statut** : Complété
- **Fichier** : `crates/bridget-daemon/src/t3code.rs`
- **Test rouge** : le lot affichait les deux UUID sans les noms attendus.
- **Test vert** : le nouveau test `spec134` et les 3 tests `spec114_lot`
  passent.
- **Résultat** : chaque élément du lot appelle le même `sender_label()` que le
  rendu unitaire.
- **Self-review XIX/XX** : une seule expression remplace un accès direct
  divergent. Aucun format parallèle n’est ajouté. Le lot reste O(n), avec sa
  borne existante.

## Self-review Article XIX/XX

- Pourquoi cette solution est nécessaire : à compléter après le diff.
- Pourquoi elle est plus simple ou plus maintenable : à compléter après le diff.
- Hypothèses prises : à compléter après le diff.
- Vérifications réalisées : à compléter après les tests.
- Non vérifié : à compléter avant livraison.
- Code supprimé ou évité : à compléter après le diff.
- Complexité ajoutée et justification : à compléter après le diff.
