# Lot 12 - Stockage documenté (T125-T127)

But : écrire ce qui existe pour les bases SQLite (versions de schéma, modes de journal) et proposer un ADR. **Aucun code ni schéma ne change.** Un changement de schéma est un format durable (R6.2). Choix : `research.md` C12. Mesures : `releves.md` section 10.
Branche : `session-260-lot12-stockage`. Worktree : `.worktrees/260-lot12-stockage`. Aucun code exécutable, aucune compilation, pas de `C-TESTS`.

## T125 - Inventaire

Créer `specs/260-assainissement-code/validation/lot12-inventaire-stockage.md`. Cinq tableaux, chacun rempli **par commande**, pas de mémoire.

1. **Mécanismes de version** (trois sont attendus : table de migrations, `user_version`, `PRAGMA table_info`) :
   ```sh
   rg -n 'user_version' crates --glob '*.rs' | rg -v '_tests?\.rs:'
   rg -n 'PRAGMA table_info|pragma_table_info' crates --glob '*.rs' | rg -v '_tests?\.rs:'
   rg -n 'MAX_IDEMPOTENCY_SCHEMA_VERSION|schema_migrations|CREATE TABLE IF NOT EXISTS schema' crates --glob '*.rs' | rg -v '_tests?\.rs:'
   ```
   Tableau : mécanisme / fichiers / ce qu'il compare / ce qui arrive si la base est plus récente que le code.
2. **Fichiers de base** : `rg -n 'Connection::open' crates --glob '*.rs' | rg -v '_tests?\.rs:' | sed -E 's/:.*//' | sort | uniq -c | sort -nr` (environ 110 ouvertures hors tests). Tableau : fichier de code / nombre d'ouvertures.
3. **Modes de journal** : `rg -n 'journal_mode|synchronous|wal_autocheckpoint|busy_timeout' crates --glob '*.rs' | rg -v '_tests?\.rs:'`. Attendu : WAL dans `store.rs` ; `journal_mode=DELETE; synchronous=FULL` dans `delivery_spool244.rs` (ou `delivery_spool.rs` si le lot 11 est intégré) et `reply_outbox244.rs` (idem `reply_outbox.rs`). Tableau : base / mode / raison si elle est écrite dans un commentaire (sinon « raison non écrite »).
4. **Schéma centralisé** : lire `crates/bridget-daemon/src/store_schema.rs` (le début suffit : 40 lignes) et noter la version maximale lue.
5. **Écarts** : une liste courte (au plus 8 lignes) : ce qui est incohérent entre mécanismes (ex. une base sans numéro de version).

Ne rien conclure qui ne soit pas lu. Une cellule inconnue s'écrit « non lu ». Commit : `docs(specs): Inventorier les mecanismes de version de schema`.

## T126 - ADR proposé

Créer `docs/decisions/260-stockage-versions-schema.md`, d'après le modèle `docs/decisions/251-communications-attentives.md` (lire ses titres : `ls docs/decisions`, `sed -n '1,25p' docs/decisions/251-communications-attentives.md`). Sections : `# 260 - Versions de schéma des bases`, ligne `Date : <jour>. Statut : Proposé. Décision à prendre par François.`, puis `## Contexte` (les trois mécanismes de T125, une phrase chacun), `## Options` (A : ne rien changer ; B : un seul mécanisme à terme, migration base par base ; C : documenter seulement et ajouter un test qui échoue si une base n'a pas de version), `## Conséquences de chaque option` (risque sur les formats durables, effort, réversibilité), `## Recommandation` (une phrase), `## Ce qui n'est pas décidé`.
Règles : aucune option ne doit être présentée comme déjà décidée. Aucune adresse, aucun chemin privé. `python3 scripts/check-docs.py` : code 0. Commit : `docs(decisions): Proposer l'unification des versions de schema`.

## T127 - Porte du lot

1. `python3 scripts/check-docs.py` et `bash scripts/check-secrets.sh` : code 0.
2. `git diff origin/main --stat -- crates` : **vide** (aucun fichier de code touché).
3. Bilan dans `implementation.md` : une phrase par mécanisme, le lien de l'ADR, « décision à prendre par François ». Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- Toute tentation de « corriger » un écart trouvé : non (R6.2, R6.10). L'écrire dans la section « Idées » du journal.
- Un fichier de code est modifié par erreur : `git checkout -- crates`.
