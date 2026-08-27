# Spécification 054 — Couvrir la garde du greffe fédéré

**Statut** : Implémentée — prête à relire

**Base gelée** : `fbb4327aa3794ffbbb8f59b360e6b2f06c3d68f7`

**Objectif Maicie** : `0a8b1c5c-1076-44ca-9802-be7112ae9ccf`

## Problème mesuré

La garde `require_local_daemon` précède correctement SQLite dans
`open_store_with_reconciliation`, mais trois entrées CLI ouvrent directement
le store : `migrate`, `plage` et `registre`. Le témoin existant protège l'ordre
dans un chemin, pas la couverture de tous les sites d'ouverture.

## Propriété

Toute entrée CLI qui ouvre la base Maicie configurée atteste d'abord que le
daemon joint appartient à la machine locale, avec la même configuration. Un
refus de connexion ou une identité fédérée ne crée ni base SQLite, ni journal.

`preflight` reste l'exception fermée : il mesure une copie jetable sans écrire
la base configurée et n'accepte pas `--migrate`.

## Scénarios

- `migrate` sans daemon refuse avant de créer ou migrer SQLite.
- `plage list`, bien que consultatif après ouverture, refuse avant SQLite car
  l'ouverture peut bootstrapper la base, activer WAL et créer l'identité.
- `registre list` refuse avant SQLite et avant la création du journal.
- un nouveau site direct `MaicieStore::open*` hors de l'unique helper gardé
  fait échouer l'oracle de couverture.
- le daemon local attesté conserve les comportements existants.

## Critères de succès

- L'oracle de couverture est rouge sur la base gelée puis vert sur la tête.
- Le mutant qui réintroduit une ouverture directe dans chacun des trois
  chemins tue l'oracle concerné.
- La garde de position historique reste verte.
