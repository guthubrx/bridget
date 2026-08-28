# Plan 059 — Mesure vérification / production

## Décision technique

Étendre la ronde passive existante plutôt que créer un second ordonnanceur :

1. interroger les objectifs dans la copie SQLite déjà isolée ;
2. construire le manifeste effectif sans heuristique, avec `indeterminate` par
   défaut et quelques annotations historiques explicitement auditées ;
3. agréger par identifiant puis par ensembles de `root_id` ;
4. publier la baseline gelée et la fenêtre glissante courante dans le même JSON
   archivé ;
5. refuser tout ratio ponctuel dont la couverture n'est pas complète.

Le manifeste visible est la projection complète incluse dans le rapport. Les
annotations versionnées vivent près du calcul et portent une justification ;
elles ne modifient jamais la base historique.

## Ordre d'implémentation

1. Poser un oracle rouge sur la baseline réelle gelée et son empreinte.
2. Poser l'oracle de couverture : les non-annotés restent visibles et le ratio
   demeure indisponible.
3. Ajouter une chaîne réelle de production/revues partageant une racine, puis
   l'oracle d'invariance au découpage.
4. Implémenter la lecture, le manifeste effectif et les deux agrégations dans
   la ronde passive.
5. Rejouer le mutant qui agrège par `objective_id`, puis restaurer au condensat
   identique.
6. Vérifier passivité, dégradation, archive atomique, syntaxe et diff.
7. Publier la tranche avec les ratios effectivement disponibles et la
   couverture, sans proposer de cible.

## Fichiers prévus

- `scripts/bridget-ronde.py` : lecture et agrégation de la mesure.
- `scripts/test-bridget-ronde.sh` : oracles de passivité et de mesure sur les
  identifiants réels gelés.
- `specs/059-mesure-verification-production/` : contrat, plan, tâches et
  résultats.

Les fichiers `plugins/maicie/src/main.rs`, `plugins/maicie/src/lib.rs`,
`plugins/maicie/src/greffe_service.rs` et `crates/bridget-daemon/src/ui.rs`
restent hors delta pour ne pas dépendre de la session 058.

## Risques et gardes

- **Ratio sur sous-ensemble** : état `unavailable` tant que la couverture n'est
  pas totale.
- **Granularité manipulable** : cardinalités d'ensembles de racines et mutant
  `root_id -> objective_id`.
- **Fenêtres confondues** : type et bornes explicites dans chaque résultat.
- **Corpus vivant** : borne `as_of` injectée et transaction de lecture sur la
  copie, jamais `MAX(cree_at)` utilisé comme horloge.
- **Classification tautologique** : aucune attente dérivée de la fonction de
  classement ; la baseline impose un cardinal et une empreinte externes.
- **Mutation de la source** : comparaison des sidecars de la base d'autorité
  déjà exercée par le harnais de ronde.

## Complexité

Pour n objectifs, la lecture, le manifeste et les deux agrégations sont O(n)
en temps et en espace. Les ensembles de racines empêchent qu'une nouvelle
charge augmente le coût autrement que linéairement ou le compte racine.

