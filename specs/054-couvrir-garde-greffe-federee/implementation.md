# Journal d'implémentation — Session 054

## Métadonnées

- Branche : `session-054-couvrir-garde-greffe-federee`
- Base gelée : `fbb4327aa3794ffbbb8f59b360e6b2f06c3d68f7`
- Statut : prête à relire

## Mesures

### Portée réelle des trois chemins directs sur la base

Le témoin runtime a exécuté les trois entrées avec une configuration jetable,
une socket absente et aucun fichier de base préexistant. Avant correction :

```text
3 sur 3 entrées contournent la garde : migrate: result=Ok("schéma migré vers 20") sqlite=true catalogue=false ; plage-list: result=Ok("plages=0") sqlite=true catalogue=false ; registre-list: result=Ok("registre list\naucune entrée") sqlite=true catalogue=true
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 21 filtered out
```

- `migrate` écrit le schéma SQLite jusqu'à la version 20 ;
- `plage list` est une lecture métier, mais l'ouverture du store initialise la
  base, le WAL et l'émetteur : ce chemin écrit donc avant sa lecture ;
- `registre list` initialise la base et le journal catalogue.

Les trois chemins doivent par conséquent être gardés avant l'ouverture. Le
pré-vol de schéma reste explicitement hors de cette frontière : il travaille
sur une copie jetable en lecture seule et ses 6 témoins dédiés restent verts.

### Oracle de couverture rouge

L'oracle structurel, encore sur la base, a inventorié les appels directs :

```text
sites d'ouverture directe hors helper gardé : ["open_maicie_store:376", "open_maicie_store:378", "run_migrate:1113"]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 21 filtered out
```

Le total `1 + 21 = 22` concorde avec l'exécution complète de la même commande
sur le même arbre, qui rend `22 passed; 0 filtered out`. Il n'est pas comparé
à un comptage statique absolu des attributs de test : les conditions `cfg`, les
tests ignorés et le périmètre de crates rendent cette égalité non générale.

## Correction

`open_guarded_maicie_store` est désormais l'unique frontière d'ouverture de
la base configurée par la CLI. Il reçoit une configuration déjà chargée,
atteste le daemon avec cette même configuration, puis ouvre ou migre le chemin
qu'elle contient. Cela évite de séparer la garde de l'ouverture et évite aussi
un second chargement de configuration entre les deux opérations.

`open_store_with_reconciliation`, `run_migrate`, `run_plage` et
`run_registre` empruntent tous cette frontière.

## Mutants causaux

Trois mutants ont chacun réintroduit une ouverture directe dans une seule
entrée. Pour chacun, le témoin runtime et l'inventaire structurel sont morts :

- `run_migrate` : `migrate` recrée SQLite et l'inventaire nomme
  `run_migrate:1120` ;
- `run_plage` : `plage-list` recrée SQLite et l'inventaire nomme ses deux
  ouvertures directes ;
- `run_registre` : `registre-list` recrée SQLite et le catalogue, et
  l'inventaire nomme ses deux ouvertures directes.

Chaque tir natif rendait :

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 21 filtered out
```

Après restauration, les deux témoins rendent chacun :

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out
```

Cette concordance ne sert qu'à comparer les exécutions du même binaire et de
la même commande ; elle ne constitue pas une preuve autonome du SHA compilé.

## Validation

```text
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Ces lignes correspondent respectivement aux tests du binaire Maicie, de la
bibliothèque Maicie et au harnais `schema_preflight`. `cargo check -p maicie
--all-targets` est vert.

`cargo clippy -p maicie --all-targets --no-deps -- -D warnings` reste rouge
sur quatre `needless_borrow` dans `plugins/maicie/src/app.rs`. La même commande
sur la base gelée rend les quatre mêmes diagnostics ; ils sont hors delta.

Un premier tir final du binaire a été invalidé : deux fixtures dépassaient la
borne de 103 octets des chemins de socket et refusaient sur la validation de
configuration, avant la garde observée. Le nom temporaire a été raccourci ; le
tir conservé est celui des 22 tests verts ci-dessus.

Deux commandes filtrées ont aussi été rejetées parce que `--exact` avait reçu
le nom sans le préfixe de module : `0 passed; 22 filtered out`. Relancées avec
le nom qualifié `tests::…`, elles rendent chacune `1 passed; 21 filtered out`.
Le tir nul est distingué du tir réel par la présence d'un test exécuté. Les
totaux concordent avec le tir complet de cette même commande, sans inférence
sur un comptage statique absolu.

Limites déclarées : la campagne workspace complète et macOS n'ont pas été
mesurés dans ce lot ciblé.
