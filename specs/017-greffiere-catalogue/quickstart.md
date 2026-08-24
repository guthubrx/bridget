# Quickstart 017 — Greffière du catalogue

Prérequis : configuration Maicie avec `catalogue_path` absolu pointant vers un
fichier régulier non-symlink sous le projet hôte (jamais `tasks.md` ni un
plan/spec).

## Commandes

```bash
# Vue humaine d'autorité (lecture pure)
./target/release/maicie registre list --config /chemin/absolu/maicie.json

# Même vue, avec les pending_qualification encore ouverts (P)
./target/release/maicie registre list --config /chemin/absolu/maicie.json --attente

# Append d'un constat fermé (sévérité et source DÉCLARÉES, jamais calculées)
./target/release/maicie registre add --config /chemin/absolu/maicie.json \
  --line '{"v":1,"kind":"add","id":"…","date":"2026-08-24T06:00:00Z","mission_source":{"kind":"incident","id":"…"},"severity":"major","text":"…"}'

# Migration prose → pending_qualification (verbatim, sans inventer de champs)
./target/release/maicie registre migrer --config /chemin/absolu/maicie.json \
  --depuis /chemin/absolu/prose-corpus.jsonl

# Qualification humaine d'un pending (sévérité/source fournies par l'humain)
./target/release/maicie registre qualifier --config /chemin/absolu/maicie.json \
  --pending <id> --severity major --source-kind review --source-id <id> \
  --date 2026-08-24T06:00:00Z

# Transcription FR-1711 : sévérité dérivée si fait couvert (pas de --severity)
./target/release/maicie registre consign --config /chemin/absolu/maicie.json \
  --fait gate_failed --source-id G1701 --date 2026-08-24T06:00:00Z \
  --text "gate G1701 rouge"

./target/release/maicie registre consign --config /chemin/absolu/maicie.json \
  --fait review_amender --source-id r-hostile --date 2026-08-24T06:00:00Z \
  --text "AMENDER : …"

# Hors table → pending_qualification (défaut = attente)
./target/release/maicie registre consign --config /chemin/absolu/maicie.json \
  --fait review_approve --source-id r-ok --date 2026-08-24T06:00:00Z \
  --text "APPROVE"
```

Le pied de page de `registre list` est toujours `N/M/K/P` :
`N` ouverts, `M` récurrents, `K` gates ratés déclarés, `P` en attente de
qualification.

## Réconciliation des clôtures (noyau journal)

La transition `open → delivered` n'existe que pour un couple déclaré
`(constat_id, objective_id)` et une clôture d'objectif **attestée**. L'API
journal `reconcile_attested_closures` consomme ces faits ; le raccord store
(T1708) et le déclenchement aux commandes catalogue (T1710) restent à câbler
hors des fichiers encore occupés par la 016. Aucune boucle résidente, aucun
polling : la réconciliation ne tourne qu'au fil d'une commande.

## Rituel agent

1. Début de session et avant toute proposition de suite : `registre list`.
2. Clôture de session : `registre list` en première opération du rituel ;
   afficher le footer `N/M/K/P`.
3. Ne jamais écrire dans un plan/tasks/issues hôte depuis le catalogue.
4. Ne jamais inventer une sévérité hors contrat FR-1711, ni classer un
   constat : Maicie propose, la table transcrit, l'humain qualifie le reste.
