# Contre-revue adverse — 107 journal WAL

- Date : 2026-09-18 — Demandeur : bdget (Claude) — Agent : horizon-cursor (Cursor, `7e9dec19-…`).
  cursor-listen (Cursor) n'accusait toujours pas les messages de la session précédente : non sollicité.
- Contrainte : lecture seule, aucun commit, aucune dépense.

| Étape | Envoi | Remise | Borne | Verdict |
|---|---|---|---|---|
| Plan (spec, research, plan, contrat, reuse-audit, tâches ; 4 questions : point unique dans `Store::open`, échec de bascule au démarrage, exploitation `-wal`/`-shm`, `synchronous=FULL`) | 06:00 | accusée (`reçu`) | 15 min → 06:15 | **pas de réponse dans le délai** |
| Implémentation (`store.rs`, tests spec107, S14, ADR, docs ; 4 questions : réessai borné, ouvreur précédant `Store::open`, `synchronous`, exploitation) | 06:18 | accusée (`reçu`) | 10 min → 06:28 | **pas de réponse dans le délai** |

## Objections

| Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|
| (aucune reçue dans les délais) | | | |

## Auto-revue de substitution (Article XX)

1. Point unique : `Store::open` est le premier ouvreur au démarrage (`daemon.rs:2919`) ; le mode est persistant,
   donc les ouvreurs suivants l'héritent. Preuve : test de conversion (connexion tierce rend `wal`).
2. Échec de bascule : la recette complète l'a révélé (8 ouvreurs simultanés) → relecture + réessai borné ; une base
   déjà WAL n'exige aucun verrou ; au-delà de 2 s l'ouverture échoue explicitement plutôt que de continuer en
   `delete`.
3. Exploitation : `-wal`/`-shm` privés (test 0600), exclusion `*.db-*` au déploiement, règle de copie documentée.
   Time Machine / copie de fichiers à chaud : documenté comme incomplet ; `VACUUM INTO` proposé.
4. `synchronous=FULL` conservé : la durabilité 099 est inchangée ; le gain d'E/S de `NORMAL` reste hors périmètre.
