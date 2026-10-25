# Audit de réutilisation — SPEC 032

**Statut** : `OK` — aucune duplication évidente, aucun arbitrage requis.
**Produit le** : 2026-08-25 21h35 · **Méthode** : recherche par nom **et** par
responsabilité dans `crates/`, `plugins/`, avant toute proposition.

## Gate avant `tasks.md`

- [x] Chaque item proposé par le plan a fait l'objet d'une recherche d'existant.
- [x] Chaque item est tranché : `RÉUTILISER`, `ÉTENDRE` ou `CRÉER`.
- [x] Aucune duplication évidente détectée.
- [x] Aucune dépendance externe nouvelle.
- [x] Aucun `NEEDS_ARBITRATION`.

## Inventaire

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/032-gui-poste-de-travail/reuse-audit.md#extrait-1).

## Arbitrages

### A1 — Remplacer les 153 lignes de front embarquées plutôt que les étendre

**Équivalent trouvé** : `crates/bridget-daemon/src/ui.rs` — HTML/JS écrit en dur,
servi par un serveur HTTP artisanal.

**Décision : CRÉER `crates/bridget-daemon/assets/ui/`, et retirer l'embarqué.**

**Justification** :

1. **Frontière posée au plan** (§8bis) : *le relais ne calcule rien*. Du HTML
   dans le Rust est exactement l'inverse — il **fabrique** l'affichage.
2. **Coût mesuré du maintien** : changer une couleur exigerait de recompiler et
   **redémarrer le daemon**, donc **couper tous les agents**. Le daemon a été
   redémarré deux fois le 25/08 ; chaque redémarrage coupe la flotte.
3. **Volume** : 153 → ~2000 lignes. Une chaîne de caractères Rust de 2000 lignes
   n'est ni relisible ni testable.

**Ce n'est pas une duplication** : l'embarqué est **retiré** dans le même lot.
Il n'y aura pas deux fronts.

### A2 — Trace inter-agents : agrégation, pas nouvelle collecte

**Équivalent trouvé** : `LedgerEntry` porte déjà tout le nécessaire —
`sender`, `target`, `ts`, `id`.

**Décision : RÉUTILISER la donnée**, ajouter une fonction d'agrégation dans
`ui.rs`.

**Justification** : aucune donnée nouvelle à collecter, aucune table, aucune
migration. **Un regroupement sur des lignes existantes.**

**Conséquence vérifiée** : **aucun lot ne touche la base de données** — donc
**aucune contrainte d'ordre de migration**, contrairement à la chaîne du 25/08
où merger dans le désordre aurait rendu la base **définitivement** incohérente.

## Risques relevés

| Risque | Constat | Parade |
|---|---|---|
| L1 et L2 modifient `ui.rs` | conflit garanti si parallèles | **séquentiel** — imposé au plan §3 |
| L5 et L6 modifient les mêmes assets | idem | **même agent, séquentiel** |
| Retrait du front embarqué | régression possible de la page actuelle | la page actuelle **n'a pas de test** ; les nouveaux critères AC2/AC9 la remplacent |
| `state` de l'agent | l'outil de ronde **confond arrêté et mort** | le contrat C4 exige la distinction ; défaut préexistant, **inscrit comme dû** |

## Dépendances externes

**Aucune.** Ni paquet Rust, ni bibliothèque web. Le front est en JavaScript nu,
servi par le serveur HTTP existant.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/032-gui-poste-de-travail/reuse-audit.md#extrait-2).
