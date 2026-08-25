# Spec 028 — Registre : traité ≠ réfuté (et list lisible)

**Statut** : en cours  
**Branche** : `session-028-registre-traite-refute`  
**Base** : `b6eea77`  
**Mandat** : objective `0e24bb3b-97f9-408b-b2e5-4ed7c86c8702` / delegation `867ff7e4-eebb-4592-a7ae-ca8db3af9219`  
**Migration SQLite** : aucune (journal catalogue + projection). v21 non consommée.

## Propriété

L'état d'un constat au registre distingue au moins **ouvert**, **traité**, **réfuté** (et **requalifié**), et la **liste** le montre — pas seulement les données.

- **Traité** : le défaut existait, il est corrigé → *ce fut vrai, ce ne l'est plus*.
- **Réfuté** : le défaut n'a jamais existé → *ce ne fut jamais vrai*.
- **Requalifié** : vrai, mais sévérité/portée changée → reste une charge réelle, ni soldée ni effacée.
- Si le registre ne sait que fermer, retirer une charge fausse **ment** (dit qu'un défaut réel a été corrigé).

## Hors périmètre

- Reclassement automatique des 82 entrées liées à un amendement (restent `ouvert` + `recurrence_of` affiché).
- Champ libre obligatoire pour motif (interdit — ensemble fermé de raisons typées).
- Solde des délégations / orphelines `a_evaluer` (déjà soldé hors lot).

## Critères d'acceptation

1. Gestes CLI `registre fermer|refuter|requalifier` avec raisons **typées** (enums fermées) et preuves selon arbitrage.
2. `registre list` affiche badges `[OUVERT]|[TRAITÉ]|[RÉFUTÉ]|[REQUALIFIÉ]`, `recurrence_of` quand présent, pied O/T/R/Q.
3. Oracle : même texte de constat, `fermer` vs `refuter` → lectures **distinctes** (pas le même rendu).
4. Fermeture/réfutation sans preuve refusée ; réfutation refuse `sha:` seul (exige `mesure:N/M`, `0/0` exclu).
5. Suite maicie verte ×3 avec comptes annoncés ; clippy lib+bins `-D warnings`.

## Arbitrages figés (écrits avant code)

Voir message Bridget cursor4 → bridget (id `mcp-15065-6a8d93a5-1`).
