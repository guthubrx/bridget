# Implementation 017 — Gate de session

**Branche** : `session-17-greffiere-catalogue`
**Gate** : `cargo test -p maicie --test catalogue_session_gate -- --nocapture`
**Fichier** : `plugins/maicie/tests/catalogue_session_gate.rs`

Ce gate n'est pas une suite : un seul scénario enchaîne cinq promesses.
Chaque assert échoue avec le nom de la promesse et la **mutation** qui la
casserait. Les oracles observent des **faits** (IDs, octets, enums, compteurs
recalculés depuis les entrées brutes) — jamais un libellé humain du rendu.

## Promesses et mutations

| # | Promesse | Preuve dans le gate | Mutation qui fait échouer |
|---|---|---|---|
| 1 | Journal qui **survit** | 3 lignes écrites, dernière tronquée au milieu, relecture : les 2 antérieures lisibles + `torn_tail_warning` | Refuser le journal entier sur queue sans LF ; ou écrire ligne puis `\n` en deux `write_all` |
| 2 | Migration **rejouable** | Corpus réel (~40) migré deux fois : 1er = N appended, 2e = 0 appended / N skipped | Omettre le dédup par `provenance_id` (le 2e passage double les pending) |
| 3 | Qualification **sans réécriture** | Texte pending capturé en octets → `qualify_pending` → texte de l'`add` **égal octet pour octet** | Normaliser/trim/réécrire le texte dans `qualify_pending` |
| 4 | Table qui **dérive**, n'invente pas | `gate_failed`→`Severity::Blocker`, `review_amender`→`Severity::Major`, `review_approve`→pending `uncovered:` | Mapper hors-table vers une sévérité ; ou changer le mapping des cases couvertes |
| 5 | Vue d'autorité = **vérité** | Compteurs N/M/K/P recalculés **hors** `project_registre` puis comparés au footer ; transition `delivered` fait baisser N et K et retire l'id des ouverts | Hardcoder le pied ; compter delivered comme ouverts ; inventer K/P ; oracle sur libellé de rendu |

Sortie observée (2026-08-24) : `gate_session_017 OK — N=2 M=0 K=0 P=40 delivered=1`
(après transition attestée sur le gate consignés).

## Ce que ce merge contient

- Journal v1 fermé, append atomique, fenêtre de corruption fermée
- Migration prose → pending ; qualification humaine verbatim
- CLI `registre list|add|migrer|qualifier|consign`
- FR-1711 : table contrat + transcription
- Lien d'arbitrage durable (`constat_id` / `pour_constat` / `delegation_arbitration_links`)
- `reconcile_catalogue_from_store` + déclenchement au fil des commandes `registre`
- Skill + quickstart (registre list en ouverture/clôture)
- Gate de session ci-dessus

## Ce qui reste ouvert

| Item | Une ligne |
|---|---|
| T1711 remède via délégation | Porter `constat_id` ; refuser message libre depuis le catalogue |
| T1713 README racine | Section journal du dû absente du README workspace ; plugin OK |
| T1715 suite workspace + revue hostile | Gate catalogue vert ; clippy/workspace complets et revue hostile humaine encore à conduire |
| Câblage producteurs → `consign` | Les chemins métier (runner de gate, routine de revue) n'appellent pas encore `registre consign` automatiquement |
| SC-1705 intégration | Preuve délégation+reçu pour un remède lancé depuis un constat |

**Merge recommandé** : livrer le journal, la migration, la table, la vue, le
lien d'arbitrage câblé et le gate. Laisser T1711 (remède) et T1715 (revue
hostile workspace) pour la passe suivante.
