# Plan 021 — Verdict lié au SHA réellement mesuré

## Pourquoi ce dessin

La greffe est le seul endroit qui voit simultanément le mandat durable et le
verdict entrant. Déplacer la comparaison dans le CLI laisserait le demandeur
juge de sa propre attente ; la placer dans Bridget déplacerait une règle métier
dans le transport. Le CLI ne fait donc qu'observer Git, Bridget transporte, et
Maicie compare puis persiste.

## Réutilisation

- Réutiliser `delivery_report` et son reçu idempotent : pas de seconde boîte,
  pas de nouveau chemin réseau.
- Ajouter un bloc optionnel `review_verdict` ; absent, le canon historique est
  inchangé.
- Réutiliser `guichet_refusal_receptions` pour les refus déterministes.
- Conserver `DecisionCoordination` et la clôture explicite de l'objectif.

## Modèle

- `Delegation.review_target: Option<ReviewTarget>` : `target_ref` et
  `expected_head`.
- `ReviewVerdictEvidence` filaire : verdict, référence, SHA attendu, SHA
  mesuré et tête distante observée.
- `MotifRefusGreffe` reçoit six variantes de revue.
- Schéma Maicie v17 : reconstruction de `guichet_refusal_receptions` avec le
  CHECK élargi ; les lignes historiques sont recopiées sans transformation.

## Flux

1. `maicie delegate` valide et persiste la cible de revue.
2. L'instruction envoyée et rejouée porte la cible typée.
3. `bridget guichet deposer delivery-report --verdict …` observe Git avant
   d'ouvrir le dépôt.
4. Bridget valide et conserve les octets canoniques.
5. Maicie relève, charge la délégation et compare en O(1).
6. Refus : reçu typé, aucune transition. Accord : délégation `terminee`,
   objectif `a_evaluer`, décision de greffe et réponse atomiques.

## Validation

- Oracles domaine pour la matrice de comparaison et les formes incomplètes.
- Oracle CLI binaire dans un dépôt + remote Git jetables.
- Oracle de réconciliation guichet sur le chemin réel, avec contrôle positif
  documenté : retirer l'appel de garde doit faire accepter le mauvais SHA et
  donc faire échouer le test.
- Oracle de migration v16 → v17 avec refus historique conservé.
- `cargo test --workspace --no-run` avant tout compte.
- Tests ciblés, puis workspace complet avec passes/rouges/ignores et liste
  imputée des rouges.
- `cargo fmt --all --check`, `git diff --check`, Clippy sur les crates touchées.

## Complexité et limites

La comparaison est O(1). `git ls-remote` ajoute une I/O réseau bornée par le
processus Git ; son échec interdit le dépôt. Aucun scan du dépôt, aucun hash du
worktree et aucune inspection du target Cargo ne sont ajoutés : ils ne
répondraient pas à la propriété demandée.
